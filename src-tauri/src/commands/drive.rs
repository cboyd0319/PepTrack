use anyhow::{Context, Result};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::state::AppState;

/// Google Drive OAuth configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
}

/// OAuth token storage
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    /// Timestamp when token expires (UTC)
    pub expires_at: Option<String>,
}

/// Drive connection status
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveStatus {
    pub connected: bool,
    pub email: Option<String>,
}

/// OAuth authorization URL response
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthUrlResponse {
    pub auth_url: String,
    pub state: String,
}

/// Storage for OAuth state during flow
#[derive(Default)]
pub struct OAuthState {
    csrf_token: Arc<Mutex<Option<String>>>,
    pkce_verifier: Arc<Mutex<Option<String>>>,
}

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const REDIRECT_URL: &str = "http://localhost:8080/oauth/callback";
const DRIVE_SCOPE: &str = "https://www.googleapis.com/auth/drive.file";

/// Starts the OAuth flow by generating an authorization URL
#[tauri::command]
pub async fn start_drive_oauth(
    config: DriveOAuthConfig,
    state: State<'_, OAuthState>,
) -> Result<AuthUrlResponse, String> {
    info!("Starting Google Drive OAuth flow");

    let client = create_oauth_client(&config).map_err(|e| {
        warn!("Failed to create OAuth client: {:#}", e);
        format!("OAuth setup failed: {}", e)
    })?;

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(DRIVE_SCOPE.to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    // Store state for verification
    *state.csrf_token.lock().await = Some(csrf_token.secret().clone());
    *state.pkce_verifier.lock().await = Some(pkce_verifier.secret().clone());

    info!("OAuth authorization URL generated");

    Ok(AuthUrlResponse {
        auth_url: auth_url.to_string(),
        state: csrf_token.secret().clone(),
    })
}

/// Completes the OAuth flow by exchanging the authorization code for tokens
#[tauri::command]
pub async fn complete_drive_oauth(
    config: DriveOAuthConfig,
    code: String,
    state_param: String,
    oauth_state: State<'_, OAuthState>,
    app_state: State<'_, AppState>,
) -> Result<DriveStatus, String> {
    info!("Completing Google Drive OAuth flow");

    // Verify CSRF token
    let stored_state = oauth_state.csrf_token.lock().await.clone();
    if stored_state.as_deref() != Some(&state_param) {
        warn!("CSRF token mismatch");
        return Err("Invalid OAuth state (CSRF mismatch)".to_string());
    }

    let pkce_verifier = oauth_state
        .pkce_verifier
        .lock()
        .await
        .clone()
        .ok_or_else(|| "PKCE verifier not found".to_string())?;

    let client = create_oauth_client(&config)
        .map_err(|e| format!("OAuth setup failed: {}", e))?;

    // Exchange authorization code for tokens
    let pkce_verifier = oauth2::PkceCodeVerifier::new(pkce_verifier);
    let token_result = client
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await
        .map_err(|e| {
            warn!("Token exchange failed: {:#}", e);
            format!("Failed to get access token: {}", e)
        })?;

    let expires_in = token_result.expires_in().map(|d| d.as_secs());
    let expires_at = expires_in.map(|secs| {
        (time::OffsetDateTime::now_utc() + time::Duration::seconds(secs as i64)).to_string()
    });

    let tokens = DriveTokens {
        access_token: token_result.access_token().secret().clone(),
        refresh_token: token_result.refresh_token().map(|t| t.secret().clone()),
        expires_in,
        expires_at,
    };

    // Store tokens and config
    store_drive_tokens(&app_state, &tokens)
        .await
        .map_err(|e| format!("Failed to store tokens: {}", e))?;

    store_drive_config(&config)
        .await
        .map_err(|e| format!("Failed to store OAuth config: {}", e))?;

    info!("Google Drive OAuth completed successfully");

    // Try to get user info
    let email = get_user_email(&tokens.access_token).await.ok();

    Ok(DriveStatus {
        connected: true,
        email,
    })
}

/// Checks Google Drive connection status
#[tauri::command]
pub async fn check_drive_status(state: State<'_, AppState>) -> Result<DriveStatus, String> {
    // Try to load and refresh tokens if needed
    let tokens = load_and_refresh_tokens(&state).await;

    if let Ok(tokens) = tokens {
        let email = get_user_email(&tokens.access_token).await.ok();
        Ok(DriveStatus {
            connected: true,
            email,
        })
    } else {
        Ok(DriveStatus {
            connected: false,
            email: None,
        })
    }
}

/// Disconnects Google Drive by removing stored tokens
#[tauri::command]
pub async fn disconnect_drive(state: State<'_, AppState>) -> Result<(), String> {
    info!("Disconnecting Google Drive");

    delete_drive_tokens(&state)
        .await
        .map_err(|e| format!("Failed to disconnect: {}", e))?;

    info!("Google Drive disconnected successfully");
    Ok(())
}

/// Uploads a backup file to Google Drive
#[tauri::command]
pub async fn upload_to_drive(
    filename: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    info!("Uploading backup to Google Drive: {}", filename);

    let tokens = load_and_refresh_tokens(&state)
        .await
        .map_err(|e| format!("Not connected to Google Drive: {}", e))?;

    let client = Client::new();

    // Create or get PepTrack folder
    let folder_id = get_or_create_folder(&client, &tokens.access_token, "PepTrack Backups")
        .await
        .map_err(|e| format!("Failed to create folder: {}", e))?;

    // Upload file
    let file_id = upload_file(&client, &tokens.access_token, &folder_id, &filename, &content)
        .await
        .map_err(|e| format!("Failed to upload file: {}", e))?;

    info!("Backup uploaded successfully: {}", file_id);
    Ok(file_id)
}

// Helper functions

fn create_oauth_client(config: &DriveOAuthConfig) -> Result<BasicClient> {
    Ok(BasicClient::new(
        ClientId::new(config.client_id.clone()),
        Some(ClientSecret::new(config.client_secret.clone())),
        AuthUrl::new(GOOGLE_AUTH_URL.to_string())?,
        Some(TokenUrl::new(GOOGLE_TOKEN_URL.to_string())?),
    )
    .set_redirect_uri(RedirectUrl::new(REDIRECT_URL.to_string())?))
}

async fn store_drive_tokens(_state: &AppState, tokens: &DriveTokens) -> Result<()> {
    // Store tokens as JSON in the app data directory
    let data_dir = dirs::data_dir()
        .context("Unable to determine data directory")?
        .join("PepTrack");
    std::fs::create_dir_all(&data_dir)?;

    let tokens_file = data_dir.join("drive_tokens.json");
    let json = serde_json::to_string(tokens)?;
    std::fs::write(&tokens_file, json)
        .context("Failed to store Drive tokens")?;

    Ok(())
}

async fn store_drive_config(config: &DriveOAuthConfig) -> Result<()> {
    let data_dir = dirs::data_dir()
        .context("Unable to determine data directory")?
        .join("PepTrack");
    std::fs::create_dir_all(&data_dir)?;

    let config_file = data_dir.join("drive_oauth_config.json");
    let json = serde_json::to_string(config)?;
    std::fs::write(&config_file, json)
        .context("Failed to store Drive OAuth config")?;

    Ok(())
}

async fn load_drive_config() -> Result<DriveOAuthConfig> {
    let data_dir = dirs::data_dir()
        .context("Unable to determine data directory")?
        .join("PepTrack");
    let config_file = data_dir.join("drive_oauth_config.json");

    let json = std::fs::read_to_string(&config_file)
        .context("Drive OAuth config not found")?;
    let config: DriveOAuthConfig = serde_json::from_str(&json)?;
    Ok(config)
}

async fn load_drive_tokens(_state: &AppState) -> Result<DriveTokens> {
    let data_dir = dirs::data_dir()
        .context("Unable to determine data directory")?
        .join("PepTrack");
    let tokens_file = data_dir.join("drive_tokens.json");

    let json = std::fs::read_to_string(&tokens_file)
        .context("Drive tokens not found")?;
    let tokens: DriveTokens = serde_json::from_str(&json)?;
    Ok(tokens)
}

// Public helper functions for use by scheduler
pub async fn load_drive_tokens_internal(state: &AppState) -> Result<DriveTokens> {
    // Use the same refresh logic as the commands
    load_and_refresh_tokens(state).await
}

async fn delete_drive_tokens(_state: &AppState) -> Result<()> {
    let data_dir = dirs::data_dir()
        .context("Unable to determine data directory")?
        .join("PepTrack");
    let tokens_file = data_dir.join("drive_tokens.json");
    let config_file = data_dir.join("drive_oauth_config.json");

    if tokens_file.exists() {
        std::fs::remove_file(&tokens_file)
            .context("Failed to delete Drive tokens")?;
    }

    // Also delete the OAuth config
    if config_file.exists() {
        std::fs::remove_file(&config_file)
            .context("Failed to delete Drive OAuth config")?;
    }

    Ok(())
}

async fn get_user_email(access_token: &str) -> Result<String> {
    let client = Client::new();
    let response = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    response
        .get("email")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .context("Email not found in user info")
}

/// Check if token needs refresh (expires in less than 5 minutes)
fn should_refresh_token(tokens: &DriveTokens) -> bool {
    if let Some(expires_at_str) = &tokens.expires_at {
        if let Ok(expires_at) = time::OffsetDateTime::parse(
            expires_at_str,
            &time::format_description::well_known::Rfc3339,
        ) {
            let now = time::OffsetDateTime::now_utc();
            let buffer = time::Duration::minutes(5);
            return now + buffer >= expires_at;
        }
    }
    // If we can't determine expiry, assume it needs refresh
    true
}

/// Refresh access token using refresh token
async fn refresh_access_token(
    tokens: &DriveTokens,
    config: &DriveOAuthConfig,
) -> Result<DriveTokens> {
    let refresh_token = tokens
        .refresh_token
        .as_ref()
        .context("No refresh token available")?;

    let client = create_oauth_client(config)?;

    let token_result = client
        .exchange_refresh_token(&oauth2::RefreshToken::new(refresh_token.clone()))
        .request_async(async_http_client)
        .await
        .context("Failed to refresh token")?;

    let expires_in = token_result.expires_in().map(|d| d.as_secs());
    let expires_at = expires_in.map(|secs| {
        (time::OffsetDateTime::now_utc() + time::Duration::seconds(secs as i64)).to_string()
    });

    Ok(DriveTokens {
        access_token: token_result.access_token().secret().clone(),
        refresh_token: Some(refresh_token.clone()), // Keep the same refresh token
        expires_in,
        expires_at,
    })
}

/// Load tokens and refresh if needed
async fn load_and_refresh_tokens(state: &AppState) -> Result<DriveTokens> {
    let mut tokens = load_drive_tokens(state).await?;

    if should_refresh_token(&tokens) {
        info!("Access token expired or expiring soon, refreshing...");

        if tokens.refresh_token.is_some() {
            // Try to load OAuth config and refresh
            match load_drive_config().await {
                Ok(config) => {
                    match refresh_access_token(&tokens, &config).await {
                        Ok(new_tokens) => {
                            info!("Successfully refreshed access token");
                            // Store the new tokens
                            store_drive_tokens(state, &new_tokens).await?;
                            tokens = new_tokens;
                        }
                        Err(e) => {
                            warn!("Failed to refresh token: {:#}", e);
                            // Return error - user needs to re-authenticate
                            return Err(e.context("Token refresh failed - please reconnect Google Drive"));
                        }
                    }
                }
                Err(e) => {
                    warn!("OAuth config not found: {:#}", e);
                    return Err(e.context("OAuth config not found - please reconnect Google Drive"));
                }
            }
        } else {
            warn!("No refresh token available");
            return Err(anyhow::anyhow!("No refresh token available - please reconnect Google Drive"));
        }
    }

    Ok(tokens)
}

async fn get_or_create_folder(
    client: &Client,
    access_token: &str,
    folder_name: &str,
) -> Result<String> {
    get_or_create_folder_internal(client, access_token, folder_name).await
}

pub async fn get_or_create_folder_internal(
    client: &Client,
    access_token: &str,
    folder_name: &str,
) -> Result<String> {
    // Search for existing folder
    let search_url = format!(
        "https://www.googleapis.com/drive/v3/files?q=name='{}' and mimeType='application/vnd.google-apps.folder' and trashed=false",
        folder_name
    );

    let response = client
        .get(&search_url)
        .bearer_auth(access_token)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    // Check if folder exists
    if let Some(files) = response.get("files").and_then(|f| f.as_array()) {
        if let Some(folder) = files.first() {
            if let Some(id) = folder.get("id").and_then(|i| i.as_str()) {
                return Ok(id.to_string());
            }
        }
    }

    // Create folder if it doesn't exist
    let create_body = serde_json::json!({
        "name": folder_name,
        "mimeType": "application/vnd.google-apps.folder"
    });

    let create_response = client
        .post("https://www.googleapis.com/drive/v3/files")
        .bearer_auth(access_token)
        .json(&create_body)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    create_response
        .get("id")
        .and_then(|i| i.as_str())
        .map(|s| s.to_string())
        .context("Failed to get folder ID")
}

async fn upload_file(
    client: &Client,
    access_token: &str,
    folder_id: &str,
    filename: &str,
    content: &str,
) -> Result<String> {
    upload_file_internal(client, access_token, folder_id, filename, content).await
}

pub async fn upload_file_internal(
    client: &Client,
    access_token: &str,
    folder_id: &str,
    filename: &str,
    content: &str,
) -> Result<String> {
    let metadata = serde_json::json!({
        "name": filename,
        "parents": [folder_id]
    });

    let boundary = "boundary_string";
    let body = format!(
        "--{}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{}\r\n--{}\r\nContent-Type: application/json\r\n\r\n{}\r\n--{}--",
        boundary,
        metadata,
        boundary,
        content,
        boundary
    );

    let response = client
        .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart")
        .bearer_auth(access_token)
        .header(
            "Content-Type",
            format!("multipart/related; boundary={}", boundary),
        )
        .body(body)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    response
        .get("id")
        .and_then(|i| i.as_str())
        .map(|s| s.to_string())
        .context("Failed to get file ID")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drive_oauth_config_serialization() {
        let config = DriveOAuthConfig {
            client_id: "test-client-id".to_string(),
            client_secret: "test-secret".to_string(),
        };

        let json = serde_json::to_string(&config);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        assert!(json_str.contains("clientId"));
        assert!(json_str.contains("clientSecret"));
        assert!(json_str.contains("test-client-id"));
    }

    #[test]
    fn test_drive_oauth_config_deserialization() {
        let json = r#"{
            "clientId": "my-client-id",
            "clientSecret": "my-secret"
        }"#;

        let config: Result<DriveOAuthConfig, _> = serde_json::from_str(json);
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.client_id, "my-client-id");
        assert_eq!(config.client_secret, "my-secret");
    }

    #[test]
    fn test_drive_oauth_config_round_trip() {
        let original = DriveOAuthConfig {
            client_id: "abc123".to_string(),
            client_secret: "secret456".to_string(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: DriveOAuthConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(original.client_id, deserialized.client_id);
        assert_eq!(original.client_secret, deserialized.client_secret);
    }

    #[test]
    fn test_drive_tokens_serialization() {
        let tokens = DriveTokens {
            access_token: "ya29.test_token".to_string(),
            refresh_token: Some("refresh_token_123".to_string()),
            expires_in: Some(3600),
            expires_at: Some("2025-11-11T12:00:00Z".to_string()),
        };

        let json = serde_json::to_string(&tokens);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        assert!(json_str.contains("accessToken"));
        assert!(json_str.contains("refreshToken"));
        assert!(json_str.contains("expiresIn"));
    }

    #[test]
    fn test_drive_tokens_deserialization() {
        let json = r#"{
            "accessToken": "token123",
            "refreshToken": "refresh456",
            "expiresIn": 3600
        }"#;

        let tokens: Result<DriveTokens, _> = serde_json::from_str(json);
        assert!(tokens.is_ok());

        let tokens = tokens.unwrap();
        assert_eq!(tokens.access_token, "token123");
        assert_eq!(tokens.refresh_token, Some("refresh456".to_string()));
        assert_eq!(tokens.expires_in, Some(3600));
    }

    #[test]
    fn test_drive_tokens_without_refresh_token() {
        let json = r#"{
            "accessToken": "token123",
            "expiresIn": 3600
        }"#;

        let tokens: Result<DriveTokens, _> = serde_json::from_str(json);
        assert!(tokens.is_ok());

        let tokens = tokens.unwrap();
        assert_eq!(tokens.access_token, "token123");
        assert_eq!(tokens.refresh_token, None);
        assert_eq!(tokens.expires_in, Some(3600));
    }

    #[test]
    fn test_drive_tokens_without_expiry() {
        let json = r#"{
            "accessToken": "token123",
            "refreshToken": "refresh456"
        }"#;

        let tokens: Result<DriveTokens, _> = serde_json::from_str(json);
        assert!(tokens.is_ok());

        let tokens = tokens.unwrap();
        assert_eq!(tokens.access_token, "token123");
        assert_eq!(tokens.refresh_token, Some("refresh456".to_string()));
        assert_eq!(tokens.expires_in, None);
    }

    #[test]
    fn test_drive_tokens_minimal() {
        let json = r#"{
            "accessToken": "token123"
        }"#;

        let tokens: Result<DriveTokens, _> = serde_json::from_str(json);
        assert!(tokens.is_ok());

        let tokens = tokens.unwrap();
        assert_eq!(tokens.access_token, "token123");
        assert_eq!(tokens.refresh_token, None);
        assert_eq!(tokens.expires_in, None);
    }

    #[test]
    fn test_drive_status_connected_serialization() {
        let status = DriveStatus {
            connected: true,
            email: Some("user@example.com".to_string()),
        };

        let json = serde_json::to_string(&status);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        assert!(json_str.contains("connected"));
        assert!(json_str.contains("email"));
        assert!(json_str.contains("user@example.com"));
    }

    #[test]
    fn test_drive_status_disconnected() {
        let status = DriveStatus {
            connected: false,
            email: None,
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"connected\":false"));
    }

    #[test]
    fn test_auth_url_response_serialization() {
        let response = AuthUrlResponse {
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth?...".to_string(),
            state: "random_state_token".to_string(),
        };

        let json = serde_json::to_string(&response);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        assert!(json_str.contains("authUrl"));
        assert!(json_str.contains("state"));
        assert!(json_str.contains("accounts.google.com"));
    }

    #[test]
    fn test_drive_oauth_config_with_long_credentials() {
        let config = DriveOAuthConfig {
            client_id: "x".repeat(1000),
            client_secret: "y".repeat(1000),
        };

        let json = serde_json::to_string(&config);
        assert!(json.is_ok());

        let deserialized: DriveOAuthConfig = serde_json::from_str(&json.unwrap()).unwrap();
        assert_eq!(deserialized.client_id.len(), 1000);
        assert_eq!(deserialized.client_secret.len(), 1000);
    }

    #[test]
    fn test_drive_tokens_with_special_characters() {
        let tokens = DriveTokens {
            access_token: "token_with-special.chars/123".to_string(),
            refresh_token: Some("refresh+token=abc".to_string()),
            expires_in: Some(7200),
            expires_at: None,
        };

        let json = serde_json::to_string(&tokens).unwrap();
        let deserialized: DriveTokens = serde_json::from_str(&json).unwrap();

        assert_eq!(tokens.access_token, deserialized.access_token);
        assert_eq!(tokens.refresh_token, deserialized.refresh_token);
    }

    #[test]
    fn test_drive_status_with_special_email() {
        let status = DriveStatus {
            connected: true,
            email: Some("user+tag@sub.example.com".to_string()),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("user+tag@sub.example.com"));
    }

    #[test]
    fn test_drive_tokens_clone() {
        let tokens = DriveTokens {
            access_token: "token1".to_string(),
            refresh_token: Some("refresh1".to_string()),
            expires_in: Some(3600),
            expires_at: None,
        };

        let cloned = tokens.clone();
        assert_eq!(tokens.access_token, cloned.access_token);
        assert_eq!(tokens.refresh_token, cloned.refresh_token);
        assert_eq!(tokens.expires_in, cloned.expires_in);
    }

    #[test]
    fn test_drive_oauth_config_clone() {
        let config = DriveOAuthConfig {
            client_id: "id1".to_string(),
            client_secret: "secret1".to_string(),
        };

        let cloned = config.clone();
        assert_eq!(config.client_id, cloned.client_id);
        assert_eq!(config.client_secret, cloned.client_secret);
    }

    #[test]
    fn test_oauth_state_default() {
        let state = OAuthState::default();
        // Should create without panicking
        assert!(state.csrf_token.try_lock().is_ok());
        assert!(state.pkce_verifier.try_lock().is_ok());
    }

    #[test]
    fn test_drive_tokens_expiry_edge_cases() {
        // Zero expiry
        let tokens = DriveTokens {
            access_token: "token".to_string(),
            refresh_token: None,
            expires_in: Some(0),
            expires_at: None,
        };
        let json = serde_json::to_string(&tokens).unwrap();
        let deserialized: DriveTokens = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.expires_in, Some(0));

        // Very large expiry
        let tokens = DriveTokens {
            access_token: "token".to_string(),
            refresh_token: None,
            expires_in: Some(u64::MAX),
            expires_at: None,
        };
        let json = serde_json::to_string(&tokens).unwrap();
        let deserialized: DriveTokens = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.expires_in, Some(u64::MAX));
    }

    #[test]
    fn test_drive_oauth_config_empty_strings() {
        let config = DriveOAuthConfig {
            client_id: String::new(),
            client_secret: String::new(),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: DriveOAuthConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.client_id, "");
        assert_eq!(deserialized.client_secret, "");
    }

    #[test]
    fn test_drive_tokens_debug_format() {
        let tokens = DriveTokens {
            access_token: "secret_token".to_string(),
            refresh_token: Some("secret_refresh".to_string()),
            expires_in: Some(3600),
            expires_at: None,
        };

        let debug_str = format!("{:?}", tokens);
        assert!(debug_str.contains("DriveTokens"));
        // Note: In production, we might want to redact tokens in debug output
    }

    #[test]
    fn test_auth_url_response_with_query_params() {
        let response = AuthUrlResponse {
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth?client_id=123&redirect_uri=http://localhost&scope=drive.file".to_string(),
            state: "csrf_token_12345".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: AuthUrlResponse = serde_json::from_str(&json).unwrap();

        assert!(deserialized.auth_url.contains("client_id"));
        assert!(deserialized.auth_url.contains("redirect_uri"));
        assert!(deserialized.auth_url.contains("scope"));
    }

    #[test]
    fn test_drive_status_camel_case_conversion() {
        let status = DriveStatus {
            connected: true,
            email: Some("test@example.com".to_string()),
        };

        let json = serde_json::to_string(&status).unwrap();

        // Should be camelCase
        assert!(json.contains("connected"));
        assert!(json.contains("email"));

        // Should NOT be snake_case (already camelCase field names)
        assert!(!json.contains("is_connected"));
    }
}
