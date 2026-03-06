# rank-wav

A Rust CLI tool that scans directories for WAV files and ranks them by acoustic features correlated with perceived sound quality.

## Features

- Scan directories for WAV files (with optional recursion)
- Extract acoustic features: RMS energy, zero-crossing rate, spectral centroid, spectral bandwidth
- Rank files by "pleasing" (warm, smooth) or "best" (balanced, present) scores
- Output as formatted table or JSON
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

## Usage

```bash
# Scan directory, sort by "most pleasing" (default)
rank-wav ./samples

# Sort by "best" (balanced, present, clear)
rank-wav ./samples --sort best

# Recursive scan into subdirectories
rank-wav ./samples -r

# Output as JSON
rank-wav ./samples --json

# Combine options
rank-wav ./samples -r --sort best --json
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

| Feature | Description | Interpretation |
|---------|-------------|----------------|
| RMS | Root mean square energy | Signal strength/loudness |
| ZCR | Zero-crossing rate | Noisiness, high-frequency content |
| Centroid | Spectral centroid (Hz) | Perceived brightness |
| Bandwidth | Spectral bandwidth (Hz) | Spectral complexity/spread |

## Command-Line Options

```
Usage: rank-wav [OPTIONS] <DIR>

Arguments:
  <DIR>  Directory to scan for WAV files

Options:
  -r, --recursive      Recurse into subdirectories
  -s, --sort <SORT>    Sort mode [default: pleasing] [possible values: best, pleasing]
      --json           Output results as JSON instead of a table
  -h, --help           Print help
  -V, --version        Print version
```

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
