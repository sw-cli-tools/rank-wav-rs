//! Audio feature extraction.
//!
//! This module provides functionality to compute acoustic features from
//! audio samples, including RMS energy, zero-crossing rate, spectral
//! centroid, and spectral bandwidth.

use anyhow::{Result, bail};
use rustfft::{FftPlanner, num_complex::Complex};
use serde::Serialize;
use std::f32::consts::PI;
use std::path::{Path, PathBuf};

/// Acoustic features extracted from a WAV file.
#[derive(Debug, Clone, Serialize)]
pub struct FeatureRow {
    /// Path to the WAV file.
    pub path: PathBuf,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Total number of samples (mono).
    pub num_samples: usize,

    /// RMS energy (signal strength indicator).
    pub rms: f32,
    /// Zero-crossing rate (noisiness indicator).
    pub zcr: f32,
    /// Spectral centroid in Hz (brightness indicator).
    pub spectral_centroid: f32,
    /// Spectral bandwidth in Hz (complexity indicator).
    pub spectral_bandwidth: f32,

    /// Normalized RMS (0.0-1.0 relative to batch).
    pub rms_norm: f32,
    /// Normalized zero-crossing rate.
    pub zcr_norm: f32,
    /// Normalized spectral centroid.
    pub centroid_norm: f32,
    /// Normalized spectral bandwidth.
    pub bandwidth_norm: f32,

    /// "Pleasing" score (higher = warmer, smoother).
    pub pleasing_score: f32,
    /// "Best" score (higher = more balanced, present).
    pub best_score: f32,
}

/// Compute acoustic features from audio samples.
///
/// # Arguments
///
/// * `path` - Path to the source file (for identification)
/// * `samples` - Normalized mono f32 samples
/// * `sample_rate` - Sample rate in Hz
///
/// # Returns
///
/// A `FeatureRow` with raw features computed. Normalized values and scores
/// are initialized to 0.0 and should be computed later via `score::normalize_rows()`.
pub fn compute_features(path: &Path, samples: &[f32], sample_rate: u32) -> Result<FeatureRow> {
    if samples.len() < 128 {
        bail!("too few samples (need at least 128, got {})", samples.len());
    }

    let rms = compute_rms(samples);
    let zcr = compute_zcr(samples);
    let (spectral_centroid, spectral_bandwidth) = compute_spectral_features(samples, sample_rate);

    Ok(FeatureRow {
        path: path.to_path_buf(),
        sample_rate,
        num_samples: samples.len(),
        rms,
        zcr,
        spectral_centroid,
        spectral_bandwidth,
        rms_norm: 0.0,
        zcr_norm: 0.0,
        centroid_norm: 0.0,
        bandwidth_norm: 0.0,
        pleasing_score: 0.0,
        best_score: 0.0,
    })
}

/// Compute RMS (root mean square) energy.
///
/// RMS is a measure of signal strength/loudness.
fn compute_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|x| x * x).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

/// Compute zero-crossing rate.
///
/// ZCR measures how often the signal crosses zero, which correlates
/// with noisiness and high-frequency content.
fn compute_zcr(samples: &[f32]) -> f32 {
    if samples.len() < 2 {
        return 0.0;
    }
    let mut crossings = 0usize;
    for w in samples.windows(2) {
        let a = w[0];
        let b = w[1];
        if (a >= 0.0 && b < 0.0) || (a < 0.0 && b >= 0.0) {
            crossings += 1;
        }
    }
    crossings as f32 / (samples.len() - 1) as f32
}

/// Compute spectral centroid and bandwidth using FFT.
///
/// The spectral centroid is the "center of mass" of the spectrum,
/// indicating perceived brightness. The bandwidth measures how
/// spread out the spectrum is.
fn compute_spectral_features(samples: &[f32], sample_rate: u32) -> (f32, f32) {
    // Use power-of-two FFT size, capped for memory efficiency
    let n = samples.len().next_power_of_two().clamp(1024, 16384);

    // Analyze center segment of the audio
    let start = if samples.len() > n {
        (samples.len() - n) / 2
    } else {
        0
    };
    let end = (start + n).min(samples.len());
    let slice = &samples[start..end];

    // Prepare FFT buffer with Hann window
    let mut buffer = vec![
        Complex {
            re: 0.0f32,
            im: 0.0f32
        };
        n
    ];
    for (i, b) in buffer.iter_mut().enumerate().take(slice.len()) {
        // Hann window: 0.5 - 0.5 * cos(2*pi*i/N)
        let w = 0.5 - 0.5 * (2.0 * PI * i as f32 / slice.len() as f32).cos();
        b.re = slice[i] * w;
    }

    // Perform FFT
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(n);
    fft.process(&mut buffer);

    // Compute magnitude spectrum (first half only - positive frequencies)
    let half = n / 2;
    let bin_hz = sample_rate as f32 / n as f32;

    let mags: Vec<f32> = buffer[..half].iter().map(|c| c.norm()).collect();

    let mag_sum: f32 = mags.iter().sum();
    if mag_sum <= 1e-12 {
        return (0.0, 0.0);
    }

    // Spectral centroid: weighted average of frequencies
    let centroid = mags
        .iter()
        .enumerate()
        .map(|(i, m)| i as f32 * bin_hz * m)
        .sum::<f32>()
        / mag_sum;

    // Spectral bandwidth: weighted standard deviation from centroid
    let bandwidth = (mags
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let f = i as f32 * bin_hz;
            let d = f - centroid;
            d * d * m
        })
        .sum::<f32>()
        / mag_sum)
        .sqrt();

    (centroid, bandwidth)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rms_silent() {
        let samples = vec![0.0; 1000];
        assert_eq!(compute_rms(&samples), 0.0);
    }

    #[test]
    fn test_rms_constant() {
        let samples = vec![0.5; 1000];
        let rms = compute_rms(&samples);
        assert!((rms - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_zcr_constant() {
        // No crossings for constant signal
        let samples = vec![0.5; 1000];
        assert_eq!(compute_zcr(&samples), 0.0);
    }

    #[test]
    fn test_zcr_alternating() {
        // Alternating signal: every pair crosses
        let samples: Vec<f32> = (0..1000)
            .map(|i| if i % 2 == 0 { 0.5 } else { -0.5 })
            .collect();
        let zcr = compute_zcr(&samples);
        assert!((zcr - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_spectral_features_sine() {
        // Generate 440 Hz sine wave
        let sample_rate = 44100;
        let freq = 440.0;
        let samples: Vec<f32> = (0..4096)
            .map(|i| (2.0 * PI * freq * i as f32 / sample_rate as f32).sin())
            .collect();

        let (centroid, bandwidth) = compute_spectral_features(&samples, sample_rate);

        // Centroid should be close to 440 Hz
        assert!(
            (centroid - freq).abs() < 50.0,
            "centroid {centroid} not near {freq}"
        );
        // Bandwidth should be narrow for pure tone
        assert!(bandwidth < 200.0, "bandwidth {bandwidth} too wide for sine");
    }

    #[test]
    fn test_compute_features() {
        let samples: Vec<f32> = (0..1024).map(|i| (i as f32 * 0.1).sin()).collect();
        let row = compute_features(Path::new("test.wav"), &samples, 44100).unwrap();

        assert_eq!(row.path, PathBuf::from("test.wav"));
        assert_eq!(row.sample_rate, 44100);
        assert_eq!(row.num_samples, 1024);
        assert!(row.rms > 0.0);
    }

    #[test]
    fn test_too_few_samples() {
        let samples = vec![0.0; 10];
        let result = compute_features(Path::new("test.wav"), &samples, 44100);
        assert!(result.is_err());
    }
}
