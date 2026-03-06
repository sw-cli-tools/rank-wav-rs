//! Normalization and scoring of audio features.
//!
//! This module provides functionality to normalize raw features across
//! a batch and compute "pleasing" and "best" ranking scores.

use crate::config::Config;
use crate::features::FeatureRow;

/// Normalize all features in the batch using min-max scaling.
///
/// After normalization, each feature will be in the range [0.0, 1.0]
/// relative to the minimum and maximum values in the batch.
pub fn normalize_rows(rows: &mut [FeatureRow]) {
    if rows.is_empty() {
        return;
    }

    normalize_field(rows, |r| r.rms, |r, v| r.rms_norm = v);
    normalize_field(rows, |r| r.zcr, |r, v| r.zcr_norm = v);
    normalize_field(rows, |r| r.spectral_centroid, |r, v| r.centroid_norm = v);
    normalize_field(rows, |r| r.spectral_bandwidth, |r, v| r.bandwidth_norm = v);

    // Normalize extended metrics if present
    let has_extended = rows.iter().any(|r| r.spectral_rolloff.is_some());
    if has_extended {
        normalize_optional_field(
            rows,
            |r| r.spectral_rolloff,
            |r, v| r.rolloff_norm = Some(v),
        );
        normalize_optional_field(
            rows,
            |r| r.spectral_flatness,
            |r, v| r.flatness_norm = Some(v),
        );
        normalize_optional_field(rows, |r| r.crest_factor, |r, v| r.crest_norm = Some(v));
    }
}

/// Normalize a single field across all rows using min-max scaling.
fn normalize_field(
    rows: &mut [FeatureRow],
    get: impl Fn(&FeatureRow) -> f32,
    mut set: impl FnMut(&mut FeatureRow, f32),
) {
    let min = rows.iter().map(&get).fold(f32::INFINITY, f32::min);
    let max = rows.iter().map(&get).fold(f32::NEG_INFINITY, f32::max);

    let span = (max - min).max(1e-9);
    for row in rows {
        let v = (get(row) - min) / span;
        set(row, v);
    }
}

/// Normalize an optional field across all rows using min-max scaling.
fn normalize_optional_field(
    rows: &mut [FeatureRow],
    get: impl Fn(&FeatureRow) -> Option<f32>,
    mut set: impl FnMut(&mut FeatureRow, f32),
) {
    let values: Vec<f32> = rows.iter().filter_map(&get).collect();
    if values.is_empty() {
        return;
    }

    let min = values.iter().copied().fold(f32::INFINITY, f32::min);
    let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);

    let span = (max - min).max(1e-9);
    for row in rows {
        if let Some(val) = get(row) {
            let v = (val - min) / span;
            set(row, v);
        }
    }
}

/// Compute pleasing and best scores for all rows.
///
/// Must be called after `normalize_rows()` to ensure normalized values
/// are populated.
///
/// ## Scoring Formulas
///
/// **Pleasing Score** (warm, smooth, low harshness):
/// - Penalizes high brightness (centroid)
/// - Penalizes high complexity (bandwidth)
/// - Penalizes high noisiness (ZCR)
/// - Slightly rewards strong signal (RMS)
///
/// **Best Score** (balanced, present, clear):
/// - Rewards strong signal (RMS)
/// - Penalizes deviation from moderate brightness
/// - Penalizes deviation from moderate complexity
/// - Penalizes high noisiness
pub fn compute_scores(rows: &mut [FeatureRow], config: &Config) {
    let p = &config.scoring.pleasing;
    let b = &config.scoring.best;

    for row in rows {
        row.pleasing_score = p.centroid_weight * row.centroid_norm
            + p.bandwidth_weight * row.bandwidth_norm
            + p.zcr_weight * row.zcr_norm
            + p.rms_weight * row.rms_norm;

        row.best_score = b.rms_weight * row.rms_norm
            + b.centroid_weight * dist(row.centroid_norm, b.centroid_target)
            + b.bandwidth_weight * dist(row.bandwidth_norm, b.bandwidth_target)
            + b.zcr_weight * row.zcr_norm;
    }
}

/// Absolute distance from a target value.
fn dist(x: f32, target: f32) -> f32 {
    (x - target).abs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_row(rms: f32, zcr: f32, centroid: f32, bandwidth: f32) -> FeatureRow {
        FeatureRow {
            path: PathBuf::from("test.wav"),
            sample_rate: 44100,
            num_samples: 1000,
            rms,
            zcr,
            spectral_centroid: centroid,
            spectral_bandwidth: bandwidth,
            spectral_rolloff: None,
            spectral_flatness: None,
            crest_factor: None,
            rms_norm: 0.0,
            zcr_norm: 0.0,
            centroid_norm: 0.0,
            bandwidth_norm: 0.0,
            rolloff_norm: None,
            flatness_norm: None,
            crest_norm: None,
            pleasing_score: 0.0,
            best_score: 0.0,
        }
    }

    #[test]
    fn test_normalize_single_row() {
        let mut rows = vec![make_row(0.5, 0.1, 1000.0, 500.0)];
        normalize_rows(&mut rows);

        // Single row: all normalized values should be 0.0 (or close to it)
        // because min == max, so (val - min) / span ≈ 0
        assert!(rows[0].rms_norm >= 0.0 && rows[0].rms_norm <= 1.0);
    }

    #[test]
    fn test_normalize_two_rows() {
        let mut rows = vec![
            make_row(0.1, 0.0, 500.0, 200.0),
            make_row(0.5, 0.2, 2000.0, 800.0),
        ];
        normalize_rows(&mut rows);

        // First row should have lower values (closer to 0)
        assert!((rows[0].rms_norm - 0.0).abs() < 0.01);
        assert!((rows[1].rms_norm - 1.0).abs() < 0.01);

        assert!((rows[0].centroid_norm - 0.0).abs() < 0.01);
        assert!((rows[1].centroid_norm - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_pleasing_score_prefers_dark_smooth() {
        let mut rows = vec![
            make_row(0.3, 0.02, 800.0, 400.0),   // dark, smooth
            make_row(0.3, 0.10, 2500.0, 2000.0), // bright, harsh
        ];
        let config = Config::default();
        normalize_rows(&mut rows);
        compute_scores(&mut rows, &config);

        // Dark/smooth should have higher pleasing score
        assert!(
            rows[0].pleasing_score > rows[1].pleasing_score,
            "dark/smooth ({}) should beat bright/harsh ({})",
            rows[0].pleasing_score,
            rows[1].pleasing_score
        );
    }

    #[test]
    fn test_best_score_prefers_strong_balanced() {
        let mut rows = vec![
            make_row(0.6, 0.03, 1500.0, 800.0), // strong, balanced
            make_row(0.1, 0.03, 1500.0, 800.0), // weak, balanced
        ];
        let config = Config::default();
        normalize_rows(&mut rows);
        compute_scores(&mut rows, &config);

        // Strong signal should have higher best score
        assert!(
            rows[0].best_score > rows[1].best_score,
            "strong ({}) should beat weak ({})",
            rows[0].best_score,
            rows[1].best_score
        );
    }

    #[test]
    fn test_dist() {
        assert!((dist(0.5, 0.5) - 0.0).abs() < 0.001);
        assert!((dist(0.0, 0.5) - 0.5).abs() < 0.001);
        assert!((dist(1.0, 0.5) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_custom_weights() {
        use crate::config::{BestWeights, PleasingWeights, ScoringConfig};

        let mut rows = vec![make_row(0.5, 0.1, 1000.0, 500.0)];
        normalize_rows(&mut rows);

        // Custom config with different weights
        let mut config = Config::default();
        config.scoring = ScoringConfig {
            pleasing: PleasingWeights {
                centroid_weight: 0.0,
                bandwidth_weight: 0.0,
                zcr_weight: 0.0,
                rms_weight: 1.0, // Only RMS matters
            },
            best: BestWeights {
                rms_weight: 1.0,
                centroid_target: 0.5,
                centroid_weight: 0.0,
                bandwidth_target: 0.5,
                bandwidth_weight: 0.0,
                zcr_weight: 0.0,
            },
        };
        compute_scores(&mut rows, &config);

        // With only RMS weight, scores should equal rms_norm
        assert!((rows[0].pleasing_score - rows[0].rms_norm).abs() < 0.001);
        assert!((rows[0].best_score - rows[0].rms_norm).abs() < 0.001);
    }
}
