//! Matrix types for 3D and 4D transformations

use crate::{Vec3, Point3, Transform3};
use nalgebra as na;
use serde::{Deserialize, Serialize};
use std::ops::{Mul, Index, IndexMut};

/// 4x4 matrix for homogeneous transformations
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Mat4 {
    pub(crate) inner: na::Matrix4<f64>,
}

impl Mat4 {
    /// Identity matrix
    pub fn identity() -> Self {
        Self {
            inner: na::Matrix4::identity(),
        }
    }

    /// Zero matrix
    pub fn zero() -> Self {
        Self {
            inner: na::Matrix4::zeros(),
        }
    }

    /// Create a new matrix from column-major array
    #[inline]
    pub fn new(m00: f64, m01: f64, m02: f64, m03: f64,
               m10: f64, m11: f64, m12: f64, m13: f64,
               m20: f64, m21: f64, m22: f64, m23: f64,
               m30: f64, m31: f64, m32: f64, m33: f64) -> Self {
        Self {
            inner: na::Matrix4::new(
                m00, m01, m02, m03,
                m10, m11, m12, m13,
                m20, m21, m22, m23,
                m30, m31, m32, m33,
            ),
        }
    }

    /// Create from a 2D array in row-major order
    #[inline]
    pub fn from_row_major(arr: [[f64; 4]; 4]) -> Self {
        Self::new(
            arr[0][0], arr[0][1], arr[0][2], arr[0][3],
            arr[1][0], arr[1][1], arr[1][2], arr[1][3],
            arr[2][0], arr[2][1], arr[2][2], arr[2][3],
            arr[3][0], arr[3][1], arr[3][2], arr[3][3],
        )
    }

    /// Create from a 2D array in column-major order
    #[inline]
    pub fn from_column_major(arr: [[f64; 4]; 4]) -> Self {
        Self::new(
            arr[0][0], arr[1][0], arr[2][0], arr[3][0],
            arr[0][1], arr[1][1], arr[2][1], arr[3][1],
            arr[0][2], arr[1][2], arr[2][2], arr[3][2],
            arr[0][3], arr[1][3], arr[2][3], arr[3][3],
        )
    }

    /// Create a translation matrix
    #[inline]
    pub fn from_translation(x: f64, y: f64, z: f64) -> Self {
        Self {
            inner: na::Matrix4::new_translation(&na::Vector3::new(x, y, z)),
        }
    }

    /// Create a scale matrix
    #[inline]
    pub fn from_scale(x: f64, y: f64, z: f64) -> Self {
        Self {
            inner: na::Matrix4::new_nonuniform_scaling(&na::Vector3::new(x, y, z)),
        }
    }

    /// Create a rotation matrix from axis and angle (radians)
    #[inline]
    pub fn from_axis_angle(axis: &Vec3, angle: f64) -> Self {
        Self {
            inner: na::Matrix4::from_axis_angle(&na::Unit::new_normalize(axis.to_nalgebra()), angle),
        }
    }

    /// Create a rotation matrix around X axis
    #[inline]
    pub fn from_rotation_x(angle: f64) -> Self {
        Self {
            inner: na::Matrix4::from_euler_angles(angle, 0.0, 0.0),
        }
    }

    /// Create a rotation matrix around Y axis
    #[inline]
    pub fn from_rotation_y(angle: f64) -> Self {
        Self {
            inner: na::Matrix4::from_euler_angles(0.0, angle, 0.0),
        }
    }

    /// Create a rotation matrix around Z axis
    #[inline]
    pub fn from_rotation_z(angle: f64) -> Self {
        Self {
            inner: na::Matrix4::from_euler_angles(0.0, 0.0, angle),
        }
    }

    /// Create from Euler angles (roll, pitch, yaw in radians)
    #[inline]
    pub fn from_euler_angles(roll: f64, pitch: f64, yaw: f64) -> Self {
        Self {
            inner: na::Matrix4::from_euler_angles(roll, pitch, yaw),
        }
    }

    /// Get element at (row, col)
    #[inline]
    pub fn get(&self, row: usize, col: usize) -> f64 {
        self.inner[(row, col)]
    }

    /// Set element at (row, col)
    #[inline]
    pub fn set(&mut self, row: usize, col: usize, value: f64) {
        self.inner[(row, col)] = value;
    }

    /// Get the 3x3 rotation/scale part (upper-left)
    #[inline]
    pub fn rotation_scale(&self) -> Mat3 {
        Mat3 {
            inner: self.inner.fixed_view::<3, 3>(0, 0).into_owned(),
        }
    }

    /// Get the translation part (last column)
    #[inline]
    pub fn translation(&self) -> Vec3 {
        Vec3::from_nalgebra(self.inner.column(3).xyz())
    }

    /// Matrix transpose
    #[inline]
    pub fn transpose(&self) -> Self {
        Self {
            inner: self.inner.transpose(),
        }
    }

    /// Matrix inverse
    #[inline]
    pub fn inverse(&self) -> Option<Self> {
        self.inner.try_inverse().map(|m| Self { inner: m })
    }

    /// Determinant
    #[inline]
    pub fn determinant(&self) -> f64 {
        self.inner.determinant()
    }

    /// Check if matrix is invertible
    #[inline]
    pub fn is_invertible(&self, tol: f64) -> bool {
        self.determinant().abs() > tol
    }

    /// Get rotation as quaternion
    #[inline]
    pub fn rotation(&self) -> na::UnitQuaternion<f64> {
        let rot_mat = self.rotation_scale();
        na::UnitQuaternion::from_matrix(&rot_mat.inner)
    }

    /// Get scale vector
    #[inline]
    pub fn scale(&self) -> Vec3 {
        let m = &self.inner;
        Vec3::new(
            m.column(0).xyz().norm(),
            m.column(1).xyz().norm(),
            m.column(2).xyz().norm(),
        )
    }

    /// Decompose into translation, rotation, scale
    #[inline]
    pub fn decompose(&self) -> (Vec3, na::UnitQuaternion<f64>, Vec3) {
        let translation = self.translation();
        let rotation = self.rotation();
        let scale = self.scale();
        (translation, rotation, scale)
    }

    /// Create look-at view matrix
    #[inline]
    pub fn look_at_rh(eye: &Point3, target: &Point3, up: &Vec3) -> Self {
        Self {
            inner: na::Matrix4::look_at_rh(&eye.to_nalgebra(), &target.to_nalgebra(), &up.to_nalgebra()),
        }
    }

    /// Create perspective projection matrix
    #[inline]
    pub fn perspective_rh(fov_y: f64, aspect: f64, near: f64, far: f64) -> Self {
        Self {
            inner: na::Matrix4::new_perspective(aspect, fov_y, near, far),
        }
    }

    /// Create orthographic projection matrix
    #[inline]
    pub fn orthographic_rh(left: f64, right: f64, bottom: f64, top: f64, near: f64, far: f64) -> Self {
        Self {
            inner: na::Matrix4::new_orthographic(left, right, bottom, top, near, far),
        }
    }

    /// Convert to nalgebra matrix
    #[inline]
    pub fn to_nalgebra(&self) -> na::Matrix4<f64> {
        self.inner
    }

    /// Create from nalgebra matrix
    #[inline]
    pub fn from_nalgebra(m: na::Matrix4<f64>) -> Self {
        Self { inner: m }
    }

    /// Convert to flat array in row-major order
    #[inline]
    pub fn to_row_major_array(&self) -> [f64; 16] {
        [
            self.get(0, 0), self.get(0, 1), self.get(0, 2), self.get(0, 3),
            self.get(1, 0), self.get(1, 1), self.get(1, 2), self.get(1, 3),
            self.get(2, 0), self.get(2, 1), self.get(2, 2), self.get(2, 3),
            self.get(3, 0), self.get(3, 1), self.get(3, 2), self.get(3, 3),
        ]
    }

    /// Convert to flat array in column-major order (OpenGL format)
    #[inline]
    pub fn to_column_major_array(&self) -> [f64; 16] {
        [
            self.get(0, 0), self.get(1, 0), self.get(2, 0), self.get(3, 0),
            self.get(0, 1), self.get(1, 1), self.get(2, 1), self.get(3, 1),
            self.get(0, 2), self.get(1, 2), self.get(2, 2), self.get(3, 2),
            self.get(0, 3), self.get(1, 3), self.get(2, 3), self.get(3, 3),
        ]
    }
}

impl Default for Mat4 {
    fn default() -> Self {
        Self::identity()
    }
}

impl Mul for Mat4 {
    type Output = Mat4;

    #[inline]
    fn mul(self, rhs: Mat4) -> Self::Output {
        Mat4 { inner: self.inner * rhs.inner }
    }
}

impl Mul<Vec3> for Mat4 {
    type Output = Vec3;

    #[inline]
    fn mul(self, rhs: Vec3) -> Self::Output {
        let v4 = self.inner * na::Vector4::new(rhs.x(), rhs.y(), rhs.z(), 0.0);
        Vec3::new(v4.x, v4.y, v4.z)
    }
}

impl Mul<Point3> for Mat4 {
    type Output = Point3;

    #[inline]
    fn mul(self, rhs: Point3) -> Self::Output {
        Point3::from_nalgebra(self.inner.transform_point(&rhs.to_nalgebra()))
    }
}

impl Index<(usize, usize)> for Mat4 {
    type Output = f64;

    #[inline]
    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        &self.inner[(row, col)]
    }
}

impl IndexMut<(usize, usize)> for Mat4 {
    #[inline]
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
        &mut self.inner[(row, col)]
    }
}

/// 3x3 matrix
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Mat3 {
    pub(crate) inner: na::Matrix3<f64>,
}

impl Mat3 {
    /// Identity matrix
    pub fn identity() -> Self {
        Self {
            inner: na::Matrix3::identity(),
        }
    }

    /// Create a new matrix
    #[inline]
    pub fn new(m00: f64, m01: f64, m02: f64,
               m10: f64, m11: f64, m12: f64,
               m20: f64, m21: f64, m22: f64) -> Self {
        Self {
            inner: na::Matrix3::new(
                m00, m01, m02,
                m10, m11, m12,
                m20, m21, m22,
            ),
        }
    }

    /// Matrix inverse
    #[inline]
    pub fn inverse(&self) -> Option<Self> {
        self.inner.try_inverse().map(|m| Self { inner: m })
    }

    /// Determinant
    #[inline]
    pub fn determinant(&self) -> f64 {
        self.inner.determinant()
    }

    /// Convert to nalgebra
    #[inline]
    pub fn to_nalgebra(&self) -> na::Matrix3<f64> {
        self.inner
    }

    /// Create from nalgebra
    #[inline]
    pub fn from_nalgebra(m: na::Matrix3<f64>) -> Self {
        Self { inner: m }
    }
}

impl Mul for Mat3 {
    type Output = Mat3;

    #[inline]
    fn mul(self, rhs: Mat3) -> Self::Output {
        Mat3 { inner: self.inner * rhs.inner }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let m = Mat4::IDENTITY;
        let v = Vec3::new(1.0, 2.0, 3.0);
        let result = m * v;
        assert!((result.x() - 1.0).abs() < 1e-10);
        assert!((result.y() - 2.0).abs() < 1e-10);
        assert!((result.z() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_translation() {
        let m = Mat4::from_translation(10.0, 20.0, 30.0);
        let p = Point3::new(1.0, 2.0, 3.0);
        let result = m * p;
        assert!((result.x() - 11.0).abs() < 1e-10);
        assert!((result.y() - 22.0).abs() < 1e-10);
        assert!((result.z() - 33.0).abs() < 1e-10);
    }

    #[test]
    fn test_scale() {
        let m = Mat4::from_scale(2.0, 3.0, 4.0);
        let v = Vec3::new(1.0, 1.0, 1.0);
        let result = m * v;
        assert!((result.x() - 2.0).abs() < 1e-10);
        assert!((result.y() - 3.0).abs() < 1e-10);
        assert!((result.z() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotation_x() {
        let m = Mat4::from_rotation_x(std::f64::consts::FRAC_PI_2);
        let v = Vec3::Y;
        let result = m * v;
        assert!((result.x() - 0.0).abs() < 1e-10);
        assert!((result.y() - 0.0).abs() < 1e-10);
        assert!((result.z() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_matrix_multiply() {
        let t = Mat4::from_translation(1.0, 2.0, 3.0);
        let s = Mat4::from_scale(2.0, 2.0, 2.0);
        let combined = t * s;
        let p = Point3::new(1.0, 1.0, 1.0);
        let result = combined * p;
        // Scale first: (1,1,1) -> (2,2,2), then translate: (3,4,5)
        assert!((result.x() - 3.0).abs() < 1e-10);
        assert!((result.y() - 4.0).abs() < 1e-10);
        assert!((result.z() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_inverse() {
        let m = Mat4::from_translation(10.0, 20.0, 30.0);
        let inv = m.inverse().unwrap();
        let p = Point3::new(11.0, 22.0, 33.0);
        let result = inv * p;
        assert!((result.x() - 1.0).abs() < 1e-10);
        assert!((result.y() - 2.0).abs() < 1e-10);
        assert!((result.z() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_transpose() {
        let m = Mat4::new(
            1.0, 2.0, 3.0, 4.0,
            5.0, 6.0, 7.0, 8.0,
            9.0, 10.0, 11.0, 12.0,
            13.0, 14.0, 15.0, 16.0,
        );
        let t = m.transpose();
        assert_eq!(t.get(0, 1), m.get(1, 0));
        assert_eq!(t.get(1, 2), m.get(2, 1));
        assert_eq!(t.get(2, 3), m.get(3, 2));
    }
}
