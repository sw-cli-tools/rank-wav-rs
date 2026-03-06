//! Output formatting for ranked results.
//!
//! This module provides functionality to format feature rows as
//! terminal tables or JSON.

use crate::features::FeatureRow;
use anyhow::Result;
use tabled::{Table, Tabled};

/// Row for terminal table display (basic metrics).
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

/// Row for terminal table display (extended metrics).
#[derive(Tabled)]
struct PrintRowExtended {
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
    #[tabled(rename = "Rolloff")]
    rolloff: String,
    #[tabled(rename = "Flatness")]
    flatness: String,
    #[tabled(rename = "Crest")]
    crest: String,
    #[tabled(rename = "Pleasing")]
    pleasing: String,
    #[tabled(rename = "Best")]
    best: String,
}

/// Truncate filename to max length with ellipsis.
fn truncate_filename(filename: &str, max_len: usize) -> String {
    if filename.len() > max_len {
        format!("{}...", &filename[..max_len - 3])
    } else {
        filename.to_string()
    }
}

/// Print rows as a formatted table or JSON.
///
/// # Arguments
///
/// * `rows` - Feature rows to print (should be pre-sorted)
/// * `json` - If true, output as JSON; otherwise output as table
/// * `extended` - If true, include extended metrics in table output
pub fn print_rows(rows: &[FeatureRow], json: bool, extended: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(rows)?);
        return Ok(());
    }

    if rows.is_empty() {
        println!("No WAV files found.");
        return Ok(());
    }

    if extended {
        print_extended_table(rows);
    } else {
        print_basic_table(rows);
    }

    Ok(())
}

/// Print basic table (without extended metrics).
fn print_basic_table(rows: &[FeatureRow]) {
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

            PrintRow {
                rank: i + 1,
                file: truncate_filename(&filename, 30),
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
}

/// Print extended table (with extended metrics).
fn print_extended_table(rows: &[FeatureRow]) {
    let print_rows: Vec<PrintRowExtended> = rows
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let filename = r
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            PrintRowExtended {
                rank: i + 1,
                file: truncate_filename(&filename, 24),
                rms: format!("{:.3}", r.rms),
                zcr: format!("{:.3}", r.zcr),
                centroid: format!("{:.0}", r.spectral_centroid),
                bandwidth: format!("{:.0}", r.spectral_bandwidth),
                rolloff: r
                    .spectral_rolloff
                    .map(|v| format!("{:.0}", v))
                    .unwrap_or_else(|| "-".to_string()),
                flatness: r
                    .spectral_flatness
                    .map(|v| format!("{:.3}", v))
                    .unwrap_or_else(|| "-".to_string()),
                crest: r
                    .crest_factor
                    .map(|v| format!("{:.1}", v))
                    .unwrap_or_else(|| "-".to_string()),
                pleasing: format!("{:.3}", r.pleasing_score),
                best: format!("{:.3}", r.best_score),
            }
        })
        .collect();

    println!("{}", Table::new(print_rows));
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
            spectral_rolloff: None,
            spectral_flatness: None,
            crest_factor: None,
            rms_norm: 0.5,
            zcr_norm: 0.5,
            centroid_norm: 0.5,
            bandwidth_norm: 0.5,
            rolloff_norm: None,
            flatness_norm: None,
            crest_norm: None,
            pleasing_score: pleasing,
            best_score: best,
        }
    }

    fn make_extended_row(name: &str, pleasing: f32, best: f32) -> FeatureRow {
        FeatureRow {
            path: PathBuf::from(name),
            sample_rate: 44100,
            num_samples: 1000,
            rms: 0.5,
            zcr: 0.05,
            spectral_centroid: 1500.0,
            spectral_bandwidth: 800.0,
            spectral_rolloff: Some(3000.0),
            spectral_flatness: Some(0.2),
            crest_factor: Some(3.5),
            rms_norm: 0.5,
            zcr_norm: 0.5,
            centroid_norm: 0.5,
            bandwidth_norm: 0.5,
            rolloff_norm: Some(0.5),
            flatness_norm: Some(0.5),
            crest_norm: Some(0.5),
            pleasing_score: pleasing,
            best_score: best,
        }
    }

    #[test]
    fn test_print_empty() {
        // Should not panic
        let rows: Vec<FeatureRow> = vec![];
        print_rows(&rows, false, false).unwrap();
    }

    #[test]
    fn test_print_json() {
        let rows = vec![make_row("test.wav", 0.5, 0.6)];
        // Should not panic
        print_rows(&rows, true, false).unwrap();
    }

    #[test]
    fn test_truncate_long_filename() {
        let rows = vec![make_row(
            "this-is-a-very-long-filename-that-should-be-truncated.wav",
            0.5,
            0.6,
        )];
        // Should not panic
        print_rows(&rows, false, false).unwrap();
    }

    #[test]
    fn test_print_extended() {
        let rows = vec![make_extended_row("test.wav", 0.5, 0.6)];
        // Should not panic
        print_rows(&rows, false, true).unwrap();
    }

    #[test]
    fn test_print_extended_json() {
        let rows = vec![make_extended_row("test.wav", 0.5, 0.6)];
        // Should not panic
        print_rows(&rows, true, true).unwrap();
    }
}
