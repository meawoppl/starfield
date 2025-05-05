//! JPL Ephemeris Information Tool
//!
//! This binary analyzes JPL ephemeris files (SPK/BSP) and prints information about
//! the data they contain, including file format, time coverage, included bodies,
//! and memory usage statistics.

use std::env;
use std::path::Path;
use std::time::Instant;

use starfield::jplephem::{calendar, names, spk::SPK};

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
fn jd_to_iso(jd: f64) -> Result<String, Box<dyn std::error::Error>> {
    Ok(calendar::format_date(jd))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get filename from command line arguments or use default test file
    let args: Vec<String> = env::args().collect();
    let filename = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("src/jplephem/test_data/de421.bsp");

    println!("Analyzing JPL Ephemeris file: {}", filename);
    println!("-------------------------------------------------------");

    // Get file metadata
    let path = Path::new(filename);
    let metadata = std::fs::metadata(path)?;
    let file_size = metadata.len();
    
    println!("File size: {}", format_size(file_size));

    // Start timer for performance metrics
    let start_time = Instant::now();
    
    // Open the ephemeris file
    let mut spk = SPK::open(filename)?;
    let elapsed = start_time.elapsed();
    println!("File loaded in {:.2?}", elapsed);
    
    // Debug file format and DAF structure
    println!("\nFile Format:");
    println!("-------------------------------------------------------");
    println!("ID Word: {}", spk.daf.locidw);
    println!("Format: {}", spk.daf.locfmt);
    println!("Internal name: {}", spk.daf.locifn);
    println!("Summary records: ND={}, NI={}", spk.daf.nd, spk.daf.ni);
    println!("Record pointers: FWARD={}, BWARD={}, FREE={}", 
             spk.daf.fward, spk.daf.bward, spk.daf.free);
             
    // Try to extract data from summaries
    println!("\nReading DAF Summaries:");
    if let Ok(summaries) = spk.daf.summaries() {
        println!("Found {} summaries", summaries.len());
        
        // Parse the summaries to extract segment information
        let mut segments: Vec<(i32, i32, f64, f64, String, String)> = Vec::new();
        
        // Debug: Dump all summary values to understand the format
        println!("\nDumping all summary values for debugging:");
        for (i, (name, values)) in summaries.iter().enumerate() {
            // Print the raw name bytes to help debug the format
            let name_bytes: Vec<u8> = name.iter().take(20).cloned().collect();
            println!("\nSummary {}: {} values, name bytes: {:?}", i+1, values.len(), name_bytes);
            
            // Print all summary values
            print!("  Values: ");
            for (j, &value) in values.iter().enumerate() {
                if j > 0 { print!(", "); }
                print!("{}={}", j, value);
            }
            println!();
            
            // According to the DAF/SPK spec, the last few integers in the summary record
            // contain various metadata including body IDs
            // For SPK (DE) files, we need the right format for target/center IDs
            if values.len() >= 8 {
                // Try different methods to extract target IDs 
                println!("  Format 1: target={}, center={}", values[values.len() - 2] as i32, values[values.len() - 1] as i32);
                println!("  Format 2: target={}, center={}", values[2] as i32, values[3] as i32);
                
                // Check if there are integer values following the ND double values
                // In an SPK file with ND=2, NI=6, we'd have 2 doubles followed by 6 integers 
                if spk.daf.nd + spk.daf.ni == values.len() as i32 {
                    println!("  Format 3: ini_jd={}, fin_jd={}, target={}, center={}", 
                        values[0], values[1], 
                        values[2 + spk.daf.nd as usize] as i32, 
                        values[3 + spk.daf.nd as usize] as i32);
                }
                
                // If there are strange bit patterns, consider them as potentially mis-interpreted
                if values[values.len() - 2].fract() != 0.0 || values[values.len() - 1].fract() != 0.0 {
                    println!("  Possible bit pattern issue detected!");
                }
            }
            
            // Our original logic
            let body: i32;
            let center: i32;
            let start_jd: f64;
            let end_jd: f64;
            
            // For DE421 with ND=2, NI=6, we expect integers after the first 2 doubles
            if spk.daf.nd == 2 && spk.daf.ni >= 2 {
                start_jd = values[0];
                end_jd = values[1]; 
                // The target and center are likely at specific positions after the doubles
                body = values[spk.daf.nd as usize] as i32;
                center = values[spk.daf.nd as usize + 1] as i32;
            } else {
                // Fall back to the original logic
                body = values[values.len() - 2] as i32;
                center = values[values.len() - 1] as i32;
                start_jd = values[0];
                end_jd = values[1];
            }
                
            println!("  Chosen format: body={}, center={}, start_jd={}, end_jd={}", 
                     body, center, start_jd, end_jd);
            
            // Skip if we have unreasonable JD values or empty body entries
            if start_jd <= 0.0 || end_jd <= 0.0 || (body == 0 && center == 0) || 
                start_jd > 5000000.0 || end_jd > 5000000.0 {
                println!("  Skipping segment with invalid values");
                continue;
            }
                
            // Extract target name if available
            let body_name = names::target_name(body)
                .unwrap_or_else(|| "Unknown");
            
            // Extract center name if available
            let center_name = names::target_name(center)
                .unwrap_or_else(|| "Unknown");
            
            // Add to our list of valid segments
            segments.push((body, center, start_jd, end_jd, body_name.to_string(), center_name.to_string()));
        }
        
        // Display all valid segments in a table
        if !segments.is_empty() {
            println!("\nExtracted Segments ({} total):", segments.len());
            println!("-------------------------------------------------------");
            println!("{:<20} {:<20} {:<15} {:<15} {:<20}", "Target", "Center", "Start Date", "End Date", "Duration");
            println!("-------------------------------------------------------");
            
            // Sort segments by target body first
            segments.sort_by_key(|s| (s.0, s.1));
            
            // Track earliest and latest dates
            let mut earliest_date = std::f64::MAX;
            let mut latest_date = std::f64::MIN;
            
            // Track unique bodies
            let mut targets = std::collections::HashSet::new();
            let mut centers = std::collections::HashSet::new();
            
            for (body, center, start_jd, end_jd, body_name, center_name) in &segments {
                targets.insert(*body);
                centers.insert(*center);
                
                let start_date = jd_to_iso(*start_jd)?;
                let end_date = jd_to_iso(*end_jd)?;
                let duration_days = end_jd - start_jd;
                let duration_years = duration_days / 365.25;
                
                println!(
                    "{:<20} {:<20} {:<15} {:<15} {:.1} days ({:.1} yr)",
                    body_name, center_name, start_date, end_date, duration_days, duration_years
                );
                
                earliest_date = earliest_date.min(*start_jd);
                latest_date = latest_date.max(*end_jd);
            }
            
            // Show overall time coverage
            if earliest_date != std::f64::MAX && latest_date != std::f64::MIN {
                let earliest_iso = jd_to_iso(earliest_date)?;
                let latest_iso = jd_to_iso(latest_date)?;
                let total_duration_days = latest_date - earliest_date;
                let total_duration_years = total_duration_days / 365.25;
                
                println!("\nOverall Time Coverage:");
                println!("-------------------------------------------------------");
                println!("Start date: {} (JD {:.1})", earliest_iso, earliest_date);
                println!("End date:   {} (JD {:.1})", latest_iso, latest_date);
                println!("Duration:   {:.1} days ({:.1} years)", total_duration_days, total_duration_years);
            }
            
            // Display available targets and centers
            println!("\nAvailable Bodies:");
            println!("-------------------------------------------------------");
            
            println!("Target bodies ({}):", targets.len());
            for &target in &targets {
                println!("  - {} (ID: {})", 
                    names::target_name(target).unwrap_or_else(|| "Unknown"), 
                    target);
            }
            
            println!("\nCenter bodies ({}):", centers.len());
            for &center in &centers {
                println!("  - {} (ID: {})", 
                    names::target_name(center).unwrap_or_else(|| "Unknown"), 
                    center);
            }
            
            // We've already displayed the info, so skip the general segment display at the end
            return Ok(());
        }
    } else {
        println!("Failed to read summaries");
    }
    
    // Read comments (if any)
    if let Ok(comments) = spk.comments() {
        if !comments.is_empty() {
            println!("\nFile Comments:");
            println!("-------------------------------------------------------");
            println!("{}", comments);
            println!("-------------------------------------------------------");
        }
    }

    // Display segment information
    println!("\nSegments ({} total):", spk.segments.len());
    println!("-------------------------------------------------------");
    println!("{:<10} {:<10} {:<15} {:<15} {:<20}", "Target", "Center", "Start Date", "End Date", "Duration");
    println!("-------------------------------------------------------");

    // Analyze and display segments
    let mut earliest_date = std::f64::MAX;
    let mut latest_date = std::f64::MIN;
    
    for segment in &spk.segments {
        let target_name = names::target_name(segment.target).unwrap_or_else(|| "Unknown");
        let center_name = names::target_name(segment.center).unwrap_or_else(|| "Unknown");

        let start_date = jd_to_iso(segment.start_jd)?;
        let end_date = jd_to_iso(segment.end_jd)?;
        let duration_days = segment.end_jd - segment.start_jd;
        let duration_years = duration_days / 365.25;
        
        println!(
            "{:<10} {:<10} {:<15} {:<15} {:.1} days ({:.1} years)",
            target_name, center_name, start_date, end_date, duration_days, duration_years
        );
        
        earliest_date = earliest_date.min(segment.start_jd);
        latest_date = latest_date.max(segment.end_jd);
    }

    // Show overall time coverage
    if earliest_date != std::f64::MAX && latest_date != std::f64::MIN {
        let earliest_iso = jd_to_iso(earliest_date)?;
        let latest_iso = jd_to_iso(latest_date)?;
        let total_duration_days = latest_date - earliest_date;
        let total_duration_years = total_duration_days / 365.25;
        
        println!("\nOverall Time Coverage:");
        println!("-------------------------------------------------------");
        println!("Start date: {} (JD {:.1})", earliest_iso, earliest_date);
        println!("End date:   {} (JD {:.1})", latest_iso, latest_date);
        println!("Duration:   {:.1} days ({:.1} years)", total_duration_days, total_duration_years);
    }

    // Display available targets and centers
    let mut targets = std::collections::HashSet::new();
    let mut centers = std::collections::HashSet::new();
    
    for segment in &spk.segments {
        targets.insert(segment.target);
        centers.insert(segment.center);
    }
    
    println!("\nAvailable Bodies:");
    println!("-------------------------------------------------------");
    
    println!("Target bodies ({}):", targets.len());
    for &target in &targets {
        println!("  - {} (ID: {})", names::target_name(target).unwrap_or_else(|| "Unknown"), target);
    }
    
    println!("\nCenter bodies ({}):", centers.len());
    for &center in &centers {
        println!("  - {} (ID: {})", names::target_name(center).unwrap_or_else(|| "Unknown"), center);
    }
    
    // Show total processing time
    let total_elapsed = start_time.elapsed();
    println!("\nTotal analysis time: {:.2?}", total_elapsed);

    Ok(())
}