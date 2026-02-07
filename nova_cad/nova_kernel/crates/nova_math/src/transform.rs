//! 3D rigid body transformations

use crate::{Vec3, Point3, Mat4, Quaternion};
use nalgebra as na;
use serde::{Deserialize, Serialize};
use std::ops::Mul;

/// 3D rigid body transformation (rotation + translation)
/// 
/// This represents an isometry in 3D space - a transformation that preserves
/// distances and angles. It consists of a rotation (stored as a unit quaternion)
/// and a translation vector.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform3 {
    /// Rotation component
    rotation: na::UnitQuaternion<f64>,
    /// Translation component
    translation: na::Vector3<f64>,
}

impl Transform3 {
    /// Identity transformation (no rotation, no translation)
    pub const IDENTITY: Self = Self {
        rotation: na::UnitQuaternion::identity(),
        translation: na::Vector3::new(0.0, 0.0, 0.0),
    };

    /// Create a new transformation from rotation and translation
    #[inline]
    pub fn new(rotation: &Quaternion, translation: Vec3) -> Self {
        Self {
            rotation: rotation.to_nalgebra(),
            translation: translation.to_nalgebra(),
        }
    }

    /// Create a pure translation
    #[inline]
    pub fn from_translation(x: f64, y: f64, z: f64) -> Self {
        Self {
            rotation: na::UnitQuaternion::identity(),
            translation: na::Vector3::new(x, y, z),
        }
    }

    /// Create a pure translation from vector
    #[inline]
    pub fn from_translation_vec(translation: Vec3) -> Self {
        Self::from_translation(translation.x(), translation.y(), translation.z())
    }

    /// Create a pure rotation from quaternion
    #[inline]
    pub fn from_rotation(rotation: &Quaternion) -> Self {
        Self {
            rotation: rotation.to_nalgebra(),
            translation: na::Vector3::zeros(),
        }
    }

    /// Create a rotation around an axis
    #[inline]
    pub fn from_axis_angle(axis: &Vec3, angle: f64) -> Self {
        Self {
            rotation: na::UnitQuaternion::from_axis_angle(
                &na::Unit::new_normalize(axis.to_nalgebra()),
                angle,
            ),
            translation: na::Vector3::zeros(),
        }
    }

    /// Create from Euler angles (roll, pitch, yaw in radians)
    #[inline]
    pub fn from_euler_angles(roll: f64, pitch: f64, yaw: f64) -> Self {
        Self {
            rotation: na::UnitQuaternion::from_euler_angles(roll, pitch, yaw),
            translation: na::Vector3::zeros(),
        }
    }

    /// Create a look-at transformation
    #[inline]
    pub fn look_at_lh(eye: &Point3, target: &Point3, up: &Vec3) -> Self {
        let rotation = na::UnitQuaternion::look_at_lh(&(target - eye).to_nalgebra(), &up.to_nalgebra());
        Self {
            rotation,
            translation: eye.to_nalgebra().coords,
        }
    }

    /// Get the rotation component
    #[inline]
    pub fn rotation(&self) -> Quaternion {
        Quaternion::from_nalgebra(self.rotation)
    }

    /// Get the translation component
    #[inline]
    pub fn translation(&self) -> Vec3 {
        Vec3::from_nalgebra(self.translation)
    }

    /// Get the translation as a point
    #[inline]
    pub fn translation_point(&self) -> Point3 {
        Point3::new(self.translation.x, self.translation.y, self.translation.z)
    }

    /// Apply transformation to a point
    #[inline]
    pub fn apply_to_point(&self, point: &Point3) -> Point3 {
        Point3::from_nalgebra(self.rotation.transform_point(&point.to_nalgebra()) + self.translation)
    }

    /// Apply transformation to a vector (ignores translation)
    #[inline]
    pub fn apply_to_vector(&self, vector: &Vec3) -> Vec3 {
        Vec3::from_nalgebra(self.rotation.transform_vector(&vector.to_nalgebra()))
    }

    /// Apply the inverse transformation to a point
    #[inline]
    pub fn inverse_apply_to_point(&self, point: &Point3) -> Point3 {
        let translated = point.to_nalgebra() - self.translation;
        Point3::from_nalgebra(self.rotation.inverse().transform_point(&translated))
    }

    /// Compose two transformations (self * other)
    #[inline]
    pub fn compose(&self, other: &Transform3) -> Transform3 {
        Transform3 {
            rotation: self.rotation * other.rotation,
            translation: self.rotation.transform_vector(&other.translation) + self.translation,
        }
    }

    /// Inverse transformation
    #[inline]
    pub fn inverse(&self) Transform3 {
        let inv_rotation = self.rotation.inverse();
        Transform3 {
            rotation: inv_rotation,
            translation: -inv_rotation.transform_vector(&self.translation),
        }
    }

    /// Linear interpolation between transformations
    #[inline]
    pub fn lerp(&self, other: &Transform3, t: f64) -> Transform3 {
        Transform3 {
            rotation: self.rotation.slerp(&other.rotation, t),
            translation: na::Vector3::lerp(&self.translation, &other.translation, t),
        }
    }

    /// Convert to 4x4 matrix
    #[inline]
    pub fn to_matrix(&self) -> Mat4 {
        let mut m = Mat4::from_nalgebra(self.rotation.to_homogeneous());
        m.set(0, 3, self.translation.x);
        m.set(1, 3, self.translation.y);
        m.set(2, 3, self.translation.z);
        m
    }

    /// Create from 4x4 matrix (assumes valid isometry)
    #[inline]
    pub fn from_matrix(matrix: &Mat4) -> Option<Self> {
        let rotation_mat = matrix.rotation_scale();
        let na_mat = rotation_mat.to_nalgebra();
        
        // Check if the matrix represents a valid rotation (orthogonal with det = 1)
        let det = na_mat.determinant();
        if (det - 1.0).abs() > 1e-6 {
            return None;
        }

        let rotation = na::UnitQuaternion::from_matrix(&na_mat);
        Some(Self {
            rotation,
            translation: matrix.translation().to_nalgebra(),
        })
    }

    /// Convert to nalgebra Isometry3
    #[inline]
    pub fn to_nalgebra(&self) -> na::Isometry3<f64> {
        na::Isometry3::from_parts(
            na::Translation3::from(self.translation),
            self.rotation,
        )
    }

    /// Create from nalgebra Isometry3
    #[inline]
    pub fn from_nalgebra(iso: na::Isometry3<f64>) -> Self {
        Self {
            rotation: iso.rotation,
            translation: iso.translation.vector,
        }
    }

    /// Check if this is approximately equal to another transform
    #[inline]
    pub fn approx_eq(&self, other: &Transform3, pos_tol: f64, rot_tol: f64) -> bool {
        let pos_diff = (self.translation - other.translation).norm();
        let rot_diff = self.rotation.angle_to(&other.rotation);
        pos_diff <= pos_tol && rot_diff <= rot_tol
    }
}

impl Default for Transform3 {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Mul for Transform3 {
    type Output = Transform3;

    #[inline]
    fn mul(self, rhs: Transform3) -> Self::Output {
        self.compose(&rhs)
    }
}

impl Mul<Point3> for Transform3 {
    type Output = Point3;

    #[inline]
    fn mul(self, rhs: Point3) -> Self::Output {
        self.apply_to_point(&rhs)
    }
}

impl Mul<Vec3> for Transform3 {
    type Output = Vec3;

    #[inline]
    fn mul(self, rhs: Vec3) -> Self::Output {
        self.apply_to_vector(&rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let t = Transform3::IDENTITY;
        let p = Point3::new(1.0, 2.0, 3.0);
        let result = t.apply_to_point(&p);
        assert!((result.x() - 1.0).abs() < 1e-10);
        assert!((result.y() - 2.0).abs() < 1e-10);
        assert!((result.z() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_translation() {
        let t = Transform3::from_translation(10.0, 20.0, 30.0);
        let p = Point3::new(1.0, 2.0, 3.0);
        let result = t.apply_to_point(&p);
        assert!((result.x() - 11.0).abs() < 1e-10);
        assert!((result.y() - 22.0).abs() < 1e-10);
        assert!((result.z() - 33.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotation_x() {
        let t = Transform3::from_axis_angle(&Vec3::X, std::f64::consts::FRAC_PI_2);
        let p = Point3::new(0.0, 1.0, 0.0);
        let result = t.apply_to_point(&p);
        assert!((result.x() - 0.0).abs() < 1e-10);
        assert!((result.y() - 0.0).abs() < 1e-10);
        assert!((result.z() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_compose() {
        let t1 = Transform3::from_translation(1.0, 0.0, 0.0);
        let t2 = Transform3::from_translation(0.0, 1.0, 0.0);
        let combined = t1.compose(&t2);
        let p = Point3::new(0.0, 0.0, 0.0);
        let result = combined.apply_to_point(&p);
        assert!((result.x() - 1.0).abs() < 1e-10);
        assert!((result.y() - 1.0).abs() < 1e-10);
        assert!((result.z() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_inverse() {
        let t = Transform3::from_translation(10.0, 20.0, 30.0);
        let inv = t.inverse();
        let p = Point3::new(11.0, 22.0, 33.0);
        let result = inv.apply_to_point(&p);
        assert!((result.x() - 1.0).abs() < 1e-10);
        assert!((result.y() - 2.0).abs() < 1e-10);
        assert!((result.z() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_to_matrix() {
        let t = Transform3::from_translation(1.0, 2.0, 3.0);
        let m = t.to_matrix();
        let p = Point3::new(0.0, 0.0, 0.0);
        let result = m * p;
        assert!((result.x() - 1.0).abs() < 1e-10);
        assert!((result.y() - 2.0).abs() < 1e-10);
        assert!((result.z() - 3.0).abs() < 1e-10);
    }
}
