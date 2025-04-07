//! Tool for analyzing magnitude distributions in Gaia catalog files
//!
//! This example processes downloaded Gaia catalog files and generates
//! a histogram of apparent magnitudes.
//!
//! Usage:
//!   cargo run --example magnitude_histogram

use std::time::Instant;

use starfield::catalogs::{GaiaCatalog, StarCatalog};
use starfield::data::list_cached_gaia_files;
use viz::histogram::{Histogram, HistogramConfig, Scale};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Gaia Magnitude Histogram Generator");
    println!("==================================");

    // Load cached Gaia files
    println!("Finding cached Gaia files...");
    let gaia_files = list_cached_gaia_files()?;

    if gaia_files.is_empty() {
        return Err("No Gaia files found. Run gaia_downloader example first.".into());
    }

    println!("Found {} Gaia files", gaia_files.len());

    // Define magnitude range
    let min_mag = -1.5;
    let max_mag = 21.0;
    let num_bins = 45; // 0.5 magnitude bins

    // Create the histogram
    let mut hist = Histogram::new_equal_bins(min_mag..max_mag, num_bins)?;

    // Set up configuration for standard display
    let mut config = HistogramConfig::default();
    config.title = Some("Gaia Magnitude Distribution".to_string());
    config.max_bar_width = 40;
    config.show_empty_bins = true;
    hist = hist.with_config(config);

    // Process each file
    let start_time = Instant::now();

    let mut total_stars: u64 = 0;

    for (i, file_path) in gaia_files.iter().enumerate() {
        println!(
            "[{}/{}] Processing {}...",
            i + 1,
            gaia_files.len(),
            file_path.display()
        );

        // Load catalog with no magnitude limit (use the highest possible value)
        match GaiaCatalog::from_file(file_path, f64::MAX) {
            Ok(catalog) => {
                // Extract magnitudes and add to histogram
                let magnitudes: Vec<f64> = catalog
                    .stars()
                    .map(|star| star.phot_g_mean_mag)
                    .filter(|&mag| mag >= min_mag && mag < max_mag)
                    .collect();

                // Add all values to histogram
                hist.add_all(magnitudes);
                total_stars += catalog.len() as u64;

                println!("  Processed {} stars", catalog.len());
            }
            Err(e) => {
                println!("  Failed to process file: {}", e);
                continue;
            }
        }
    }

    let elapsed = start_time.elapsed();
    println!(
        "\nProcessing completed in {:.2} seconds",
        elapsed.as_secs_f64()
    );
    println!("Total stars analyzed: {}", total_stars);

    // Get references to counts and bin_edges before we move the histogram
    let counts = hist.counts().to_vec();
    let bin_edges = hist.bin_edges().to_vec();

    // Print standard histogram
    println!("\n{}", hist.format()?);

    // Create log-scaled histogram for better visualization of distribution
    let mut log_config = HistogramConfig::default();
    log_config.title = Some("Gaia Magnitude Distribution (Log Scale)".to_string());
    log_config.scale = Scale::Log10;
    log_config.max_bar_width = 40;
    log_config.show_empty_bins = true;

    let log_hist = hist.with_config(log_config);
    println!("\n{}", log_hist.format()?);

    // Find brightest stars (first non-empty bin)
    if let Some((idx, _)) = counts.iter().enumerate().find(|(_, &count)| count > 0) {
        println!(
            "\nBrightest stars found: magnitude range {:.2} - {:.2}",
            bin_edges[idx],
            bin_edges[idx + 1]
        );
    }

    // Find most common magnitude range
    if let Some(max_idx) = counts
        .iter()
        .enumerate()
        .max_by_key(|(_, &count)| count)
        .map(|(idx, _)| idx)
    {
        println!(
            "Most common magnitude range: {:.2} - {:.2} ({} stars)",
            bin_edges[max_idx],
            bin_edges[max_idx + 1],
            counts[max_idx]
        );
    }

    // Calculate stars visible to naked eye (magnitude < 6.0)
    let naked_eye_idx = bin_edges
        .iter()
        .position(|&edge| edge >= 6.0)
        .unwrap_or(bin_edges.len());

    let naked_eye_stars: u64 = counts.iter().take(naked_eye_idx).sum();
    let total_count: u64 = counts.iter().sum();
    let percentage = (naked_eye_stars as f64 / total_count as f64) * 100.0;

    println!("\nStars brighter than magnitude 6 (visible to naked eye):");
    println!("  {} stars ({:.4}% of total)", naked_eye_stars, percentage);

    Ok(())
}
