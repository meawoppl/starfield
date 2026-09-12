//! Downloader module for retrieving astronomical data
//!
//! This module handles downloading and caching of astronomical data files.

use std::env;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::Result;
use crate::StarfieldError;

use indicatif::{ProgressBar, ProgressStyle};

// Hipparcos catalog URL
const HIPPARCOS_URL: &str = "https://cdsarc.cds.unistra.fr/ftp/cats/I/239/hip_main.dat";

/// Base URL for JPL planetary ephemeris BSP files
const JPL_BSP_URL: &str = "https://ssd.jpl.nasa.gov/ftp/eph/planets/bsp/";

/// Base URL for NAIF satellite SPK files
const NAIF_SATELLITES_URL: &str =
    "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/satellites/";

/// Base URL for NAIF generic PCK kernels
///
/// Holds both the text kernels (`.tpc`, e.g. `pck00011.tpc`, which carries the
/// IAU/WGCCRE radii and rotational elements) and the binary kernels (`.bpc`,
/// e.g. `moon_pa_de440_200625.bpc` and `earth_latest_high_prec.bpc`).
pub const NAIF_PCK_URL: &str = "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/";

/// Base URL for NAIF satellite frame kernels
///
/// Holds the text frame kernels (`.tf`) that name the body-fixed frames a
/// binary PCK orients, such as `moon_080317.tf`, which defines
/// `MOON_PA_DE421` and `MOON_ME`.
pub const NAIF_FK_SATELLITES_URL: &str =
    "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/fk/satellites/";

/// Base URL for NAIF leapsecond kernels
///
/// Holds the `.tls` kernels, such as `naif0012.tls`, that carry the TAI-UTC
/// leap second table.
pub const NAIF_LSK_URL: &str = "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/lsk/";

/// Get the cache directory path
pub fn get_cache_dir() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".cache").join("starfield")
}

/// Ensure that the cache directory exists
pub fn ensure_cache_dir() -> io::Result<PathBuf> {
    let cache_dir = get_cache_dir();
    fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir)
}

/// Check if a file exists and is not empty
pub(crate) fn file_exists_and_not_empty<P: AsRef<Path>>(path: P) -> bool {
    match fs::metadata(path) {
        Ok(metadata) => metadata.is_file() && metadata.len() > 0,
        Err(_) => false,
    }
}

/// Download a file from URL to a local path
fn download_file<P: AsRef<Path>>(url: &str, path: P) -> Result<()> {
    // Create parent directories if they don't exist
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent).map_err(StarfieldError::IoError)?;
    }

    // Create a temporary file first to avoid partial downloads
    let temp_path = path.as_ref().with_extension("tmp");
    let mut file = BufWriter::new(File::create(&temp_path).map_err(StarfieldError::IoError)?);

    // Create HTTP client with timeout
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| StarfieldError::DataError(format!("Failed to create HTTP client: {}", e)))?;

    // Make the request
    let mut response = client
        .get(url)
        .send()
        .map_err(|e| StarfieldError::DataError(format!("Failed to download file: {}", e)))?;

    // Check if the request was successful
    if !response.status().is_success() {
        return Err(StarfieldError::DataError(format!(
            "Failed to download file, status: {}",
            response.status()
        )));
    }

    // Copy the response body to the file
    let mut buffer = [0; 8192];
    loop {
        let bytes_read = response
            .read(&mut buffer)
            .map_err(|e| StarfieldError::DataError(format!("Failed to read response: {}", e)))?;

        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])
            .map_err(StarfieldError::IoError)?;
    }

    // Flush and sync the file
    file.flush().map_err(StarfieldError::IoError)?;
    drop(file);

    // Rename the temporary file to the final path
    fs::rename(temp_path, path).map_err(StarfieldError::IoError)?;

    Ok(())
}

/// Decompress a gzipped file
/// Currently unused as we're using synthetic data, but kept for future reference
#[allow(dead_code)]
fn decompress_gzip<P: AsRef<Path>, Q: AsRef<Path>>(gz_path: P, output_path: Q) -> Result<()> {
    let file = File::open(&gz_path).map_err(StarfieldError::IoError)?;

    // Check if file is a valid gzip file (gzip header starts with magic numbers 0x1F 0x8B)
    let mut header = [0u8; 2];
    {
        let mut file_clone = file.try_clone().map_err(StarfieldError::IoError)?;
        if file_clone.read_exact(&mut header).is_err() || header != [0x1F, 0x8B] {
            return Err(StarfieldError::DataError(format!(
                "Invalid gzip file: {:?} is not a valid gzip header",
                header
            )));
        }
    }

    let gz = BufReader::new(file);
    let mut decoder = flate2::read::GzDecoder::new(gz);

    // Try to validate the gzip file by reading a bit
    let mut test_buffer = [0u8; 1024];
    if decoder.read(&mut test_buffer).is_err() {
        // If we get an error, the file might be corrupted
        // Remove the file and return an error
        let _ = fs::remove_file(&gz_path);
        return Err(StarfieldError::DataError(
            "Downloaded file appears to be corrupt. File removed, please try again.".to_string(),
        ));
    }

    // Reset the decoder and actually decompress
    let file = File::open(gz_path).map_err(StarfieldError::IoError)?;
    let gz = BufReader::new(file);
    let mut decoder = flate2::read::GzDecoder::new(gz);

    let output_file = File::create(&output_path).map_err(StarfieldError::IoError)?;
    let mut writer = BufWriter::new(output_file);

    match io::copy(&mut decoder, &mut writer) {
        Ok(_) => {
            writer.flush().map_err(StarfieldError::IoError)?;
            Ok(())
        }
        Err(e) => {
            // Clean up partial files on error
            let _ = fs::remove_file(&output_path);
            Err(StarfieldError::DataError(format!(
                "Failed to decompress file: {}",
                e
            )))
        }
    }
}

/// Resolve the download URL for a known data filename.
///
/// Returns `Some(full_url)` if the filename matches a known pattern,
/// `None` otherwise. Full URLs (containing `://`) pass through unchanged.
///
/// Recognized patterns:
///
/// - `*.bsp` — SPK ephemerides, from JPL, or from the NAIF satellite
///   directory when the name starts with `jup`
/// - `*.tpc`, `*.bpc` — text and binary PCK kernels, from [`NAIF_PCK_URL`]
/// - `*.tf` — text frame kernels, from [`NAIF_FK_SATELLITES_URL`]
/// - `*.tls` — leapsecond kernels, from [`NAIF_LSK_URL`]
pub fn resolve_url(filename: &str) -> Option<String> {
    if filename.contains("://") {
        return Some(filename.to_string());
    }

    if filename.ends_with(".bsp") {
        let base = if filename.starts_with("jup") {
            NAIF_SATELLITES_URL
        } else {
            JPL_BSP_URL
        };
        return Some(format!("{}{}", base, filename));
    }

    if filename.ends_with(".tpc") || filename.ends_with(".bpc") {
        return Some(format!("{}{}", NAIF_PCK_URL, filename));
    }

    if filename.ends_with(".tf") {
        return Some(format!("{}{}", NAIF_FK_SATELLITES_URL, filename));
    }

    if filename.ends_with(".tls") {
        return Some(format!("{}{}", NAIF_LSK_URL, filename));
    }

    None
}

/// The SPDX-ish licence assertion carried by every NAIF and JPL generic
/// kernel: they are US Government works, published without restriction.
/// Mirroring is redistribution, so the datastore asks for it per artifact.
#[cfg(feature = "datastore")]
const KERNEL_LICENSE: &str = "public-domain";

/// One kernel that `starfield-datastore`'s `manifests/ephemeris.toml` already
/// describes, and that the organisation's ephemeris server therefore serves
/// under the key given here. The key, the source URL and the byte count must
/// stay identical to that manifest: they are what the mirror is keyed on.
#[cfg(feature = "datastore")]
struct ManifestKernel {
    filename: &'static str,
    key: &'static str,
    url: &'static str,
    bytes: u64,
    description: &'static str,
    citation: Option<&'static str>,
}

/// The kernels named in `starfield-datastore`'s `manifests/ephemeris.toml`.
///
/// Their source URLs are the manifest's, which are not always the ones
/// [`resolve_url`] would build — DE421 is served from the NAIF archive's
/// `a_old_versions` directory there, and from JPL's `ssd` mirror here.
#[cfg(feature = "datastore")]
const MANIFEST_KERNELS: [ManifestKernel; 6] = [
    ManifestKernel {
        filename: "de421.bsp",
        key: "naif/spk/de421.bsp",
        url: "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/a_old_versions/de421.bsp",
        bytes: 16_790_528,
        description: "JPL DE421 planetary and lunar ephemeris, 1900-2050 (NAIF generic kernels)",
        citation: Some(
            "Folkner, W. M., Williams, J. G., Boggs, D. H. (2009). The Planetary and Lunar Ephemeris DE 421. IPN Progress Report 42-178.",
        ),
    },
    ManifestKernel {
        filename: "de440.bsp",
        key: "naif/spk/de440.bsp",
        url: "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440.bsp",
        bytes: 119_799_808,
        description: "JPL DE440 planetary and lunar ephemeris, 1550-2650 (NAIF generic kernels)",
        citation: Some(
            "Park, R. S., Folkner, W. M., Williams, J. G., Boggs, D. H. (2021). The JPL Planetary and Lunar Ephemerides DE440 and DE441. AJ 161:105.",
        ),
    },
    ManifestKernel {
        filename: "pck00011.tpc",
        key: "naif/pck/pck00011.tpc",
        url: "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/pck00011.tpc",
        bytes: 131_226,
        description: "NAIF text PCK: IAU 2015 body orientation and shape constants",
        citation: None,
    },
    ManifestKernel {
        filename: "moon_pa_de421_1900-2050.bpc",
        key: "naif/pck/moon_pa_de421_1900-2050.bpc",
        url: "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/moon_pa_de421_1900-2050.bpc",
        bytes: 1_770_496,
        description: "Binary PCK: lunar principal-axis orientation from DE421, 1900-2050",
        citation: None,
    },
    ManifestKernel {
        filename: "moon_080317.tf",
        key: "naif/fk/moon_080317.tf",
        url: "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/fk/satellites/moon_080317.tf",
        bytes: 21_437,
        description: "Lunar frame kernel: MOON_PA and MOON_ME frame definitions",
        citation: None,
    },
    ManifestKernel {
        filename: "naif0012.tls",
        key: "naif/lsk/naif0012.tls",
        url: "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/lsk/naif0012.tls",
        bytes: 5_257,
        description: "NAIF leap seconds kernel (through 2017-01-01)",
        citation: None,
    },
];

/// The key prefix and content check for a kernel filename, by extension.
///
/// The prefix follows the manifest's scheme — kernel type, not archive
/// directory, so a frame kernel served from `fk/satellites/` is keyed
/// `naif/fk/…`. The check rejects the wrong *kind* of file (an HTML login
/// page or a soft-404 catalogue page under a `.bsp` key); validating the
/// format is the kernel parser's job, which runs afterwards and is better at
/// it. Text kernels may open with blank lines, so their check trims leading
/// whitespace; a DAF identification word sits at bytes 0..8, so the binary
/// checks do not. `NAIF/DAF` is the pre-N0052 identification word, still
/// found in archived kernels that [`crate::jplephem`] reads today.
#[cfg(feature = "datastore")]
fn kernel_kind(filename: &str) -> Option<(&'static str, starfield_datastore::ContentCheck)> {
    use starfield_datastore::ContentCheck;

    let text_kernel = || {
        ContentCheck::magic(
            crate::planetarylib::TEXT_MAGIC_NUMBERS
                .iter()
                .map(|magic| magic.as_bytes().to_vec())
                .collect(),
            true,
        )
    };

    if filename.ends_with(".bsp") {
        return Some((
            "naif/spk/",
            ContentCheck::magic(vec![b"DAF/SPK".to_vec(), b"NAIF/DAF".to_vec()], false),
        ));
    }
    if filename.ends_with(".bpc") {
        return Some((
            "naif/pck/",
            ContentCheck::magic(vec![b"DAF/PCK".to_vec(), b"NAIF/DAF".to_vec()], false),
        ));
    }
    if filename.ends_with(".tpc") {
        return Some(("naif/pck/", text_kernel()));
    }
    if filename.ends_with(".tf") {
        return Some(("naif/fk/", text_kernel()));
    }
    if filename.ends_with(".tls") {
        return Some((
            "naif/lsk/",
            ContentCheck::magic(vec![b"KPL/LSK".to_vec()], true),
        ));
    }
    None
}

/// The datastore artifact for a SPICE kernel filename, or `None` when the
/// name is not a kernel this crate knows how to key.
///
/// This is the same table as [`resolve_url`], one entry wider: a filename maps
/// to a stable archive-shaped key, the source to fetch it from, and the
/// content check that says what shape of bytes may be cached under that key.
/// Keeping the two beside each other keeps one mapping to maintain.
///
/// The six kernels in `starfield-datastore`'s `manifests/ephemeris.toml` are
/// reproduced verbatim — key, source URL, byte count, provenance — because the
/// organisation's ephemeris server serves exactly those keys. Any other kernel
/// name [`resolve_url`] recognises is keyed by the same scheme and sourced
/// from the archive URL that function builds; it will miss the mirror until
/// someone adds it to the manifest.
///
/// Returns `None` for a full URL (there is no stable key for an arbitrary URL)
/// and for a non-kernel file such as the Hipparcos catalogue, both of which
/// keep the direct download path.
///
/// # Example
///
/// ```
/// let artifact = starfield::data::kernel_artifact("de421.bsp").unwrap();
/// assert_eq!(artifact.key.as_str(), "naif/spk/de421.bsp");
/// assert_eq!(artifact.expected_bytes, Some(16_790_528));
/// ```
#[cfg(feature = "datastore")]
pub fn kernel_artifact(filename: &str) -> Option<starfield_datastore::Artifact> {
    use starfield_datastore::{Artifact, ArtifactKey, Freshness, Provenance, Source};

    if filename.contains("://") {
        return None;
    }
    let (prefix, check) = kernel_kind(filename)?;

    let known = MANIFEST_KERNELS
        .iter()
        .find(|kernel| kernel.filename == filename);
    let (key, url, description, citation, bytes) = match known {
        Some(kernel) => (
            kernel.key.to_string(),
            kernel.url.to_string(),
            kernel.description.to_string(),
            kernel.citation.map(str::to_string),
            Some(kernel.bytes),
        ),
        None => (
            format!("{}{}", prefix, filename),
            resolve_url(filename)?,
            format!("NAIF/JPL generic kernel {}", filename),
            None,
            None,
        ),
    };

    let artifact = Artifact::new(ArtifactKey::new(key).ok()?, vec![Source::new(url)])
        .with_check(check)
        .with_freshness(Freshness::Immutable)
        .with_provenance(Provenance {
            description,
            license: KERNEL_LICENSE.to_string(),
            citation,
        });
    Some(match bytes {
        Some(bytes) => artifact.with_expected_bytes(bytes),
        None => artifact,
    })
}

/// Resolve one artifact through a datastore built from `builder`, seeding the
/// cache from a legacy flat file first.
///
/// Before the datastore existed, [`download_or_cache`] wrote kernels straight
/// into `<cache root>/<filename>`. Such a file is imported — validated exactly
/// as a download would be — so that nobody re-downloads a kernel they already
/// have. An import failure is an error rather than a silent refetch: it means
/// the bytes on disk are not the kernel the key names, and the file should be
/// looked at rather than quietly replaced.
#[cfg(feature = "datastore")]
fn datastore_resolve(
    builder: starfield_datastore::DatastoreBuilder,
    artifact: &starfield_datastore::Artifact,
    filename: &str,
) -> Result<PathBuf> {
    let store = builder.build()?;
    let legacy = store.cache_root().join(filename);
    if store.peek(&artifact.key).is_none() && file_exists_and_not_empty(&legacy) {
        store.import(artifact, &legacy).map_err(|e| {
            StarfieldError::DataError(format!(
                "the cached file {} is not a usable {}: {e}",
                legacy.display(),
                artifact.key
            ))
        })?;
    }
    Ok(store.get(artifact)?)
}

/// Download a file from URL to a local path, showing a progress bar.
///
/// Uses a longer timeout (600s) suitable for large ephemeris files.
/// Downloads to a temporary file first, then atomically renames.
pub fn download_file_with_progress<P: AsRef<Path>>(url: &str, path: P) -> Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent).map_err(StarfieldError::IoError)?;
    }

    let temp_path = path.as_ref().with_extension("tmp");
    let mut file = BufWriter::new(File::create(&temp_path).map_err(StarfieldError::IoError)?);

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(600))
        .build()
        .map_err(|e| StarfieldError::DataError(format!("Failed to create HTTP client: {}", e)))?;

    let response = client
        .get(url)
        .send()
        .map_err(|e| StarfieldError::DataError(format!("Failed to download {}: {}", url, e)))?;

    if !response.status().is_success() {
        return Err(StarfieldError::DataError(format!(
            "Download failed for {}: HTTP {}",
            url,
            response.status()
        )));
    }

    let total_size = response.content_length().unwrap_or(0);
    let pb = if total_size > 0 {
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{bar:40}] {percent}% {bytes}/{total_bytes} ({bytes_per_sec})")
                .unwrap()
                .progress_chars("##-"),
        );
        Some(pb)
    } else {
        None
    };

    let mut reader = io::BufReader::new(response);
    let mut downloaded: u64 = 0;
    let mut buffer = [0u8; 131_072]; // 128KB chunks

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .map_err(|e| StarfieldError::DataError(format!("Failed to read response: {}", e)))?;

        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])
            .map_err(StarfieldError::IoError)?;

        downloaded += bytes_read as u64;
        if let Some(ref pb) = pb {
            pb.set_position(downloaded);
        }
    }

    if let Some(ref pb) = pb {
        pb.finish_and_clear();
    }

    file.flush().map_err(StarfieldError::IoError)?;
    drop(file);

    fs::rename(temp_path, path).map_err(StarfieldError::IoError)?;

    Ok(())
}

/// Ensure a data file is available locally, downloading it if necessary.
///
/// Checks `data_dir` (or the default cache `~/.cache/starfield/`) for the file.
/// If not found, resolves the URL from the filename and downloads it.
/// Returns the path to the local file.
///
/// # Kernels and the datastore
///
/// With the `datastore` feature (on by default) a filename that
/// [`kernel_artifact`] recognises is resolved through `starfield-datastore`
/// instead: local cache, then the organisation's mirror, then — only when
/// `STARFIELD_ALLOW_UPSTREAM=1` — the upstream archive. The returned path is
/// then the cache's content-addressed blob, not `<dir>/<filename>`; open it
/// by the path returned rather than rebuilding it. `STARFIELD_MIRROR`,
/// `STARFIELD_ALLOW_UPSTREAM`, `STARFIELD_CACHE_DIR` and
/// `~/.config/starfield/datastore.toml` are honoured as the datastore
/// documents them.
///
/// The strict chain is the default: with no mirror configured and
/// `STARFIELD_ALLOW_UPSTREAM` unset, a cached kernel is served and a miss is
/// an error naming that variable. A kernel left in the old flat layout at
/// `<cache root>/<filename>` is imported into the cache on first use, so
/// nothing already downloaded is downloaded again.
///
/// Everything else — the Hipparcos catalogue, a full URL, an unrecognised
/// name — and every build without the feature takes the direct download path
/// through [`resolve_url`] and [`download_file_with_progress`].
pub fn download_or_cache(filename: &str, data_dir: Option<&Path>) -> Result<PathBuf> {
    #[cfg(feature = "datastore")]
    if let Some(artifact) = kernel_artifact(filename) {
        let builder = starfield_datastore::DatastoreBuilder::from_env()?;
        // An explicit data directory wins; otherwise the datastore resolves
        // its own root, which defaults to the same `~/.cache/starfield`.
        let builder = match data_dir {
            Some(dir) => builder.cache_root(dir.to_path_buf()),
            None => builder,
        };
        return datastore_resolve(builder, &artifact, filename);
    }

    let dir = match data_dir {
        Some(d) => {
            fs::create_dir_all(d).map_err(StarfieldError::IoError)?;
            d.to_path_buf()
        }
        None => ensure_cache_dir().map_err(StarfieldError::IoError)?,
    };

    let local_path = dir.join(filename);

    if file_exists_and_not_empty(&local_path) {
        return Ok(local_path);
    }

    let url = resolve_url(filename).ok_or_else(|| {
        StarfieldError::DataError(format!(
            "Unknown file '{}'. Provide a recognized filename (e.g. de421.bsp) or a full URL.",
            filename
        ))
    })?;

    eprintln!("Downloading {} ...", url);
    download_file_with_progress(&url, &local_path)?;
    eprintln!("Saved to {}", local_path.display());

    Ok(local_path)
}

/// Download the Hipparcos catalog
pub fn download_hipparcos() -> Result<PathBuf> {
    let cache_dir = ensure_cache_dir().map_err(StarfieldError::IoError)?;

    // File paths
    let dat_path = cache_dir.join("hip_main.dat");

    // If the file already exists and is not empty, return its path
    if file_exists_and_not_empty(&dat_path) {
        println!("Using cached Hipparcos catalog from {}", dat_path.display());
        return Ok(dat_path);
    }

    // Check if hip_main.dat exists in the project root (for CI environments)
    let project_root_dat = PathBuf::from("hip_main.dat");
    if file_exists_and_not_empty(&project_root_dat) {
        println!(
            "Using Hipparcos catalog from project root: {}",
            project_root_dat.display()
        );

        // Copy the file to the cache directory
        fs::copy(&project_root_dat, &dat_path).map_err(StarfieldError::IoError)?;
        println!("Copied Hipparcos catalog to cache: {}", dat_path.display());
        return Ok(dat_path);
    }

    // Download the real Hipparcos catalog
    println!("Downloading Hipparcos catalog from {}...", HIPPARCOS_URL);
    println!("This may take a moment as the catalog is approximately 36MB");

    // Attempt to download the file
    match download_file(HIPPARCOS_URL, &dat_path) {
        Ok(_) => {
            println!(
                "Hipparcos catalog downloaded successfully to {}",
                dat_path.display()
            );
            Ok(dat_path)
        }
        Err(e) => {
            // If download fails, we could provide a fallback to synthetic data, but
            // for now we'll just return the error
            println!("Failed to download Hipparcos catalog: {}", e);
            println!("Check your internet connection or try again later.");
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_dir() {
        let cache_dir = get_cache_dir();
        assert!(cache_dir.to_str().unwrap().contains(".cache/starfield"));
    }

    #[test]
    fn test_resolve_url_bsp() {
        assert_eq!(
            resolve_url("de421.bsp"),
            Some("https://ssd.jpl.nasa.gov/ftp/eph/planets/bsp/de421.bsp".to_string())
        );
    }

    #[test]
    fn test_resolve_url_jupiter_bsp() {
        assert_eq!(
            resolve_url("jup365.bsp"),
            Some(
                "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/satellites/jup365.bsp"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_resolve_url_text_pck() {
        assert_eq!(
            resolve_url("pck00011.tpc"),
            Some("https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/pck00011.tpc".to_string())
        );
    }

    #[test]
    fn test_resolve_url_binary_pck() {
        assert_eq!(
            resolve_url("moon_pa_de440_200625.bpc"),
            Some(
                "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/moon_pa_de440_200625.bpc"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_resolve_url_frame_kernel() {
        assert_eq!(
            resolve_url("moon_080317.tf"),
            Some(
                "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/fk/satellites/moon_080317.tf"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_resolve_url_full_url_passthrough() {
        let url = "https://example.com/custom.bsp";
        assert_eq!(resolve_url(url), Some(url.to_string()));
    }

    #[test]
    fn test_resolve_url_unknown() {
        assert_eq!(resolve_url("unknown.xyz"), None);
    }

    #[test]
    fn test_download_or_cache_cached_file() {
        let dir = tempfile::tempdir().unwrap();
        let test_file = dir.path().join("test.bsp");
        std::fs::write(&test_file, b"fake bsp data").unwrap();

        let result = download_or_cache("test.bsp", Some(dir.path()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_file);
    }

    #[test]
    fn test_download_or_cache_unknown_file() {
        let dir = tempfile::tempdir().unwrap();
        let result = download_or_cache("unknown.xyz", Some(dir.path()));
        assert!(result.is_err());
    }

    /// Verify all known download endpoints respond to HEAD requests.
    ///
    /// This catches broken URLs in CI without streaming large files.
    #[test]
    #[ignore = "makes live HEAD requests to JPL and NAIF"]
    fn test_known_endpoints_reachable() {
        let filenames = [
            "de421.bsp",
            "de405.bsp",
            "de430t.bsp",
            "jup365.bsp",
            "pck00011.tpc",
            "moon_pa_de440_200625.bpc",
        ];

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("Failed to build HTTP client");

        for filename in filenames {
            let url = resolve_url(filename).expect("resolve_url returned None");
            let response = client
                .head(&url)
                .send()
                .unwrap_or_else(|e| panic!("HEAD request failed for {}: {}", url, e));

            assert!(
                response.status().is_success(),
                "Endpoint {} returned HTTP {}",
                url,
                response.status()
            );
        }
    }
}
