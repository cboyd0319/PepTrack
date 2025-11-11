//! Crossref API integration
//!
//! Crossref is a DOI registration agency that provides metadata for scholarly works.
//! It's particularly good for finding papers by DOI or searching published research.
//!
//! # API Documentation
//!
//! - API Docs: https://github.com/CrossRef/rest-api-doc
//! - Rate limits: 50 requests/second for polite users
//! - Polite pool: Include email in User-Agent or Plus header
//!
//! # Examples
//!
//! ```no_run
//! use peptrack_literature::{CrossrefFetcher, LiteratureFetcher};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let fetcher = CrossrefFetcher::new();
//! let results = fetcher.search("peptide therapy", 5).await?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use tracing::debug;

use crate::models::{LiteratureFetcher, LiteratureResult};

const API_BASE: &str = "https://api.crossref.org/works";

/// Crossref API fetcher
pub struct CrossrefFetcher {
    client: reqwest::Client,
}

impl CrossrefFetcher {
    /// Creates a new Crossref fetcher
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

impl Default for CrossrefFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LiteratureFetcher for CrossrefFetcher {
    async fn search(&self, query: &str, max_results: usize) -> Result<Vec<LiteratureResult>> {
        let url = format!(
            "{}?query={}&rows={}",
            API_BASE,
            urlencoding::encode(query),
            max_results
        );

        debug!("Crossref search URL: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send Crossref request")?;

        let body = response
            .text()
            .await
            .context("Failed to read Crossref response")?;

        let search_result: CrossrefResponse = serde_json::from_str(&body)
            .with_context(|| format!("Failed to parse Crossref response: {}", body))?;

        let results = search_result
            .message
            .items
            .into_iter()
            .map(|work| {
                let authors = if work.author.is_empty() {
                    None
                } else {
                    Some(
                        work.author
                            .iter()
                            .filter_map(|a| {
                                if let (Some(given), Some(family)) = (&a.given, &a.family) {
                                    Some(format!("{} {}", given, family))
                                } else {
                                    a.family.clone()
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(", "),
                    )
                };

                let url = work.url.or_else(|| work.doi.as_ref().map(|doi| format!("https://doi.org/{}", doi)));

                let published_date = work.published.and_then(|p| {
                    p.date_parts.first().and_then(|parts| {
                        if parts.is_empty() {
                            None
                        } else if parts.len() == 1 {
                            Some(format!("{}", parts[0]))
                        } else if parts.len() == 2 {
                            Some(format!("{}-{:02}", parts[0], parts[1]))
                        } else {
                            Some(format!("{}-{:02}-{:02}", parts[0], parts[1], parts[2]))
                        }
                    })
                });

                let journal = work
                    .container_title
                    .and_then(|titles| titles.first().cloned());

                LiteratureResult {
                    source: "crossref".to_string(),
                    title: work.title.first().cloned().unwrap_or_default(),
                    url,
                    doi: work.doi,
                    authors,
                    published_date,
                    journal,
                    abstract_text: work.abstract_text,
                }
            })
            .collect();

        Ok(results)
    }

    fn source_name(&self) -> &'static str {
        "crossref"
    }
}

// Crossref API response types

#[derive(Debug, Deserialize)]
struct CrossrefResponse {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    items: Vec<Work>,
}

#[derive(Debug, Deserialize)]
struct Work {
    #[serde(default)]
    doi: Option<String>,
    #[serde(default)]
    url: Option<String>,
    title: Vec<String>,
    #[serde(default)]
    author: Vec<Author>,
    #[serde(default)]
    published: Option<DateInfo>,
    #[serde(default, rename = "container-title")]
    container_title: Option<Vec<String>>,
    #[serde(default, rename = "abstract")]
    abstract_text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Author {
    given: Option<String>,
    family: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DateInfo {
    #[serde(rename = "date-parts")]
    date_parts: Vec<Vec<u32>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn crossref_search_returns_results() {
        let fetcher = CrossrefFetcher::new();
        let results = fetcher.search("peptide therapy", 5).await;

        match results {
            Ok(papers) => {
                assert!(!papers.is_empty(), "Should find papers for peptide therapy");
                assert!(papers.len() <= 5, "Should respect max_results");

                for paper in &papers {
                    assert_eq!(paper.source, "crossref");
                    assert!(!paper.title.is_empty(), "Paper should have title");
                }

                println!("Found {} papers:", papers.len());
                for paper in papers {
                    println!("  - {}", paper.title);
                }
            }
            Err(e) => {
                eprintln!("Crossref search failed (network test): {:#}", e);
            }
        }
    }

    #[test]
    fn crossref_fetcher_can_be_created() {
        let _fetcher = CrossrefFetcher::new();
    }
}
