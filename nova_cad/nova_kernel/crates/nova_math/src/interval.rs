//! Closed interval arithmetic for robust geometric computations

use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub, Mul, Div, Neg};

/// Closed interval [min, max]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Interval {
    /// Lower bound (inclusive)
    pub min: f64,
    /// Upper bound (inclusive)
    pub max: f64,
}

impl Interval {
    /// Empty interval
    pub fn empty() -> Self {
        Self {
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        }
    }

    /// Interval containing all real numbers
    pub const ENTIRE: Self = Self {
        min: f64::NEG_INFINITY,
        max: f64::INFINITY,
    };

    /// Zero interval [0, 0]
    pub const ZERO: Self = Self { min: 0.0, max: 0.0 };

    /// Unit interval [0, 1]
    pub const UNIT: Self = Self { min: 0.0, max: 1.0 };

    /// Create a new interval
    #[inline]
    pub fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    /// Create a point interval [x, x]
    #[inline]
    pub fn point(x: f64) -> Self {
        Self { min: x, max: x }
    }

    /// Check if the interval is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.min > self.max
    }

    /// Check if the interval is valid (min <= max)
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.min <= self.max
    }

    /// Check if the interval is a single point
    #[inline]
    pub fn is_point(&self, tol: f64) -> bool {
        (self.max - self.min).abs() <= tol
    }

    /// Width of the interval
    #[inline]
    pub fn width(&self) -> f64 {
        if self.is_empty() {
            0.0
        } else {
            self.max - self.min
        }
    }

    /// Midpoint of the interval
    #[inline]
    pub fn midpoint(&self) -> f64 {
        (self.min + self.max) * 0.5
    }

    /// Check if a value is contained in the interval
    #[inline]
    pub fn contains(&self, x: f64) -> bool {
        x >= self.min && x <= self.max
    }

    /// Check if another interval is contained in this interval
    #[inline]
    pub fn contains_interval(&self, other: &Interval) -> bool {
        self.min <= other.min && self.max >= other.max
    }

    /// Check if this interval intersects another
    #[inline]
    pub fn intersects(&self, other: &Interval) -> bool {
        self.min <= other.max && self.max >= other.min
    }

    /// Compute the intersection of two intervals
    #[inline]
    pub fn intersection(&self, other: &Interval) -> Self {
        Self {
            min: self.min.max(other.min),
            max: self.max.min(other.max),
        }
    }

    /// Compute the union (hull) of two intervals
    #[inline]
    pub fn union(&self, other: &Interval) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// Expand the interval by a margin
    #[inline]
    pub fn expand(&mut self, margin: f64) {
        self.min -= margin;
        self.max += margin;
    }

    /// Expand to include a value
    #[inline]
    pub fn include(&mut self, x: f64) {
        self.min = self.min.min(x);
        self.max = self.max.max(x);
    }

    /// Split the interval at the midpoint
    #[inline]
    pub fn split(&self) -> (Self, Self) {
        let mid = self.midpoint();
        (
            Self::new(self.min, mid),
            Self::new(mid, self.max),
        )
    }

    /// Check if the interval is strictly positive (> 0)
    #[inline]
    pub fn is_strictly_positive(&self) -> bool {
        self.min > 0.0
    }

    /// Check if the interval is strictly negative (< 0)
    #[inline]
    pub fn is_strictly_negative(&self) -> bool {
        self.max < 0.0
    }

    /// Check if the interval contains zero
    #[inline]
    pub fn contains_zero(&self) -> bool {
        self.min <= 0.0 && self.max >= 0.0
    }

    /// Check if the interval is entirely non-negative
    #[inline]
    pub fn is_non_negative(&self) -> bool {
        self.min >= 0.0
    }

    /// Check if the interval is entirely non-positive
    #[inline]
    pub fn is_non_positive(&self) -> bool {
        self.max <= 0.0
    }

    /// Absolute value of the interval
    #[inline]
    pub fn abs(&self) -> Self {
        if self.contains_zero() {
            Self::new(0.0, self.min.abs().max(self.max.abs()))
        } else if self.min >= 0.0 {
            *self
        } else {
            Self::new(self.max.abs(), self.min.abs())
        }
    }

    /// Square of the interval
    #[inline]
    pub fn sqr(&self) -> Self {
        if self.contains_zero() {
            Self::new(0.0, self.min.abs().max(self.max.abs()).powi(2))
        } else {
            let a = self.min.powi(2);
            let b = self.max.powi(2);
            Self::new(a.min(b), a.max(b))
        }
    }

    /// Square root of the interval (requires non-negative)
    #[inline]
    pub fn sqrt(&self) -> Option<Self> {
        if self.min < 0.0 {
            None
        } else {
            Some(Self::new(self.min.sqrt(), self.max.sqrt()))
        }
    }

    /// Clamp a value to this interval
    #[inline]
    pub fn clamp(&self, x: f64) -> f64 {
        x.clamp(self.min, self.max)
    }

    /// Linear interpolation within the interval
    #[inline]
    pub fn lerp(&self, t: f64) -> f64 {
        crate::lerp(self.min, self.max, t)
    }

    /// Inverse lerp: find t such that lerp(t) = x
    #[inline]
    pub fn inverse_lerp(&self, x: f64) -> f64 {
        if self.width() == 0.0 {
            0.0
        } else {
            (x - self.min) / self.width()
        }
    }

    /// Check if approximately equal to another interval
    #[inline]
    pub fn approx_eq(&self, other: &Interval, tol: f64) -> bool {
        (self.min - other.min).abs() <= tol && (self.max - other.max).abs() <= tol
    }

    /// Create from array [min, max]
    #[inline]
    pub fn from_array(arr: [f64; 2]) -> Self {
        Self::new(arr[0], arr[1])
    }

    /// Convert to array [min, max]
    #[inline]
    pub fn to_array(&self) -> [f64; 2] {
        [self.min, self.max]
    }
}

impl Default for Interval {
    fn default() -> Self {
        Self::empty()
    }
}

impl Add for Interval {
    type Output = Interval;

    #[inline]
    fn add(self, rhs: Interval) -> Self::Output {
        Self::new(self.min + rhs.min, self.max + rhs.max)
    }
}

impl Sub for Interval {
    type Output = Interval;

    #[inline]
    fn sub(self, rhs: Interval) -> Self::Output {
        Self::new(self.min - rhs.max, self.max - rhs.min)
    }
}

impl Mul for Interval {
    type Output = Interval;

    #[inline]
    fn mul(self, rhs: Interval) -> Self::Output {
        let a = self.min * rhs.min;
        let b = self.min * rhs.max;
        let c = self.max * rhs.min;
        let d = self.max * rhs.max;
        Self::new(a.min(b).min(c).min(d), a.max(b).max(c).max(d))
    }
}

impl Mul<f64> for Interval {
    type Output = Interval;

    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        if rhs >= 0.0 {
            Self::new(self.min * rhs, self.max * rhs)
        } else {
            Self::new(self.max * rhs, self.min * rhs)
        }
    }
}

impl Div for Interval {
    type Output = Option<Interval>;

    #[inline]
    fn div(self, rhs: Interval) -> Self::Output {
        // Check if division by zero is possible
        if rhs.contains_zero() {
            if rhs.min == 0.0 && rhs.max == 0.0 {
                return None; // Division by zero
            }
            // Interval straddles zero - result is unbounded
            return Some(Interval::ENTIRE);
        }
        
        let a = self.min / rhs.min;
        let b = self.min / rhs.max;
        let c = self.max / rhs.min;
        let d = self.max / rhs.max;
        Some(Self::new(a.min(b).min(c).min(d), a.max(b).max(c).max(d)))
    }
}

impl Neg for Interval {
    type Output = Interval;

    #[inline]
    fn neg(self) -> Self::Output {
        Self::new(-self.max, -self.min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let i = Interval::new(0.0, 1.0);
        assert_eq!(i.min, 0.0);
        assert_eq!(i.max, 1.0);
    }

    #[test]
    fn test_point() {
        let i = Interval::point(5.0);
        assert_eq!(i.min, 5.0);
        assert_eq!(i.max, 5.0);
        assert!(i.is_point(1e-10));
    }

    #[test]
    fn test_width() {
        let i = Interval::new(1.0, 5.0);
        assert_eq!(i.width(), 4.0);
    }

    #[test]
    fn test_midpoint() {
        let i = Interval::new(0.0, 10.0);
        assert_eq!(i.midpoint(), 5.0);
    }

    #[test]
    fn test_contains() {
        let i = Interval::new(0.0, 10.0);
        assert!(i.contains(5.0));
        assert!(i.contains(0.0));
        assert!(i.contains(10.0));
        assert!(!i.contains(-1.0));
        assert!(!i.contains(11.0));
    }

    #[test]
    fn test_intersects() {
        let i1 = Interval::new(0.0, 5.0);
        let i2 = Interval::new(3.0, 8.0);
        let i3 = Interval::new(6.0, 10.0);
        assert!(i1.intersects(&i2));
        assert!(!i1.intersects(&i3));
    }

    #[test]
    fn test_intersection() {
        let i1 = Interval::new(0.0, 5.0);
        let i2 = Interval::new(3.0, 8.0);
        let inter = i1.intersection(&i2);
        assert_eq!(inter.min, 3.0);
        assert_eq!(inter.max, 5.0);
    }

    #[test]
    fn test_union() {
        let i1 = Interval::new(0.0, 3.0);
        let i2 = Interval::new(5.0, 8.0);
        let union = i1.union(&i2);
        assert_eq!(union.min, 0.0);
        assert_eq!(union.max, 8.0);
    }

    #[test]
    fn test_add() {
        let i1 = Interval::new(1.0, 2.0);
        let i2 = Interval::new(3.0, 4.0);
        let sum = i1 + i2;
        assert_eq!(sum.min, 4.0);
        assert_eq!(sum.max, 6.0);
    }

    #[test]
    fn test_sub() {
        let i1 = Interval::new(5.0, 10.0);
        let i2 = Interval::new(1.0, 2.0);
        let diff = i1 - i2;
        assert_eq!(diff.min, 3.0);
        assert_eq!(diff.max, 9.0);
    }

    #[test]
    fn test_mul() {
        let i1 = Interval::new(2.0, 3.0);
        let i2 = Interval::new(4.0, 5.0);
        let prod = i1 * i2;
        assert_eq!(prod.min, 8.0);
        assert_eq!(prod.max, 15.0);
    }

    #[test]
    fn test_contains_zero() {
        let i1 = Interval::new(-1.0, 1.0);
        let i2 = Interval::new(1.0, 2.0);
        assert!(i1.contains_zero());
        assert!(!i2.contains_zero());
    }

    #[test]
    fn test_split() {
        let i = Interval::new(0.0, 10.0);
        let (left, right) = i.split();
        assert_eq!(left.min, 0.0);
        assert_eq!(left.max, 5.0);
        assert_eq!(right.min, 5.0);
        assert_eq!(right.max, 10.0);
    }

    #[test]
    fn test_lerp() {
        let i = Interval::new(0.0, 10.0);
        assert_eq!(i.lerp(0.5), 5.0);
        assert_eq!(i.lerp(0.0), 0.0);
        assert_eq!(i.lerp(1.0), 10.0);
    }

    #[test]
    fn test_inverse_lerp() {
        let i = Interval::new(0.0, 10.0);
        assert_eq!(i.inverse_lerp(5.0), 0.5);
        assert_eq!(i.inverse_lerp(0.0), 0.0);
        assert_eq!(i.inverse_lerp(10.0), 1.0);
    }
}
