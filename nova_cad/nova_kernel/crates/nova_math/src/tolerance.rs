//! Hierarchical tolerance context for geometric comparisons

use crate::{DEFAULT_RESABS, DEFAULT_RESREL, DEFAULT_ANGLE_TOL};
use serde::{Deserialize, Serialize};

/// Tolerance context for geometric operations
/// 
/// Provides a hierarchical tolerance system similar to Parasolid's SPAresabs
/// and SPAresrel, with support for both absolute and relative tolerances.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ToleranceContext {
    /// Absolute resolution (SPAresabs equivalent)
    pub resabs: f64,
    /// Relative resolution (SPAresrel equivalent)
    pub resrel: f64,
    /// Angular tolerance in radians
    pub angle_tol: f64,
    /// Square of resabs (precomputed for efficiency)
    resabs_sq: f64,
}

impl ToleranceContext {
    /// Default tolerance context
    pub const DEFAULT: Self = Self {
        resabs: DEFAULT_RESABS,
        resrel: DEFAULT_RESREL,
        angle_tol: DEFAULT_ANGLE_TOL,
        resabs_sq: DEFAULT_RESABS * DEFAULT_RESABS,
    };

    /// Create a new tolerance context
    #[inline]
    pub fn new(resabs: f64, resrel: f64, angle_tol: f64) -> Self {
        Self {
            resabs,
            resrel,
            angle_tol,
            resabs_sq: resabs * resabs,
        }
    }

    /// Create with only absolute tolerance
    #[inline]
    pub fn with_resabs(resabs: f64) -> Self {
        Self::new(resabs, DEFAULT_RESREL, DEFAULT_ANGLE_TOL)
    }

    /// Get the absolute resolution
    #[inline]
    pub fn resabs(&self) -> f64 {
        self.resabs
    }

    /// Get the squared absolute resolution
    #[inline]
    pub fn resabs_sq(&self) -> f64 {
        self.resabs_sq
    }

    /// Get the relative resolution
    #[inline]
    pub fn resrel(&self) -> f64 {
        self.resrel
    }

    /// Get the angular tolerance
    #[inline]
    pub fn angle_tol(&self) -> f64 {
        self.angle_tol
    }

    /// Compute effective tolerance for a given magnitude
    /// 
    /// Returns max(resabs, magnitude * resrel)
    #[inline]
    pub fn effective_tol(&self, magnitude: f64) -> f64 {
        self.resabs.max(magnitude * self.resrel)
    }

    /// Check if a value is approximately zero
    #[inline]
    pub fn is_zero(&self, val: f64) -> bool {
        val.abs() <= self.resabs
    }

    /// Check if a value is approximately zero (squared comparison)
    #[inline]
    pub fn is_zero_sq(&self, val_sq: f64) -> bool {
        val_sq <= self.resabs_sq
    }

    /// Check if two values are approximately equal
    #[inline]
    pub fn is_equal(&self, a: f64, b: f64) -> bool {
        (a - b).abs() <= self.effective_tol(a.abs().max(b.abs()))
    }

    /// Check if two squared values are approximately equal
    #[inline]
    pub fn is_equal_sq(&self, a_sq: f64, b_sq: f64) -> bool {
        (a_sq - b_sq).abs() <= self.resabs_sq
    }

    /// Compare two values with tolerance
    /// 
    /// Returns:
    /// - -1 if a < b - tol
    /// -  0 if |a - b| <= tol
    /// -  1 if a > b + tol
    #[inline]
    pub fn compare(&self, a: f64, b: f64) -> i32 {
        let diff = a - b;
        let tol = self.effective_tol(a.abs().max(b.abs()));
        if diff < -tol {
            -1
        } else if diff > tol {
            1
        } else {
            0
        }
    }

    /// Check if a value is positive (with tolerance)
    #[inline]
    pub fn is_positive(&self, val: f64) -> bool {
        val > self.resabs
    }

    /// Check if a value is negative (with tolerance)
    #[inline]
    pub fn is_negative(&self, val: f64) -> bool {
        val < -self.resabs
    }

    /// Check if two angles are approximately equal
    #[inline]
    pub fn angles_equal(&self, a: f64, b: f64) -> bool {
        let diff = (a - b).abs();
        let diff_wrapped = diff.min((std::f64::consts::TAU - diff).abs());
        diff_wrapped <= self.angle_tol
    }

    /// Clamp a value to zero if within tolerance
    #[inline]
    pub fn clamp_to_zero(&self, val: f64) -> f64 {
        if self.is_zero(val) { 0.0 } else { val }
    }

    /// Scale the tolerance by a factor
    #[inline]
    pub fn scale(&self, factor: f64) -> Self {
        Self::new(
            self.resabs * factor,
            self.resrel * factor,
            self.angle_tol * factor,
        )
    }
}

impl Default for ToleranceContext {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Global tolerance functions using default context
pub struct Tolerance;

impl Tolerance {
    /// Check if approximately zero using default tolerance
    #[inline]
    pub fn is_zero(val: f64) -> bool {
        ToleranceContext::DEFAULT.is_zero(val)
    }

    /// Check if approximately equal using default tolerance
    #[inline]
    pub fn is_equal(a: f64, b: f64) -> bool {
        ToleranceContext::DEFAULT.is_equal(a, b)
    }

    /// Compare two values using default tolerance
    #[inline]
    pub fn compare(a: f64, b: f64) -> i32 {
        ToleranceContext::DEFAULT.compare(a, b)
    }

    /// Check if angles are equal using default tolerance
    #[inline]
    pub fn angles_equal(a: f64, b: f64) -> bool {
        ToleranceContext::DEFAULT.angles_equal(a, b)
    }

    /// Get default absolute resolution
    #[inline]
    pub fn resabs() -> f64 {
        DEFAULT_RESABS
    }

    /// Get default relative resolution
    #[inline]
    pub fn resrel() -> f64 {
        DEFAULT_RESREL
    }

    /// Get default angular tolerance
    #[inline]
    pub fn angle_tol() -> f64 {
        DEFAULT_ANGLE_TOL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let tol = ToleranceContext::DEFAULT;
        assert_eq!(tol.resabs(), DEFAULT_RESABS);
        assert_eq!(tol.resrel(), DEFAULT_RESREL);
    }

    #[test]
    fn test_is_zero() {
        let tol = ToleranceContext::DEFAULT;
        assert!(tol.is_zero(0.0));
        assert!(tol.is_zero(1e-7));
        assert!(!tol.is_zero(1e-4));
    }

    #[test]
    fn test_is_equal() {
        let tol = ToleranceContext::DEFAULT;
        assert!(tol.is_equal(1.0, 1.0 + 1e-7));
        assert!(!tol.is_equal(1.0, 1.1));
    }

    #[test]
    fn test_compare() {
        let tol = ToleranceContext::DEFAULT;
        assert_eq!(tol.compare(1.0, 2.0), -1);
        assert_eq!(tol.compare(2.0, 1.0), 1);
        assert_eq!(tol.compare(1.0, 1.0 + 1e-7), 0);
    }

    #[test]
    fn test_effective_tol() {
        let tol = ToleranceContext::DEFAULT;
        // For small values, resabs dominates
        assert_eq!(tol.effective_tol(1e-10), DEFAULT_RESABS);
        // For large values, resrel may dominate
        let large = 1.0 / DEFAULT_RESREL;
        assert!(tol.effective_tol(large) > DEFAULT_RESABS);
    }

    #[test]
    fn test_angles_equal() {
        let tol = ToleranceContext::DEFAULT;
        assert!(tol.angles_equal(0.0, 0.0));
        assert!(tol.angles_equal(0.0, std::f64::consts::TAU));
        assert!(!tol.angles_equal(0.0, 1.0));
    }

    #[test]
    fn test_scale() {
        let tol1 = ToleranceContext::DEFAULT;
        let tol2 = tol1.scale(2.0);
        assert_eq!(tol2.resabs(), tol1.resabs() * 2.0);
        assert_eq!(tol2.resrel(), tol1.resrel() * 2.0);
    }

    #[test]
    fn test_global_functions() {
        assert!(Tolerance::is_zero(1e-7));
        assert!(Tolerance::is_equal(1.0, 1.0 + 1e-7));
        assert_eq!(Tolerance::compare(1.0, 2.0), -1);
    }
}
