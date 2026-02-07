//! 3D plane representation

use crate::{Point3, Vec3, Transform3, BoundingBox3, Bounded};
use nalgebra as na;
use serde::{Deserialize, Serialize};

/// 3D plane defined by a point and unit normal
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Plane {
    /// A point on the plane
    origin: Point3,
    /// Unit normal vector (points outward for half-space)
    normal: Vec3,
}

impl Plane {
    /// XY plane at origin (Z = 0, normal = +Z)
    pub const XY: Self = Self {
        origin: Point3::ORIGIN,
        normal: Vec3::Z,
    };

    /// XZ plane at origin (Y = 0, normal = +Y)
    pub const XZ: Self = Self {
        origin: Point3::ORIGIN,
        normal: Vec3::Y,
    };

    /// YZ plane at origin (X = 0, normal = +X)
    pub const YZ: Self = Self {
        origin: Point3::ORIGIN,
        normal: Vec3::X,
    };

    /// Create a new plane from origin and normal
    /// 
    /// The normal will be normalized automatically.
    #[inline]
    pub fn new(origin: Point3, normal: Vec3) -> Self {
        Self {
            origin,
            normal: normal.normalized(),
        }
    }

    /// Create a plane from origin and normal (assumes normal is already unit length)
    #[inline]
    pub fn from_normalized(origin: Point3, normal: Vec3) -> Self {
        debug_assert!(normal.is_normalized(1e-10), "Normal must be unit length");
        Self { origin, normal }
    }

    /// Create a plane from three points
    /// 
    /// Returns None if the points are collinear.
    #[inline]
    pub fn from_points(p1: &Point3, p2: &Point3, p3: &Point3) -> Option<Self> {
        let v1 = *p2 - *p1;
        let v2 = *p3 - *p1;
        let normal = v1.cross(&v2);
        
        if normal.is_zero(1e-10) {
            return None; // Points are collinear
        }
        
        Some(Self::new(*p1, normal))
    }

    /// Create a plane from a point and two direction vectors in the plane
    /// 
    /// Returns None if the direction vectors are parallel.
    #[inline]
    pub fn from_point_and_directions(origin: Point3, dir1: &Vec3, dir2: &Vec3) -> Option<Self> {
        let normal = dir1.cross(dir2);
        
        if normal.is_zero(1e-10) {
            return None; // Directions are parallel
        }
        
        Some(Self::new(origin, normal))
    }

    /// Get the origin point
    #[inline]
    pub fn origin(&self) -> Point3 {
        self.origin
    }

    /// Get the unit normal
    #[inline]
    pub fn normal(&self) -> Vec3 {
        self.normal
    }

    /// Get the D coefficient of the plane equation (ax + by + cz + d = 0)
    /// 
    /// Note: This returns -n·origin, so the plane equation is n·p + d = 0
    #[inline]
    pub fn d(&self) -> f64 {
        -self.normal.dot(&self.origin.to_vector())
    }

    /// Get plane coefficients (a, b, c, d) for equation ax + by + cz + d = 0
    #[inline]
    pub fn coefficients(&self) -> (f64, f64, f64, f64) {
        (self.normal.x(), self.normal.y(), self.normal.z(), self.d())
    }

    /// Signed distance from a point to the plane
    /// 
    /// Positive if the point is on the side of the normal,
    /// negative if on the opposite side.
    #[inline]
    pub fn signed_distance_to_point(&self, p: &Point3) -> f64 {
        self.normal.dot(&(*p - self.origin).to_vector())
    }

    /// Absolute distance from a point to the plane
    #[inline]
    pub fn distance_to_point(&self, p: &Point3) -> f64 {
        self.signed_distance_to_point(p).abs()
    }

    /// Project a point onto the plane
    #[inline]
    pub fn project_point(&self, p: &Point3) -> Point3 {
        let dist = self.signed_distance_to_point(p);
        *p - self.normal * dist
    }

    /// Project a vector onto the plane (removes normal component)
    #[inline]
    pub fn project_vector(&self, v: &Vec3) -> Vec3 {
        *v - self.normal * v.dot(&self.normal)
    }

    /// Reflect a point across the plane
    #[inline]
    pub fn reflect_point(&self, p: &Point3) -> Point3 {
        let dist = self.signed_distance_to_point(p);
        *p - self.normal * (2.0 * dist)
    }

    /// Reflect a vector across the plane
    #[inline]
    pub fn reflect_vector(&self, v: &Vec3) -> Vec3 {
        v.reflect(&self.normal)
    }

    /// Check which side of the plane a point is on
    /// 
    /// Returns:
    /// -  1 if on the normal side
    /// - -1 if on the opposite side
    /// -  0 if on the plane (within tolerance)
    #[inline]
    pub fn point_side(&self, p: &Point3, tol: f64) -> i32 {
        let dist = self.signed_distance_to_point(p);
        if dist > tol {
            1
        } else if dist < -tol {
            -1
        } else {
            0
        }
    }

    /// Check if a point is on the plane (within tolerance)
    #[inline]
    pub fn contains_point(&self, p: &Point3, tol: f64) -> bool {
        self.distance_to_point(p) <= tol
    }

    /// Get two orthonormal basis vectors in the plane
    /// 
    /// Returns (u_axis, v_axis) such that:
    /// - u_axis × v_axis = normal
    /// - Both are unit length and perpendicular to each other
    #[inline]
    pub fn basis_vectors(&self) -> (Vec3, Vec3) {
        // Find a vector not parallel to normal
        let arbitrary = if self.normal.x().abs() < 0.9 {
            Vec3::X
        } else {
            Vec3::Y
        };
        
        // u_axis is perpendicular to normal
        let u_axis = arbitrary.reject_from(&self.normal).normalized();
        
        // v_axis completes the right-handed system
        let v_axis = self.normal.cross(&u_axis);
        
        (u_axis, v_axis)
    }

    /// Convert a point in plane coordinates (u, v) to 3D point
    #[inline]
    pub fn point_at(&self, u: f64, v: f64) -> Point3 {
        let (u_axis, v_axis) = self.basis_vectors();
        self.origin + u_axis * u + v_axis * v
    }

    /// Convert a 3D point to plane coordinates (u, v, signed_dist)
    #[inline]
    pub fn to_plane_coords(&self, p: &Point3) -> (f64, f64, f64) {
        let (u_axis, v_axis) = self.basis_vectors();
        let local = *p - self.origin;
        let u = local.dot(&u_axis);
        let v = local.dot(&v_axis);
        let d = self.signed_distance_to_point(p);
        (u, v, d)
    }

    /// Get the line of intersection with another plane
    /// 
    /// Returns None if the planes are parallel.
    #[inline]
    pub fn intersect_plane(&self, other: &Plane) -> Option<(Point3, Vec3)> {
        let dir = self.normal.cross(&other.normal);
        
        if dir.is_zero(1e-10) {
            return None; // Planes are parallel
        }
        
        // Find a point on the intersection line
        // Solve: n1·p = n1·origin1 and n2·p = n2·origin2
        let n1 = self.normal.to_nalgebra();
        let n2 = other.normal.to_nalgebra();
        let d1 = n1.dot(&self.origin.to_nalgebra().coords);
        let d2 = n2.dot(&other.origin.to_nalgebra().coords);
        
        // Use the formula for line of intersection
        let n1_cross_n2 = n1.cross(&n2);
        let n1_cross_n2_sq = n1_cross_n2.norm_squared();
        
        let point = na::Point3::from(
            (n1_cross_n2.cross(&n2) * d1 + n1.cross(&n1_cross_n2) * d2) / n1_cross_n2_sq
        );
        
        Some((Point3::from_nalgebra(point), Vec3::from_nalgebra(dir)))
    }

    /// Get the intersection point of three planes
    /// 
    /// Returns None if the planes don't intersect at a single point.
    #[inline]
    pub fn intersect_three_planes(p1: &Plane, p2: &Plane, p3: &Plane) -> Option<Point3> {
        let n1 = p1.normal.to_nalgebra();
        let n2 = p2.normal.to_nalgebra();
        let n3 = p3.normal.to_nalgebra();
        
        let d1 = -p1.d();
        let d2 = -p2.d();
        let d3 = -p3.d();
        
        // Build matrix of normals
        let mat = na::Matrix3::from_columns(&[n1, n2, n3]);
        
        // Check if planes intersect at a point
        let det = mat.determinant();
        if det.abs() < 1e-10 {
            return None;
        }
        
        // Solve for intersection point
        let rhs = na::Vector3::new(d1, d2, d3);
        let solution = mat.try_inverse()? * rhs;
        
        Some(Point3::new(solution.x, solution.y, solution.z))
    }

    /// Transform the plane
    #[inline]
    pub fn transform(&self, transform: &Transform3) -> Self {
        let new_origin = transform.apply_to_point(&self.origin);
        let new_normal = transform.apply_to_vector(&self.normal).normalized();
        Self::from_normalized(new_origin, new_normal)
    }

    /// Flip the plane normal
    #[inline]
    pub fn flip(&self) -> Self {
        Self::from_normalized(self.origin, -self.normal)
    }

    /// Check if approximately equal to another plane
    #[inline]
    pub fn approx_eq(&self, other: &Plane, pos_tol: f64, normal_tol: f64) -> bool {
        self.contains_point(&other.origin, pos_tol) &&
        self.normal.dot(&other.normal).abs() >= 1.0 - normal_tol
    }

    /// Check if parallel to another plane
    #[inline]
    pub fn is_parallel_to(&self, other: &Plane, tol: f64) -> bool {
        self.normal.cross(&other.normal).is_zero(tol)
    }

    /// Check if perpendicular to another plane
    #[inline]
    pub fn is_perpendicular_to(&self, other: &Plane, tol: f64) -> bool {
        self.normal.dot(&other.normal).abs() <= tol
    }

    /// Get the plane's bounding box (unbounded, returns infinite)
    #[inline]
    pub fn bounding_box(&self) -> BoundingBox3 {
        BoundingBox3::EMPTY
    }
}

impl Bounded for Plane {
    fn bounding_box(&self) -> BoundingBox3 {
        BoundingBox3::EMPTY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let plane = Plane::new(Point3::new(0.0, 0.0, 0.0), Vec3::Z);
        assert_eq!(plane.origin().x(), 0.0);
        assert!(plane.normal().is_normalized(1e-10));
    }

    #[test]
    fn test_from_points() {
        let p1 = Point3::new(0.0, 0.0, 0.0);
        let p2 = Point3::new(1.0, 0.0, 0.0);
        let p3 = Point3::new(0.0, 1.0, 0.0);
        let plane = Plane::from_points(&p1, &p2, &p3).unwrap();
        assert!(plane.normal().dot(&Vec3::Z).abs() > 0.99);
    }

    #[test]
    fn test_from_points_collinear() {
        let p1 = Point3::new(0.0, 0.0, 0.0);
        let p2 = Point3::new(1.0, 0.0, 0.0);
        let p3 = Point3::new(2.0, 0.0, 0.0);
        assert!(Plane::from_points(&p1, &p2, &p3).is_none());
    }

    #[test]
    fn test_distance_to_point() {
        let plane = Plane::XY;
        let p = Point3::new(1.0, 2.0, 3.0);
        assert_eq!(plane.signed_distance_to_point(&p), 3.0);
        assert_eq!(plane.distance_to_point(&p), 3.0);
    }

    #[test]
    fn test_project_point() {
        let plane = Plane::XY;
        let p = Point3::new(1.0, 2.0, 3.0);
        let projected = plane.project_point(&p);
        assert_eq!(projected.x(), 1.0);
        assert_eq!(projected.y(), 2.0);
        assert_eq!(projected.z(), 0.0);
    }

    #[test]
    fn test_reflect_point() {
        let plane = Plane::XY;
        let p = Point3::new(1.0, 2.0, 3.0);
        let reflected = plane.reflect_point(&p);
        assert_eq!(reflected.x(), 1.0);
        assert_eq!(reflected.y(), 2.0);
        assert_eq!(reflected.z(), -3.0);
    }

    #[test]
    fn test_basis_vectors() {
        let plane = Plane::XY;
        let (u, v) = plane.basis_vectors();
        assert!(u.is_normalized(1e-10));
        assert!(v.is_normalized(1e-10));
        assert!(u.dot(&v).abs() < 1e-10);
        assert!(u.cross(&v).dot(&Vec3::Z).abs() > 0.99);
    }

    #[test]
    fn test_point_at() {
        let plane = Plane::XY;
        let p = plane.point_at(2.0, 3.0);
        assert_eq!(p.x(), 2.0);
        assert_eq!(p.y(), 3.0);
        assert_eq!(p.z(), 0.0);
    }

    #[test]
    fn test_to_plane_coords() {
        let plane = Plane::XY;
        let p = Point3::new(2.0, 3.0, 4.0);
        let (u, v, d) = plane.to_plane_coords(&p);
        assert_eq!(u, 2.0);
        assert_eq!(v, 3.0);
        assert_eq!(d, 4.0);
    }

    #[test]
    fn test_intersect_plane() {
        let plane1 = Plane::XY;
        let plane2 = Plane::YZ;
        let (origin, dir) = plane1.intersect_plane(&plane2).unwrap();
        // Intersection should be the X-axis
        assert!(origin.distance_to(&Point3::ORIGIN) < 1e-10);
        assert!(dir.dot(&Vec3::X).abs() > 0.99);
    }

    #[test]
    fn test_intersect_parallel_planes() {
        let plane1 = Plane::XY;
        let plane2 = Plane::new(Point3::new(0.0, 0.0, 1.0), Vec3::Z);
        assert!(plane1.intersect_plane(&plane2).is_none());
    }

    #[test]
    fn test_is_parallel_to() {
        let plane1 = Plane::XY;
        let plane2 = Plane::new(Point3::new(0.0, 0.0, 1.0), Vec3::Z);
        assert!(plane1.is_parallel_to(&plane2, 1e-10));
    }

    #[test]
    fn test_is_perpendicular_to() {
        let plane1 = Plane::XY;
        let plane2 = Plane::YZ;
        assert!(plane1.is_perpendicular_to(&plane2, 1e-10));
    }

    #[test]
    fn test_flip() {
        let plane = Plane::XY;
        let flipped = plane.flip();
        assert_eq!(flipped.normal().z(), -1.0);
    }

    #[test]
    fn test_transform() {
        let plane = Plane::XY;
        let transform = Transform3::from_translation(1.0, 2.0, 3.0);
        let transformed = plane.transform(&transform);
        assert_eq!(transformed.origin().z(), 3.0);
        assert_eq!(transformed.normal().z(), 1.0);
    }
}
