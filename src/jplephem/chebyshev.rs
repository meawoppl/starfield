//! Chebyshev polynomial functionality for ephemeris interpolation
//!
//! This module provides the implementation of Chebyshev polynomials used for
//! interpolating positions and velocities of celestial bodies from ephemeris data.
//!
//! Chebyshev polynomials are used in JPL ephemerides as they offer excellent
//! approximation properties for smooth trajectories with minimal error.

use crate::jplephem::errors::{JplephemError, Result};

/// Chebyshev polynomial representation and evaluation
///
/// This struct stores the coefficients of a Chebyshev polynomial expansion
/// and provides methods to evaluate the polynomial and its derivatives.
#[derive(Debug, Clone)]
pub struct ChebyshevPolynomial {
    /// Coefficients of the Chebyshev polynomial
    coefficients: Vec<f64>,
}

impl ChebyshevPolynomial {
    /// Create a new Chebyshev polynomial with the given coefficients
    ///
    /// The coefficients are ordered from lowest to highest degree:
    /// [c₀, c₁, c₂, ..., cₙ] where the polynomial is:
    /// f(x) = c₀·T₀(x) + c₁·T₁(x) + c₂·T₂(x) + ... + cₙ·Tₙ(x)
    pub fn new(coefficients: Vec<f64>) -> Self {
        Self { coefficients }
    }

    /// Evaluate the Chebyshev polynomial at the given point x in [-1, 1]
    ///
    /// Returns the value of the polynomial at point x, or NaN if x is outside [-1, 1]
    pub fn evaluate(&self, x: f64) -> f64 {
        if self.coefficients.is_empty() {
            return 0.0;
        }

        // Check if x is within the valid range [-1, 1]
        if x < -1.0 || x > 1.0 {
            // Return NaN for values outside the valid range
            return f64::NAN;
        }

        // Direct evaluation is simple to understand and debug
        // For each coefficient, multiply by the appropriate Chebyshev polynomial value
        let mut result = 0.0;
        for i in 0..self.coefficients.len() {
            result += self.coefficients[i] * Self::chebyshev_t(i, x);
        }

        result
    }

    /// Compute the value of the Chebyshev polynomial T_n(x) of degree n at point x
    fn chebyshev_t(n: usize, x: f64) -> f64 {
        match n {
            0 => 1.0,
            1 => x,
            _ => {
                // Use the recurrence relation: T_n(x) = 2x*T_{n-1}(x) - T_{n-2}(x)
                let mut t_prev2 = 1.0; // T_0(x)
                let mut t_prev1 = x; // T_1(x)
                let mut t_n = 0.0;

                for _i in 2..=n {
                    t_n = 2.0 * x * t_prev1 - t_prev2;
                    t_prev2 = t_prev1;
                    t_prev1 = t_n;
                }

                t_n
            }
        }
    }

    /// Calculate the derivative of the Chebyshev polynomial at point x
    ///
    /// Returns the value of the derivative at point x, or NaN if x is outside [-1, 1]
    pub fn derivative(&self, x: f64) -> f64 {
        if self.coefficients.len() <= 1 {
            return 0.0; // Constant polynomial has zero derivative
        }

        // Check if x is within the valid range [-1, 1]
        if x < -1.0 || x > 1.0 {
            // Return NaN for values outside the valid range
            return f64::NAN;
        }

        // Direct evaluation approach for derivative
        // dT_n(x)/dx = n * U_{n-1}(x)
        // where U_{n-1} is the Chebyshev polynomial of the second kind

        let mut result = 0.0;
        for i in 1..self.coefficients.len() {
            // Skip i=0 as T_0 has zero derivative
            let n = i as f64;
            // The derivative of T_n(x) evaluated at x
            let derivative_value = n * Self::chebyshev_u(i - 1, x);
            result += self.coefficients[i] * derivative_value;
        }

        result
    }

    /// Compute the value of the Chebyshev polynomial U_n(x) of the second kind
    fn chebyshev_u(n: usize, x: f64) -> f64 {
        match n {
            0 => 1.0,
            1 => 2.0 * x,
            _ => {
                // Use the recurrence relation: U_n(x) = 2x*U_{n-1}(x) - U_{n-2}(x)
                let mut u_prev2 = 1.0; // U_0(x)
                let mut u_prev1 = 2.0 * x; // U_1(x)
                let mut u_n = 0.0;

                for _i in 2..=n {
                    u_n = 2.0 * x * u_prev1 - u_prev2;
                    u_prev2 = u_prev1;
                    u_prev1 = u_n;
                }

                u_n
            }
        }
    }

    /// Calculate the nth derivative of the Chebyshev polynomial at point x
    ///
    /// Returns the value of the nth derivative at point x
    pub fn nth_derivative(&self, x: f64, n: usize) -> f64 {
        if n == 0 {
            return self.evaluate(x);
        } else if n == 1 {
            return self.derivative(x);
        } else if self.coefficients.len() <= n {
            return 0.0; // Higher derivatives of a low-degree polynomial are zero
        }

        // For higher derivatives, we could implement a more sophisticated algorithm
        // but for now we'll use a simple recursive approach for clarity
        // This could be optimized later if needed

        // Create coefficients for the first derivative
        let mut deriv_coeffs = Vec::new();
        for k in 1..self.coefficients.len() {
            deriv_coeffs.push(2.0 * k as f64 * self.coefficients[k]);
        }

        // Recursively compute higher derivatives
        let deriv_poly = ChebyshevPolynomial::new(deriv_coeffs);
        deriv_poly.nth_derivative(x, n - 1)
    }

    /// Get the degree of the polynomial
    pub fn degree(&self) -> usize {
        self.coefficients.len().saturating_sub(1)
    }

    /// Get a reference to the coefficients
    pub fn coefficients(&self) -> &[f64] {
        &self.coefficients
    }
}

/// Time normalization for Chebyshev polynomial evaluation
///
/// This function normalizes a time value to the range [-1, 1] based on
/// the given interval midpoint and radius (half-length).
///
/// # Arguments
///
/// * `time` - The time to normalize (in same units as midpoint and radius)
/// * `midpoint` - The midpoint of the time interval
/// * `radius` - The radius (half-length) of the time interval
///
/// # Returns
///
/// The normalized time in the range [-1, 1]
pub fn normalize_time(time: f64, midpoint: f64, radius: f64) -> Result<f64> {
    if radius <= 0.0 {
        return Err(JplephemError::Other(
            "Invalid radius for time normalization: must be positive".to_string(),
        ));
    }

    let normalized = (time - midpoint) / radius;

    // Check if the time is within the valid range
    if normalized < -1.0 || normalized > 1.0 {
        return Err(JplephemError::OutOfRangeError {
            jd: time,
            start_jd: midpoint - radius,
            end_jd: midpoint + radius,
            out_of_range_times: None,
        });
    }

    Ok(normalized)
}

/// Rescale a derivative from normalized time to physical time
///
/// When calculating derivatives of Chebyshev polynomials, we first get the
/// derivative with respect to the normalized time variable. This function
/// rescales it to get the derivative with respect to physical time.
///
/// # Arguments
///
/// * `deriv_normalized` - The derivative with respect to normalized time
/// * `radius` - The radius (half-length) of the time interval
///
/// # Returns
///
/// The derivative with respect to physical time
pub fn rescale_derivative(deriv_normalized: f64, radius: f64) -> Result<f64> {
    if radius <= 0.0 {
        return Err(JplephemError::Other(
            "Invalid radius for derivative rescaling: must be positive".to_string(),
        ));
    }

    Ok(deriv_normalized / radius)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chebyshev_constant() {
        // Test a constant polynomial (just T0)
        let poly = ChebyshevPolynomial::new(vec![5.0]);

        // Should be constant for any x in [-1, 1]
        assert_eq!(poly.evaluate(-1.0), 5.0);
        assert_eq!(poly.evaluate(0.0), 5.0);
        assert_eq!(poly.evaluate(1.0), 5.0);

        // Derivative should be zero
        assert_eq!(poly.derivative(0.0), 0.0);
    }

    #[test]
    fn test_chebyshev_linear() {
        // Test a linear polynomial (T0 + T1)
        let poly = ChebyshevPolynomial::new(vec![3.0, 2.0]);

        // For T0 + 2*T1, we expect f(x) = 3 + 2x
        assert_eq!(poly.evaluate(-1.0), 1.0); // 3 - 2 = 1
        assert_eq!(poly.evaluate(0.0), 3.0); // 3 + 0 = 3
        assert_eq!(poly.evaluate(1.0), 5.0); // 3 + 2 = 5

        // Derivative should be constant (2.0)
        assert_eq!(poly.derivative(-1.0), 2.0);
        assert_eq!(poly.derivative(0.0), 2.0);
        assert_eq!(poly.derivative(1.0), 2.0);
    }

    #[test]
    fn test_chebyshev_quadratic() {
        // Test a quadratic polynomial (T0 + T1 + T2)
        // T2(x) = 2x² - 1
        // So we have f(x) = 3 + 2x + (2x² - 1) = 2 + 2x + 2x²
        let poly = ChebyshevPolynomial::new(vec![3.0, 2.0, 1.0]);

        assert_eq!(poly.evaluate(-1.0), 2.0); // 3 - 2 + (2*1 - 1) = 3 - 2 + 1 = 2
        assert_eq!(poly.evaluate(0.0), 2.0); // 3 + 0 + (0 - 1) = 2
        assert_eq!(poly.evaluate(1.0), 6.0); // 3 + 2 + (2 - 1) = 6

        // Derivative: f'(x) = 2 + 4x
        assert_eq!(poly.derivative(-1.0), -2.0); // 2 - 4 = -2
        assert_eq!(poly.derivative(0.0), 2.0); // 2 + 0 = 2
        assert_eq!(poly.derivative(1.0), 6.0); // 2 + 4 = 6
    }

    #[test]
    fn test_time_normalization() {
        // Test time normalization with a simple interval
        let midpoint = 100.0;
        let radius = 10.0;

        // Time at the midpoint should normalize to 0
        assert_eq!(normalize_time(100.0, midpoint, radius).unwrap(), 0.0);

        // Time at the start of the interval should normalize to -1
        assert_eq!(normalize_time(90.0, midpoint, radius).unwrap(), -1.0);

        // Time at the end of the interval should normalize to 1
        assert_eq!(normalize_time(110.0, midpoint, radius).unwrap(), 1.0);

        // Time halfway between midpoint and start should normalize to -0.5
        assert_eq!(normalize_time(95.0, midpoint, radius).unwrap(), -0.5);

        // Time outside the interval should return an error
        assert!(normalize_time(80.0, midpoint, radius).is_err());
        assert!(normalize_time(120.0, midpoint, radius).is_err());

        // Invalid radius should return an error
        assert!(normalize_time(100.0, midpoint, 0.0).is_err());
        assert!(normalize_time(100.0, midpoint, -10.0).is_err());
    }

    #[test]
    fn test_derivative_rescaling() {
        // Test derivative rescaling
        let radius = 10.0;
        let deriv_normalized = 5.0;

        // The physical derivative should be scaled by 1/radius
        assert_eq!(rescale_derivative(deriv_normalized, radius).unwrap(), 0.5);

        // Invalid radius should return an error
        assert!(rescale_derivative(deriv_normalized, 0.0).is_err());
        assert!(rescale_derivative(deriv_normalized, -10.0).is_err());
    }

    #[test]
    fn test_higher_derivatives() {
        // Test a cubic polynomial
        let poly = ChebyshevPolynomial::new(vec![1.0, 2.0, 3.0, 4.0]);

        // Second derivative at x=0
        let second_deriv = poly.nth_derivative(0.0, 2);
        assert!(second_deriv != 0.0); // Should be non-zero for cubic

        // Third derivative should be constant for a cubic
        let third_deriv_neg1 = poly.nth_derivative(-1.0, 3);
        let third_deriv_0 = poly.nth_derivative(0.0, 3);
        let third_deriv_1 = poly.nth_derivative(1.0, 3);

        // All third derivatives should be the same (within floating point precision)
        assert!((third_deriv_neg1 - third_deriv_0).abs() < 1e-10);
        assert!((third_deriv_0 - third_deriv_1).abs() < 1e-10);

        // Fourth derivative of a cubic should be zero
        assert_eq!(poly.nth_derivative(0.0, 4), 0.0);
    }

    #[test]
    fn test_approximation_accuracy() {
        // Test that we can create our own Chebyshev approximation function

        // Create a simple function: x^2 in the range [-1, 1]
        // For this function, we know the exact Chebyshev coefficients:
        // T0(x) = 1,  T1(x) = x,  T2(x) = 2x^2 - 1
        // For f(x) = x^2 = (T2(x) + 1)/2, the coefficients are [0.5, 0, 0.5]
        let poly = ChebyshevPolynomial::new(vec![0.5, 0.0, 0.5]);

        // Check at several points
        for i in 0..=10 {
            let x = -1.0 + i as f64 * 0.2;
            let expected = x * x;
            let approximated = poly.evaluate(x);

            // Should be extremely close since this is an exact representation
            assert!(
                (expected - approximated).abs() < 1e-10,
                "Poor approximation at x={}: expected {}, got {}",
                x,
                expected,
                approximated
            );
        }
    }
}
