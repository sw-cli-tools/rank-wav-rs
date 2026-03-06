# rank-wav

A Rust CLI tool that scans directories for WAV files and ranks them by acoustic features correlated with perceived sound quality.

## Features

- Scan directories for WAV files (with optional recursion)
- Extract acoustic features: RMS energy, zero-crossing rate, spectral centroid, spectral bandwidth
- Optional extended metrics: spectral rolloff, spectral flatness, crest factor
- Rank files by "pleasing" (warm, smooth) or "best" (balanced, present) scores
- Output as formatted table or JSON
- Configurable metrics and scoring weights via TOML config file
- Supports 8/16/24/32-bit integer and 32-bit float WAV formats
- Pure Rust implementation (no C dependencies)

## Installation

```bash
# Clone and build
git clone https://github.com/sw-cli-tools/rank-wav-rs.git
cd rank-wav-rs
cargo build --release

# Binary will be at ./target/release/rank-wav
```

### Setting up the alias

Add to your shell config (`~/.bashrc`, `~/.zshrc`, etc.):

```bash
alias rank-wav='/path/to/rank-wav-rs/target/release/rank-wav'
```

Or install system-wide:

```bash
cargo install --path .
# Installs to ~/.cargo/bin/rank-wav (ensure ~/.cargo/bin is in your PATH)
```

## Usage

```bash
# Scan directory, sort by "most pleasing" (default)
rank-wav ./samples

# Sort by "best" (balanced, present, clear)
rank-wav ./samples --sort best

# Recursive scan into subdirectories
rank-wav ./samples -r

# Enable extended metrics (rolloff, flatness, crest factor)
rank-wav ./samples -e

# Use custom config file
rank-wav ./samples -c custom-config.toml

# Output as JSON
rank-wav ./samples --json

# Combine options
rank-wav ./samples -r -e --sort best --json
```

## Example Output

```
+---+------------------------+--------+--------+----------+-----------+----------+--------+
| # | File                   | RMS    | ZCR    | Centroid | Bandwidth | Pleasing | Best   |
+---+------------------------+--------+--------+----------+-----------+----------+--------+
| 1 | motif-warm.wav         | 0.0271 | 0.0190 | 763      | 1079      | 0.812    | 0.641  |
| 2 | motif-balanced.wav     | 0.0647 | 0.0480 | 1502     | 1515      | 0.487    | 0.844  |
| 3 | motif-bright.wav       | 0.0361 | 0.0530 | 1782     | 1469      | 0.362    | 0.611  |
+---+------------------------+--------+--------+----------+-----------+----------+--------+
```

## Scoring Methodology

### "Pleasing" Score

Favors warm, smooth, easy-to-listen sounds:
- Lower spectral centroid (less bright)
- Lower spectral bandwidth (less complex)
- Lower zero-crossing rate (less noisy)
- Moderate RMS (not too quiet)

### "Best" Score

Favors clear, balanced, present sounds:
- Strong RMS (present, not weak)
- Moderate spectral centroid (not too bright or dark)
- Moderate spectral bandwidth (not too complex or thin)
- Low zero-crossing rate (not noisy)

## Acoustic Features

### Basic Metrics (always computed)

| Feature | Description | Interpretation |
|---------|-------------|----------------|
| RMS | Root mean square energy | Signal strength/loudness |
| ZCR | Zero-crossing rate | Noisiness, high-frequency content |
| Centroid | Spectral centroid (Hz) | Perceived brightness |
| Bandwidth | Spectral bandwidth (Hz) | Spectral complexity/spread |

### Extended Metrics (with -e/--extended flag)

| Feature | Description | Interpretation |
|---------|-------------|----------------|
| Rolloff | Spectral rolloff (Hz) | Frequency below which 85% of energy lies |
| Flatness | Spectral flatness (0-1) | 0 = tonal, 1 = noisy |
| Crest | Crest factor (dB) | Peak to RMS ratio; higher = more dynamic |

## Command-Line Options

```
Usage: rank-wav [OPTIONS] <DIR>

Arguments:
  <DIR>  Directory to scan for WAV files

Options:
  -r, --recursive         Recurse into subdirectories
  -s, --sort <SORT>       Sort mode [default: pleasing] [possible values: best, pleasing]
  -e, --extended          Enable extended metrics (rolloff, flatness, crest factor)
  -c, --config <CONFIG>   Configuration file path [default: config.toml]
      --json              Output results as JSON instead of a table
  -h, --help              Print help
  -V, --version           Print version
```

## Configuration

Create a `config.toml` file to customize metrics and scoring weights:

```toml
# Enable/disable individual metrics
[metrics.basic]
rms = true
zcr = true
spectral_centroid = true
spectral_bandwidth = true

[metrics.extended]
spectral_rolloff = false
spectral_flatness = false
crest_factor = false

# Customize scoring weights
[scoring.pleasing]
centroid_weight = -0.40
bandwidth_weight = -0.30
zcr_weight = -0.20
rms_weight = 0.10

[scoring.best]
rms_weight = 0.35
centroid_target = 0.45
centroid_weight = -0.25
bandwidth_target = 0.40
bandwidth_weight = -0.20
zcr_weight = -0.20
```

The config file is optional. Missing or empty files use sensible defaults. At least one metric must be enabled.

## Use Cases

- **Procedural Audio**: Compare synthesized sound variants to find the most pleasing
- **Sample Libraries**: Quickly triage large collections of audio samples
- **Sound Design**: Rank synthesis parameter variations
- **Music Production**: Sort one-shots by tonal characteristics

## Technical Details

- FFT-based spectral analysis using rustfft
- Hann window applied before FFT to reduce spectral leakage
- Center segment analysis for long files (up to 16384 samples)
- Batch normalization for relative ranking within a set

## License

MIT License - See [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome! Please ensure:
- All tests pass: `cargo test`
- No clippy warnings: `cargo clippy -- -D warnings`
- Code formatted: `cargo fmt`
