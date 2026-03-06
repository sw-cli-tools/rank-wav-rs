# Implementation Plan

## Phase 1: Foundation (MVP)

### Goal
Get a working CLI that can scan WAV files and output basic metrics.

### Tasks

#### 1.1 Project Setup
- [ ] Update Cargo.toml with dependencies
- [ ] Create module structure (cli.rs, scan.rs, wav.rs, features.rs, score.rs, output.rs)
- [ ] Add basic main.rs with module imports

#### 1.2 CLI Module
- [ ] Define `Cli` struct with clap derive
- [ ] Define `SortMode` enum (Best, Pleasing)
- [ ] Parse directory argument
- [ ] Add --recursive, --sort, --json flags

#### 1.3 WAV Reader
- [ ] Implement `read_wav_mono_f32()` function
- [ ] Support 16-bit integer format (most common)
- [ ] Downmix stereo to mono
- [ ] Add unit tests with test WAV files

#### 1.4 Directory Scanner
- [ ] Implement `scan_dir()` function
- [ ] Filter for .wav files
- [ ] Support recursive flag
- [ ] Handle errors gracefully (skip unreadable files)

### Deliverable
`cargo run -- ./test-wavs` prints list of discovered WAV files.

---

## Phase 2: Feature Extraction

### Goal
Compute acoustic features for each WAV file.

### Tasks

#### 2.1 Basic Features
- [ ] Define `FeatureRow` struct
- [ ] Implement `compute_rms()` function
- [ ] Implement `compute_zcr()` function
- [ ] Add unit tests

#### 2.2 Spectral Features
- [ ] Implement `compute_spectral_features()` function
- [ ] Add Hann window before FFT
- [ ] Compute spectral centroid
- [ ] Compute spectral bandwidth
- [ ] Add unit tests with known signals

#### 2.3 Integration
- [ ] Wire feature extraction into scan pipeline
- [ ] Populate FeatureRow for each file
- [ ] Handle edge cases (too few samples)

### Deliverable
`cargo run -- ./test-wavs` prints raw feature values for each file.

---

## Phase 3: Scoring and Ranking

### Goal
Normalize features and compute ranking scores.

### Tasks

#### 3.1 Normalization
- [ ] Implement `normalize_rows()` function
- [ ] Min-max scaling for each feature
- [ ] Handle edge case (single file)

#### 3.2 Scoring
- [ ] Implement `compute_scores()` function
- [ ] Implement pleasing score formula
- [ ] Implement best score formula
- [ ] Add unit tests

#### 3.3 Sorting
- [ ] Sort by selected score in main.rs
- [ ] Descending order (best first)

### Deliverable
`cargo run -- ./test-wavs --sort pleasing` shows ranked results.

---

## Phase 4: Output Formatting

### Goal
Professional output in table and JSON formats.

### Tasks

#### 4.1 Table Output
- [ ] Implement table formatting with tabled
- [ ] Show file, raw features, scores
- [ ] Format numbers appropriately

#### 4.2 JSON Output
- [ ] Implement JSON output with serde_json
- [ ] Include all FeatureRow fields
- [ ] Pretty-print for readability

#### 4.3 Polish
- [ ] Add rank column (#1, #2, etc.)
- [ ] Truncate long filenames
- [ ] Add color highlighting (optional)

### Deliverable
Complete v1.0 with `--json` flag working.

---

## Phase 5: Quality and Documentation

### Goal
Production-ready code with full documentation.

### Tasks

#### 5.1 Extended WAV Support
- [ ] Add 8-bit integer support
- [ ] Add 24-bit integer support
- [ ] Add 32-bit integer support
- [ ] Add 32-bit float support
- [ ] Test each format

#### 5.2 Documentation
- [ ] Doc comments on all public items
- [ ] Module-level documentation
- [ ] Update README with usage examples
- [ ] Add --help examples

#### 5.3 Error Messages
- [ ] Improve error messages
- [ ] Add file path to all errors
- [ ] Suggest fixes where possible

#### 5.4 Testing
- [ ] Create test WAV files in tests/
- [ ] Integration tests for full pipeline
- [ ] Edge case tests (empty dir, no wavs)

### Deliverable
Fully documented, tested v1.0 release.

---

## Completed Phases

- **Phase 1-4**: Foundation, Features, Scoring, Output (v1.0)
- **Phase 5**: Quality and Documentation
- **Phase 7**: Extended Metrics (rolloff, flatness, crest factor)
- **Phase 8**: Configuration File Support (TOML config, custom weights)

---

## Future Phases (Post v1.2)

### Phase 6: CSV Export
- [ ] Add --csv flag
- [ ] Output to file or stdout
- [ ] Include all features and scores

### Phase 9: Multi-Format Support
- [ ] Replace hound with symphonia
- [ ] Add MP3 support
- [ ] Add FLAC support
- [ ] Add OGG support

### Phase 10: Output Enhancements
- [ ] `--top N` flag to limit output to top N results
- [ ] `--quiet` flag for minimal output (filenames only)
- [ ] `--threshold` flag to filter by minimum score

### Phase 11: Performance
- [ ] Parallel processing with rayon for large directories
- [ ] Watch mode to re-scan on file changes

### Phase 12: Calibration Mode
- [ ] Accept user rankings
- [ ] Fit custom weights
- [ ] Save/load weight profiles

---

## Current Status

See [status.md](status.md) for current progress.
