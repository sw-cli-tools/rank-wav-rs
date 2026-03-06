//! rank-wav: Scan WAV files and rank them by acoustic features.
//!
//! This CLI tool analyzes WAV files in a directory and ranks them based on
//! perceptual quality metrics including RMS energy, zero-crossing rate,
//! spectral centroid, and spectral bandwidth.

mod cli;
mod config;
mod features;
mod output;
mod scan;
mod score;
mod wav;

use anyhow::{bail, Result};
use clap::Parser;
use cli::{Cli, SortMode};
use config::Config;

fn main() -> Result<()> {
    // Handle version flag with detailed build info
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && (args[1] == "-V" || args[1] == "--version") {
        println!("{}", sw_cli::version!());
        return Ok(());
    }

    let cli = Cli::parse();

    if !cli.dir.exists() {
        bail!("Directory does not exist: {}", cli.dir.display());
    }

    if !cli.dir.is_dir() {
        bail!("Not a directory: {}", cli.dir.display());
    }

    // Load configuration (missing or empty file uses defaults)
    let config = Config::load(&cli.config)?.with_extended(cli.extended);

    // Validate that at least one metric is enabled
    config.validate()?;

    let mut rows = scan::scan_dir(&cli.dir, cli.recursive, &config)?;

    if rows.is_empty() {
        println!("No WAV files found in {}", cli.dir.display());
        return Ok(());
    }

    // Normalize features across the batch
    score::normalize_rows(&mut rows);

    // Compute ranking scores
    score::compute_scores(&mut rows, &config);

    // Sort by selected mode (descending - best first)
    match cli.sort {
        SortMode::Pleasing => {
            rows.sort_by(|a, b| b.pleasing_score.total_cmp(&a.pleasing_score));
        }
        SortMode::Best => {
            rows.sort_by(|a, b| b.best_score.total_cmp(&a.best_score));
        }
    }

    output::print_rows(&rows, cli.json, config.has_extended())?;

    Ok(())
}
