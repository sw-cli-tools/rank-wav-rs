//! WAV file reading and decoding.
//!
//! This module provides functionality to read WAV files and convert them
//! to normalized mono f32 samples suitable for audio analysis.

use anyhow::{Result, bail};
use hound::{SampleFormat, WavReader};
use std::path::Path;

/// Read a WAV file and return normalized mono f32 samples.
///
/// Supports 8, 16, 24, and 32-bit integer formats, as well as 32-bit float.
/// Stereo and multi-channel files are downmixed to mono by averaging.
///
/// # Arguments
///
/// * `path` - Path to the WAV file
///
/// # Returns
///
/// A tuple of (samples, sample_rate) where samples are normalized to [-1.0, 1.0].
pub fn read_wav_mono_f32(path: &Path) -> Result<(Vec<f32>, u32)> {
    let mut reader = WavReader::open(path)?;
    let spec = reader.spec();

    let channels = spec.channels as usize;
    let sample_rate = spec.sample_rate;

    if channels == 0 {
        bail!("WAV has zero channels");
    }

    let interleaved: Vec<f32> = match (spec.sample_format, spec.bits_per_sample) {
        (SampleFormat::Int, 8) => reader
            .samples::<i8>()
            .map(|s| s.map(|v| v as f32 / i8::MAX as f32))
            .collect::<Result<Vec<_>, _>>()?,

        (SampleFormat::Int, 16) => reader
            .samples::<i16>()
            .map(|s| s.map(|v| v as f32 / i16::MAX as f32))
            .collect::<Result<Vec<_>, _>>()?,

        (SampleFormat::Int, 24) => reader
            .samples::<i32>()
            .map(|s| s.map(|v| v as f32 / 8_388_607.0)) // 2^23 - 1
            .collect::<Result<Vec<_>, _>>()?,

        (SampleFormat::Int, 32) => reader
            .samples::<i32>()
            .map(|s| s.map(|v| v as f32 / i32::MAX as f32))
            .collect::<Result<Vec<_>, _>>()?,

        (SampleFormat::Float, 32) => reader.samples::<f32>().collect::<Result<Vec<_>, _>>()?,

        _ => bail!(
            "unsupported WAV format: {:?} {} bits",
            spec.sample_format,
            spec.bits_per_sample
        ),
    };

    if channels == 1 {
        return Ok((interleaved, sample_rate));
    }

    // Downmix to mono by averaging channels.
    let mut mono = Vec::with_capacity(interleaved.len() / channels);
    for frame in interleaved.chunks_exact(channels) {
        let sum: f32 = frame.iter().copied().sum();
        mono.push(sum / channels as f32);
    }

    Ok((mono, sample_rate))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn create_test_wav_16bit_mono(samples: &[i16]) -> Vec<u8> {
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
            for &sample in samples {
                writer.write_sample(sample).unwrap();
            }
            writer.finalize().unwrap();
        }
        buffer
    }

    fn create_test_wav_16bit_stereo(samples: &[(i16, i16)]) -> Vec<u8> {
        let mut buffer = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer);
            let spec = hound::WavSpec {
                channels: 2,
                sample_rate: 44100,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };
            let mut writer = hound::WavWriter::new(cursor, spec).unwrap();
            for &(left, right) in samples {
                writer.write_sample(left).unwrap();
                writer.write_sample(right).unwrap();
            }
            writer.finalize().unwrap();
        }
        buffer
    }

    #[test]
    fn test_read_mono_16bit() {
        let wav_data = create_test_wav_16bit_mono(&[0, 16383, -16384, 32767]);
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), &wav_data).unwrap();

        let (samples, sr) = read_wav_mono_f32(temp.path()).unwrap();

        assert_eq!(sr, 44100);
        assert_eq!(samples.len(), 4);
        assert!((samples[0] - 0.0).abs() < 0.001);
        assert!((samples[3] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_read_stereo_downmix() {
        // Left: max positive, Right: max negative => average ~0
        let wav_data = create_test_wav_16bit_stereo(&[(32767, -32767), (16384, 16384)]);
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), &wav_data).unwrap();

        let (samples, sr) = read_wav_mono_f32(temp.path()).unwrap();

        assert_eq!(sr, 44100);
        assert_eq!(samples.len(), 2);
        // First frame: (1.0 + -1.0) / 2 = 0.0
        assert!(samples[0].abs() < 0.001);
        // Second frame: both positive, should average to ~0.5
        assert!((samples[1] - 0.5).abs() < 0.01);
    }
}
