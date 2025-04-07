//! Time module for astronomical time calculations

use chrono::{DateTime, Datelike, TimeZone, Utc};
use std::fmt;
use thiserror::Error;

/// Error type for time operations
#[derive(Debug, Error)]
pub enum TimeError {
    #[error("Invalid time format: {0}")]
    InvalidFormat(String),

    #[error("Time out of range: {0}")]
    OutOfRange(String),

    #[error("Parsing error: {0}")]
    ParseError(String),
}

/// Result type for time operations
pub type Result<T> = std::result::Result<T, TimeError>;

/// Represents astronomical time with high precision
#[derive(Debug, Clone, PartialEq)]
pub struct Time {
    /// UTC datetime
    utc: DateTime<Utc>,
    /// Delta-T in seconds (difference between UT1 and TT)
    delta_t: f64,
}

impl Time {
    /// Create a new time from a UTC datetime
    pub fn new(utc: DateTime<Utc>) -> Self {
        // Calculate an approximate delta_t for the given date
        // This is a simplified calculation and should be replaced with a more accurate one
        let year = utc.year() as f64;
        let delta_t = if year >= 2005.0 && year < 2050.0 {
            // Simple approximation for recent years
            62.92 + 0.32 * (year - 2000.0)
        } else {
            // Default for other years
            60.0
        };

        Self { utc, delta_t }
    }

    /// Get the current time
    pub fn now() -> Self {
        Self::new(Utc::now())
    }

    /// Get the UTC datetime
    pub fn utc(&self) -> DateTime<Utc> {
        self.utc
    }

    /// Get the TT (Terrestrial Time) as seconds since J2000.0
    pub fn tt_seconds_from_j2000(&self) -> f64 {
        // J2000.0 is 2000-01-01T12:00:00Z
        let j2000 = Utc.with_ymd_and_hms(2000, 1, 1, 12, 0, 0).unwrap();

        // Calculate seconds since J2000.0 in UTC
        let seconds_since_j2000 = (self.utc - j2000).num_seconds() as f64;

        // Add delta_t to convert to TT
        seconds_since_j2000 + self.delta_t
    }

    /// Get the Julian Date
    pub fn jd(&self) -> f64 {
        // J2000.0 is JD 2451545.0
        let j2000_jd = 2451545.0;

        // Convert seconds to days
        let days_since_j2000 = self.tt_seconds_from_j2000() / 86400.0;

        j2000_jd + days_since_j2000
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (ΔT: {:.2}s)",
            self.utc.format("%Y-%m-%d %H:%M:%S UTC"),
            self.delta_t
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_time_creation() {
        let date = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
        let time = Time::new(date);

        // Test delta_t calculation for 2020
        assert_relative_eq!(
            time.delta_t,
            62.92 + 0.32 * (2020.0 - 2000.0),
            epsilon = 0.01
        );
    }

    #[test]
    fn test_tt_seconds_from_j2000() {
        // 2020-01-01 is 20 years after J2000
        let date = Utc.with_ymd_and_hms(2020, 1, 1, 12, 0, 0).unwrap();
        let time = Time::new(date);

        // Expected: 20 years in seconds plus delta_t
        let expected = 20.0 * 365.25 * 24.0 * 3600.0 + time.delta_t;
        let result = time.tt_seconds_from_j2000();

        assert_relative_eq!(result, expected, epsilon = 0.1);
    }

    #[test]
    fn test_julian_date() {
        let date = Utc.with_ymd_and_hms(2000, 1, 1, 12, 0, 0).unwrap();
        let time = Time::new(date);

        // J2000.0 is JD 2451545.0
        let expected = 2451545.0;
        let result = time.jd();

        assert_relative_eq!(result, expected, max_relative = 1e-10);
    }
}
