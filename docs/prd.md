# Product Requirements Document (PRD)

## Problem Statement

When generating procedural audio (synthesized motifs, sound effects, etc.), multiple variants are often produced. Manually listening to and comparing each file to determine which sounds "best" or "most pleasing" is time-consuming and subjective.

## Solution

A Rust CLI tool that automatically analyzes WAV files and ranks them based on objective acoustic features that correlate with perceived sound quality.

## Target Users

- Audio developers generating procedural sounds
- Sound designers comparing synthesis parameter variations
- Musicians curating sample libraries
- Anyone needing to quickly triage many audio files

## Core Requirements

### Functional Requirements

#### FR-1: Directory Scanning
- MUST scan a specified directory for WAV files
- MUST support optional recursive scanning
- MUST gracefully skip non-WAV files
- MUST report errors for unreadable files without crashing

#### FR-2: WAV File Support
- MUST support 8-bit, 16-bit, 24-bit, and 32-bit integer formats
- MUST support 32-bit float format
- MUST support mono and stereo files
- MUST downmix stereo to mono for analysis

#### FR-3: Feature Extraction
- MUST compute RMS energy (signal strength indicator)
- MUST compute zero-crossing rate (noisiness indicator)
- MUST compute spectral centroid (brightness indicator)
- MUST compute spectral bandwidth (complexity indicator)

#### FR-4: Ranking
- MUST normalize features across the batch being analyzed
- MUST provide a "pleasing" ranking (favors dark, smooth, moderate)
- MUST provide a "best" ranking (favors balanced, present, clear)
- MUST allow user to select sort mode

#### FR-5: Output
- MUST display results in a formatted terminal table
- MUST support JSON output for machine processing
- SHOULD support CSV output for spreadsheet analysis

### Non-Functional Requirements

#### NFR-1: Performance
- SHOULD process typical motif files (< 10 seconds) within 100ms each
- SHOULD handle directories with hundreds of files

#### NFR-2: Portability
- MUST be pure Rust (no C dependencies)
- MUST build on macOS, Linux, and Windows

#### NFR-3: Usability
- MUST provide clear error messages
- MUST have --help documentation
- SHOULD show progress for large batches

#### NFR-4: Extensibility
- Architecture SHOULD support adding new features easily
- Architecture SHOULD support adding new output formats easily
- Architecture SHOULD support future format support (MP3, FLAC)

## Scoring Methodology

### "Most Pleasing" Score
Biases toward warm, smooth, easy-to-listen sounds:
- Lower spectral centroid (less bright)
- Lower spectral bandwidth (less complex)
- Lower zero-crossing rate (less noisy)
- Moderate RMS (not too quiet)

Formula:
```
pleasing_score = -0.40 * norm(centroid)
               - 0.30 * norm(bandwidth)
               - 0.20 * norm(zcr)
               + 0.10 * norm(rms)
```

### "Best" Score
Biases toward clear, balanced, present sounds:
- Strong RMS (present, not weak)
- Moderate centroid (not too bright or dark)
- Moderate bandwidth (not too complex or thin)
- Low noisiness

Formula:
```
best_score = +0.35 * norm(rms)
           - 0.25 * distance_from_target(norm(centroid), 0.45)
           - 0.20 * distance_from_target(norm(bandwidth), 0.40)
           - 0.20 * norm(zcr)
```

## CLI Interface

### Commands

```bash
# Basic usage
rank-wav ./motifs

# Sort by "best" instead of default "pleasing"
rank-wav ./motifs --sort best

# Recursive scan
rank-wav ./motifs --recursive

# JSON output
rank-wav ./motifs --json

# CSV output (future)
rank-wav ./motifs --csv output.csv
```

### Arguments

| Argument | Short | Description | Default |
|----------|-------|-------------|---------|
| `<dir>` | - | Directory to scan | Required |
| `--sort` | `-s` | Sort mode (pleasing, best) | pleasing |
| `--recursive` | `-r` | Scan subdirectories | false |
| `--json` | - | Output as JSON | false |
| `--csv` | - | Output as CSV to file | - |

## Success Metrics

1. User can rank a directory of WAV files in < 5 seconds
2. Rankings correlate reasonably with human perception
3. Tool is easy to integrate into scripted workflows (via JSON output)

## Out of Scope (v1)

- Non-WAV format support
- GUI interface
- Audio playback
- Spectrogram visualization
- Machine learning models

## Future Enhancements (v2+)

### Output Enhancements
- `--csv` flag for CSV export
- `--top N` flag to limit output to top N results
- `--quiet` flag for minimal output (filenames only)
- `--threshold` flag to filter by minimum score

### Performance
- Parallel processing with rayon for large directories
- Watch mode to re-scan on file changes

### Format Support
- MP3 support via symphonia
- FLAC support via symphonia
- OGG support via symphonia

### Advanced Features
- Calibration mode: accept user rankings, fit custom weights
- Save/load weight profiles
