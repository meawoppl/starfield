# Starfield

Star catalog and celestial mechanics calculations inspired by Skyfield.

## Features

- Celestial coordinate transformations
- Star catalog management (Hipparcos, GAIA)
- Precession, nutation, and earth rotation calculations
- Time and date handling for astronomical applications
- Synthetic catalog generation for testing

## Installation

```bash
cargo add starfield
```

## Example

```rust
use starfield::time::Time;
use starfield::catalogs::hipparcos::HipparcosCatalog;
use starfield::catalogs::StarCatalog;

fn main() {
    // Create a synthetic Hipparcos catalog for testing
    let catalog = HipparcosCatalog::create_synthetic();
    
    // Get current time
    let time = Time::now();
    
    // Find bright stars
    let bright_stars = catalog.brighter_than(3.0);
    
    println!("Found {} bright stars at {}", bright_stars.len(), time);
    
    // Print the brightest star information
    if let Some(brightest) = catalog.stars().min_by(|a, b| a.mag.partial_cmp(&b.mag).unwrap()) {
        println!(
            "Brightest star: HIP {} (magnitude {:.2})",
            brightest.hip, brightest.mag
        );
    }
}
```

## Command Line Tools

The package includes command-line tools for working with star catalogs:

```bash
# Basic catalog statistics
cargo stats --catalog hipparcos --operation stats

# Filter a catalog by magnitude and save it
cargo stats --catalog hipparcos --operation filter --magnitude 6.0 --output bright_stars.bin

# Download Gaia catalog data
cargo run --example gaia_downloader -- --download 1

# Filter Gaia data by magnitude and export to binary format
cargo run --example gaia_filter -- --input /path/to/gaia_file.csv.gz --output filtered_stars.bin --magnitude 18.0
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.