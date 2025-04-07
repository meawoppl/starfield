//! Tool for filtering Gaia catalog files to keep only bright stars
//!
//! This utility filters Gaia catalog files to keep only stars brighter than
//! a specified magnitude threshold (default: 20.0) and saves a smaller file
//! containing only essential fields: source_id, ra, dec, and phot_g_mean_mag.
//!
//! Usage:
//!   cargo run --example gaia_filter -- [options]
//!
//! Options:
//!   --input PATH       Input Gaia catalog file (CSV or gzipped CSV)
//!   --output PATH      Output file path (.bin extension)
//!   --magnitude FLOAT  Maximum magnitude threshold (default: 20.0)
//!   --list             List cached Gaia files
//!   --all              Process all cached Gaia files
//!   --max-files NUM    Maximum number of files to process when using --all

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use starfield::catalogs::{BinaryCatalog, StarData};
use starfield::data::list_cached_gaia_files;

/// Parses CSV headers to find column indices for required fields
fn parse_headers(
    headers: &[&str],
) -> Result<(usize, usize, usize, usize), Box<dyn std::error::Error>> {
    let find_column = |name: &str| -> Result<usize, Box<dyn std::error::Error>> {
        headers
            .iter()
            .position(|&h| h == name)
            .ok_or_else(|| format!("Missing column: {}", name).into())
    };

    // Find required column indices
    let source_id_idx = find_column("source_id")?;
    let ra_idx = find_column("ra")?;
    let dec_idx = find_column("dec")?;
    let g_mag_idx = find_column("phot_g_mean_mag")?;

    Ok((source_id_idx, ra_idx, dec_idx, g_mag_idx))
}

/// Create a reader for a file, handling gzip if needed
fn create_reader(path: &Path) -> Result<Box<dyn BufRead>, Box<dyn std::error::Error>> {
    let input_file = File::open(path)?;

    // Determine if the file is gzipped or not
    let path_str = path.to_string_lossy().to_string();
    let is_gzipped = path_str.ends_with(".gz");

    // Create appropriate reader
    let reader: Box<dyn BufRead> = if is_gzipped {
        println!("Detected gzipped file, decompressing...");
        let decoder = GzDecoder::new(input_file);
        Box::new(BufReader::new(decoder))
    } else {
        Box::new(BufReader::new(input_file))
    };

    Ok(reader)
}

/// Creates an iterator over StarData from a CSV reader
struct GaiaFileIterator<R: BufRead> {
    reader: R,
    source_id_idx: usize,
    ra_idx: usize,
    dec_idx: usize,
    g_mag_idx: usize,
    magnitude_limit: f64,
    processed_lines: usize,
    next_progress_marker: usize,
    file_path: String,
}

impl<R: BufRead> GaiaFileIterator<R> {
    fn new(
        reader: R,
        source_id_idx: usize,
        ra_idx: usize,
        dec_idx: usize,
        g_mag_idx: usize,
        magnitude_limit: f64,
        file_path: String,
    ) -> Self {
        Self {
            reader,
            source_id_idx,
            ra_idx,
            dec_idx,
            g_mag_idx,
            magnitude_limit,
            processed_lines: 0,
            next_progress_marker: 100000,
            file_path,
        }
    }
}

impl<R: BufRead> Iterator for GaiaFileIterator<R> {
    type Item = StarData;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();

        // Read lines until we find a valid star or reach EOF
        loop {
            line.clear();
            match self.reader.read_line(&mut line) {
                Ok(0) => return None, // End of file
                Ok(_) => {
                    self.processed_lines += 1;

                    // Show progress
                    if self.processed_lines >= self.next_progress_marker {
                        println!(
                            "Processing {}: {} lines processed",
                            self.file_path, self.processed_lines
                        );
                        self.next_progress_marker += 100000;
                    }

                    // Skip empty lines
                    if line.trim().is_empty() {
                        continue;
                    }

                    // Split line into fields
                    let fields: Vec<&str> = line.trim().split(',').collect();

                    // Skip lines with insufficient fields
                    if fields.len() <= self.g_mag_idx
                        || fields.len() <= self.ra_idx
                        || fields.len() <= self.dec_idx
                        || fields.len() <= self.source_id_idx
                    {
                        continue;
                    }

                    // Parse the magnitude first for early filtering
                    let g_mag = match fields[self.g_mag_idx].parse::<f64>() {
                        Ok(mag) => mag,
                        Err(_) => continue,
                    };

                    // Skip stars fainter than magnitude limit
                    if g_mag > self.magnitude_limit {
                        continue;
                    }

                    // Parse required fields
                    let source_id = match fields[self.source_id_idx].parse::<u64>() {
                        Ok(id) => id,
                        Err(_) => continue,
                    };

                    let ra = match fields[self.ra_idx].parse::<f64>() {
                        Ok(ra) => ra,
                        Err(_) => continue,
                    };

                    let dec = match fields[self.dec_idx].parse::<f64>() {
                        Ok(dec) => dec,
                        Err(_) => continue,
                    };

                    // Return a valid star
                    return Some(StarData::new(source_id, ra, dec, g_mag, None));
                }
                Err(_) => continue, // Skip error lines
            }
        }
    }
}

/// Process a single Gaia catalog file, returning an iterator over StarData
fn process_file<P: AsRef<Path>>(
    input_path: P,
    magnitude_limit: f64,
) -> Result<Box<dyn Iterator<Item = StarData>>, Box<dyn std::error::Error>> {
    println!(
        "Processing Gaia catalog file: {}",
        input_path.as_ref().display()
    );

    // Create reader for the file
    let mut reader = create_reader(input_path.as_ref())?;

    // Read header line to determine column positions
    let mut header = String::new();
    reader.read_line(&mut header)?;

    // Parse header to find column indices
    let headers: Vec<&str> = header.trim().split(',').collect();
    let (source_id_idx, ra_idx, dec_idx, g_mag_idx) = parse_headers(&headers)?;

    // Create iterator over the file's stars
    let file_path = input_path.as_ref().display().to_string();
    let iterator = GaiaFileIterator::new(
        reader,
        source_id_idx,
        ra_idx,
        dec_idx,
        g_mag_idx,
        magnitude_limit,
        file_path,
    );

    Ok(Box::new(iterator))
}

/// Create an iterator that processes files sequentially
struct SequentialFileProcessor {
    pending_files: Vec<PathBuf>,
    current_iterator: Option<Box<dyn Iterator<Item = StarData>>>,
    magnitude_limit: f64,
}

impl SequentialFileProcessor {
    fn new(files: Vec<PathBuf>, magnitude_limit: f64) -> Self {
        Self {
            pending_files: files,
            current_iterator: None,
            magnitude_limit,
        }
    }

    fn load_next_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(next_path) = self.pending_files.pop() {
            self.current_iterator = Some(process_file(next_path, self.magnitude_limit)?);
            Ok(())
        } else {
            // No more files to load
            self.current_iterator = None;
            Ok(())
        }
    }
}

impl Iterator for SequentialFileProcessor {
    type Item = StarData;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If we have a current iterator, try to get next item
            if let Some(iter) = self.current_iterator.as_mut() {
                if let Some(star) = iter.next() {
                    return Some(star);
                }
            }

            // Current iterator exhausted or not initialized, try loading next file
            match self.load_next_file() {
                Ok(()) if self.current_iterator.is_some() => {
                    // Successfully loaded next file, continue to try getting items
                    continue;
                }
                _ => {
                    // No more files or error loading
                    return None;
                }
            }
        }
    }
}

/// Process multiple files sequentially, opening only one file at a time
fn process_multiple_files(
    input_files: Vec<PathBuf>,
    magnitude_limit: f64,
) -> Result<Box<dyn Iterator<Item = StarData>>, Box<dyn std::error::Error>> {
    let total_files = input_files.len();

    if total_files == 0 {
        return Err("No input files provided".into());
    }

    println!("Processing {} Gaia catalog files", total_files);

    // Reverse the files so we can pop from the end efficiently
    let mut files: Vec<PathBuf> = input_files.into_iter().collect();
    files.reverse();

    // Create sequential processor that will open files one at a time
    let processor = SequentialFileProcessor::new(files, magnitude_limit);

    Ok(Box::new(processor))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // Default values
    let mut input_path = None;
    let mut output_path = None;
    let mut magnitude_limit = 20.0;
    let mut list_files = false;
    let mut all_files = false;
    let mut max_files = None;

    // Parse command-line arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                if i + 1 < args.len() {
                    input_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --input".into());
                }
            }
            "--output" => {
                if i + 1 < args.len() {
                    output_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --output".into());
                }
            }
            "--magnitude" => {
                if i + 1 < args.len() {
                    magnitude_limit = args[i + 1].parse()?;
                    i += 2;
                } else {
                    return Err("Missing value for --magnitude".into());
                }
            }
            "--list" => {
                list_files = true;
                i += 1;
            }
            "--all" => {
                all_files = true;
                i += 1;
            }
            "--max-files" => {
                if i + 1 < args.len() {
                    max_files = Some(args[i + 1].parse()?);
                    i += 2;
                } else {
                    return Err("Missing value for --max-files".into());
                }
            }
            _ => {
                println!("Unknown argument: {}", args[i]);
                i += 1;
            }
        }
    }

    println!("Gaia Catalog Filter Tool");
    println!("=======================");

    if list_files {
        println!("Listing cached Gaia catalog files:");
        let files = list_cached_gaia_files()?;

        if files.is_empty() {
            println!("No files found in cache.");
        } else {
            for (i, path) in files.iter().enumerate() {
                println!("  {}. {}", i + 1, path.display());
            }
            println!("Total: {} files", files.len());
        }
        return Ok(());
    }

    // Check if output path is provided
    if output_path.is_none() {
        println!("Usage:");
        println!("  cargo run --example gaia_filter -- --input <input_file> --output <output_file> [--magnitude <limit>]");
        println!("  cargo run --example gaia_filter -- --all --output <output_file> [--magnitude <limit>] [--max-files <num>]");
        println!("  cargo run --example gaia_filter -- --list");
        println!();
        println!("Options:");
        println!("  --input PATH       Input Gaia catalog file (CSV or gzipped CSV)");
        println!("  --output PATH      Output file path (.bin extension)");
        println!("  --magnitude FLOAT  Maximum magnitude threshold (default: 20.0)");
        println!("  --list             List cached Gaia files");
        println!("  --all              Process all cached Gaia files");
        println!("  --max-files NUM    Maximum number of files to process when using --all");

        return Err("Missing required arguments".into());
    }

    let output_file = output_path.unwrap();
    if !output_file.ends_with(".bin") {
        return Err("Output file must have .bin extension".into());
    }

    // Get iterator over stars based on input mode
    let star_iterator = if all_files {
        // Process all cached files (or up to max_files)
        let mut files = list_cached_gaia_files()?;

        if files.is_empty() {
            return Err("No Gaia files found in cache.".into());
        }

        // Limit number of files if max_files is specified
        if let Some(limit) = max_files {
            if limit < files.len() {
                println!(
                    "Limiting to {} files (out of {} available)",
                    limit,
                    files.len()
                );
                files.truncate(limit);
            }
        }

        // Process the files
        Box::new(process_multiple_files(files, magnitude_limit)?)
            as Box<dyn Iterator<Item = StarData>>
    } else if let Some(input) = input_path {
        // Process single file
        Box::new(process_file(input, magnitude_limit)?) as Box<dyn Iterator<Item = StarData>>
    } else {
        return Err("Must specify either --input or --all".into());
    };

    // Create description for catalog
    let desc = format!("Gaia catalog filtered to magnitude {}", magnitude_limit);

    // Stream-write stars to binary catalog file
    println!("Streaming stars to binary catalog file...");

    // Use a counting adapter to monitor progress
    let mut count = 0;
    let counting_iterator = star_iterator.inspect(|_| {
        count += 1;
        if count % 10000 == 0 {
            println!("  Processed {} stars", count);
        }
    });

    // Write catalog directly from iterator
    let final_count =
        BinaryCatalog::write_from_star_data(&output_file, counting_iterator, &desc, None)?;

    println!("Completed filtering:");
    println!(
        "  Kept {} stars with magnitude <= {}",
        final_count, magnitude_limit
    );
    println!("  Output written to: {}", output_file);

    Ok(())
}
