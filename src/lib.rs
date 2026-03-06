//! # rank-wav
//!
//! A library for analyzing WAV files and ranking them by acoustic features.
//!
//! This crate provides functionality to:
//! - Read WAV files in various formats (8/16/24/32-bit int, 32-bit float)
//! - Extract acoustic features (RMS, ZCR, spectral centroid, bandwidth)
//! - Optional extended metrics (rolloff, flatness, crest factor)
//! - Score and rank files by perceived quality
//! - Configurable metrics and scoring weights via TOML
//!
//! ## Example
//!
//! ```no_run
//! use rank_wav_rs::{config::Config, scan, score, features::FeatureRow};
//! use std::path::Path;
//!
//! // Load config (or use defaults)
//! let config = Config::default();
//!
//! // Scan a directory for WAV files
//! let mut rows = scan::scan_dir(Path::new("./samples"), false, &config).unwrap();
//!
//! // Normalize and score
//! score::normalize_rows(&mut rows);
//! score::compute_scores(&mut rows, &config);
//!
//! // Sort by pleasing score (highest first)
//! rows.sort_by(|a, b| b.pleasing_score.total_cmp(&a.pleasing_score));
//!
//! for row in &rows {
//!     println!("{}: {:.3}", row.path.display(), row.pleasing_score);
//! }
//! ```
//!
//! ## Extended Metrics
//!
//! Use `Config::default().with_extended(true)` to enable extended metrics:
//! - Spectral rolloff: frequency below which 85% of energy lies
//! - Spectral flatness: 0 = tonal, 1 = noisy
//! - Crest factor: peak to RMS ratio in dB
//!
//! ## Configuration
//!
//! Configuration can be loaded from a TOML file or customized programmatically:
//!
//! ```no_run
//! use rank_wav_rs::config::Config;
//! use std::path::Path;
//!
//! // Load from file (missing file uses defaults)
//! let config = Config::load(Path::new("config.toml")).unwrap();
//!
//! // Enable extended metrics via flag
//! let config = config.with_extended(true);
//! ```

pub mod cli;
pub mod config;
pub mod features;
pub mod output;
pub mod scan;
pub mod score;
pub mod wav;
