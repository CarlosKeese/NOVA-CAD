//! Nova Math - Mathematical Foundation for Nova Kernel 3D
//! 
//! Provides core mathematical types and operations:
//! - Points, vectors, matrices in 3D space
//! - Transformations and quaternions
//! - Bounding boxes and intervals
//! - Tolerance-based geometric comparisons
//! - Robust geometric predicates

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod point;
pub mod vector;
pub mod matrix;
pub mod transform;
pub mod quaternion;
pub mod bbox;
pub mod interval;
pub mod tolerance;
pub mod plane;
pub mod predicates;

pub use point::{Point2, Point3, Point4};
pub use vector::{Vec2, Vec3, Vec4};
pub use matrix::{Mat3, Mat4};
pub use transform::Transform3;
pub use quaternion::Quaternion;
pub use bbox::{BoundingBox2, BoundingBox3};
pub use interval::Interval;
pub use tolerance::{Tolerance, ToleranceContext};
pub use plane::Plane;
pub use predicates::{orient2d, orient3d, incircle, insphere};

/// Default absolute resolution (SPAresabs equivalent)
pub const DEFAULT_RESABS: f64 = 1e-6;

/// Default relative resolution
pub const DEFAULT_RESREL: f64 = 1e-10;

/// Default angular tolerance in radians
pub const DEFAULT_ANGLE_TOL: f64 = 1e-10;

/// Common trait for geometric entities that can be transformed
pub trait Transformable {
    /// Apply a transformation to this entity
    fn transform(&mut self, transform: &Transform3);
    
    /// Return a transformed copy
    fn transformed(&self, transform: &Transform3) -> Self where Self: Clone;
}

/// Trait for entities with a bounding box
pub trait Bounded {
    /// Compute axis-aligned bounding box
    fn bounding_box(&self) -> BoundingBox3;
}

/// Trait for geometric evaluation
pub trait Evaluable {
    /// Evaluate at parameter t
    fn evaluate(&self, t: f64) -> Point3;
    
    /// Evaluate derivative at parameter t
    fn derivative(&self, t: f64, order: u32) -> Vec3;
}

/// Linear interpolation between two values
#[inline]
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + t * (b - a)
}

/// Clamp a value to the range [min, max]
#[inline]
pub fn clamp(val: f64, min: f64, max: f64) -> f64 {
    val.max(min).min(max)
}

/// Check if two f64 values are approximately equal within tolerance
#[inline]
pub fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol
}

/// Check if a value is approximately zero within tolerance
#[inline]
pub fn approx_zero(val: f64, tol: f64) -> bool {
    val.abs() <= tol
}

/// Square a value
#[inline]
pub fn sqr(x: f64) -> f64 {
    x * x
}

/// Cube a value
#[inline]
pub fn cube(x: f64) -> f64 {
    x * x * x
}

/// Convert degrees to radians
#[inline]
pub fn to_radians(deg: f64) -> f64 {
    deg * std::f64::consts::PI / 180.0
}

/// Convert radians to degrees
#[inline]
pub fn to_degrees(rad: f64) -> f64 {
    rad * 180.0 / std::f64::consts::PI
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5.0, 0.0, 10.0), 5.0);
        assert_eq!(clamp(-5.0, 0.0, 10.0), 0.0);
        assert_eq!(clamp(15.0, 0.0, 10.0), 10.0);
    }

    #[test]
    fn test_approx_eq() {
        assert!(approx_eq(1.0, 1.000001, 1e-5));
        assert!(!approx_eq(1.0, 1.1, 1e-5));
    }

    #[test]
    fn test_angle_conversions() {
        assert!(approx_eq(to_radians(180.0), std::f64::consts::PI, 1e-10));
        assert!(approx_eq(to_degrees(std::f64::consts::PI), 180.0, 1e-10));
    }
}
