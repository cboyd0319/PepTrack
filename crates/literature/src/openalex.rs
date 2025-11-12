//! OpenAlex API integration
//!
//! OpenAlex is a free, open catalog of the world's scholarly research system.
//! It indexes papers, authors, institutions, and more.
//!
//! # API Documentation
//!
//! - API Docs: https://docs.openalex.org/
//! - Rate limits: 100,000 requests/day, 10/second (no key required)
//! - Polite pool: Email in User-Agent header for faster rate limits
//!
//! # Examples
//!
//! ```no_run
//! use peptrack_literature::{OpenAlexFetcher, LiteratureFetcher};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let fetcher = OpenAlexFetcher::new();
//! let results = fetcher.search("peptide therapy", 5).await?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use tracing::debug;

use crate::models::{LiteratureFetcher, LiteratureResult};

const API_BASE: &str = "https://api.openalex.org/works";

/// OpenAlex API fetcher
pub struct OpenAlexFetcher {
    client: reqwest::Client,
}

impl OpenAlexFetcher {
    /// Creates a new OpenAlex fetcher
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("PepTrack/1.0 (mailto:support@peptrack.app)")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }
}

impl Default for OpenAlexFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LiteratureFetcher for OpenAlexFetcher {
    async fn search(&self, query: &str, max_results: usize) -> Result<Vec<LiteratureResult>> {
        let url = format!(
            "{}?search={}&per-page={}",
            API_BASE,
            urlencoding::encode(query),
            max_results
        );

        debug!("OpenAlex search URL: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send OpenAlex request")?;

        let body = response
            .text()
            .await
            .context("Failed to read OpenAlex response")?;

        let search_result: OpenAlexResponse = serde_json::from_str(&body)
            .with_context(|| format!("Failed to parse OpenAlex response: {}", body))?;

        let results = search_result
            .results
            .into_iter()
            .map(|work| {
                let authors = if work.authorships.is_empty() {
                    None
                } else {
                    Some(
                        work.authorships
                            .iter()
                            .filter_map(|a| a.author.display_name.clone())
                            .collect::<Vec<_>>()
                            .join(", "),
                    )
                };

                // Extract DOI from id (format: https://openalex.org/W1234567)
                let doi = work.doi.clone();

                let abstract_text = work
                    .abstract_inverted_index
                    .as_ref()
                    .and_then(reconstruct_abstract)
                    .or_else(|| {
                        work.abstract_inverted_index
                            .as_ref()
                            .map(|_| String::from("[Abstract available at source]"))
                    });

                LiteratureResult {
                    source: "openalex".to_string(),
                    title: work.title.clone(),
                    url: work.doi.or_else(|| Some(work.id.clone())),
                    doi,
                    authors,
                    published_date: work.publication_date,
                    journal: work
                        .primary_location
                        .and_then(|loc| loc.source.map(|s| s.display_name)),
                    abstract_text,
                }
            })
            .collect();

        Ok(results)
    }

    fn source_name(&self) -> &'static str {
        "openalex"
    }
}

fn reconstruct_abstract(index: &Value) -> Option<String> {
    let obj = index.as_object()?;
    let mut max_pos = 0usize;

    for positions in obj.values() {
        if let Some(arr) = positions.as_array() {
            for pos in arr {
                if let Some(p) = pos.as_u64() {
                    if (p as usize) > max_pos {
                        max_pos = p as usize;
                    }
                }
            }
        }
    }

    let mut words = vec![String::new(); max_pos + 1];
    for (word, positions) in obj.iter() {
        if let Some(arr) = positions.as_array() {
            for pos in arr {
                if let Some(p) = pos.as_u64() {
                    if let Some(slot) = words.get_mut(p as usize) {
                        *slot = word.clone();
                    }
                }
            }
        }
    }

    if words.iter().all(|w| w.is_empty()) {
        None
    } else {
        Some(words.join(" "))
    }
}

// OpenAlex API response types

#[derive(Debug, Deserialize)]
struct OpenAlexResponse {
    results: Vec<Work>,
}

#[derive(Debug, Deserialize)]
struct Work {
    id: String,
    #[serde(default)]
    doi: Option<String>,
    title: String,
    #[serde(default)]
    publication_date: Option<String>,
    #[serde(default)]
    authorships: Vec<Authorship>,
    #[serde(default)]
    primary_location: Option<Location>,
    #[serde(default)]
    abstract_inverted_index: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct Authorship {
    author: AuthorInfo,
}

#[derive(Debug, Deserialize)]
struct AuthorInfo {
    display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Location {
    source: Option<Source>,
}

#[derive(Debug, Deserialize)]
struct Source {
    display_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn openalex_search_returns_results() {
        let fetcher = OpenAlexFetcher::new();
        let results = fetcher.search("peptide therapy", 5).await;

        match results {
            Ok(papers) => {
                assert!(!papers.is_empty(), "Should find papers for peptide therapy");
                assert!(papers.len() <= 5, "Should respect max_results");

                for paper in &papers {
                    assert_eq!(paper.source, "openalex");
                    assert!(!paper.title.is_empty(), "Paper should have title");
                }

                println!("Found {} papers:", papers.len());
                for paper in papers {
                    println!("  - {}", paper.title);
                }
            }
            Err(e) => {
                eprintln!("OpenAlex search failed (network test): {:#}", e);
            }
        }
    }

    #[test]
    fn openalex_fetcher_can_be_created() {
        let _fetcher = OpenAlexFetcher::new();
    }
}
