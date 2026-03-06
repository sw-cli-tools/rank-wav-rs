# Design Document

## Design Decisions

### DD-1: Pure Rust Implementation

**Decision**: Use pure Rust crates without C/C++ dependencies.

**Rationale**:
- Simplifies cross-platform builds
- No system library requirements
- Easier to install via cargo

**Trade-offs**:
- hound is WAV-only (no MP3/FLAC)
- rustfft is slightly slower than FFTW but sufficient for our needs

### DD-2: Mono Analysis

**Decision**: Convert all audio to mono before analysis.

**Rationale**:
- Simplifies feature computation
- Stereo width is not a primary quality indicator
- Consistent analysis regardless of channel count

**Implementation**: Average all channels together.

### DD-3: Batch Normalization

**Decision**: Normalize features relative to the batch being analyzed.

**Rationale**:
- Rankings are meaningful within a comparison set
- Absolute thresholds vary by use case
- Allows relative ranking without calibration

**Trade-off**: Scores are not comparable across different runs.

### DD-4: Windowed FFT

**Decision**: Apply Hann window before FFT.

**Rationale**:
- Reduces spectral leakage
- Improves centroid/bandwidth accuracy
- Standard practice in audio analysis

### DD-5: Center Segment Analysis

**Decision**: For long files, analyze a center segment rather than the whole file.

**Rationale**:
- Avoids attack/decay transients skewing results
- Captures the sustained portion of sounds
- Limits memory usage for very long files

**Implementation**: Use up to 16384 samples from the center.

### DD-6: Explicit Scoring Formulas

**Decision**: Use simple, documented weighted formulas for scoring.

**Rationale**:
- Explainable results (users can understand rankings)
- No hidden complexity or "magic"
- Easy to adjust weights in future versions

### DD-7: Two Distinct Scores

**Decision**: Provide separate "pleasing" and "best" scores.

**Rationale**:
- "Pleasing" and "best" are subtly different
- Different use cases require different biases
- Users can choose which matters more

**Definitions**:
- Pleasing: warm, smooth, low harshness
- Best: balanced, present, clear signal

## API Design

### FeatureRow Struct

```rust
#[derive(Debug, Clone, Serialize)]
pub struct FeatureRow {
    // Identity
    pub path: PathBuf,
    pub sample_rate: u32,
    pub num_samples: usize,

    // Raw features (absolute values)
    pub rms: f32,
    pub zcr: f32,
    pub spectral_centroid: f32,
    pub spectral_bandwidth: f32,

    // Normalized features (0.0-1.0 relative to batch)
    pub rms_norm: f32,
    pub zcr_norm: f32,
    pub centroid_norm: f32,
    pub bandwidth_norm: f32,

    // Computed scores
    pub pleasing_score: f32,
    pub best_score: f32,
}
```

### CLI Structure

```rust
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SortMode {
    Best,
    Pleasing,
}

#[derive(Debug, Parser)]
pub struct Cli {
    pub dir: PathBuf,

    #[arg(short, long)]
    pub recursive: bool,

    #[arg(short, long, value_enum, default_value = "pleasing")]
    pub sort: SortMode,

    #[arg(long)]
    pub json: bool,
}
```

## Error Handling

### Strategy

- Use `anyhow::Result` for all fallible operations
- Print warnings for individual file failures, continue processing
- Exit with error only for fatal issues (missing directory, no files)

### Error Categories

| Category | Handling |
|----------|----------|
| Directory not found | Exit with error |
| No WAV files found | Exit with message |
| Individual file unreadable | Warn, skip file |
| Unsupported WAV format | Warn, skip file |
| Too few samples | Warn, skip file |

## Testing Strategy

### Unit Tests

- `wav.rs`: Test each sample format conversion
- `features.rs`: Test feature computation with known signals
- `score.rs`: Test normalization with edge cases

### Integration Tests

- Full pipeline with test WAV files
- Verify JSON output structure
- Verify table output format

### Test Signals

Create synthetic test WAV files:
- Pure sine wave (predictable centroid)
- White noise (high ZCR, wide bandwidth)
- Silent file (zero RMS, edge case)

## Future Considerations

### Multi-Format Support

Replace `hound` with `symphonia` for MP3/FLAC/AAC/OGG.

### Additional Features

Potential future metrics:
- Spectral rolloff (frequency below which X% of energy)
- Spectral flatness (how noise-like vs tonal)
- Crest factor (peak to RMS ratio)
- Attack time (onset characteristics)
- Harmonic-to-noise ratio

### Custom Weights

Allow user-defined scoring weights via config file:
```toml
[scoring.pleasing]
centroid_weight = -0.40
bandwidth_weight = -0.30
zcr_weight = -0.20
rms_weight = 0.10
```

### Calibration Mode

Let users hand-rank a set of files, then fit weights to match:
1. User ranks 20 files manually
2. Tool optimizes weights to minimize ranking error
3. Custom weights saved for future use
