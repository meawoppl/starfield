//! Star Catalog Statistics CLI
//!
//! A unified CLI tool for loading, analyzing, and visualizing star catalog data
//! from multiple catalog sources (Hipparcos, Gaia, Binary formats).
//!
//! Features:
//! - Star magnitude histogram
//! - RA/Dec density map (ASCII art visualization)
//! - Statistical analysis of star properties
//! - Support for multiple catalog formats

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::Instant;

use flate2::read::GzDecoder;

use starfield::catalogs::{BinaryCatalog, StarCatalog, StarPosition};
use starfield::data::list_cached_gaia_files;
use starfield::Loader;
use viz::density_map;
use viz::histogram::{Histogram, HistogramConfig, Scale};

/// Print a simple progress bar
fn _print_progress(progress: f64, width: usize) {
    let filled_width = (progress * width as f64).round() as usize;
    let empty_width = width - filled_width;

    print!("\r[");
    for _ in 0..filled_width {
        print!("#");
    }
    for _ in 0..empty_width {
        print!(" ");
    }
    print!("] {:.1}%", progress * 100.0);
    io::stdout().flush().unwrap();
}

// Implementation moved to the viz::density_map module

/// Process a Gaia catalog file directly, streaming through it
fn process_gaia_file(
    file_path: &PathBuf,
    mag_histogram: &mut Histogram<f64>,
    density_grid: &mut [Vec<u32>],
    grid_width: usize,
    grid_height: usize,
    magnitude_limit: f64,
) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    println!("Processing {}...", file_path.display());

    // Open file
    let file = File::open(file_path)?;

    // Create appropriate reader based on file extension
    let is_gzipped = file_path.to_string_lossy().ends_with(".gz");
    let reader: Box<dyn BufRead> = if is_gzipped {
        Box::new(BufReader::new(GzDecoder::new(file)))
    } else {
        Box::new(BufReader::new(file))
    };

    // Process the file line by line
    let mut lines_iter = reader.lines();

    // Read header to find column positions
    let header = match lines_iter.next() {
        Some(Ok(line)) => line,
        _ => return Err("Failed to read header from Gaia file".into()),
    };

    // Parse header
    let headers: Vec<&str> = header.split(',').collect();
    let find_column = |name: &str| -> Result<usize, Box<dyn std::error::Error>> {
        headers
            .iter()
            .position(|&h| h == name)
            .ok_or_else(|| format!("Missing column: {}", name).into())
    };

    // Find required column indices
    let g_mag_idx = find_column("phot_g_mean_mag")?;
    let ra_idx = find_column("ra")?;
    let dec_idx = find_column("dec")?;

    let mut processed_lines = 0;
    let mut kept_stars = 0;
    let mut progress_marker = 100000;

    // Process data lines
    for line_result in lines_iter {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error reading line: {}", e);
                continue;
            }
        };

        processed_lines += 1;

        // Show progress
        if processed_lines >= progress_marker {
            println!(
                "Processed {} lines, kept {} stars",
                processed_lines, kept_stars
            );
            progress_marker += 100000;
        }

        if line.trim().is_empty() {
            continue;
        }

        // Split line into fields
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < headers.len() {
            continue; // Skip lines with insufficient columns
        }

        // Parse the magnitude
        let g_mag = match fields[g_mag_idx].parse::<f64>() {
            Ok(mag) => mag,
            Err(_) => continue,
        };

        // Skip stars fainter than magnitude limit
        if g_mag > magnitude_limit {
            continue;
        }

        // Parse coordinates
        let ra = match fields[ra_idx].parse::<f64>() {
            Ok(val) => val,
            Err(_) => continue,
        };

        let dec = match fields[dec_idx].parse::<f64>() {
            Ok(val) => val,
            Err(_) => continue,
        };

        // Add to magnitude histogram
        mag_histogram.add(g_mag);

        // Add to density grid
        let x = ((ra / 360.0) * grid_width as f64) as usize % grid_width;
        let y = ((90.0 - dec) / 180.0 * grid_height as f64) as usize;
        let y = y.min(grid_height - 1);
        density_grid[y][x] += 1;

        kept_stars += 1;
    }

    Ok((processed_lines, kept_stars))
}

/// Display help message
fn print_help() {
    println!("Star Catalog Statistics Tool");
    println!("===========================");
    println!("Usage: cargo run --bin star_catalog_stats -- [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -t, --type TYPE       Catalog type: hipparcos, gaia, or binary (required)");
    println!("  -f, --file PATH       Input file path (for binary catalog or specific file)");
    println!("  -m, --magnitude MAG   Magnitude limit (default: 6.0 for Hipparcos, 20.0 for Gaia)");
    println!("  -w, --width N         Width of the density map (default: 100)");
    println!("  -h, --height N        Height of the density map (default: 40)");
    println!("  --histogram-bins N    Number of bins for magnitude histogram (default: 40)");
    println!("  --help                Show this help message");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    // Parse command line args
    let args: Vec<String> = env::args().collect();

    // Default parameters
    let mut catalog_type = None;
    let mut file_path = None;
    let mut magnitude_limit = None;
    let mut density_width = 100;
    let mut density_height = 40;
    let mut histogram_bins = 40;

    // Parse command-line arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-t" | "--type" => {
                if i + 1 < args.len() {
                    catalog_type = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --type".into());
                }
            }
            "-f" | "--file" => {
                if i + 1 < args.len() {
                    file_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --file".into());
                }
            }
            "-m" | "--magnitude" => {
                if i + 1 < args.len() {
                    magnitude_limit = Some(args[i + 1].parse()?);
                    i += 2;
                } else {
                    return Err("Missing value for --magnitude".into());
                }
            }
            "-w" | "--width" => {
                if i + 1 < args.len() {
                    density_width = args[i + 1].parse()?;
                    i += 2;
                } else {
                    return Err("Missing value for --width".into());
                }
            }
            "-h" | "--height" => {
                if i + 1 < args.len() {
                    density_height = args[i + 1].parse()?;
                    i += 2;
                } else {
                    return Err("Missing value for --height".into());
                }
            }
            "--histogram-bins" => {
                if i + 1 < args.len() {
                    histogram_bins = args[i + 1].parse()?;
                    i += 2;
                } else {
                    return Err("Missing value for --histogram-bins".into());
                }
            }
            "--help" => {
                print_help();
                return Ok(());
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                i += 1;
            }
        }
    }

    // Check if catalog type is provided
    let catalog_type = match catalog_type {
        Some(t) => t,
        None => {
            print_help();
            return Err("Missing required --type argument".into());
        }
    };

    // Set default magnitude limits based on catalog type
    let magnitude_limit = magnitude_limit.unwrap_or(match catalog_type.as_str() {
        "hipparcos" => 6.0,        // Default for Hipparcos (naked eye visibility)
        "gaia" | "binary" => 20.0, // Default for Gaia and binary
        _ => 10.0,
    });

    println!("Star Catalog Statistics CLI");
    println!("==========================");
    println!("Catalog type: {}", catalog_type);
    println!("Magnitude limit: {:.1}", magnitude_limit);
    if let Some(ref path) = file_path {
        println!("Input file: {}", path);
    }
    println!();

    // Create histogram for magnitude distribution
    let min_mag = -2.0;
    let max_mag = magnitude_limit;
    let mut mag_histogram = Histogram::new_equal_bins(min_mag..max_mag, histogram_bins)?;

    // Configure histogram display
    let config = HistogramConfig {
        title: Some(format!("{} Star Magnitude Distribution", catalog_type)),
        max_bar_width: 40,
        show_empty_bins: true,
        ..Default::default()
    };
    mag_histogram = mag_histogram.with_config(config);

    // Create density grid for the sky map
    let mut density_grid = vec![vec![0u32; density_width]; density_height];

    // Process data based on catalog type
    match catalog_type.as_str() {
        "hipparcos" => {
            // Create a loader
            let loader = Loader::new();

            // Load Hipparcos catalog
            println!("Loading Hipparcos catalog...");
            let catalog = loader.load_hipparcos_catalog(magnitude_limit)?;

            println!("Catalog loaded: {} stars", catalog.len());

            // Process all stars for histogram and density map
            for star in catalog.stars() {
                // Add to magnitude histogram
                mag_histogram.add(star.mag);

                // Add to density grid
                let ra_deg = star.ra(); // Use the trait method instead of direct field access
                let dec_deg = star.dec();
                let x = ((ra_deg / 360.0) * density_width as f64) as usize % density_width;
                let y = ((90.0 - dec_deg) / 180.0 * density_height as f64) as usize;
                let y = y.min(density_height - 1);
                density_grid[y][x] += 1;
            }

            // Print statistics
            println!("\nCatalog Information:");
            println!("  Total stars: {}", catalog.len());

            // Find brightest star
            let brightest = catalog
                .stars()
                .min_by(|a, b| a.mag.partial_cmp(&b.mag).unwrap())
                .unwrap();

            println!(
                "  Brightest star: HIP {} (magnitude {:.2})",
                brightest.hip, brightest.mag
            );

            // Count bright stars (visible to naked eye, mag < 6.0)
            let naked_eye_count = catalog.filter(|star| star.mag < 6.0).len();
            println!(
                "  Stars visible to naked eye (mag < 6.0): {}",
                naked_eye_count
            );
        }
        "gaia" => {
            // Check if a specific file is specified
            if let Some(path) = file_path {
                // Process single Gaia file
                process_gaia_file(
                    &PathBuf::from(path),
                    &mut mag_histogram,
                    &mut density_grid,
                    density_width,
                    density_height,
                    magnitude_limit,
                )?;
            } else {
                // Process all cached Gaia files
                println!("Finding cached Gaia files...");
                let gaia_files = list_cached_gaia_files()?;

                if gaia_files.is_empty() {
                    return Err("No Gaia files found. Run gaia_downloader example first.".into());
                }

                println!("Found {} Gaia files to process", gaia_files.len());

                let mut total_processed = 0;
                let mut total_kept = 0;

                for (i, file_path) in gaia_files.iter().enumerate() {
                    println!(
                        "[{}/{}] Processing {}...",
                        i + 1,
                        gaia_files.len(),
                        file_path.display()
                    );

                    match process_gaia_file(
                        file_path,
                        &mut mag_histogram,
                        &mut density_grid,
                        density_width,
                        density_height,
                        magnitude_limit,
                    ) {
                        Ok((processed, kept)) => {
                            total_processed += processed;
                            total_kept += kept;
                        }
                        Err(e) => {
                            println!("  Error processing file: {}", e);
                        }
                    }
                }

                // Print statistics
                println!("\nCatalog Information:");
                println!("  Total lines processed: {}", total_processed);
                println!("  Total stars analyzed: {}", total_kept);
            }
        }
        "binary" => {
            // Check if a file path is provided
            let path = match file_path {
                Some(p) => p,
                None => return Err("Binary catalog requires --file parameter".into()),
            };

            // Load binary catalog
            println!("Loading binary catalog from {}...", path);
            let catalog = BinaryCatalog::load(path)?;

            println!("Catalog loaded: {} stars", catalog.len());
            println!("Catalog description: {}", catalog.description());

            // Process stars for histogram and density map
            for star in catalog.stars() {
                // Add to magnitude histogram
                mag_histogram.add(star.magnitude);

                // Add to density grid
                let ra_deg = star.ra(); // Use the trait method instead of direct field access
                let dec_deg = star.dec();
                let x = ((ra_deg / 360.0) * density_width as f64) as usize % density_width;
                let y = ((90.0 - dec_deg) / 180.0 * density_height as f64) as usize;
                let y = y.min(density_height - 1);
                density_grid[y][x] += 1;
            }

            // Find brightest star
            if let Some(brightest) = catalog
                .stars()
                .iter()
                .min_by(|a, b| a.magnitude.partial_cmp(&b.magnitude).unwrap())
            {
                println!(
                    "  Brightest star: ID {} (magnitude {:.2})",
                    brightest.id, brightest.magnitude
                );
            }

            // Count bright stars (visible to naked eye, mag < 6.0)
            let naked_eye_count = catalog.filter(|star| star.magnitude < 6.0).len();
            println!(
                "  Stars visible to naked eye (mag < 6.0): {}",
                naked_eye_count
            );
        }
        _ => {
            return Err(format!("Unknown catalog type: {}", catalog_type).into());
        }
    }

    // Print magnitude histogram
    println!("\n{}", mag_histogram.format()?);

    // Print log-scaled magnitude histogram
    let log_config = HistogramConfig {
        title: Some(format!(
            "{} Magnitude Distribution (Log Scale)",
            catalog_type
        )),
        scale: Scale::Log10,
        max_bar_width: 40,
        show_empty_bins: true,
        ..Default::default()
    };

    let log_hist = mag_histogram.with_config(log_config);
    println!("\n{}", log_hist.format()?);

    // Get all RA/Dec coordinates for the density map
    let mut ra_dec_points = Vec::new();

    // Process data based on catalog type to extract RA/Dec
    match catalog_type.as_str() {
        "hipparcos" => {
            // Get RA/Dec from Hipparcos data
            let catalog = Loader::new().load_hipparcos_catalog(magnitude_limit)?;
            ra_dec_points = catalog.stars().map(|star| (star.ra, star.dec)).collect();
        }
        "gaia" | "binary" => {
            // Extract RA/Dec from the already processed density grid
            for (y, row) in density_grid.iter().enumerate() {
                for (x, &count) in row.iter().enumerate() {
                    if count > 0 {
                        // Map from density grid to celestial coordinates
                        let ra = (x as f64 / density_width as f64) * 360.0;
                        let dec = 90.0 - ((y as f64 / density_height as f64) * 180.0);

                        // Add the point count times (to preserve density)
                        for _ in 0..count {
                            ra_dec_points.push((ra, dec));
                        }
                    }
                }
            }
        }
        _ => {}
    }

    // Create and print density map using the viz module
    if let Ok(density_map_str) = density_map::create_celestial_density_map(
        &ra_dec_points,
        density_width,
        density_height,
        " .:+*#@",
    ) {
        println!("\n{}", density_map_str);
    }

    let elapsed = start_time.elapsed();
    println!(
        "\nProcessing completed in {:.2} seconds",
        elapsed.as_secs_f64()
    );

    Ok(())
}
