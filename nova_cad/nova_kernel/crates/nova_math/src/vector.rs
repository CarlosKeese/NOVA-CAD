//! Vector types for 2D, 3D, and 4D space

use nalgebra as na;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub, Mul, Div, Neg, Index, IndexMut, AddAssign, SubAssign, MulAssign, DivAssign};

/// 3D vector with f64 components
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec3 {
    pub(crate) inner: na::Vector3<f64>,
}

impl Vec3 {
    /// Zero vector
    pub const ZERO: Self = Self {
        inner: na::Vector3::new(0.0, 0.0, 0.0),
    };

    /// Unit X vector
    pub const X: Self = Self {
        inner: na::Vector3::new(1.0, 0.0, 0.0),
    };

    /// Unit Y vector
    pub const Y: Self = Self {
        inner: na::Vector3::new(0.0, 1.0, 0.0),
    };

    /// Unit Z vector
    pub const Z: Self = Self {
        inner: na::Vector3::new(0.0, 0.0, 1.0),
    };

    /// Create a new vector
    #[inline]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            inner: na::Vector3::new(x, y, z),
        }
    }

    /// X component
    #[inline]
    pub fn x(&self) -> f64 {
        self.inner.x
    }

    /// Y component
    #[inline]
    pub fn y(&self) -> f64 {
        self.inner.y
    }

    /// Z component
    #[inline]
    pub fn z(&self) -> f64 {
        self.inner.z
    }

    /// Set X component
    #[inline]
    pub fn set_x(&mut self, x: f64) {
        self.inner.x = x;
    }

    /// Set Y component
    #[inline]
    pub fn set_y(&mut self, y: f64) {
        self.inner.y = y;
    }

    /// Set Z component
    #[inline]
    pub fn set_z(&mut self, z: f64) {
        self.inner.z = z;
    }

    /// Vector length (magnitude)
    #[inline]
    pub fn length(&self) -> f64 {
        self.inner.norm()
    }

    /// Squared length (faster, avoids sqrt)
    #[inline]
    pub fn length_squared(&self) -> f64 {
        self.inner.norm_squared()
    }

    /// Normalize to unit length
    #[inline]
    pub fn normalize(&mut self) {
        self.inner.normalize_mut();
    }

    /// Return normalized copy
    #[inline]
    pub fn normalized(&self) -> Self {
        Self {
            inner: self.inner.normalize(),
        }
    }

    /// Check if vector is normalized (unit length)
    #[inline]
    pub fn is_normalized(&self, tol: f64) -> bool {
        (self.length_squared() - 1.0).abs() <= tol
    }

    /// Check if vector is zero
    #[inline]
    pub fn is_zero(&self, tol: f64) -> bool {
        self.length_squared() <= tol * tol
    }

    /// Dot product with another vector
    #[inline]
    pub fn dot(&self, other: &Vec3) -> f64 {
        self.inner.dot(&other.inner)
    }

    /// Cross product with another vector
    #[inline]
    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3 {
            inner: self.inner.cross(&other.inner),
        }
    }

    /// Angle between this vector and another (in radians)
    #[inline]
    pub fn angle_to(&self, other: &Vec3) -> f64 {
        self.inner.angle(&other.inner)
    }

    /// Project this vector onto another
    #[inline]
    pub fn project_onto(&self, other: &Vec3) -> Vec3 {
        let other_norm_sq = other.length_squared();
        if other_norm_sq < f64::EPSILON {
            return Vec3::ZERO;
        }
        let scale = self.dot(other) / other_norm_sq;
        *other * scale
    }

    /// Reject this vector from another (component perpendicular to other)
    #[inline]
    pub fn reject_from(&self, other: &Vec3) -> Vec3 {
        *self - self.project_onto(other)
    }

    /// Reflect this vector across a normal
    #[inline]
    pub fn reflect(&self, normal: &Vec3) -> Vec3 {
        *self - *normal * (2.0 * self.dot(normal))
    }

    /// Linear interpolation with another vector
    #[inline]
    pub fn lerp(&self, other: &Vec3, t: f64) -> Vec3 {
        Vec3::new(
            crate::lerp(self.x(), other.x(), t),
            crate::lerp(self.y(), other.y(), t),
            crate::lerp(self.z(), other.z(), t),
        )
    }

    /// Spherical linear interpolation with another vector
    #[inline]
    pub fn slerp(&self, other: &Vec3, t: f64) -> Vec3 {
        Vec3 {
            inner: self.inner.slerp(&other.inner, t),
        }
    }

    /// Scale to a specific length
    #[inline]
    pub fn with_length(&self, length: f64) -> Vec3 {
        self.normalized() * length
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

    /// Convert to nalgebra vector
    #[inline]
    pub fn to_nalgebra(&self) -> na::Vector3<f64> {
        self.inner
    }

    /// Create from nalgebra vector
    #[inline]
    pub fn from_nalgebra(v: na::Vector3<f64>) -> Self {
        Self { inner: v }
    }
}

impl Default for Vec3 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Add for Vec3 {
    type Output = Vec3;

    #[inline]
    fn add(self, rhs: Vec3) -> Self::Output {
        Vec3 { inner: self.inner + rhs.inner }
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    #[inline]
    fn sub(self, rhs: Vec3) -> Self::Output {
        Vec3 { inner: self.inner - rhs.inner }
    }
}

impl Mul<f64> for Vec3 {
    type Output = Vec3;

    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        Vec3 { inner: self.inner * rhs }
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;

    #[inline]
    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs * self
    }
}

impl Div<f64> for Vec3 {
    type Output = Vec3;

    #[inline]
    fn div(self, rhs: f64) -> Self::Output {
        Vec3 { inner: self.inner / rhs }
    }
}

impl Neg for Vec3 {
    type Output = Vec3;

    #[inline]
    fn neg(self) -> Self::Output {
        Vec3 { inner: -self.inner }
    }
}

impl AddAssign for Vec3 {
    #[inline]
    fn add_assign(&mut self, rhs: Vec3) {
        self.inner += rhs.inner;
    }
}

impl SubAssign for Vec3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Vec3) {
        self.inner -= rhs.inner;
    }
}

impl MulAssign<f64> for Vec3 {
    #[inline]
    fn mul_assign(&mut self, rhs: f64) {
        self.inner *= rhs;
    }
}

impl DivAssign<f64> for Vec3 {
    #[inline]
    fn div_assign(&mut self, rhs: f64) {
        self.inner /= rhs;
    }
}

impl Index<usize> for Vec3 {
    type Output = f64;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl IndexMut<usize> for Vec3 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

/// 2D vector with f64 components
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub(crate) inner: na::Vector2<f64>,
}

impl Vec2 {
    /// Zero vector
    pub const ZERO: Self = Self {
        inner: na::Vector2::new(0.0, 0.0),
    };

    /// Create a new vector
    #[inline]
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            inner: na::Vector2::new(x, y),
        }
    }

    #[inline]
    pub fn x(&self) -> f64 { self.inner.x }
    
    #[inline]
    pub fn y(&self) -> f64 { self.inner.y }

    #[inline]
    pub fn length(&self) -> f64 {
        self.inner.norm()
    }

    #[inline]
    pub fn dot(&self, other: &Vec2) -> f64 {
        self.inner.dot(&other.inner)
    }

    #[inline]
    pub fn perp(&self) -> Vec2 {
        Vec2::new(-self.y(), self.x())
    }

    #[inline]
    pub fn cross(&self, other: &Vec2) -> f64 {
        self.x() * other.y() - self.y() * other.x()
    }
}

/// 4D vector with f64 components
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec4 {
    pub(crate) inner: na::Vector4<f64>,
}

impl Vec4 {
    /// Create a new vector
    #[inline]
    pub fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self {
            inner: na::Vector4::new(x, y, z, w),
        }
    }

    #[inline]
    pub fn x(&self) -> f64 { self.inner.x }
    #[inline]
    pub fn y(&self) -> f64 { self.inner.y }
    #[inline]
    pub fn z(&self) -> f64 { self.inner.z }
    #[inline]
    pub fn w(&self) -> f64 { self.inner.w }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3_new() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(v.x(), 1.0);
        assert_eq!(v.y(), 2.0);
        assert_eq!(v.z(), 3.0);
    }

    #[test]
    fn test_dot_product() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        assert_eq!(v1.dot(&v2), 32.0); // 1*4 + 2*5 + 3*6
    }

    #[test]
    fn test_cross_product() {
        let v1 = Vec3::X;
        let v2 = Vec3::Y;
        let cross = v1.cross(&v2);
        assert!((cross.x() - 0.0).abs() < 1e-10);
        assert!((cross.y() - 0.0).abs() < 1e-10);
        assert!((cross.z() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_normalize() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        let n = v.normalized();
        assert!((n.length() - 1.0).abs() < 1e-10);
        assert!((n.x() - 0.6).abs() < 1e-10);
        assert!((n.y() - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_angle_between() {
        let v1 = Vec3::X;
        let v2 = Vec3::Y;
        let angle = v1.angle_to(&v2);
        assert!((angle - std::f64::consts::FRAC_PI_2).abs() < 1e-10);
    }

    #[test]
    fn test_project_onto() {
        let v = Vec3::new(1.0, 1.0, 0.0);
        let onto = Vec3::X;
        let proj = v.project_onto(&onto);
        assert!((proj.x() - 1.0).abs() < 1e-10);
        assert!((proj.y() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_lerp() {
        let v1 = Vec3::new(0.0, 0.0, 0.0);
        let v2 = Vec3::new(10.0, 20.0, 30.0);
        let mid = v1.lerp(&v2, 0.5);
        assert_eq!(mid.x(), 5.0);
        assert_eq!(mid.y(), 10.0);
        assert_eq!(mid.z(), 15.0);
    }
}
