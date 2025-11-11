//! PepTrack Literature - Research paper fetching and caching
//!
//! This crate provides integrations with scientific literature APIs:
//! - PubMed: Free biomedical literature database
//! - OpenAlex: Open catalog of scholarly works
//! - Crossref: DOI-based metadata service
//!
//! # Architecture
//!
//! Each API has a dedicated fetcher module that implements normalized search.
//! All fetchers return `LiteratureResult` structs that can be converted to
//! `LiteratureEntry` for storage.
//!
//! # Examples
//!
//! ```no_run
//! use peptrack_literature::{PubMedFetcher, LiteratureFetcher};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let fetcher = PubMedFetcher::new();
//! let results = fetcher.search("BPC-157 wound healing", 10).await?;
//! for result in results {
//!     println!("{}: {}", result.title, result.url.unwrap_or_default());
//! }
//! # Ok(())
//! # }
//! ```

pub mod models;
pub mod pubmed;
pub mod openalex;
pub mod crossref;

pub use models::{LiteratureResult, LiteratureFetcher};
pub use pubmed::PubMedFetcher;
pub use openalex::OpenAlexFetcher;
pub use crossref::CrossrefFetcher;
