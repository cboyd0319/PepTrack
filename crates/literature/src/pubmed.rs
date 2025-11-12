//! PubMed E-utilities API integration
//!
//! PubMed is a free database of biomedical literature from MEDLINE and life science journals.
//! This module uses the E-utilities API to search for and fetch article metadata.
//!
//! # API Documentation
//!
//! - E-utilities Guide: https://www.ncbi.nlm.nih.gov/books/NBK25501/
//! - Rate limits: Max 3 requests/second without API key, 10/second with key
//!
//! # Examples
//!
//! ```no_run
//! use peptrack_literature::{PubMedFetcher, LiteratureFetcher};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let fetcher = PubMedFetcher::new();
//! let results = fetcher.search("peptide therapy", 5).await?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use tracing::{debug, warn};

use crate::models::{LiteratureFetcher, LiteratureResult};

const ESEARCH_BASE: &str = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi";
const ESUMMARY_BASE: &str = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esummary.fcgi";

/// PubMed API fetcher using E-utilities
pub struct PubMedFetcher {
    client: reqwest::Client,
    api_key: Option<String>,
}

impl PubMedFetcher {
    /// Creates a new PubMed fetcher
    ///
    /// Use `with_api_key()` to provide an NCBI API key for higher rate limits.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("PepTrack/1.0")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            api_key: None,
        }
    }

    /// Creates a fetcher with an NCBI API key
    ///
    /// With an API key, rate limit increases from 3 req/s to 10 req/s.
    pub fn with_api_key(api_key: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("PepTrack/1.0")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            api_key: Some(api_key),
        }
    }

    /// Searches PubMed and returns PMIDs
    async fn search_pmids(&self, query: &str, max_results: usize) -> Result<Vec<String>> {
        let mut url = format!(
            "{}?db=pubmed&term={}&retmode=json&retmax={}",
            ESEARCH_BASE,
            urlencoding::encode(query),
            max_results
        );

        if let Some(key) = &self.api_key {
            url.push_str(&format!("&api_key={}", key));
        }

        debug!("PubMed search URL: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send PubMed search request")?;

        let body = response
            .text()
            .await
            .context("Failed to read PubMed search response")?;

        let search_result: ESearchResult = serde_json::from_str(&body)
            .with_context(|| format!("Failed to parse PubMed search response: {}", body))?;

        Ok(search_result.esearchresult.idlist)
    }

    /// Fetches article summaries for given PMIDs
    async fn fetch_summaries(&self, pmids: &[String]) -> Result<Vec<LiteratureResult>> {
        if pmids.is_empty() {
            return Ok(Vec::new());
        }

        let id_list = pmids.join(",");
        let mut url = format!("{}?db=pubmed&id={}&retmode=json", ESUMMARY_BASE, id_list);

        if let Some(key) = &self.api_key {
            url.push_str(&format!("&api_key={}", key));
        }

        debug!("PubMed summary URL: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send PubMed summary request")?;

        let body = response
            .text()
            .await
            .context("Failed to read PubMed summary response")?;

        let summary_result: ESummaryResult = serde_json::from_str(&body)
            .with_context(|| format!("Failed to parse PubMed summary response: {}", body))?;

        let mut results = Vec::new();

        for pmid in pmids {
            if let Some(article_value) = summary_result.result.articles.get(pmid) {
                // Try to deserialize the article data
                match serde_json::from_value::<ArticleSummary>(article_value.clone()) {
                    Ok(article) => {
                        // Skip error entries
                        if article.title.is_none() {
                            warn!("PMID {} has no title, skipping", pmid);
                            continue;
                        }

                        let title = article.title.clone().unwrap_or_default();
                        let authors = article.authors.as_ref().map(|a| {
                            a.iter()
                                .map(|author| author.name.clone())
                                .collect::<Vec<_>>()
                                .join(", ")
                        });

                        let doi = article.articleids.as_ref().and_then(|ids| {
                            ids.iter()
                                .find(|id| id.idtype == "doi")
                                .map(|id| id.value.clone())
                        });

                        results.push(LiteratureResult {
                            source: "pubmed".to_string(),
                            title,
                            url: Some(format!("https://pubmed.ncbi.nlm.nih.gov/{}/", pmid)),
                            doi,
                            authors,
                            published_date: article.pubdate.clone(),
                            journal: article.fulljournalname.clone(),
                            abstract_text: None, // Summary API doesn't include abstracts
                        });
                    }
                    Err(e) => {
                        warn!("Failed to parse article {}: {}", pmid, e);
                    }
                }
            }
        }

        Ok(results)
    }
}

impl Default for PubMedFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LiteratureFetcher for PubMedFetcher {
    async fn search(&self, query: &str, max_results: usize) -> Result<Vec<LiteratureResult>> {
        let pmids = self.search_pmids(query, max_results).await?;
        self.fetch_summaries(&pmids).await
    }

    fn source_name(&self) -> &'static str {
        "pubmed"
    }
}

// PubMed API response types

#[derive(Debug, Deserialize)]
struct ESearchResult {
    esearchresult: ESearchData,
}

#[derive(Debug, Deserialize)]
struct ESearchData {
    idlist: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ESummaryResult {
    result: ESummaryData,
}

#[derive(Debug, Deserialize)]
struct ESummaryData {
    #[serde(default)]
    #[allow(dead_code)]
    uids: Vec<String>,
    #[serde(flatten)]
    articles: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ArticleSummary {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    authors: Option<Vec<Author>>,
    #[serde(default)]
    pubdate: Option<String>,
    #[serde(default)]
    fulljournalname: Option<String>,
    #[serde(default)]
    articleids: Option<Vec<ArticleId>>,
}

#[derive(Debug, Deserialize)]
struct Author {
    name: String,
}

#[derive(Debug, Deserialize)]
struct ArticleId {
    idtype: String,
    value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn pubmed_search_returns_results() {
        let fetcher = PubMedFetcher::new();
        let results = fetcher.search("BPC-157", 5).await;

        match results {
            Ok(papers) => {
                assert!(!papers.is_empty(), "Should find papers for BPC-157");
                assert!(papers.len() <= 5, "Should respect max_results");

                for paper in &papers {
                    assert_eq!(paper.source, "pubmed");
                    assert!(!paper.title.is_empty(), "Paper should have title");
                    assert!(paper.url.is_some(), "Paper should have URL");
                }

                println!("Found {} papers:", papers.len());
                for paper in papers {
                    println!("  - {}", paper.title);
                }
            }
            Err(e) => {
                // Network tests can fail - don't fail the build
                eprintln!("PubMed search failed (network test): {:#}", e);
            }
        }
    }

    #[tokio::test]
    async fn pubmed_search_empty_query() {
        let fetcher = PubMedFetcher::new();
        let results = fetcher.search("xyznonexistentpeptide12345", 5).await;

        match results {
            Ok(papers) => {
                // Empty results are valid
                assert!(papers.is_empty(), "Should return empty for nonsense query");
            }
            Err(e) => {
                eprintln!("PubMed search failed (network test): {:#}", e);
            }
        }
    }

    #[test]
    fn pubmed_fetcher_can_be_created() {
        let _fetcher = PubMedFetcher::new();
        let _fetcher_with_key = PubMedFetcher::with_api_key("test_key".to_string());
    }
}
