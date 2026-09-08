//! Planetary constants: body shapes and body-fixed orientation.
//!
//! This module is the Rust counterpart of Skyfield's `planetarylib.py`. It
//! owns two sources of the same information:
//!
//! * [`PlanetaryConstants`], which holds the variables read from a NAIF text
//!   kernel such as `pck00011.tpc` (see [`text_pck`]), and
//! * an embedded, network-free table of the IAU WGCCRE 2015 values for the
//!   Sun, the eight planets, the Moon and Pluto, reachable through
//!   [`body_constants`] and through the accessors on
//!   [`planetlib::Body`](crate::planetlib::Body).
//!
//! Both produce the same [`RotationalElements`] type, so code can start with
//! the embedded table and switch to a downloaded kernel without changing.
//!
//! # Example
//!
//! ```
//! use starfield::planetarylib::body_constants;
//!
//! let mars = body_constants(499).unwrap();
//! assert_eq!(mars.radii, [3396.19, 3396.19, 3376.20]);
//! assert_eq!(mars.elements.pole_ra[0], 317.269202);
//! ```
//!
//! Evaluating the elements at a given time — turning them into a rotation
//! matrix — is the job of the body-fixed frames, which land separately.

#[cfg(all(test, feature = "python-tests"))]
mod python_tests;
pub mod text_pck;

use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

pub use text_pck::KernelValue;

use crate::{Result, StarfieldError};

/// The IAU WGCCRE shape and orientation constants of one body.
///
/// The `nut_prec_*` terms and `nut_prec_angles` together give the periodic
/// corrections that some bodies need; a body without them carries empty
/// vectors.
#[derive(Debug, Clone, PartialEq)]
pub struct RotationalElements {
    /// Right ascension of the north pole: deg, deg/century, deg/century².
    pub pole_ra: [f64; 3],
    /// Declination of the north pole: deg, deg/century, deg/century².
    pub pole_dec: [f64; 3],
    /// Prime meridian angle W: deg, deg/day, deg/day².
    pub pm: [f64; 3],
    /// Amplitudes, in degrees, of the periodic terms added to the pole RA.
    pub nut_prec_ra: Vec<f64>,
    /// Amplitudes, in degrees, of the periodic terms added to the pole
    /// declination.
    pub nut_prec_dec: Vec<f64>,
    /// Amplitudes, in degrees, of the periodic terms added to W.
    pub nut_prec_pm: Vec<f64>,
    /// The nutation and precession angles of the body's barycentre, as
    /// `(deg, deg/century)` pairs. The amplitudes above index into this list.
    pub nut_prec_angles: Vec<(f64, f64)>,
    /// The deg/century² term of each angle in `nut_prec_angles`, in the same
    /// order and of the same length.
    ///
    /// The kernels only supply these when the barycentre declares
    /// `BODYn_MAX_PHASE_DEGREE = 2`, which as of `pck00011.tpc` is true of the
    /// Mars system alone; every other body gets a vector of zeros.
    pub nut_prec_angle_accel: Vec<f64>,
}

/// One row of the embedded IAU 2015 table.
#[derive(Debug, Clone, PartialEq)]
pub struct BodyConstants {
    /// NAIF integer code of the body, such as 499 for Mars.
    pub naif_id: i32,
    /// Body name, matching [`planetlib::Body::name`](crate::planetlib::Body::name).
    pub name: &'static str,
    /// Triaxial ellipsoid radii in km: two equatorial, then polar.
    pub radii: [f64; 3],
    /// Orientation of the body-fixed frame.
    pub elements: RotationalElements,
}

impl BodyConstants {
    /// The mean radius in km: the arithmetic mean of the three ellipsoid axes.
    pub fn mean_radius_km(&self) -> f64 {
        (self.radii[0] + self.radii[1] + self.radii[2]) / 3.0
    }

    /// The flattening `(a − c) / a` of the reference ellipsoid.
    ///
    /// Returns zero for a sphere.
    pub fn flattening(&self) -> f64 {
        (self.radii[0] - self.radii[2]) / self.radii[0]
    }
}

/// The embedded IAU WGCCRE 2015 table, with its provenance header.
const IAU2015_CSV: &str = include_str!("iau2015.csv");

/// The embedded table, parsed once on first use and keyed by NAIF code.
static IAU2015: LazyLock<HashMap<i32, BodyConstants>> = LazyLock::new(|| {
    parse_iau2015(IAU2015_CSV).expect("the embedded IAU 2015 table is well formed")
});

/// Look up a body in the embedded IAU WGCCRE 2015 table by NAIF code.
///
/// Covers the Sun (10), the eight planets (199 … 899), the Moon (301) and
/// Pluto (999); every other code returns `None`.
///
/// # Example
///
/// ```
/// let earth = starfield::planetarylib::body_constants(399).unwrap();
/// assert_eq!(earth.radii, [6378.1366, 6378.1366, 6356.7519]);
/// ```
pub fn body_constants(naif_id: i32) -> Option<&'static BodyConstants> {
    IAU2015.get(&naif_id)
}

/// Every body in the embedded IAU WGCCRE 2015 table, in NAIF code order.
pub fn all_body_constants() -> Vec<&'static BodyConstants> {
    let mut bodies: Vec<&'static BodyConstants> = IAU2015.values().collect();
    bodies.sort_by_key(|b| b.naif_id);
    bodies
}

/// Parse the embedded CSV table.
fn parse_iau2015(csv: &'static str) -> Result<HashMap<i32, BodyConstants>> {
    let mut table = HashMap::new();

    for line in csv.lines() {
        if line.starts_with('#') || line.is_empty() || line.starts_with("naif_id,") {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 12 {
            return Err(StarfieldError::DataError(format!(
                "iau2015.csv: expected 12 fields, found {} in {:?}",
                fields.len(),
                line
            )));
        }

        let naif_id: i32 = fields[0].parse().map_err(|_| {
            StarfieldError::DataError(format!("iau2015.csv: bad NAIF code {:?}", fields[0]))
        })?;
        let radii = [
            parse_field(fields[2])?,
            parse_field(fields[3])?,
            parse_field(fields[4])?,
        ];
        let (nut_prec_angles, nut_prec_angle_accel) = parse_angle_triples(fields[11])?;

        table.insert(
            naif_id,
            BodyConstants {
                naif_id,
                name: fields[1],
                radii,
                elements: RotationalElements {
                    pole_ra: parse_coefficients(fields[5])?,
                    pole_dec: parse_coefficients(fields[6])?,
                    pm: parse_coefficients(fields[7])?,
                    nut_prec_ra: parse_list(fields[8])?,
                    nut_prec_dec: parse_list(fields[9])?,
                    nut_prec_pm: parse_list(fields[10])?,
                    nut_prec_angles,
                    nut_prec_angle_accel,
                },
            },
        );
    }

    Ok(table)
}

/// Parse one number out of the embedded table.
fn parse_field(field: &str) -> Result<f64> {
    field.trim().parse::<f64>().map_err(|_| {
        StarfieldError::DataError(format!("iau2015.csv: cannot parse {:?} as a number", field))
    })
}

/// Parse a space-separated list of numbers out of the embedded table.
fn parse_list(field: &str) -> Result<Vec<f64>> {
    field.split_whitespace().map(parse_field).collect()
}

/// The nutation and precession angles of a barycentre: `(deg, deg/century)`
/// pairs alongside their `deg/century²` terms, one per pair.
type NutPrecAngles = (Vec<(f64, f64)>, Vec<f64>);

/// Parse the flattened `(deg, deg/century, deg/century²)` angle triples.
fn parse_angle_triples(field: &str) -> Result<NutPrecAngles> {
    let flat = parse_list(field)?;
    if flat.len() % 3 != 0 {
        return Err(StarfieldError::DataError(format!(
            "iau2015.csv: {} angle terms is not a whole number of triples",
            flat.len()
        )));
    }
    let angles = flat.chunks(3).map(|c| (c[0], c[1])).collect();
    let accel = flat.chunks(3).map(|c| c[2]).collect();
    Ok((angles, accel))
}

/// Parse a three-term polynomial out of the embedded table.
fn parse_coefficients(field: &str) -> Result<[f64; 3]> {
    let values = parse_list(field)?;
    coefficients(&values).ok_or_else(|| {
        StarfieldError::DataError(format!(
            "iau2015.csv: expected at most 3 polynomial terms, found {}",
            values.len()
        ))
    })
}

/// Widen a polynomial of up to three terms to a fixed-size array.
fn coefficients(values: &[f64]) -> Option<[f64; 3]> {
    if values.len() > 3 {
        return None;
    }
    let mut out = [0.0; 3];
    out[..values.len()].copy_from_slice(values);
    Some(out)
}

/// The magic numbers that open a NAIF text kernel.
const TEXT_MAGIC_NUMBERS: [&str; 2] = ["KPL/FK", "KPL/PCK"];

/// The variables read from one or more NAIF text kernels.
///
/// This mirrors Skyfield's `PlanetaryConstants`: text kernels are read into
/// the [`variables`](Self::variables) map, and the accessors pick the
/// body-related entries back out of it.
///
/// # Example
///
/// ```
/// use starfield::planetarylib::PlanetaryConstants;
///
/// let mut pc = PlanetaryConstants::new();
/// pc.read_text("KPL/PCK\n\\begindata\nBODY499_RADII = ( 3396.19 3396.19 3376.20 )\n\\begintext\n")
///     .unwrap();
/// assert_eq!(pc.radii(499), Some([3396.19, 3396.19, 3376.20]));
/// ```
#[derive(Debug, Clone, Default)]
pub struct PlanetaryConstants {
    /// Every name assigned by the kernels read so far.
    pub variables: HashMap<String, KernelValue>,
}

impl PlanetaryConstants {
    /// An empty set of constants.
    pub fn new() -> Self {
        Self::default()
    }

    /// Read the text of a NAIF text kernel, merging it into `variables`.
    ///
    /// The text must begin with `KPL/PCK` or `KPL/FK`, the magic numbers that
    /// mark a text kernel; anything else is rejected rather than silently
    /// parsed as an empty kernel.
    ///
    /// # Errors
    ///
    /// Returns [`StarfieldError::DataError`] if the magic number is missing or
    /// if the kernel is malformed.
    pub fn read_text(&mut self, text: &str) -> Result<()> {
        let head = text.trim_start();
        if !TEXT_MAGIC_NUMBERS.iter().any(|m| head.starts_with(m)) {
            return Err(StarfieldError::DataError(format!(
                "a text kernel must start with one of {:?}",
                TEXT_MAGIC_NUMBERS
            )));
        }
        text_pck::load(text, &mut self.variables)
    }

    /// Read a text kernel from a file, merging it into `variables`.
    ///
    /// # Errors
    ///
    /// Returns [`StarfieldError::IoError`] if the file cannot be read, and the
    /// errors of [`read_text`](Self::read_text) otherwise.
    pub fn open_text<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let text = std::fs::read_to_string(path)?;
        self.read_text(&text)
    }

    /// Look up one kernel variable by name.
    pub fn get(&self, name: &str) -> Option<&KernelValue> {
        self.variables.get(name)
    }

    /// The triaxial ellipsoid radii of a body in km: two equatorial, then polar.
    ///
    /// Returns `None` if the kernels read so far carry no `BODYnnn_RADII`.
    pub fn radii(&self, body: i32) -> Option<[f64; 3]> {
        let values = self.variables.get(&format!("BODY{}_RADII", body))?.clone();
        let values = values.to_numbers()?;
        if values.len() != 3 {
            return None;
        }
        Some([values[0], values[1], values[2]])
    }

    /// The body-fixed orientation of a body.
    ///
    /// Requires `BODYnnn_POLE_RA`, `BODYnnn_POLE_DEC` and `BODYnnn_PM`;
    /// returns `None` if any of the three is missing. The nutation and
    /// precession angles come from the body's barycentre, that is from
    /// `BODYn_NUT_PREC_ANGLES` where `n` is `body / 100`, and are empty when
    /// the barycentre declares none.
    pub fn rotational_elements(&self, body: i32) -> Option<RotationalElements> {
        let pole_ra = self.polynomial(&format!("BODY{}_POLE_RA", body))?;
        let pole_dec = self.polynomial(&format!("BODY{}_POLE_DEC", body))?;
        let pm = self.polynomial(&format!("BODY{}_PM", body))?;

        let (nut_prec_angles, nut_prec_angle_accel) = self.nut_prec_angles(body / 100);

        Some(RotationalElements {
            pole_ra,
            pole_dec,
            pm,
            nut_prec_ra: self.list(&format!("BODY{}_NUT_PREC_RA", body)),
            nut_prec_dec: self.list(&format!("BODY{}_NUT_PREC_DEC", body)),
            nut_prec_pm: self.list(&format!("BODY{}_NUT_PREC_PM", body)),
            nut_prec_angles,
            nut_prec_angle_accel,
        })
    }

    /// A variable read as a polynomial of up to three terms.
    fn polynomial(&self, name: &str) -> Option<[f64; 3]> {
        coefficients(&self.variables.get(name)?.to_numbers()?)
    }

    /// A variable read as a list of numbers, empty when it is absent.
    fn list(&self, name: &str) -> Vec<f64> {
        self.variables
            .get(name)
            .and_then(|v| v.to_numbers())
            .unwrap_or_default()
    }

    /// The nutation and precession angles of a barycentre.
    ///
    /// `BODYn_MAX_PHASE_DEGREE = 2` means the angles are quadratic and arrive
    /// in threes rather than in pairs; `pck00011.tpc` uses that form for the
    /// Mars system.
    fn nut_prec_angles(&self, barycentre: i32) -> NutPrecAngles {
        let flat = self.list(&format!("BODY{}_NUT_PREC_ANGLES", barycentre));
        let quadratic = self
            .variables
            .get(&format!("BODY{}_MAX_PHASE_DEGREE", barycentre))
            .and_then(|v| v.as_number())
            == Some(2.0);
        let stride = if quadratic { 3 } else { 2 };

        let mut angles = Vec::new();
        let mut accel = Vec::new();
        for chunk in flat.chunks(stride) {
            if chunk.len() < stride {
                break;
            }
            angles.push((chunk[0], chunk[1]));
            accel.push(if quadratic { chunk[2] } else { 0.0 });
        }
        (angles, accel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planetlib::Body;

    /// A verbatim excerpt of `pck00011.tpc`, checked in so the tests need no
    /// network. The full kernel is 4318 lines; this is the part that concerns
    /// the eleven bodies of [`Body`].
    const EXCERPT: &str = include_str!("pck00011_excerpt.tpc");

    fn excerpt() -> PlanetaryConstants {
        let mut pc = PlanetaryConstants::new();
        pc.read_text(EXCERPT).unwrap();
        pc
    }

    #[test]
    fn test_excerpt_parses_every_body_key() {
        let pc = excerpt();
        let body_keys = pc
            .variables
            .keys()
            .filter(|k| k.starts_with("BODY"))
            .count();
        assert_eq!(body_keys, pc.variables.len());
        assert_eq!(pc.variables.len(), 77);
    }

    #[test]
    fn test_excerpt_radii() {
        let pc = excerpt();
        assert_eq!(pc.radii(399), Some([6378.1366, 6378.1366, 6356.7519]));
        assert_eq!(pc.radii(499), Some([3396.19, 3396.19, 3376.20]));
        assert_eq!(pc.radii(301), Some([1737.4, 1737.4, 1737.4]));
        assert_eq!(pc.radii(599), Some([71492.0, 71492.0, 66854.0]));
        assert_eq!(pc.radii(-1), None);
    }

    #[test]
    fn test_excerpt_earth_elements() {
        let e = excerpt().rotational_elements(399).unwrap();
        assert_eq!(e.pole_ra, [0.0, -0.641, 0.0]);
        assert_eq!(e.pole_dec, [90.0, -0.557, 0.0]);
        assert_eq!(e.pm, [190.147, 360.9856235, 0.0]);
        assert!(e.nut_prec_ra.is_empty());
        // The Earth takes its angles from barycentre 3, shared with the Moon.
        assert_eq!(e.nut_prec_angles.len(), 13);
        assert_eq!(e.nut_prec_angles[0], (125.045, -1935.5364525));
        assert!(e.nut_prec_angle_accel.iter().all(|&a| a == 0.0));
    }

    #[test]
    fn test_excerpt_moon_elements() {
        let e = excerpt().rotational_elements(301).unwrap();
        assert_eq!(e.pole_ra, [269.9949, 0.0031, 0.0]);
        assert_eq!(e.pm, [38.3213, 13.17635815, -1.4e-12]);
        assert_eq!(e.nut_prec_ra.len(), 13);
        assert_eq!(e.nut_prec_dec.len(), 13);
        assert_eq!(e.nut_prec_pm.len(), 13);
        assert_eq!(e.nut_prec_ra[0], -3.8787);
        assert_eq!(e.nut_prec_angles.len(), 13);
    }

    #[test]
    fn test_excerpt_mars_elements_are_quadratic() {
        let e = excerpt().rotational_elements(499).unwrap();
        assert_eq!(e.pole_ra, [317.269202, -0.10927547, 0.0]);
        assert_eq!(e.pm, [176.049863, 350.891982443297, 0.0]);
        assert_eq!(e.nut_prec_ra.len(), 15);
        assert_eq!(e.nut_prec_dec.len(), 20);
        assert_eq!(e.nut_prec_pm.len(), 26);
        // BODY4_MAX_PHASE_DEGREE is 2, so the 78 terms are 26 quadratic angles.
        assert_eq!(e.nut_prec_angles.len(), 26);
        assert_eq!(e.nut_prec_angles[4], (189.6327156, 41215158.1842005));
        assert_eq!(e.nut_prec_angle_accel[4], 12.711923222);
        assert_eq!(e.nut_prec_angle_accel[0], 0.0);
    }

    #[test]
    fn test_excerpt_jupiter_elements() {
        let e = excerpt().rotational_elements(599).unwrap();
        assert_eq!(e.pole_ra, [268.056595, -0.006499, 0.0]);
        assert_eq!(e.pole_dec, [64.495303, 0.002413, 0.0]);
        assert_eq!(e.pm, [284.95, 870.536, 0.0]);
        assert_eq!(e.nut_prec_ra.len(), 15);
        assert_eq!(e.nut_prec_angles.len(), 15);
        assert_eq!(e.nut_prec_angles[0], (73.32, 91472.9));
    }

    #[test]
    fn test_excerpt_barycentre_angle_counts() {
        let pc = excerpt();
        for (barycentre, count) in [(1, 5), (3, 13), (4, 26), (5, 15), (6, 8), (7, 18), (8, 17)] {
            let (angles, accel) = pc.nut_prec_angles(barycentre);
            assert_eq!(angles.len(), count, "barycentre {}", barycentre);
            assert_eq!(accel.len(), count, "barycentre {}", barycentre);
        }
    }

    #[test]
    fn test_missing_body_has_no_elements() {
        assert!(excerpt().rotational_elements(-1).is_none());
    }

    #[test]
    fn test_read_text_rejects_a_file_without_a_magic_number() {
        let mut pc = PlanetaryConstants::new();
        assert!(pc.read_text("\\begindata\nX = 1\n\\begintext\n").is_err());
    }

    #[test]
    fn test_read_text_accepts_a_frame_kernel() {
        let mut pc = PlanetaryConstants::new();
        pc.read_text("KPL/FK\n\\begindata\nFRAME_MOON_PA = 31000\n\\begintext\n")
            .unwrap();
        assert_eq!(pc.get("FRAME_MOON_PA").unwrap().as_number(), Some(31000.0));
    }

    #[test]
    fn test_open_text_reads_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("excerpt.tpc");
        std::fs::write(&path, EXCERPT).unwrap();

        let mut pc = PlanetaryConstants::new();
        pc.open_text(&path).unwrap();
        assert_eq!(pc.radii(499), Some([3396.19, 3396.19, 3376.20]));
    }

    #[test]
    fn test_embedded_table_covers_every_body() {
        for body in [
            Body::Sun,
            Body::Mercury,
            Body::Venus,
            Body::Earth,
            Body::Moon,
            Body::Mars,
            Body::Jupiter,
            Body::Saturn,
            Body::Uranus,
            Body::Neptune,
            Body::Pluto,
        ] {
            let constants = body_constants(body.naif_id())
                .unwrap_or_else(|| panic!("{} is missing from iau2015.csv", body.name()));
            assert_eq!(constants.name, body.name());
        }
        assert_eq!(all_body_constants().len(), 11);
    }

    #[test]
    fn test_embedded_table_matches_the_kernel_excerpt() {
        let pc = excerpt();
        for constants in all_body_constants() {
            let id = constants.naif_id;
            assert_eq!(
                Some(constants.radii),
                pc.radii(id),
                "{} radii disagree",
                constants.name
            );
            assert_eq!(
                Some(&constants.elements),
                pc.rotational_elements(id).as_ref(),
                "{} elements disagree",
                constants.name
            );
        }
    }

    #[test]
    fn test_embedded_spot_values() {
        let mars = body_constants(499).unwrap();
        assert_eq!(mars.radii, [3396.19, 3396.19, 3376.20]);
        let earth = body_constants(399).unwrap();
        assert_eq!(earth.radii, [6378.1366, 6378.1366, 6356.7519]);
        assert_eq!(body_constants(10).unwrap().radii[0], 695700.0);
        assert!(body_constants(4).is_none());
    }

    #[test]
    fn test_mean_radius_and_flattening() {
        let earth = body_constants(399).unwrap();
        assert!((earth.mean_radius_km() - 6371.0084).abs() < 1e-4);
        assert!((earth.flattening() - 1.0 / 298.2572).abs() < 1e-6);
        assert_eq!(body_constants(301).unwrap().flattening(), 0.0);
    }

    #[test]
    fn test_body_accessors_delegate_to_the_table() {
        assert_eq!(Body::Mars.radii_km(), [3396.19, 3396.19, 3376.20]);
        assert!((Body::Mars.mean_radius_km() - 3389.5266666666666).abs() < 1e-9);
        assert!((Body::Mars.flattening() - (3396.19 - 3376.20) / 3396.19).abs() < 1e-15);
        assert_eq!(Body::Mars.rotational_elements().pm[1], 350.891982443297);
    }
}
