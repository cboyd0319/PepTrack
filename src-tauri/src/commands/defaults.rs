use peptrack_core::StorageManager;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::info;

use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultProtocol {
    pub peptide_name: String,
    pub common_name: String,
    pub typical_dose_range: String,
    pub notes: String,
}

/// Get list of popular peptides for pre-population
#[tauri::command]
pub async fn get_default_peptides() -> Result<Vec<DefaultProtocol>, String> {
    Ok(get_popular_peptides())
}

/// Populate database with popular peptide protocols
#[tauri::command]
pub async fn populate_default_peptides(
    state: State<'_, std::sync::Arc<AppState>>,
) -> Result<usize, String> {
    info!("Populating default peptides");

    let peptides = get_popular_peptides();
    let mut created_count = 0;

    for peptide in peptides {
        // Check if this peptide already exists (by peptide_name)
        let existing = state
            .storage
            .list_protocols()
            .map_err(|e| format!("Failed to check existing protocols: {}", e))?
            .into_iter()
            .any(|p| p.peptide_name == peptide.peptide_name);

        if existing {
            continue; // Skip if already exists
        }

        // Create protocol
        let protocol = peptrack_core::models::PeptideProtocol {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("{} Protocol", peptide.common_name),
            peptide_name: peptide.peptide_name,
            notes: Some(format!(
                "{}\n\nTypical dose range: {}",
                peptide.notes, peptide.typical_dose_range
            )),
            target_concentration_mg_ml: None,
            created_at: time::OffsetDateTime::now_utc(),
            updated_at: time::OffsetDateTime::now_utc(),
        };

        state
            .storage
            .upsert_protocol(&protocol)
            .map_err(|e| format!("Failed to create protocol: {}", e))?;

        created_count += 1;
    }

    info!("Created {} default peptide protocols", created_count);
    Ok(created_count)
}

fn get_popular_peptides() -> Vec<DefaultProtocol> {
    vec![
        DefaultProtocol {
            peptide_name: "BPC-157".to_string(),
            common_name: "Body Protection Compound-157".to_string(),
            typical_dose_range: "200-500 mcg/day".to_string(),
            notes: "Known for tissue repair and gut health. Commonly injected subcutaneously or taken orally.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "GHK-Cu".to_string(),
            common_name: "Copper Peptide (GHK-Cu)".to_string(),
            typical_dose_range: "0.5-2 mg/day".to_string(),
            notes: "Supports skin health, wound healing, and anti-aging. Often used topically or injected.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "Tesamorelin".to_string(),
            common_name: "Tesamorelin (GHRH)".to_string(),
            typical_dose_range: "1-2 mg/day".to_string(),
            notes: "FDA-approved for reducing abdominal fat. Growth hormone releasing hormone analog.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "MOTS-c".to_string(),
            common_name: "MOTS-c".to_string(),
            typical_dose_range: "5-15 mg/week".to_string(),
            notes: "Mitochondrial peptide supporting metabolism and exercise capacity.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "CJC-1295".to_string(),
            common_name: "CJC-1295 (GHRH analog)".to_string(),
            typical_dose_range: "1-2 mg/week (without DAC)".to_string(),
            notes: "Growth hormone releasing hormone analog. Often combined with Ipamorelin.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "DSIP".to_string(),
            common_name: "Delta Sleep-Inducing Peptide".to_string(),
            typical_dose_range: "100-300 mcg before bed".to_string(),
            notes: "May support sleep quality and stress reduction.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "Ipamorelin".to_string(),
            common_name: "Ipamorelin (GHRP)".to_string(),
            typical_dose_range: "200-300 mcg, 2-3x/day".to_string(),
            notes: "Growth hormone secretagogue. Minimal effect on cortisol/prolactin.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "Retatrutide".to_string(),
            common_name: "Retatrutide (Triple Agonist)".to_string(),
            typical_dose_range: "1-12 mg/week (titrate)".to_string(),
            notes: "Triple agonist (GLP-1/GIP/glucagon) for weight management. Clinical trial phase.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "Sermorelin".to_string(),
            common_name: "Sermorelin (GHRH)".to_string(),
            typical_dose_range: "200-500 mcg before bed".to_string(),
            notes: "Growth hormone releasing hormone. Shorter half-life than CJC-1295.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "Kisspeptin-10".to_string(),
            common_name: "Kisspeptin-10".to_string(),
            typical_dose_range: "1-5 mcg/kg".to_string(),
            notes: "Reproductive hormone regulation. Research phase for fertility support.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "Gonadorelin".to_string(),
            common_name: "Gonadorelin (GnRH)".to_string(),
            typical_dose_range: "100-200 mcg/injection".to_string(),
            notes: "Gonadotropin-releasing hormone. Supports testosterone production.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "GHRP-6".to_string(),
            common_name: "Growth Hormone Releasing Peptide-6".to_string(),
            typical_dose_range: "100-200 mcg, 2-3x/day".to_string(),
            notes: "Potent GH secretagogue. May increase appetite.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "GHRP-2".to_string(),
            common_name: "Growth Hormone Releasing Peptide-2".to_string(),
            typical_dose_range: "100-200 mcg, 2-3x/day".to_string(),
            notes: "Similar to GHRP-6 but less appetite stimulation.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "MK-677".to_string(),
            common_name: "Ibutamoren (MK-677)".to_string(),
            typical_dose_range: "10-25 mg/day (oral)".to_string(),
            notes: "Oral GH secretagogue. Not technically a peptide but commonly grouped.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "AOD-9604".to_string(),
            common_name: "AOD-9604 (Fragment 176-191)".to_string(),
            typical_dose_range: "300-600 mcg/day".to_string(),
            notes: "GH fragment targeting fat metabolism without GH's other effects.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "Semaglutide".to_string(),
            common_name: "Semaglutide (GLP-1 agonist)".to_string(),
            typical_dose_range: "0.25-2.4 mg/week (titrate)".to_string(),
            notes: "FDA-approved for weight management and diabetes. Weekly injection.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "Tirzepatide".to_string(),
            common_name: "Tirzepatide (GIP/GLP-1 dual agonist)".to_string(),
            typical_dose_range: "2.5-15 mg/week (titrate)".to_string(),
            notes: "FDA-approved dual agonist for weight loss and diabetes management.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "SLU-PP-332".to_string(),
            common_name: "SLU-PP-332 (Exercise Mimetic)".to_string(),
            typical_dose_range: "Research phase - no established dose".to_string(),
            notes: "Novel exercise mimetic peptide. Currently in early research phase.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "PT-141".to_string(),
            common_name: "Bremelanotide (PT-141)".to_string(),
            typical_dose_range: "1.75 mg as needed".to_string(),
            notes: "FDA-approved for hypoactive sexual desire disorder. Melanocortin receptor agonist.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "TB-500".to_string(),
            common_name: "Thymosin Beta-4 Fragment (TB-500)".to_string(),
            typical_dose_range: "2-10 mg/week".to_string(),
            notes: "Promotes healing and tissue repair. Often used for injury recovery.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "Epithalon".to_string(),
            common_name: "Epitalon (Epithalon)".to_string(),
            typical_dose_range: "5-10 mg/day for 10-20 days".to_string(),
            notes: "Telomerase activator. Used in longevity protocols.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "NAD+".to_string(),
            common_name: "NAD+ (Nicotinamide Adenine Dinucleotide)".to_string(),
            typical_dose_range: "50-500 mg IV or SubQ".to_string(),
            notes: "Cellular energy and metabolism support. Various administration methods.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "Semax".to_string(),
            common_name: "Semax".to_string(),
            typical_dose_range: "300-600 mcg/day (nasal or SubQ)".to_string(),
            notes: "Neuroprotective and cognitive enhancing peptide. Russian nootropic.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "Selank".to_string(),
            common_name: "Selank".to_string(),
            typical_dose_range: "250-500 mcg/day (nasal or SubQ)".to_string(),
            notes: "Anxiolytic and cognitive peptide. Related to tuftsin.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "KPV".to_string(),
            common_name: "KPV (Lys-Pro-Val)".to_string(),
            typical_dose_range: "250-500 mcg/day (oral or topical)".to_string(),
            notes: "Anti-inflammatory tripeptide. Supports gut and skin health.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "Oxytocin".to_string(),
            common_name: "Oxytocin".to_string(),
            typical_dose_range: "10-40 IU nasal as needed".to_string(),
            notes: "Social bonding and trust hormone. Various wellness applications.".to_string(),
        },
        DefaultProtocol {
            peptide_name: "Melanotan II".to_string(),
            common_name: "Melanotan II (MT-II)".to_string(),
            typical_dose_range: "250-500 mcg/day".to_string(),
            notes: "Melanocortin receptor agonist. Tanning and libido effects.".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_peptides_count() {
        let peptides = get_popular_peptides();
        assert_eq!(peptides.len(), 27, "Should have exactly 27 popular peptides");
    }

    #[test]
    fn test_peptide_names_unique() {
        let peptides = get_popular_peptides();
        let mut names: Vec<String> = peptides.iter().map(|p| p.peptide_name.clone()).collect();
        let original_len = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), original_len, "All peptide names should be unique");
    }

    #[test]
    fn test_all_peptides_have_data() {
        let peptides = get_popular_peptides();
        for peptide in peptides {
            assert!(!peptide.peptide_name.is_empty(), "Peptide name should not be empty");
            assert!(!peptide.common_name.is_empty(), "Common name should not be empty");
            assert!(!peptide.typical_dose_range.is_empty(), "Dose range should not be empty");
            assert!(!peptide.notes.is_empty(), "Notes should not be empty");
        }
    }
}
