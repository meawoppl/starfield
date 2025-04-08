//! Starfield: Rust astronomical calculations library inspired by Python's skyfield
//!
//! This crate provides high-precision astronomical calculations for positions
//! of stars, planets, and other celestial objects.

use crate::catalogs::StarCatalog;
use std::path::Path;
use thiserror::Error;

pub mod almanac;
pub mod catalogs;
pub mod celestial;
pub mod constants;
pub mod coordinates;
pub mod data;
pub mod earthlib;
pub mod errors;
pub mod framelib;
pub mod nutationlib;
pub mod planetlib;
pub mod positions;
pub mod precessionlib;
pub mod time;
pub mod units;

// Re-export commonly used types
pub use coordinates::RaDec;
pub use time::{CalendarTuple, Time, Timescale};

/// Main error type for the starfield library
#[derive(Debug, Error)]
pub enum StarfieldError {
    #[error("Time error: {0}")]
    TimeError(String),

    #[error("Data error: {0}")]
    DataError(String),

    #[error("Calculation error: {0}")]
    CalculationError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Object not found: {0}")]
    ObjectNotFound(String),
}

/// Result type for starfield operations
pub type Result<T> = std::result::Result<T, StarfieldError>;

/// Entry point for loading standard astronomical data
pub struct Loader {
    data_dir: Option<std::path::PathBuf>,
}

impl Loader {
    /// Create a new loader with default data directory
    pub fn new() -> Self {
        Self { data_dir: None }
    }

    /// Set a custom data directory
    pub fn with_data_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.data_dir = Some(path.as_ref().to_path_buf());
        self
    }

    /// Load the Hipparcos star catalog with a specified magnitude limit
    pub fn load_hipparcos_catalog(
        &self,
        magnitude_limit: f64,
    ) -> Result<catalogs::HipparcosCatalog> {
        use crate::data::download_hipparcos;

        // Download/cache the Hipparcos catalog
        let dat_path = download_hipparcos()?;

        // Load the catalog
        catalogs::HipparcosCatalog::from_dat_file(dat_path, magnitude_limit)
    }

    /// Load the Gaia star catalog from a specific file (CSV or gzipped CSV) with a magnitude limit
    pub fn load_gaia_catalog_from_file<P: AsRef<Path>>(
        &self,
        path: P,
        magnitude_limit: f64,
    ) -> Result<catalogs::GaiaCatalog> {
        // Load the catalog from the provided file
        catalogs::GaiaCatalog::from_file(path, magnitude_limit)
    }

    /// Load the Gaia star catalog from all cached files (CSV or gzipped CSV) with a magnitude limit
    pub fn load_gaia_catalog(&self, magnitude_limit: f64) -> Result<catalogs::GaiaCatalog> {
        use crate::data::list_cached_gaia_files;

        // Get list of all cached Gaia files
        let files = list_cached_gaia_files()?;

        if files.is_empty() {
            return Err(StarfieldError::DataError(
                "No Gaia catalog files found in cache. Use the gaia_downloader tool to download them.".to_string()
            ));
        }

        println!("Loading Gaia catalog from {} cached files...", files.len());

        // Load the first file to initialize the catalog
        let mut catalog = self.load_gaia_catalog_from_file(&files[0], magnitude_limit)?;

        // Load the rest of the files and merge them into the catalog
        for file in files.iter().skip(1) {
            println!("Loading additional file: {}", file.display());
            let additional_catalog = self.load_gaia_catalog_from_file(file, magnitude_limit)?;
            catalog.merge(additional_catalog)?;
        }

        println!(
            "Successfully loaded Gaia catalog with {} stars",
            catalog.len()
        );
        Ok(catalog)
    }

    /// Load the Gaia catalog in synthetic mode (for testing or when real data is unavailable)
    pub fn load_synthetic_gaia_catalog(&self) -> catalogs::GaiaCatalog {
        catalogs::GaiaCatalog::create_synthetic()
    }

    /// Load planetary ephemeris
    pub fn load_ephemeris(&self) -> Result<planetlib::Ephemeris> {
        // This is a placeholder until we implement proper ephemeris calculations
        Ok(planetlib::Ephemeris::new())
    }

    /// Load a timescale for time conversions
    pub fn timescale(&self) -> time::Timescale {
        // For now, we return a default timescale with basic data
        // In the future, this could load delta_t data and leap second files
        time::Timescale::default()
    }
}

/// A central object representing the solar system
pub struct Starfield {
    // Implementation would mirror skyfield's Skyfield class
}

impl Default for Loader {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export CelestialObject trait from celestial module
pub use celestial::CelestialObject;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalogs::StarCatalog;

    // Skip this test in CI as it requires downloading data
    #[test]
    #[ignore]
    fn test_synthetic_hip_catalog() {
        // This test uses our synthetic catalog data that mimics Hipparcos format

        // Create a loader
        let loader = Loader::new();

        // Load the synthetic Hipparcos catalog with magnitude 6.0 (naked eye visibility)
        let catalog = loader
            .load_hipparcos_catalog(6.0)
            .expect("Failed to load synthetic catalog");

        // Print the number of stars
        let star_count = catalog.len();
        println!(
            "Loaded {} stars from synthetic Hipparcos catalog",
            star_count
        );

        // Verify we have a reasonable number of stars (should be around 5000)
        assert!(star_count > 1000, "Too few stars loaded: {}", star_count);

        // Check for some well-known bright stars we explicitly added to the synthetic data
        let sirius = catalog.get_star(32349); // Sirius
        assert!(sirius.is_some(), "Sirius not found in catalog");
        println!("Found Sirius: {:?}", sirius.unwrap());

        let vega = catalog.get_star(91262); // Vega
        assert!(vega.is_some(), "Vega not found in catalog");

        // Verify magnitude filtering works
        let bright_stars = catalog.brighter_than(1.0);
        println!("Stars brighter than magnitude 1.0: {}", bright_stars.len());
        assert!(!bright_stars.is_empty(), "No bright stars found");

        // Test filtering by sky region (Orion's belt)
        let orion_belt_stars = catalog.filter(|star| {
            (star.ra >= 80.0 && star.ra <= 85.0) && (star.dec >= -2.0 && star.dec <= 0.0)
        });

        println!(
            "Found {} stars in Orion's belt region",
            orion_belt_stars.len()
        );
        assert!(
            !orion_belt_stars.is_empty(),
            "No stars found in Orion's belt region"
        );
    }

    #[test]
    fn test_synthetic_hipparcos() {
        // Instead of downloading the catalog, we'll use a synthetic one for testing
        use crate::catalogs::hipparcos::HipparcosCatalog;

        // Create a synthetic catalog
        let catalog = HipparcosCatalog::create_synthetic();

        // Print the number of stars
        let star_count = catalog.len();
        println!("Loaded {} stars from Hipparcos catalog", star_count);

        // Verify we have a reasonable number of stars
        // The Hipparcos catalog has about 118,000 stars total, but we're limiting by magnitude
        assert!(star_count > 1000, "Too few stars loaded: {}", star_count);

        // Verify some bright stars are present
        // Sirius (HIP 32349) - The brightest star in the night sky
        let sirius = catalog.get_star(32349);
        assert!(sirius.is_some(), "Sirius not found in catalog");

        // Vega (HIP 91262)
        let vega = catalog.get_star(91262);
        assert!(vega.is_some(), "Vega not found in catalog");

        // Test magnitude filtering
        let bright_stars = catalog.brighter_than(1.0);
        println!("Stars brighter than magnitude 1.0: {}", bright_stars.len());
        assert!(!bright_stars.is_empty(), "No bright stars found");
    }
}
