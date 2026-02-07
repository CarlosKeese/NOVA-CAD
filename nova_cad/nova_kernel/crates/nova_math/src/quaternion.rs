//! Quaternion for 3D rotations

use crate::{Vec3, Mat4};
use nalgebra as na;
use serde::{Deserialize, Serialize};
use std::ops::{Mul, Neg};

/// Unit quaternion for representing 3D rotations
/// 
/// Quaternions provide a compact and numerically stable representation
/// of rotations in 3D space, avoiding gimbal lock issues of Euler angles.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Quaternion {
    pub(crate) inner: na::UnitQuaternion<f64>,
}

impl Quaternion {
    /// Identity quaternion (no rotation)
    pub fn identity() -> Self {
        Self {
            inner: na::UnitQuaternion::identity(),
        }
    }

    /// Create a quaternion from components (will be normalized)
    #[inline]
    pub fn new(w: f64, x: f64, y: f64, z: f64) -> Self {
        let q = na::Quaternion::new(w, x, y, z);
        Self {
            inner: na::UnitQuaternion::from_quaternion(q.normalize()),
        }
    }

    /// Create from raw quaternion (assumes already normalized)
    #[inline]
    pub fn from_normalized(w: f64, x: f64, y: f64, z: f64) -> Self {
        Self {
            inner: na::UnitQuaternion::new_unchecked(na::Quaternion::new(w, x, y, z)),
        }
    }

    /// Create a rotation around an axis
    #[inline]
    pub fn from_axis_angle(axis: &Vec3, angle: f64) -> Self {
        Self {
            inner: na::UnitQuaternion::from_axis_angle(
                &na::Unit::new_normalize(axis.to_nalgebra()),
                angle,
            ),
        }
    }

    /// Create from Euler angles (roll, pitch, yaw in radians)
    /// 
    /// Applied in order: roll (X), pitch (Y), yaw (Z)
    #[inline]
    pub fn from_euler_angles(roll: f64, pitch: f64, yaw: f64) -> Self {
        Self {
            inner: na::UnitQuaternion::from_euler_angles(roll, pitch, yaw),
        }
    }

    /// Create from a rotation matrix
    #[inline]
    pub fn from_rotation_matrix(m: &na::Rotation3<f64>) -> Self {
        Self {
            inner: na::UnitQuaternion::from_rotation_matrix(m),
        }
    }

    /// W component (scalar part)
    #[inline]
    pub fn w(&self) -> f64 {
        self.inner.w
    }

    /// X component (vector part)
    #[inline]
    pub fn x(&self) -> f64 {
        self.inner.i
    }

    /// Y component (vector part)
    #[inline]
    pub fn y(&self) -> f64 {
        self.inner.j
    }

    /// Z component (vector part)
    #[inline]
    pub fn z(&self) -> f64 {
        self.inner.k
    }

    /// Get all components as (w, x, y, z)
    #[inline]
    pub fn components(&self) -> (f64, f64, f64, f64) {
        (self.w(), self.x(), self.y(), self.z())
    }

    /// Get the vector/imaginary part
    #[inline]
    pub fn vector_part(&self) -> Vec3 {
        Vec3::new(self.x(), self.y(), self.z())
    }

    /// Get the rotation axis
    #[inline]
    pub fn axis(&self) -> Option<Vec3> {
        self.inner.axis().map(|a| Vec3::from_nalgebra(a.into_inner()))
    }

    /// Get the rotation angle in radians
    #[inline]
    pub fn angle(&self) -> f64 {
        self.inner.angle()
    }

    /// Conjugate (inverse for unit quaternion)
    #[inline]
    pub fn conjugate(&self) -> Self {
        Self {
            inner: self.inner.conjugate(),
        }
    }

    /// Inverse rotation
    #[inline]
    pub fn inverse(&self) -> Self {
        Self {
            inner: self.inner.inverse(),
        }
    }

    /// Spherical linear interpolation with another quaternion
    #[inline]
    pub fn slerp(&self, other: &Quaternion, t: f64) -> Self {
        Self {
            inner: self.inner.slerp(&other.inner, t),
        }
    }

    /// Normalized linear interpolation (faster than slerp, less accurate)
    #[inline]
    pub fn nlerp(&self, other: &Quaternion, t: f64) -> Self {
        let result = self.inner.into_inner().lerp(&other.inner.into_inner(), t);
        Self {
            inner: na::UnitQuaternion::from_quaternion(result.normalize()),
        }
    }

    /// Convert to Euler angles (roll, pitch, yaw in radians)
    #[inline]
    pub fn to_euler_angles(&self) -> (f64, f64, f64) {
        self.inner.euler_angles()
    }

    /// Convert to 4x4 rotation matrix
    #[inline]
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_nalgebra(self.inner.to_homogeneous())
    }

    /// Convert to 3x3 rotation matrix
    #[inline]
    pub fn to_rotation_matrix(&self) -> na::Rotation3<f64> {
        self.inner.to_rotation_matrix()
    }

    /// Rotate a vector
    #[inline]
    pub fn rotate_vector(&self, v: &Vec3) -> Vec3 {
        Vec3::from_nalgebra(self.inner.transform_vector(&v.to_nalgebra()))
    }

    /// Angle between this quaternion and another
    #[inline]
    pub fn angle_to(&self, other: &Quaternion) -> f64 {
        self.inner.angle_to(&other.inner)
    }

    /// Check if approximately equal to another quaternion
    #[inline]
    pub fn approx_eq(&self, other: &Quaternion, tol: f64) -> bool {
        // Quaternions q and -q represent the same rotation
        let dot = self.w() * other.w() + self.x() * other.x() + 
                  self.y() * other.y() + self.z() * other.z();
        (dot.abs() - 1.0).abs() <= tol
    }

    /// Convert to nalgebra UnitQuaternion
    #[inline]
    pub fn to_nalgebra(&self) -> na::UnitQuaternion<f64> {
        self.inner
    }

    /// Create from nalgebra UnitQuaternion
    #[inline]
    pub fn from_nalgebra(q: na::UnitQuaternion<f64>) -> Self {
        Self { inner: q }
    }

    /// Convert to array [w, x, y, z]
    #[inline]
    pub fn to_array(&self) -> [f64; 4] {
        [self.w(), self.x(), self.y(), self.z()]
    }

    /// Create from array [w, x, y, z]
    #[inline]
    pub fn from_array(arr: [f64; 4]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3])
    }
}

impl Default for Quaternion {
    fn default() -> Self {
        Self::identity()
    }
}

impl Mul for Quaternion {
    type Output = Quaternion;

    #[inline]
    fn mul(self, rhs: Quaternion) -> Self::Output {
        Quaternion {
            inner: self.inner * rhs.inner,
        }
    }
}

impl Neg for Quaternion {
    type Output = Quaternion;

    #[inline]
    fn neg(self) -> Self::Output {
        Quaternion {
            inner: na::UnitQuaternion::new_unchecked(-self.inner.into_inner()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let q = Quaternion::IDENTITY;
        let v = Vec3::new(1.0, 0.0, 0.0);
        let result = q.rotate_vector(&v);
        assert!((result.x() - 1.0).abs() < 1e-10);
        assert!((result.y() - 0.0).abs() < 1e-10);
        assert!((result.z() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotation_x() {
        let q = Quaternion::from_axis_angle(&Vec3::X, std::f64::consts::FRAC_PI_2);
        let v = Vec3::Y;
        let result = q.rotate_vector(&v);
        assert!((result.x() - 0.0).abs() < 1e-10);
        assert!((result.y() - 0.0).abs() < 1e-10);
        assert!((result.z() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotation_y() {
        let q = Quaternion::from_axis_angle(&Vec3::Y, std::f64::consts::FRAC_PI_2);
        let v = Vec3::Z;
        let result = q.rotate_vector(&v);
        assert!((result.x() - 1.0).abs() < 1e-10);
        assert!((result.y() - 0.0).abs() < 1e-10);
        assert!((result.z() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotation_z() {
        let q = Quaternion::from_axis_angle(&Vec3::Z, std::f64::consts::FRAC_PI_2);
        let v = Vec3::X;
        let result = q.rotate_vector(&v);
        assert!((result.x() - 0.0).abs() < 1e-10);
        assert!((result.y() - 1.0).abs() < 1e-10);
        assert!((result.z() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_quaternion_multiply() {
        let q1 = Quaternion::from_axis_angle(&Vec3::Z, std::f64::consts::FRAC_PI_2);
        let q2 = Quaternion::from_axis_angle(&Vec3::Z, std::f64::consts::FRAC_PI_2);
        let combined = q1 * q2;
        let v = Vec3::X;
        let result = combined.rotate_vector(&v);
        // 90° + 90° = 180° rotation around Z
        assert!((result.x() + 1.0).abs() < 1e-10);
        assert!((result.y() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_conjugate() {
        let q = Quaternion::from_axis_angle(&Vec3::Z, std::f64::consts::FRAC_PI_4);
        let q_conj = q.conjugate();
        let v = Vec3::new(1.0, 0.0, 0.0);
        let rotated = q.rotate_vector(&v);
        let back = q_conj.rotate_vector(&rotated);
        assert!((back.x() - 1.0).abs() < 1e-10);
        assert!((back.y() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_slerp() {
        let q1 = Quaternion::IDENTITY;
        let q2 = Quaternion::from_axis_angle(&Vec3::Z, std::f64::consts::PI);
        let mid = q1.slerp(&q2, 0.5);
        let angle = mid.angle();
        assert!((angle - std::f64::consts::FRAC_PI_2).abs() < 1e-10);
    }

    #[test]
    fn test_euler_angles() {
        let roll = 0.1;
        let pitch = 0.2;
        let yaw = 0.3;
        let q = Quaternion::from_euler_angles(roll, pitch, yaw);
        let (r, p, y) = q.to_euler_angles();
        assert!((r - roll).abs() < 1e-10);
        assert!((p - pitch).abs() < 1e-10);
        assert!((y - yaw).abs() < 1e-10);
    }
}
