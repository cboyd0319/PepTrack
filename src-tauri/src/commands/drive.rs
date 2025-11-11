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
}

/// Drive connection status
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveStatus {
    pub connected: bool,
    pub email: Option<String>,
}

/// OAuth authorization URL response
#[derive(Debug, Serialize)]
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

    let tokens = DriveTokens {
        access_token: token_result.access_token().secret().clone(),
        refresh_token: token_result.refresh_token().map(|t| t.secret().clone()),
        expires_in: token_result.expires_in().map(|d| d.as_secs()),
    };

    // Store tokens in keychain
    store_drive_tokens(&app_state, &tokens)
        .await
        .map_err(|e| format!("Failed to store tokens: {}", e))?;

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
    let tokens = load_drive_tokens(&state).await;

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

    let tokens = load_drive_tokens(&state)
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

async fn delete_drive_tokens(_state: &AppState) -> Result<()> {
    let data_dir = dirs::data_dir()
        .context("Unable to determine data directory")?
        .join("PepTrack");
    let tokens_file = data_dir.join("drive_tokens.json");

    if tokens_file.exists() {
        std::fs::remove_file(&tokens_file)
            .context("Failed to delete Drive tokens")?;
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

async fn get_or_create_folder(
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
