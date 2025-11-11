//! Common types for literature fetching

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// Normalized literature search result from any API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteratureResult {
    /// Source API (e.g., "pubmed", "openalex", "crossref")
    pub source: String,
    /// Paper title
    pub title: String,
    /// URL to the paper or abstract
    pub url: Option<String>,
    /// DOI if available
    pub doi: Option<String>,
    /// Authors list (comma-separated)
    pub authors: Option<String>,
    /// Publication date if available
    pub published_date: Option<String>,
    /// Journal or venue name
    pub journal: Option<String>,
    /// Abstract or summary text
    pub abstract_text: Option<String>,
}

impl LiteratureResult {
    /// Converts this result into a `peptrack_core::LiteratureEntry`
    ///
    /// This is the format that gets stored in the encrypted database.
    /// The `summary` field is left empty - it will be filled by AI summarization later.
    pub fn to_entry(&self) -> peptrack_core::LiteratureEntry {
        peptrack_core::LiteratureEntry {
            id: Uuid::new_v4().to_string(),
            source: self.source.clone(),
            title: self.title.clone(),
            url: self.url.clone(),
            summary: self.abstract_text.clone(),
            relevance_score: None,
            indexed_at: OffsetDateTime::now_utc(),
        }
    }
}

/// Trait for all literature fetchers
///
/// Each API implementation (PubMed, OpenAlex, Crossref) implements this trait
/// to provide a unified search interface.
#[async_trait]
pub trait LiteratureFetcher: Send + Sync {
    /// Searches for literature matching the query
    ///
    /// # Arguments
    ///
    /// * `query` - Search terms (e.g., "BPC-157 wound healing")
    /// * `max_results` - Maximum number of results to return
    ///
    /// # Returns
    ///
    /// A vector of normalized literature results
    async fn search(&self, query: &str, max_results: usize) -> Result<Vec<LiteratureResult>>;

    /// Returns the source name for this fetcher (e.g., "pubmed")
    fn source_name(&self) -> &'static str;
}
