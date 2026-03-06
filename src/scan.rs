//! Directory scanning for WAV files.
//!
//! This module provides functionality to traverse directories and find
//! WAV files, extracting acoustic features from each.

use crate::features::{FeatureRow, compute_features};
use crate::wav::read_wav_mono_f32;
use anyhow::Result;
use std::path::Path;
use walkdir::WalkDir;

/// Scan a directory for WAV files and compute features for each.
///
/// # Arguments
///
/// * `dir` - Directory to scan
/// * `recursive` - If true, recurse into subdirectories
///
/// # Returns
///
/// A vector of `FeatureRow` for each successfully processed WAV file.
/// Files that cannot be read or processed are skipped with a warning.
pub fn scan_dir(dir: &Path, recursive: bool) -> Result<Vec<FeatureRow>> {
    let max_depth = if recursive { usize::MAX } else { 1 };
    let mut rows = Vec::new();

    for entry in WalkDir::new(dir).max_depth(max_depth) {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                eprintln!("Warning: {err}");
                continue;
            }
        };

        let path = entry.path();

        if !entry.file_type().is_file() {
            continue;
        }

        // Check for .wav extension (case-insensitive)
        let is_wav = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.eq_ignore_ascii_case("wav"))
            == Some(true);

        if !is_wav {
            continue;
        }

        match process_wav_file(path) {
            Ok(row) => rows.push(row),
            Err(err) => {
                eprintln!("Skipping {}: {err}", path.display());
            }
        }
    }

    Ok(rows)
}

/// Process a single WAV file and return its features.
fn process_wav_file(path: &Path) -> Result<FeatureRow> {
    let (samples, sample_rate) = read_wav_mono_f32(path)?;
    let mut row = compute_features(path, &samples, sample_rate)?;
    row.sample_rate = sample_rate;
    row.num_samples = samples.len();
    Ok(row)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Cursor;

    fn create_test_wav() -> Vec<u8> {
        let mut buffer = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer);
            let spec = hound::WavSpec {
                channels: 1,
                sample_rate: 44100,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };
            let mut writer = hound::WavWriter::new(cursor, spec).unwrap();
            // Write enough samples for analysis
            for i in 0..1024 {
                let sample = ((i as f32 * 0.1).sin() * 16384.0) as i16;
                writer.write_sample(sample).unwrap();
            }
            writer.finalize().unwrap();
        }
        buffer
    }

    #[test]
    fn test_scan_empty_dir() {
        let temp = tempfile::tempdir().unwrap();
        let rows = scan_dir(temp.path(), false).unwrap();
        assert!(rows.is_empty());
    }

    #[test]
    fn test_scan_with_wav_files() {
        let temp = tempfile::tempdir().unwrap();

        // Create test WAV files
        let wav_data = create_test_wav();
        fs::write(temp.path().join("test1.wav"), &wav_data).unwrap();
        fs::write(temp.path().join("test2.WAV"), &wav_data).unwrap(); // uppercase

        // Create a non-WAV file
        fs::write(temp.path().join("ignore.txt"), "not a wav").unwrap();

        let rows = scan_dir(temp.path(), false).unwrap();
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn test_scan_recursive() {
        let temp = tempfile::tempdir().unwrap();
        let subdir = temp.path().join("subdir");
        fs::create_dir(&subdir).unwrap();

        let wav_data = create_test_wav();
        fs::write(temp.path().join("root.wav"), &wav_data).unwrap();
        fs::write(subdir.join("nested.wav"), &wav_data).unwrap();

        // Non-recursive should find only root
        let rows = scan_dir(temp.path(), false).unwrap();
        assert_eq!(rows.len(), 1);

        // Recursive should find both
        let rows = scan_dir(temp.path(), true).unwrap();
        assert_eq!(rows.len(), 2);
    }
}
