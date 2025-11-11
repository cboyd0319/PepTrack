use anyhow::Result;
use peptrack_core::models::LiteratureEntry;
use peptrack_literature::{CrossrefFetcher, LiteratureFetcher, OpenAlexFetcher, PubMedFetcher};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

/// Result from a literature search across multiple sources
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiteratureSearchResult {
    pub source: String,
    pub results: Vec<peptrack_literature::LiteratureResult>,
}

/// Request to search for literature
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchLiteraturePayload {
    pub query: String,
    pub max_results: Option<usize>,
    pub sources: Option<Vec<String>>, // ["pubmed", "openalex", "crossref"]
}

/// Lists all cached literature entries
#[tauri::command]
pub async fn list_literature(state: State<'_, std::sync::Arc<AppState>>) -> Result<Vec<LiteratureEntry>, String> {
    state
        .storage
        .list_literature()
        .map_err(|err| err.to_string())
}

/// Searches cached literature by query
#[tauri::command]
pub async fn search_cached_literature(
    state: State<'_, std::sync::Arc<AppState>>,
    query: String,
) -> Result<Vec<LiteratureEntry>, String> {
    state
        .storage
        .search_literature(&query)
        .map_err(|err| err.to_string())
}

/// Searches external APIs for new literature and caches results
#[tauri::command]
pub async fn search_literature(
    state: State<'_, std::sync::Arc<AppState>>,
    payload: SearchLiteraturePayload,
) -> Result<Vec<LiteratureSearchResult>, String> {
    let max_results = payload.max_results.unwrap_or(10);
    let sources = payload
        .sources
        .unwrap_or_else(|| vec!["pubmed".to_string(), "openalex".to_string()]);

    let mut all_results = Vec::new();

    // Search each requested source
    for source_name in sources {
        let fetcher_result: Result<Box<dyn LiteratureFetcher>, String> = match source_name.as_str()
        {
            "pubmed" => Ok(Box::new(PubMedFetcher::new())),
            "openalex" => Ok(Box::new(OpenAlexFetcher::new())),
            "crossref" => Ok(Box::new(CrossrefFetcher::new())),
            _ => Err(format!("Unknown source: {}", source_name)),
        };

        let fetcher = fetcher_result?;

        match fetcher.search(&payload.query, max_results).await {
            Ok(results) => {
                // Cache all results
                for result in &results {
                    let entry = result.to_entry();
                    if let Err(e) = state.storage.cache_literature(&entry) {
                        eprintln!("Failed to cache literature entry: {:#}", e);
                    }
                }

                all_results.push(LiteratureSearchResult {
                    source: source_name,
                    results,
                });
            }
            Err(e) => {
                eprintln!("Failed to search {}: {:#}", source_name, e);
                // Continue with other sources even if one fails
            }
        }
    }

    Ok(all_results)
}
