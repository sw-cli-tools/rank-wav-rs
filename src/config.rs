//! Configuration file handling.
//!
//! This module provides functionality to load and merge configuration
//! from TOML files. Configuration controls which metrics are computed
//! and the weights used for scoring.

use anyhow::{Result, bail};
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Main configuration structure.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Metrics configuration.
    pub metrics: MetricsConfig,
    /// Scoring weights configuration.
    pub scoring: ScoringConfig,
}

/// Configuration for which metrics to compute.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct MetricsConfig {
    /// Basic metrics (always available).
    pub basic: BasicMetrics,
    /// Extended metrics (require --extended or config).
    pub extended: ExtendedMetrics,
}

/// Basic metric toggles.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct BasicMetrics {
    /// Compute RMS energy.
    pub rms: bool,
    /// Compute zero-crossing rate.
    pub zcr: bool,
    /// Compute spectral centroid.
    pub spectral_centroid: bool,
    /// Compute spectral bandwidth.
    pub spectral_bandwidth: bool,
}

/// Extended metric toggles.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ExtendedMetrics {
    /// Compute spectral rolloff.
    pub spectral_rolloff: bool,
    /// Compute spectral flatness.
    pub spectral_flatness: bool,
    /// Compute crest factor.
    pub crest_factor: bool,
}

/// Scoring weights configuration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ScoringConfig {
    /// Weights for "pleasing" score.
    pub pleasing: PleasingWeights,
    /// Weights for "best" score.
    pub best: BestWeights,
}

/// Weights for the "pleasing" score formula.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct PleasingWeights {
    pub centroid_weight: f32,
    pub bandwidth_weight: f32,
    pub zcr_weight: f32,
    pub rms_weight: f32,
}

/// Weights for the "best" score formula.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct BestWeights {
    pub rms_weight: f32,
    pub centroid_target: f32,
    pub centroid_weight: f32,
    pub bandwidth_target: f32,
    pub bandwidth_weight: f32,
    pub zcr_weight: f32,
}

// Default implementations

impl Default for BasicMetrics {
    fn default() -> Self {
        Self {
            rms: true,
            zcr: true,
            spectral_centroid: true,
            spectral_bandwidth: true,
        }
    }
}

impl Default for PleasingWeights {
    fn default() -> Self {
        Self {
            centroid_weight: -0.40,
            bandwidth_weight: -0.30,
            zcr_weight: -0.20,
            rms_weight: 0.10,
        }
    }
}

impl Default for BestWeights {
    fn default() -> Self {
        Self {
            rms_weight: 0.35,
            centroid_target: 0.45,
            centroid_weight: -0.25,
            bandwidth_target: 0.40,
            bandwidth_weight: -0.20,
            zcr_weight: -0.20,
        }
    }
}

impl Config {
    /// Load configuration from a TOML file.
    ///
    /// If the file doesn't exist or is empty, returns default config.
    /// If the file exists but has parse errors, returns an error.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)?;

        // Empty file = use defaults
        if content.trim().is_empty() {
            return Ok(Self::default());
        }

        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Apply --extended flag to enable all extended metrics.
    pub fn with_extended(mut self, extended: bool) -> Self {
        if extended {
            self.metrics.extended.spectral_rolloff = true;
            self.metrics.extended.spectral_flatness = true;
            self.metrics.extended.crest_factor = true;
        }
        self
    }

    /// Check if at least one metric is enabled.
    pub fn validate(&self) -> Result<()> {
        let basic = &self.metrics.basic;
        let extended = &self.metrics.extended;

        let any_enabled = basic.rms
            || basic.zcr
            || basic.spectral_centroid
            || basic.spectral_bandwidth
            || extended.spectral_rolloff
            || extended.spectral_flatness
            || extended.crest_factor;

        if !any_enabled {
            bail!("At least one metric must be enabled");
        }

        Ok(())
    }

    /// Check if any extended metrics are enabled.
    pub fn has_extended(&self) -> bool {
        let ext = &self.metrics.extended;
        ext.spectral_rolloff || ext.spectral_flatness || ext.crest_factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.metrics.basic.rms);
        assert!(config.metrics.basic.zcr);
        assert!(!config.metrics.extended.spectral_rolloff);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_load_missing_file() {
        let config = Config::load(Path::new("nonexistent.toml")).unwrap();
        assert!(config.metrics.basic.rms);
    }

    #[test]
    fn test_load_empty_file() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "").unwrap();

        let config = Config::load(file.path()).unwrap();
        assert!(config.metrics.basic.rms);
    }

    #[test]
    fn test_load_partial_config() {
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            r#"
[metrics.basic]
rms = false
"#
        )
        .unwrap();

        let config = Config::load(file.path()).unwrap();
        assert!(!config.metrics.basic.rms);
        assert!(config.metrics.basic.zcr); // default
    }

    #[test]
    fn test_load_custom_weights() {
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            r#"
[scoring.pleasing]
centroid_weight = -0.50
"#
        )
        .unwrap();

        let config = Config::load(file.path()).unwrap();
        assert!((config.scoring.pleasing.centroid_weight - (-0.50)).abs() < 0.001);
        assert!((config.scoring.pleasing.rms_weight - 0.10).abs() < 0.001); // default
    }

    #[test]
    fn test_with_extended() {
        let config = Config::default().with_extended(true);
        assert!(config.metrics.extended.spectral_rolloff);
        assert!(config.metrics.extended.spectral_flatness);
        assert!(config.metrics.extended.crest_factor);
    }

    #[test]
    fn test_validate_all_disabled() {
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            r#"
[metrics.basic]
rms = false
zcr = false
spectral_centroid = false
spectral_bandwidth = false

[metrics.extended]
spectral_rolloff = false
spectral_flatness = false
crest_factor = false
"#
        )
        .unwrap();

        let config = Config::load(file.path()).unwrap();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_has_extended() {
        let config = Config::default();
        assert!(!config.has_extended());

        let config = config.with_extended(true);
        assert!(config.has_extended());
    }
}
