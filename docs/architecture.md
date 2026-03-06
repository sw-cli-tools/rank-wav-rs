# Architecture

## Overview

rank-wav-rs is a Rust CLI tool that scans directories for WAV files, extracts acoustic features, and ranks them based on perceptual quality metrics.

## System Architecture

```
+-------------------+
|     CLI Layer     |  cli.rs - Command parsing (clap)
+-------------------+
         |
         v
+-------------------+
|   Scan Layer      |  scan.rs - Directory traversal (walkdir)
+-------------------+
         |
         v
+-------------------+
|   WAV Decoder     |  wav.rs - WAV reading (hound)
+-------------------+
         |
         v
+-------------------+
|  Feature Extract  |  features.rs - Audio analysis (rustfft)
+-------------------+
         |
         v
+-------------------+
|   Scoring Layer   |  score.rs - Normalization and ranking
+-------------------+
         |
         v
+-------------------+
|   Output Layer    |  output.rs - Table/JSON formatting
+-------------------+
```

## Module Responsibilities

### cli.rs
- Parse command-line arguments using clap derive macros
- Define `SortMode` enum (Best, Pleasing)
- Validate input directory exists

### scan.rs
- Traverse directories using walkdir
- Filter for .wav files only
- Handle recursive vs flat scanning
- Gracefully skip unreadable files

### wav.rs
- Decode WAV files using hound
- Support multiple sample formats (8/16/24/32-bit int, 32-bit float)
- Convert to normalized f32 samples
- Downmix stereo to mono

### features.rs
- Define `FeatureRow` struct with all metrics
- Compute RMS energy (signal strength)
- Compute zero-crossing rate (noisiness proxy)
- Compute spectral centroid (brightness) via FFT
- Compute spectral bandwidth (complexity) via FFT
- Apply Hann window before FFT

### score.rs
- Normalize features across batch (min-max scaling)
- Compute "pleasing" score (dark, smooth, moderate)
- Compute "best" score (balanced, present, clear)

### output.rs
- Format results as terminal table using tabled
- Format results as JSON using serde_json
- Support CSV output (future)

## Data Flow

```
Directory Path
     |
     v
[scan.rs] --> List of WAV paths
     |
     v
[wav.rs] --> Vec<f32> samples + sample_rate per file
     |
     v
[features.rs] --> FeatureRow per file (raw metrics)
     |
     v
[score.rs] --> FeatureRow with normalized + scored values
     |
     v
[output.rs] --> Terminal table or JSON
```

## Key Data Structures

### FeatureRow
```rust
pub struct FeatureRow {
    pub path: PathBuf,
    pub sample_rate: u32,
    pub num_samples: usize,

    // Raw features
    pub rms: f32,
    pub zcr: f32,
    pub spectral_centroid: f32,
    pub spectral_bandwidth: f32,

    // Normalized (0.0 - 1.0)
    pub rms_norm: f32,
    pub zcr_norm: f32,
    pub centroid_norm: f32,
    pub bandwidth_norm: f32,

    // Computed scores
    pub pleasing_score: f32,
    pub best_score: f32,
}
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| anyhow | Error handling |
| clap | CLI argument parsing |
| hound | WAV file decoding |
| rustfft | FFT for spectral analysis |
| walkdir | Directory traversal |
| serde | Serialization |
| serde_json | JSON output |
| tabled | Terminal tables |

## Performance Considerations

- FFT size capped at 16384 samples to limit memory
- Analyze center segment of long files
- Streaming WAV decode (hound)
- Single-threaded for v1 (sufficient for typical use)

## Future Extensions

1. **Multi-format support**: Use symphonia for MP3/FLAC/AAC
2. **Parallel processing**: Use rayon for concurrent file analysis
3. **Additional metrics**: Spectral rolloff, flatness, crest factor
4. **Calibration mode**: User-ranked training data to fit custom weights
5. **CSV export**: Machine-readable output for spreadsheets
