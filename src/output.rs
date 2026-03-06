//! Output formatting for ranked results.
//!
//! This module provides functionality to format feature rows as
//! terminal tables or JSON.

use crate::features::FeatureRow;
use anyhow::Result;
use tabled::{Table, Tabled};

/// Row for terminal table display.
#[derive(Tabled)]
struct PrintRow {
    #[tabled(rename = "#")]
    rank: usize,
    #[tabled(rename = "File")]
    file: String,
    #[tabled(rename = "RMS")]
    rms: String,
    #[tabled(rename = "ZCR")]
    zcr: String,
    #[tabled(rename = "Centroid")]
    centroid: String,
    #[tabled(rename = "Bandwidth")]
    bandwidth: String,
    #[tabled(rename = "Pleasing")]
    pleasing: String,
    #[tabled(rename = "Best")]
    best: String,
}

/// Print rows as a formatted table or JSON.
///
/// # Arguments
///
/// * `rows` - Feature rows to print (should be pre-sorted)
/// * `json` - If true, output as JSON; otherwise output as table
pub fn print_rows(rows: &[FeatureRow], json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(rows)?);
        return Ok(());
    }

    if rows.is_empty() {
        println!("No WAV files found.");
        return Ok(());
    }

    let print_rows: Vec<PrintRow> = rows
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let filename = r
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            // Truncate long filenames
            let file = if filename.len() > 30 {
                format!("{}...", &filename[..27])
            } else {
                filename
            };

            PrintRow {
                rank: i + 1,
                file,
                rms: format!("{:.4}", r.rms),
                zcr: format!("{:.4}", r.zcr),
                centroid: format!("{:.0}", r.spectral_centroid),
                bandwidth: format!("{:.0}", r.spectral_bandwidth),
                pleasing: format!("{:.3}", r.pleasing_score),
                best: format!("{:.3}", r.best_score),
            }
        })
        .collect();

    println!("{}", Table::new(print_rows));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_row(name: &str, pleasing: f32, best: f32) -> FeatureRow {
        FeatureRow {
            path: PathBuf::from(name),
            sample_rate: 44100,
            num_samples: 1000,
            rms: 0.5,
            zcr: 0.05,
            spectral_centroid: 1500.0,
            spectral_bandwidth: 800.0,
            rms_norm: 0.5,
            zcr_norm: 0.5,
            centroid_norm: 0.5,
            bandwidth_norm: 0.5,
            pleasing_score: pleasing,
            best_score: best,
        }
    }

    #[test]
    fn test_print_empty() {
        // Should not panic
        let rows: Vec<FeatureRow> = vec![];
        print_rows(&rows, false).unwrap();
    }

    #[test]
    fn test_print_json() {
        let rows = vec![make_row("test.wav", 0.5, 0.6)];
        // Should not panic
        print_rows(&rows, true).unwrap();
    }

    #[test]
    fn test_truncate_long_filename() {
        let rows = vec![make_row(
            "this-is-a-very-long-filename-that-should-be-truncated.wav",
            0.5,
            0.6,
        )];
        // Should not panic
        print_rows(&rows, false).unwrap();
    }
}
