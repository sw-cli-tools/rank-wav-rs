//! Integration tests for rank-wav.
//!
//! These tests verify the full pipeline from WAV file to ranked output.

use hound::{SampleFormat, WavSpec, WavWriter};
use rank_wav_rs::{scan, score};
use std::f32::consts::PI;
use std::fs;
use std::io::Cursor;
use tempfile::tempdir;

/// Create a WAV file with a sine wave at the given frequency.
fn create_sine_wav(freq: f32, amplitude: f32, duration_samples: usize) -> Vec<u8> {
    let sample_rate = 44100;
    let mut buffer = Vec::new();
    {
        let cursor = Cursor::new(&mut buffer);
        let spec = WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        let mut writer = WavWriter::new(cursor, spec).unwrap();

        for i in 0..duration_samples {
            let t = i as f32 / sample_rate as f32;
            let sample = (amplitude * (2.0 * PI * freq * t).sin() * 32767.0) as i16;
            writer.write_sample(sample).unwrap();
        }
        writer.finalize().unwrap();
    }
    buffer
}

/// Create a WAV file with white noise.
fn create_noise_wav(amplitude: f32, duration_samples: usize) -> Vec<u8> {
    let sample_rate = 44100;
    let mut buffer = Vec::new();
    {
        let cursor = Cursor::new(&mut buffer);
        let spec = WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        let mut writer = WavWriter::new(cursor, spec).unwrap();

        // Simple pseudo-random noise using linear congruential generator
        let mut seed: u32 = 12345;
        for _ in 0..duration_samples {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let random = ((seed >> 16) as i32 - 16384) as f32 / 16384.0;
            let sample = (amplitude * random * 32767.0) as i16;
            writer.write_sample(sample).unwrap();
        }
        writer.finalize().unwrap();
    }
    buffer
}

#[test]
fn test_full_pipeline_with_synthetic_wavs() {
    let temp = tempdir().unwrap();

    // Create test WAV files with different characteristics
    // Low frequency sine (should be "pleasing" - dark, smooth)
    let low_sine = create_sine_wav(220.0, 0.5, 8192);
    fs::write(temp.path().join("low_sine.wav"), &low_sine).unwrap();

    // High frequency sine (should be less pleasing - bright)
    let high_sine = create_sine_wav(4000.0, 0.5, 8192);
    fs::write(temp.path().join("high_sine.wav"), &high_sine).unwrap();

    // Noise (should be least pleasing - harsh, high ZCR)
    let noise = create_noise_wav(0.3, 8192);
    fs::write(temp.path().join("noise.wav"), &noise).unwrap();

    // Scan and score
    let mut rows = scan::scan_dir(temp.path(), false, false).unwrap();
    assert_eq!(rows.len(), 3, "Should find 3 WAV files");

    score::normalize_rows(&mut rows);
    score::compute_scores(&mut rows);

    // Sort by pleasing score
    rows.sort_by(|a, b| b.pleasing_score.total_cmp(&a.pleasing_score));

    // Low sine should rank highest for pleasing (darkest)
    let top = &rows[0];
    assert!(
        top.path.to_string_lossy().contains("low_sine"),
        "Low sine should be most pleasing, got: {}",
        top.path.display()
    );

    // Noise should rank lowest for pleasing
    let bottom = &rows[2];
    assert!(
        bottom.path.to_string_lossy().contains("noise"),
        "Noise should be least pleasing, got: {}",
        bottom.path.display()
    );
}

#[test]
fn test_best_score_prefers_strong_signal() {
    let temp = tempdir().unwrap();

    // Strong signal
    let strong = create_sine_wav(1000.0, 0.8, 8192);
    fs::write(temp.path().join("strong.wav"), &strong).unwrap();

    // Weak signal
    let weak = create_sine_wav(1000.0, 0.1, 8192);
    fs::write(temp.path().join("weak.wav"), &weak).unwrap();

    let mut rows = scan::scan_dir(temp.path(), false, false).unwrap();
    score::normalize_rows(&mut rows);
    score::compute_scores(&mut rows);

    // Sort by best score
    rows.sort_by(|a, b| b.best_score.total_cmp(&a.best_score));

    // Strong signal should rank higher for "best"
    let top = &rows[0];
    assert!(
        top.path.to_string_lossy().contains("strong"),
        "Strong signal should rank higher for best, got: {}",
        top.path.display()
    );
}

#[test]
fn test_recursive_scan() {
    let temp = tempdir().unwrap();
    let subdir = temp.path().join("subdir");
    fs::create_dir(&subdir).unwrap();

    let wav = create_sine_wav(440.0, 0.5, 4096);
    fs::write(temp.path().join("root.wav"), &wav).unwrap();
    fs::write(subdir.join("nested.wav"), &wav).unwrap();

    // Non-recursive
    let rows = scan::scan_dir(temp.path(), false, false).unwrap();
    assert_eq!(rows.len(), 1, "Non-recursive should find 1 file");

    // Recursive
    let rows = scan::scan_dir(temp.path(), true, false).unwrap();
    assert_eq!(rows.len(), 2, "Recursive should find 2 files");
}

#[test]
fn test_json_output_structure() {
    let temp = tempdir().unwrap();

    let wav = create_sine_wav(440.0, 0.5, 4096);
    fs::write(temp.path().join("test.wav"), &wav).unwrap();

    let mut rows = scan::scan_dir(temp.path(), false, false).unwrap();
    score::normalize_rows(&mut rows);
    score::compute_scores(&mut rows);

    // Serialize to JSON
    let json = serde_json::to_string(&rows).unwrap();

    // Verify JSON contains expected fields
    assert!(json.contains("\"rms\""), "JSON should contain rms");
    assert!(json.contains("\"zcr\""), "JSON should contain zcr");
    assert!(
        json.contains("\"spectral_centroid\""),
        "JSON should contain spectral_centroid"
    );
    assert!(
        json.contains("\"pleasing_score\""),
        "JSON should contain pleasing_score"
    );
    assert!(
        json.contains("\"best_score\""),
        "JSON should contain best_score"
    );
}

#[test]
fn test_stereo_downmix() {
    let temp = tempdir().unwrap();

    // Create a stereo WAV file
    let sample_rate = 44100;
    let mut buffer = Vec::new();
    {
        let cursor = Cursor::new(&mut buffer);
        let spec = WavSpec {
            channels: 2,
            sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        let mut writer = WavWriter::new(cursor, spec).unwrap();

        for i in 0..4096 {
            let t = i as f32 / sample_rate as f32;
            // Left channel: 440 Hz
            let left = (0.5 * (2.0 * PI * 440.0 * t).sin() * 32767.0) as i16;
            // Right channel: 880 Hz
            let right = (0.5 * (2.0 * PI * 880.0 * t).sin() * 32767.0) as i16;
            writer.write_sample(left).unwrap();
            writer.write_sample(right).unwrap();
        }
        writer.finalize().unwrap();
    }

    fs::write(temp.path().join("stereo.wav"), &buffer).unwrap();

    let rows = scan::scan_dir(temp.path(), false, false).unwrap();
    assert_eq!(rows.len(), 1);

    // Mono samples should be half the stereo frame count
    assert_eq!(rows[0].num_samples, 4096);
}

#[test]
fn test_extended_metrics() {
    let temp = tempdir().unwrap();

    // Create test WAV files
    let sine = create_sine_wav(440.0, 0.5, 8192);
    fs::write(temp.path().join("sine.wav"), &sine).unwrap();

    let noise = create_noise_wav(0.3, 8192);
    fs::write(temp.path().join("noise.wav"), &noise).unwrap();

    // Scan with extended metrics
    let mut rows = scan::scan_dir(temp.path(), false, true).unwrap();
    assert_eq!(rows.len(), 2);

    // All extended metrics should be present
    for row in &rows {
        assert!(row.spectral_rolloff.is_some());
        assert!(row.spectral_flatness.is_some());
        assert!(row.crest_factor.is_some());
    }

    // Normalize and verify extended norms are computed
    score::normalize_rows(&mut rows);
    for row in &rows {
        assert!(row.rolloff_norm.is_some());
        assert!(row.flatness_norm.is_some());
        assert!(row.crest_norm.is_some());
    }

    // Noise should have higher flatness than sine
    let sine_row = rows
        .iter()
        .find(|r| r.path.to_string_lossy().contains("sine"))
        .unwrap();
    let noise_row = rows
        .iter()
        .find(|r| r.path.to_string_lossy().contains("noise"))
        .unwrap();

    assert!(
        noise_row.spectral_flatness.unwrap() > sine_row.spectral_flatness.unwrap(),
        "Noise flatness ({}) should exceed sine flatness ({})",
        noise_row.spectral_flatness.unwrap(),
        sine_row.spectral_flatness.unwrap()
    );
}
