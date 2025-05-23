//! # Angle Representation Module
//!
//! This module provides exact angle representation that preserves the original
//! precision and format (degrees vs radians) of angular measurements.
//!
//! ## Design Philosophy
//!
//! The `Angle` type maintains the exact numerical value and format as provided
//! by the user, avoiding unnecessary precision loss that would occur from
//! immediate conversion. This is particularly important for astronomical
//! calculations where precision matters.
//!
//! ## Internal Storage
//!
//! The `Angle` struct uses an enum-based storage system:
//! - Values provided in degrees are stored exactly as degrees
//! - Values provided in radians are stored exactly as radians
//! - Conversion only occurs when explicitly requested via `to_degrees()` or `to_radians()`
//!
//! This approach ensures that:
//! - No precision is lost during construction
//! - The original format is preserved for round-trip accuracy
//! - Conversion artifacts are minimized
//!
//! ## Examples
//!
//! ```rust
//! use starfield::coordinates::angle::{Angle, AngleFormat};
//!
//! // Create angle from degrees - stored exactly as 45.0 degrees
//! let angle_deg = Angle::from_degrees(45.0);
//! assert_eq!(angle_deg.to_degrees(), 45.0);
//!
//! // Create angle from radians - stored exactly as π/4 radians
//! let angle_rad = Angle::from_radians(std::f64::consts::PI / 4.0);
//! assert_eq!(angle_rad.to_radians(), std::f64::consts::PI / 4.0);
//! ```

use std::f64::consts::PI;

/// Internal representation format for angle values
///
/// This enum allows the `Angle` struct to maintain the exact numerical
/// value in its original format, preventing precision loss from unnecessary
/// conversions during construction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AngleFormat {
    /// Angle stored in degrees
    Degrees(f64),
    /// Angle stored in radians
    Radians(f64),
}

/// Represents an angular measurement with exact precision preservation
///
/// The `Angle` type stores angular values in their original format
/// (degrees or radians) to maintain maximum precision. Conversion
/// between formats only occurs when explicitly requested.
///
/// # Internal Storage Strategy
///
/// - Values are stored in their original format to prevent precision loss
/// - Degrees are stored as degrees, radians as radians
/// - No automatic conversion occurs during construction
/// - Conversion factors use high-precision constants
///
/// # Precision Guarantees
///
/// - Round-trip conversions preserve maximum possible precision
/// - Original format values are returned exactly when accessed
/// - Conversion uses `std::f64::consts::PI` for mathematical accuracy
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Angle {
    /// Internal storage maintaining original format and value
    angle: AngleFormat,
}

impl Angle {
    /// Creates a new angle from the given value and format
    ///
    /// # Arguments
    ///
    /// * `value` - The numerical angle value
    /// * `format` - The format containing the value (degrees or radians)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starfield::coordinates::angle::{Angle, AngleFormat};
    ///
    /// let angle = Angle::new(90.0, AngleFormat::Degrees(90.0));
    /// ```
    fn new(value: f64, format: AngleFormat) -> Self {
        match format {
            AngleFormat::Degrees(_) => Angle { angle: AngleFormat::Degrees(value) },
            AngleFormat::Radians(_) => Angle { angle: AngleFormat::Radians(value) },
        }
    }

    /// Creates an angle from a value in degrees
    ///
    /// The value is stored exactly as provided, maintaining full precision.
    ///
    /// # Arguments
    ///
    /// * `degrees` - Angle value in degrees
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starfield::coordinates::angle::Angle;
    ///
    /// let right_angle = Angle::from_degrees(90.0);
    /// assert_eq!(right_angle.to_degrees(), 90.0);
    /// ```
    pub fn from_degrees(degrees: f64) -> Self {
        Angle {
            angle: AngleFormat::Degrees(degrees),
        }
    }

    /// Creates an angle from a value in radians
    ///
    /// The value is stored exactly as provided, maintaining full precision.
    ///
    /// # Arguments
    ///
    /// * `radians` - Angle value in radians
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starfield::coordinates::angle::Angle;
    ///
    /// let right_angle = Angle::from_radians(std::f64::consts::PI / 2.0);
    /// assert_eq!(right_angle.to_radians(), std::f64::consts::PI / 2.0);
    /// ```
    pub fn from_radians(radians: f64) -> Self {
        Angle {
            angle: AngleFormat::Radians(radians),
        }
    }

    /// Returns the angle value in degrees
    ///
    /// If the angle was originally stored in degrees, returns the exact
    /// original value. If stored in radians, performs high-precision
    /// conversion using `std::f64::consts::PI`.
    ///
    /// # Precision Notes
    ///
    /// - Angles stored as degrees return exact original values
    /// - Conversion from radians uses: `radians * (180.0 / π)`
    /// - Uses `std::f64::consts::PI` for maximum precision
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starfield::coordinates::angle::Angle;
    ///
    /// let angle_deg = Angle::from_degrees(45.0);
    /// assert_eq!(angle_deg.to_degrees(), 45.0); // Exact
    ///
    /// let angle_rad = Angle::from_radians(std::f64::consts::PI / 4.0);
    /// assert!((angle_rad.to_degrees() - 45.0).abs() < 1e-14); // High precision
    /// ```
    pub fn to_degrees(&self) -> f64 {
        match self.angle {
            AngleFormat::Degrees(deg) => deg,
            AngleFormat::Radians(rad) => rad * (180.0 / PI),
        }
    }

    /// Returns the angle value in radians
    ///
    /// If the angle was originally stored in radians, returns the exact
    /// original value. If stored in degrees, performs high-precision
    /// conversion using `std::f64::consts::PI`.
    ///
    /// # Precision Notes
    ///
    /// - Angles stored as radians return exact original values
    /// - Conversion from degrees uses: `degrees * (π / 180.0)`
    /// - Uses `std::f64::consts::PI` for maximum precision
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starfield::coordinates::angle::Angle;
    ///
    /// let angle_rad = Angle::from_radians(std::f64::consts::PI / 2.0);
    /// assert_eq!(angle_rad.to_radians(), std::f64::consts::PI / 2.0); // Exact
    ///
    /// let angle_deg = Angle::from_degrees(90.0);
    /// assert!((angle_deg.to_radians() - std::f64::consts::PI / 2.0).abs() < 1e-14);
    /// ```
    pub fn to_radians(&self) -> f64 {
        match self.angle {
            AngleFormat::Degrees(deg) => deg * (PI / 180.0),
            AngleFormat::Radians(rad) => rad,
        }
    }

    /// Returns the internal format of this angle
    ///
    /// This method allows inspection of how the angle is stored internally,
    /// which can be useful for debugging or optimization decisions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starfield::coordinates::angle::{Angle, AngleFormat};
    ///
    /// let angle = Angle::from_degrees(45.0);
    /// match angle.format() {
    ///     AngleFormat::Degrees(val) => assert_eq!(val, 45.0),
    ///     AngleFormat::Radians(_) => panic!("Expected degrees format"),
    /// }
    /// ```
    pub fn format(&self) -> AngleFormat {
        self.angle
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_angle_from_degrees_exact_storage() {
        let angle = Angle::from_degrees(45.0);
        assert_eq!(angle.to_degrees(), 45.0);
        
        // Verify internal storage format
        match angle.format() {
            AngleFormat::Degrees(val) => assert_eq!(val, 45.0),
            AngleFormat::Radians(_) => panic!("Expected degrees format"),
        }
    }

    #[test]
    fn test_angle_from_radians_exact_storage() {
        let angle = Angle::from_radians(PI / 4.0);
        assert_eq!(angle.to_radians(), PI / 4.0);
        
        // Verify internal storage format
        match angle.format() {
            AngleFormat::Radians(val) => assert_eq!(val, PI / 4.0),
            AngleFormat::Degrees(_) => panic!("Expected radians format"),
        }
    }

    #[test]
    fn test_degree_to_radian_conversion() {
        let angle = Angle::from_degrees(180.0);
        let radians = angle.to_radians();
        assert!((radians - PI).abs() < 1e-15);
    }

    #[test]
    fn test_radian_to_degree_conversion() {
        let angle = Angle::from_radians(PI);
        let degrees = angle.to_degrees();
        assert!((degrees - 180.0).abs() < 1e-13);
    }

    #[test]
    fn test_common_angles_degrees() {
        let test_cases = vec![
            (0.0, 0.0),
            (90.0, PI / 2.0),
            (180.0, PI),
            (270.0, 3.0 * PI / 2.0),
            (360.0, 2.0 * PI),
            (45.0, PI / 4.0),
        ];

        for (degrees, expected_radians) in test_cases {
            let angle = Angle::from_degrees(degrees);
            assert!((angle.to_radians() - expected_radians).abs() < 1e-14,
                   "Failed for {} degrees", degrees);
            assert_eq!(angle.to_degrees(), degrees); // Exact for degrees
        }
    }

    #[test]
    fn test_common_angles_radians() {
        let test_cases = vec![
            (0.0, 0.0),
            (PI / 2.0, 90.0),
            (PI, 180.0),
            (3.0 * PI / 2.0, 270.0),
            (2.0 * PI, 360.0),
            (PI / 4.0, 45.0),
        ];

        for (radians, expected_degrees) in test_cases {
            let angle = Angle::from_radians(radians);
            assert!((angle.to_degrees() - expected_degrees).abs() < 1e-13,
                   "Failed for {} radians", radians);
            assert_eq!(angle.to_radians(), radians); // Exact for radians
        }
    }

    #[test]
    fn test_precision_preservation() {
        // Test that very precise values are preserved exactly when stored in original format
        let precise_degrees = 123.456789012345;
        let angle_deg = Angle::from_degrees(precise_degrees);
        assert_eq!(angle_deg.to_degrees(), precise_degrees);

        let precise_radians = 2.154321098765432;
        let angle_rad = Angle::from_radians(precise_radians);
        assert_eq!(angle_rad.to_radians(), precise_radians);
    }

    #[test]
    fn test_round_trip_conversion_precision() {
        // Test round-trip precision: degrees -> radians -> degrees
        let original_degrees = 37.5;
        let angle = Angle::from_degrees(original_degrees);
        let radians = angle.to_radians();
        let angle_from_rad = Angle::from_radians(radians);
        let back_to_degrees = angle_from_rad.to_degrees();
        
        assert!((back_to_degrees - original_degrees).abs() < 1e-14);

        // Test round-trip precision: radians -> degrees -> radians
        let original_radians = PI / 3.0; // 60 degrees
        let angle = Angle::from_radians(original_radians);
        let degrees = angle.to_degrees();
        let angle_from_deg = Angle::from_degrees(degrees);
        let back_to_radians = angle_from_deg.to_radians();
        
        assert!((back_to_radians - original_radians).abs() < 1e-14);
    }

    #[test]
    fn test_negative_angles() {
        let neg_degrees = Angle::from_degrees(-45.0);
        assert_eq!(neg_degrees.to_degrees(), -45.0);
        assert!((neg_degrees.to_radians() - (-PI / 4.0)).abs() < 1e-15);

        let neg_radians = Angle::from_radians(-PI / 6.0);
        assert_eq!(neg_radians.to_radians(), -PI / 6.0);
        assert!((neg_radians.to_degrees() - (-30.0)).abs() < 1e-14);
    }

    #[test]
    fn test_zero_angle() {
        let zero_deg = Angle::from_degrees(0.0);
        let zero_rad = Angle::from_radians(0.0);
        
        assert_eq!(zero_deg.to_degrees(), 0.0);
        assert_eq!(zero_deg.to_radians(), 0.0);
        assert_eq!(zero_rad.to_radians(), 0.0);
        assert_eq!(zero_rad.to_degrees(), 0.0);
    }

    #[test]
    fn test_angle_equality() {
        let angle1 = Angle::from_degrees(90.0);
        let angle2 = Angle::from_degrees(90.0);
        let angle3 = Angle::from_radians(PI / 2.0);
        
        assert_eq!(angle1, angle2);
        assert_ne!(angle1, angle3); // Different internal representations
    }

    #[test]
    fn test_very_small_angles() {
        let tiny_degrees = 1e-10;
        let angle = Angle::from_degrees(tiny_degrees);
        assert_eq!(angle.to_degrees(), tiny_degrees);
        
        let tiny_radians = 1e-15;
        let angle = Angle::from_radians(tiny_radians);
        assert_eq!(angle.to_radians(), tiny_radians);
    }

    #[test]
    fn test_large_angles() {
        let large_degrees = 720.0; // Two full rotations
        let angle = Angle::from_degrees(large_degrees);
        assert_eq!(angle.to_degrees(), large_degrees);
        assert!((angle.to_radians() - 4.0 * PI).abs() < 1e-13);
        
        let large_radians = 10.0 * PI;
        let angle = Angle::from_radians(large_radians);
        assert_eq!(angle.to_radians(), large_radians);
        assert!((angle.to_degrees() - 1800.0).abs() < 1e-12);
    }

    #[test]
    fn test_format_inspection() {
        let deg_angle = Angle::from_degrees(30.0);
        let rad_angle = Angle::from_radians(PI / 6.0);
        
        match deg_angle.format() {
            AngleFormat::Degrees(val) => assert_eq!(val, 30.0),
            _ => panic!("Expected degrees format"),
        }
        
        match rad_angle.format() {
            AngleFormat::Radians(val) => assert_eq!(val, PI / 6.0),
            _ => panic!("Expected radians format"),
        }
    }
}

