pub mod angle;
pub mod cartesian;

use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Celestial coordinate in right ascension and declination
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RaDec {
    /// Right ascension in radians
    pub ra: f64,
    /// Declination in radians
    pub dec: f64,
}

impl RaDec {
    /// Create a new RaDec coordinate with values in radians
    pub fn new(ra: f64, dec: f64) -> Self {
        Self { ra, dec }
    }

    /// Create a new RaDec coordinate with values in degrees
    pub fn from_degrees(ra_deg: f64, dec_deg: f64) -> Self {
        Self {
            ra: ra_deg * PI / 180.0,
            dec: dec_deg * PI / 180.0,
        }
    }

    /// Get right ascension in degrees
    pub fn ra_degrees(&self) -> f64 {
        self.ra * 180.0 / PI
    }

    /// Get declination in degrees
    pub fn dec_degrees(&self) -> f64 {
        self.dec * 180.0 / PI
    }

    /// Calculate angular distance to another RaDec in radians
    pub fn angular_distance(&self, other: &RaDec) -> f64 {
        let cos_dist = self.dec.sin() * other.dec.sin()
            + self.dec.cos() * other.dec.cos() * (self.ra - other.ra).cos();

        // Handle numerical precision issues
        if cos_dist >= 1.0 {
            0.0
        } else if cos_dist <= -1.0 {
            PI
        } else {
            cos_dist.acos()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radec_conversions() {
        let ra_rad = 1.5;
        let dec_rad = 0.5;

        let coord = RaDec::new(ra_rad, dec_rad);

        assert_eq!(coord.ra, ra_rad);
        assert_eq!(coord.dec, dec_rad);

        let ra_deg = ra_rad * 180.0 / PI;
        let dec_deg = dec_rad * 180.0 / PI;

        assert!((coord.ra_degrees() - ra_deg).abs() < 1e-10);
        assert!((coord.dec_degrees() - dec_deg).abs() < 1e-10);

        let from_deg = RaDec::from_degrees(ra_deg, dec_deg);
        assert!((from_deg.ra - ra_rad).abs() < 1e-10);
        assert!((from_deg.dec - dec_rad).abs() < 1e-10);
    }

    #[test]
    fn test_angular_distance() {
        // Same point should have zero distance
        let p1 = RaDec::new(1.0, 0.5);
        assert!((p1.angular_distance(&p1)).abs() < 1e-10);

        // Points on opposite sides of sphere should have distance of PI
        let p2 = RaDec::new(1.0 + PI, -0.5);
        assert!((p1.angular_distance(&p2) - PI).abs() < 1e-10);

        // Check a known angular distance
        let polaris = RaDec::from_degrees(37.95, 89.26); // Close to North pole
        let vega = RaDec::from_degrees(279.23, 38.78);

        // Angular distance should be around 51 degrees
        let dist_rad = polaris.angular_distance(&vega);
        let dist_deg = dist_rad * 180.0 / PI;
        assert!((dist_deg - 51.0).abs() < 1.0);
    }
}
