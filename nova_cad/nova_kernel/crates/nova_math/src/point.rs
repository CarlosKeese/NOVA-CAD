//! Point types for 2D, 3D, and 4D space

use crate::{vector::{Vec2, Vec3, Vec4}, Transform3, BoundingBox3, Bounded};
use nalgebra as na;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub, Mul, Div, Neg, Index, IndexMut};

/// 3D point with f64 coordinates
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point3 {
    /// Internal nalgebra point
    pub(crate) inner: na::Point3<f64>,
}

impl Point3 {
    /// Origin point (0, 0, 0)
    pub const ORIGIN: Self = Self {
        inner: na::Point3::new(0.0, 0.0, 0.0),
    };

    /// Create a new point from coordinates
    #[inline]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            inner: na::Point3::new(x, y, z),
        }
    }

    /// X coordinate
    #[inline]
    pub fn x(&self) -> f64 {
        self.inner.x
    }

    /// Y coordinate
    #[inline]
    pub fn y(&self) -> f64 {
        self.inner.y
    }

    /// Z coordinate
    #[inline]
    pub fn z(&self) -> f64 {
        self.inner.z
    }

    /// Set X coordinate
    #[inline]
    pub fn set_x(&mut self, x: f64) {
        self.inner.x = x;
    }

    /// Set Y coordinate
    #[inline]
    pub fn set_y(&mut self, y: f64) {
        self.inner.y = y;
    }

    /// Set Z coordinate
    #[inline]
    pub fn set_z(&mut self, z: f64) {
        self.inner.z = z;
    }

    /// Distance to another point
    #[inline]
    pub fn distance_to(&self, other: &Point3) -> f64 {
        (self.inner - other.inner).norm()
    }

    /// Squared distance to another point (faster, avoids sqrt)
    #[inline]
    pub fn distance_squared_to(&self, other: &Point3) -> f64 {
        (self.inner - other.inner).norm_squared()
    }

    /// Midpoint between this point and another
    #[inline]
    pub fn midpoint(&self, other: &Point3) -> Point3 {
        Point3::new(
            (self.x() + other.x()) * 0.5,
            (self.y() + other.y()) * 0.5,
            (self.z() + other.z()) * 0.5,
        )
    }

    /// Linear interpolation between this point and another
    #[inline]
    pub fn lerp(&self, other: &Point3, t: f64) -> Point3 {
        Point3::new(
            crate::lerp(self.x(), other.x(), t),
            crate::lerp(self.y(), other.y(), t),
            crate::lerp(self.z(), other.z(), t),
        )
    }

    /// Convert to homogeneous coordinates (x, y, z, 1)
    #[inline]
    pub fn to_homogeneous(&self) -> na::Point4<f64> {
        na::Point4::new(self.x(), self.y(), self.z(), 1.0)
    }

    /// Convert to vector (from origin)
    #[inline]
    pub fn to_vector(&self) -> Vec3 {
        Vec3::new(self.x(), self.y(), self.z())
    }

    /// Convert to array
    #[inline]
    pub fn to_array(&self) -> [f64; 3] {
        [self.x(), self.y(), self.z()]
    }

    /// Create from array
    #[inline]
    pub fn from_array(arr: [f64; 3]) -> Self {
        Self::new(arr[0], arr[1], arr[2])
    }

    /// Convert to nalgebra point
    #[inline]
    pub fn to_nalgebra(&self) -> na::Point3<f64> {
        self.inner
    }

    /// Create from nalgebra point
    #[inline]
    pub fn from_nalgebra(p: na::Point3<f64>) -> Self {
        Self { inner: p }
    }
}

impl Default for Point3 {
    fn default() -> Self {
        Self::ORIGIN
    }
}

impl Add<Vec3> for Point3 {
    type Output = Point3;

    #[inline]
    fn add(self, rhs: Vec3) -> Self::Output {
        Point3::from_nalgebra(self.inner + rhs.to_nalgebra())
    }
}

impl Sub<Vec3> for Point3 {
    type Output = Point3;

    #[inline]
    fn sub(self, rhs: Vec3) -> Self::Output {
        Point3::from_nalgebra(self.inner - rhs.to_nalgebra())
    }
}

impl Sub<Point3> for Point3 {
    type Output = Vec3;

    #[inline]
    fn sub(self, rhs: Point3) -> Self::Output {
        Vec3::from_nalgebra(self.inner - rhs.inner)
    }
}

impl Index<usize> for Point3 {
    type Output = f64;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl IndexMut<usize> for Point3 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

impl Bounded for Point3 {
    fn bounding_box(&self) -> BoundingBox3 {
        BoundingBox3::from_point(*self)
    }
}

/// 2D point with f64 coordinates
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point2 {
    pub(crate) inner: na::Point2<f64>,
}

impl Point2 {
    /// Origin point (0, 0)
    pub const ORIGIN: Self = Self {
        inner: na::Point2::new(0.0, 0.0),
    };

    /// Create a new point
    #[inline]
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            inner: na::Point2::new(x, y),
        }
    }

    /// X coordinate
    #[inline]
    pub fn x(&self) -> f64 {
        self.inner.x
    }

    /// Y coordinate
    #[inline]
    pub fn y(&self) -> f64 {
        self.inner.y
    }

    /// Distance to another point
    #[inline]
    pub fn distance_to(&self, other: &Point2) -> f64 {
        (self.inner - other.inner).norm()
    }

    /// Convert to array
    #[inline]
    pub fn to_array(&self) -> [f64; 2] {
        [self.x(), self.y()]
    }
}

/// 4D point (homogeneous coordinates)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point4 {
    pub(crate) inner: na::Point4<f64>,
}

impl Point4 {
    /// Create a new point
    #[inline]
    pub fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self {
            inner: na::Point4::new(x, y, z, w),
        }
    }

    /// X coordinate
    #[inline]
    pub fn x(&self) -> f64 {
        self.inner.x
    }

    /// Y coordinate
    #[inline]
    pub fn y(&self) -> f64 {
        self.inner.y
    }

    /// Z coordinate
    #[inline]
    pub fn z(&self) -> f64 {
        self.inner.z
    }

    /// W coordinate (homogeneous)
    #[inline]
    pub fn w(&self) -> f64 {
        self.inner.w
    }

    /// Project to 3D (divide by w)
    #[inline]
    pub fn to_point3(&self) -> Point3 {
        Point3::from_nalgebra(self.inner.xyz() / self.inner.w)
    }

    /// Convert from nalgebra
    #[inline]
    pub fn from_nalgebra(p: na::Point4<f64>) -> Self {
        Self { inner: p }
    }

    /// Convert to nalgebra
    #[inline]
    pub fn to_nalgebra(&self) -> na::Point4<f64> {
        self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point3_new() {
        let p = Point3::new(1.0, 2.0, 3.0);
        assert_eq!(p.x(), 1.0);
        assert_eq!(p.y(), 2.0);
        assert_eq!(p.z(), 3.0);
    }

    #[test]
    fn test_distance() {
        let p1 = Point3::new(0.0, 0.0, 0.0);
        let p2 = Point3::new(3.0, 4.0, 0.0);
        assert_eq!(p1.distance_to(&p2), 5.0);
    }

    #[test]
    fn test_midpoint() {
        let p1 = Point3::new(0.0, 0.0, 0.0);
        let p2 = Point3::new(2.0, 4.0, 6.0);
        let mid = p1.midpoint(&p2);
        assert_eq!(mid.x(), 1.0);
        assert_eq!(mid.y(), 2.0);
        assert_eq!(mid.z(), 3.0);
    }

    #[test]
    fn test_lerp() {
        let p1 = Point3::new(0.0, 0.0, 0.0);
        let p2 = Point3::new(10.0, 20.0, 30.0);
        let mid = p1.lerp(&p2, 0.5);
        assert_eq!(mid.x(), 5.0);
        assert_eq!(mid.y(), 10.0);
        assert_eq!(mid.z(), 15.0);
    }

    #[test]
    fn test_vector_subtraction() {
        let p1 = Point3::new(3.0, 4.0, 5.0);
        let p2 = Point3::new(1.0, 1.0, 1.0);
        let v = p1 - p2;
        assert_eq!(v.x(), 2.0);
        assert_eq!(v.y(), 3.0);
        assert_eq!(v.z(), 4.0);
    }

    #[test]
    fn test_point_vector_addition() {
        let p = Point3::new(1.0, 2.0, 3.0);
        let v = Vec3::new(1.0, 1.0, 1.0);
        let result = p + v;
        assert_eq!(result.x(), 2.0);
        assert_eq!(result.y(), 3.0);
        assert_eq!(result.z(), 4.0);
    }
}
