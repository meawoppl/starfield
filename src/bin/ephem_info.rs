//! JPL Ephemeris Information Tool
//!
//! This binary analyzes JPL ephemeris files (SPK/BSP) and prints information about
//! the data they contain, including file format, time coverage, included bodies,
//! and memory usage statistics.
//!
//! Usage:
//!   cargo run --bin ephem_info -- [--comments] [path/to/ephem.bsp]

use std::collections::HashSet;
use std::path::Path;
use std::time::Instant;

use clap::{ArgAction, Parser};
use starfield::jplephem::{
    calendar, names,
    spk::{seconds_to_jd, SPK},
};

/// Type alias for the error type used throughout this module
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// JPL Ephemeris Information Tool
#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "Analyzes and displays information about JPL ephemeris files (SPK/BSP/PCK)",
    long_about = None
)]
struct Args {
    /// Display only file comments
    #[arg(short, long, action = ArgAction::SetTrue)]
    comments: bool,

    /// Display detailed debugging information
    #[arg(short, long, action = ArgAction::SetTrue)]
    debug: bool,

    /// Ephemeris file to analyze
    #[arg(default_value = "src/jplephem/test_data/de421.bsp")]
    filename: String,
}

/// Format bytes as KB, MB, or GB
fn format_size(size_bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size_bytes >= GB {
        format!("{:.2} GB", size_bytes as f64 / GB as f64)
    } else if size_bytes >= MB {
        format!("{:.2} MB", size_bytes as f64 / MB as f64)
    } else if size_bytes >= KB {
        format!("{:.2} KB", size_bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", size_bytes)
    }
}

/// Convert Julian date to ISO format (YYYY-MM-DD)
fn jd_to_iso(jd: f64) -> Result<String> {
    Ok(calendar::format_date(jd))
}

/// Prints a section header with a title and separator line
fn print_section_header(title: &str) {
    println!("\n{}:", title);
    println!("-------------------------------------------------------");
}

/// Helper to print named values in a formatted way
fn print_named_value(name: &str, value: impl std::fmt::Display) {
    println!("{}: {}", name, value);
}

/// Displays file comments from an SPK file
fn display_comments(spk: &mut SPK) -> Result<()> {
    if let Ok(comments) = spk.comments() {
        if !comments.is_empty() {
            print_section_header("File Comments");
            println!("{}", comments);
            println!("-------------------------------------------------------");
        } else {
            println!("\nNo comments found in file.");
        }
    } else {
        println!("\nFailed to read comments from file.");
    }

    Ok(())
}

/// Displays basic information about the file format and structure
fn display_file_format(spk: &SPK) {
    print_section_header("File Format");
    print_named_value("ID Word", &spk.daf.locidw);
    print_named_value("Endian", &format!("{:?}", spk.daf.endian));
    print_named_value("Internal name", &spk.daf.ifname);
    print_named_value(
        "Summary records",
        format!("ND={}, NI={}", spk.daf.nd, spk.daf.ni),
    );
    print_named_value(
        "Record pointers",
        format!(
            "FWARD={}, BWARD={}, FREE={}",
            spk.daf.fward, spk.daf.bward, spk.daf.free
        ),
    );
}

/// Displays segment information in a formatted table
fn display_segments(spk: &SPK) -> Result<(HashSet<i32>, HashSet<i32>)> {
    if spk.segments.is_empty() {
        println!("\nNo valid segments found in the file.");
        return Ok((HashSet::new(), HashSet::new()));
    }

    print_section_header(format!("Segments ({} total)", spk.segments.len()).as_str());
    println!(
        "{:<20} {:<20} {:<15} {:<15} {:<20}",
        "Target", "Center", "Start Date", "End Date", "Duration"
    );
    println!("-------------------------------------------------------");

    // Track earliest and latest dates
    let mut earliest_date = std::f64::MAX;
    let mut latest_date = std::f64::MIN;

    // Track unique bodies
    let mut targets = HashSet::new();
    let mut centers = HashSet::new();

    // Create a sorted copy of the segments
    let mut sorted_segments = spk.segments.clone();

    // Sort segments by center ID first, then by target ID
    sorted_segments.sort_by(|a, b| {
        a.center
            .cmp(&b.center)
            .then_with(|| a.target.cmp(&b.target))
    });

    for segment in &sorted_segments {
        targets.insert(segment.target);
        centers.insert(segment.center);

        let target_name = names::target_name(segment.target).unwrap_or_else(|| "Unknown");
        let center_name = names::target_name(segment.center).unwrap_or_else(|| "Unknown");

        let start_date = jd_to_iso(segment.start_jd)?;
        let end_date = jd_to_iso(segment.end_jd)?;
        let duration_days = segment.end_jd - segment.start_jd;
        let duration_years = duration_days / 365.25;

        println!(
            "{:<20} {:<20} {:<15} {:<15} {:.1} days ({:.1} yr)",
            target_name, center_name, start_date, end_date, duration_days, duration_years
        );

        earliest_date = earliest_date.min(segment.start_jd);
        latest_date = latest_date.max(segment.end_jd);
    }

    // Display time coverage if we have valid dates
    if earliest_date != std::f64::MAX && latest_date != std::f64::MIN {
        display_time_coverage(earliest_date, latest_date)?;
    }

    Ok((targets, centers))
}

/// Displays time coverage information
fn display_time_coverage(earliest_date: f64, latest_date: f64) -> Result<()> {
    let earliest_iso = jd_to_iso(earliest_date)?;
    let latest_iso = jd_to_iso(latest_date)?;
    let total_duration_days = latest_date - earliest_date;
    let total_duration_years = total_duration_days / 365.25;

    print_section_header("Overall Time Coverage");
    print_named_value(
        "Start date",
        format!("{} (JD {:.1})", earliest_iso, earliest_date),
    );
    print_named_value(
        "End date",
        format!("{} (JD {:.1})", latest_iso, latest_date),
    );
    print_named_value(
        "Duration",
        format!(
            "{:.1} days ({:.1} years)",
            total_duration_days, total_duration_years
        ),
    );

    Ok(())
}

/// Displays information about available bodies
fn display_available_bodies(targets: &HashSet<i32>, centers: &HashSet<i32>) {
    print_section_header("Available Bodies");

    // Convert HashSets to sorted vectors for ordered display
    let mut sorted_targets: Vec<i32> = targets.iter().cloned().collect();
    let mut sorted_centers: Vec<i32> = centers.iter().cloned().collect();

    // Sort bodies by ID
    sorted_targets.sort();
    sorted_centers.sort();

    println!("Target bodies ({}):", targets.len());
    for &target in &sorted_targets {
        println!(
            "  - {} (ID: {})",
            names::target_name(target).unwrap_or_else(|| "Unknown"),
            target
        );
    }

    println!("\nCenter bodies ({}):", centers.len());
    for &center in &sorted_centers {
        println!(
            "  - {} (ID: {})",
            names::target_name(center).unwrap_or_else(|| "Unknown"),
            center
        );
    }
}

/// Displays detailed debug information about DAF summaries
fn display_debug_info(spk: &mut SPK) -> Result<()> {
    print_section_header("Debug Information");

    if let Ok(summaries) = spk.daf.summaries() {
        println!("Found {} DAF summaries", summaries.len());

        println!("\nDumping summary values for debugging:");
        for (i, (name, values)) in summaries.iter().enumerate() {
            if values.len() < 8 || values.iter().all(|&v| v == 0.0) {
                continue; // Skip empty summaries
            }

            // Print the raw name bytes
            let name_bytes: Vec<u8> = name.iter().take(20).cloned().collect();
            println!(
                "\nSummary {}: {} values, name bytes: {:?}",
                i + 1,
                values.len(),
                name_bytes
            );

            // Print all summary values
            print!("  Values: ");
            for (j, &value) in values.iter().enumerate() {
                if j > 0 {
                    print!(", ");
                }
                print!("{}={}", j, value);
            }
            println!();

            // Check for Julian date limits
            let start_second = values[0];
            let end_second = values[1];
            let start_jd = seconds_to_jd(start_second);
            let end_jd = seconds_to_jd(end_second);

            println!(
                "  Seconds since J2000: start={}, end={}",
                start_second, end_second
            );
            println!("  Julian dates: start={}, end={}", start_jd, end_jd);
            println!(
                "  Dates: start={}, end={}",
                calendar::format_date(start_jd),
                calendar::format_date(end_jd)
            );

            // Target and center bodies
            if spk.daf.nd == 2 && spk.daf.ni >= 6 {
                let target = values[2] as i32;
                let center = values[3] as i32;
                println!(
                    "  Target: {} ({}), Center: {} ({})",
                    target,
                    names::target_name(target).unwrap_or_else(|| "Unknown"),
                    center,
                    names::target_name(center).unwrap_or_else(|| "Unknown")
                );
            }
        }
    } else {
        println!("Failed to read summaries");
    }

    Ok(())
}

fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    println!("Analyzing JPL Ephemeris file: {}", args.filename);
    println!("-------------------------------------------------------");

    // Get file metadata
    let path = Path::new(&args.filename);
    let metadata = std::fs::metadata(path)?;
    let file_size = metadata.len();

    println!("File size: {}", format_size(file_size));

    // Start timer for performance metrics
    let start_time = Instant::now();

    // Open the ephemeris file
    let mut spk = SPK::open(&args.filename)?;
    println!("File loaded in {:.2?}", start_time.elapsed());

    // If comments-only mode is enabled, just display comments and exit
    if args.comments {
        display_comments(&mut spk)?;
        return Ok(());
    }

    // Display file format information
    display_file_format(&spk);

    // Display segments and get unique body IDs
    let (targets, centers) = display_segments(&spk)?;

    // Display available bodies
    display_available_bodies(&targets, &centers);

    // If debug mode is enabled, show detailed debug information
    if args.debug {
        display_debug_info(&mut spk)?;
    } else if spk.segments.is_empty() {
        // If no segments were found, show comments and debug info anyway
        display_comments(&mut spk)?;
        display_debug_info(&mut spk)?;
    } else {
        // Show comments at the end for normal mode
        display_comments(&mut spk)?;
    }

    // Show total processing time
    let total_elapsed = start_time.elapsed();
    println!("\nTotal analysis time: {:.2?}", total_elapsed);

    Ok(())
}
