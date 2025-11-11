use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::db::now_timestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeptideProtocol {
    pub id: String,
    pub name: String,
    pub peptide_name: String,
    pub notes: Option<String>,
    pub current_vial_status: Option<String>,
    pub target_concentration_mg_ml: Option<f32>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl PeptideProtocol {
    pub fn new<S: Into<String>>(name: S, peptide_name: S) -> Self {
        let now = now_timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            peptide_name: peptide_name.into(),
            notes: None,
            current_vial_status: None,
            target_concentration_mg_ml: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoseLog {
    pub id: String,
    pub protocol_id: String,
    pub site: String,
    pub amount_mg: f32,
    pub notes: Option<String>,
    pub logged_at: OffsetDateTime,
}

impl DoseLog {
    pub fn new<S: Into<String>>(protocol_id: S, site: S, amount_mg: f32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            protocol_id: protocol_id.into(),
            site: site.into(),
            amount_mg,
            notes: None,
            logged_at: now_timestamp(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteratureEntry {
    pub id: String,
    pub source: String,
    pub title: String,
    pub url: Option<String>,
    pub summary: Option<String>,
    pub relevance_score: Option<f32>,
    pub indexed_at: OffsetDateTime,
}

impl LiteratureEntry {
    pub fn new<S: Into<String>>(source: S, title: S) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            source: source.into(),
            title: title.into(),
            url: None,
            summary: None,
            relevance_score: None,
            indexed_at: now_timestamp(),
        }
    }
}
