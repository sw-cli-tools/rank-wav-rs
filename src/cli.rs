//! Command-line interface definitions.
//!
//! This module defines the CLI structure using clap derive macros.

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Sort mode for ranking WAV files.
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum SortMode {
    /// Rank by "best" score (balanced, present, clear).
    Best,
    /// Rank by "pleasing" score (warm, smooth, low harshness).
    #[default]
    Pleasing,
}

/// Scan WAV files and rank them by acoustic features.
#[derive(Debug, Parser)]
#[command(name = "rank-wav")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Directory to scan for WAV files.
    pub dir: PathBuf,

    /// Recurse into subdirectories.
    #[arg(short, long)]
    pub recursive: bool,

    /// Sort mode for ranking.
    #[arg(short, long, value_enum, default_value = "pleasing")]
    pub sort: SortMode,

    /// Output results as JSON instead of a table.
    #[arg(long)]
    pub json: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_defaults() {
        let cli = Cli::parse_from(["rank-wav", "."]);
        assert!(!cli.recursive);
        assert!(matches!(cli.sort, SortMode::Pleasing));
        assert!(!cli.json);
    }

    #[test]
    fn test_cli_with_flags() {
        let cli = Cli::parse_from(["rank-wav", "./wavs", "-r", "--sort", "best", "--json"]);
        assert!(cli.recursive);
        assert!(matches!(cli.sort, SortMode::Best));
        assert!(cli.json);
    }
}
