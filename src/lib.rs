//! # rank-wav
//!
//! A library for analyzing WAV files and ranking them by acoustic features.
//!
//! This crate provides functionality to:
//! - Read WAV files in various formats (8/16/24/32-bit int, 32-bit float)
//! - Extract acoustic features (RMS, ZCR, spectral centroid, bandwidth)
//! - Optional extended metrics (rolloff, flatness, crest factor)
//! - Score and rank files by perceived quality
//!
//! ## Example
//!
//! ```no_run
//! use rank_wav_rs::{scan, score, features::FeatureRow};
//! use std::path::Path;
//!
//! // Scan a directory for WAV files (without extended metrics)
//! let mut rows = scan::scan_dir(Path::new("./samples"), false, false).unwrap();
//!
//! // Normalize and score
//! score::normalize_rows(&mut rows);
//! score::compute_scores(&mut rows);
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
//! Pass `true` as the third argument to `scan_dir` to enable extended metrics:
//! - Spectral rolloff: frequency below which 85% of energy lies
//! - Spectral flatness: 0 = tonal, 1 = noisy
//! - Crest factor: peak to RMS ratio in dB

pub mod cli;
pub mod features;
pub mod output;
pub mod scan;
pub mod score;
pub mod wav;
