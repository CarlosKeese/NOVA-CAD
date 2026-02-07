//! Axis-aligned bounding boxes for 2D and 3D

use crate::{Point2, Point3, Vec3};
use nalgebra as na;
use serde::{Deserialize, Serialize};

/// 3D axis-aligned bounding box
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox3 {
    /// Minimum corner (lower bounds)
    pub min: Point3,
    /// Maximum corner (upper bounds)
    pub max: Point3,
}

impl BoundingBox3 {
    /// Empty bounding box (invalid, min > max)
    pub const EMPTY: Self = Self {
        min: Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
        max: Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
    };

    /// Create a new bounding box from min and max corners
    #[inline]
    pub fn new(min: Point3, max: Point3) -> Self {
        Self { min, max }
    }

    /// Create a bounding box from a single point
    #[inline]
    pub fn from_point(p: Point3) -> Self {
        Self { min: p, max: p }
    }

    /// Create a bounding box from center and half-extents
    #[inline]
    pub fn from_center_half_extents(center: &Point3, half_extents: &Vec3) -> Self {
        Self {
            min: Point3::new(
                center.x() - half_extents.x(),
                center.y() - half_extents.y(),
                center.z() - half_extents.z(),
            ),
            max: Point3::new(
                center.x() + half_extents.x(),
                center.y() + half_extents.y(),
                center.z() + half_extents.z(),
            ),
        }
    }

    /// Create a bounding box containing two points
    #[inline]
    pub fn from_points(a: &Point3, b: &Point3) -> Self {
        Self {
            min: Point3::new(
                a.x().min(b.x()),
                a.y().min(b.y()),
                a.z().min(b.z()),
            ),
            max: Point3::new(
                a.x().max(b.x()),
                a.y().max(b.y()),
                a.z().max(b.z()),
            ),
        }
    }

    /// Check if the bounding box is empty/invalid
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.min.x() > self.max.x() || 
        self.min.y() > self.max.y() || 
        self.min.z() > self.max.z()
    }

    /// Check if the bounding box is valid (min <= max in all dimensions)
    #[inline]
    pub fn is_valid(&self) -> bool {
        !self.is_empty()
    }

    /// Get the center point
    #[inline]
    pub fn center(&self) -> Point3 {
        self.min.midpoint(&self.max)
    }

    /// Get the extents (size in each dimension)
    #[inline]
    pub fn extents(&self) -> Vec3 {
        Vec3::new(
            self.max.x() - self.min.x(),
            self.max.y() - self.min.y(),
            self.max.z() - self.min.z(),
        )
    }

    /// Get half-extents
    #[inline]
    pub fn half_extents(&self) -> Vec3 {
        self.extents() * 0.5
    }

    /// Get the diagonal length
    #[inline]
    pub fn diagonal(&self) -> f64 {
        self.extents().length()
    }

    /// Get the squared diagonal length
    #[inline]
    pub fn diagonal_squared(&self) -> f64 {
        self.extents().length_squared()
    }

    /// Get the volume
    #[inline]
    pub fn volume(&self) -> f64 {
        let e = self.extents();
        e.x() * e.y() * e.z()
    }

    /// Get the surface area
    #[inline]
    pub fn surface_area(&self) -> f64 {
        let e = self.extents();
        2.0 * (e.x() * e.y() + e.y() * e.z() + e.z() * e.x())
    }

    /// Check if a point is inside the bounding box
    #[inline]
    pub fn contains(&self, p: &Point3) -> bool {
        p.x() >= self.min.x() && p.x() <= self.max.x() &&
        p.y() >= self.min.y() && p.y() <= self.max.y() &&
        p.z() >= self.min.z() && p.z() <= self.max.z()
    }

    /// Check if a point is strictly inside (not on boundary)
    #[inline]
    pub fn contains_strict(&self, p: &Point3) -> bool {
        p.x() > self.min.x() && p.x() < self.max.x() &&
        p.y() > self.min.y() && p.y() < self.max.y() &&
        p.z() > self.min.z() && p.z() < self.max.z()
    }

    /// Check if this bounding box contains another
    #[inline]
    pub fn contains_bbox(&self, other: &BoundingBox3) -> bool {
        self.contains(&other.min) && self.contains(&other.max)
    }

    /// Check if this bounding box intersects another
    #[inline]
    pub fn intersects(&self, other: &BoundingBox3) -> bool {
        self.min.x() <= other.max.x() && self.max.x() >= other.min.x() &&
        self.min.y() <= other.max.y() && self.max.y() >= other.min.y() &&
        self.min.z() <= other.max.z() && self.max.z() >= other.min.z()
    }

    /// Compute the intersection of two bounding boxes
    #[inline]
    pub fn intersection(&self, other: &BoundingBox3) -> Self {
        Self {
            min: Point3::new(
                self.min.x().max(other.min.x()),
                self.min.y().max(other.min.y()),
                self.min.z().max(other.min.z()),
            ),
            max: Point3::new(
                self.max.x().min(other.max.x()),
                self.max.y().min(other.max.y()),
                self.max.z().min(other.max.z()),
            ),
        }
    }

    /// Compute the union of two bounding boxes
    #[inline]
    pub fn union(&self, other: &BoundingBox3) -> Self {
        Self {
            min: Point3::new(
                self.min.x().min(other.min.x()),
                self.min.y().min(other.min.y()),
                self.min.z().min(other.min.z()),
            ),
            max: Point3::new(
                self.max.x().max(other.max.x()),
                self.max.y().max(other.max.y()),
                self.max.z().max(other.max.z()),
            ),
        }
    }

    /// Expand to include a point
    #[inline]
    pub fn expand(&mut self, p: &Point3) {
        self.min = Point3::new(
            self.min.x().min(p.x()),
            self.min.y().min(p.y()),
            self.min.z().min(p.z()),
        );
        self.max = Point3::new(
            self.max.x().max(p.x()),
            self.max.y().max(p.y()),
            self.max.z().max(p.z()),
        );
    }

    /// Expand by a margin in all directions
    #[inline]
    pub fn expand_by_margin(&mut self, margin: f64) {
        self.min = Point3::new(
            self.min.x() - margin,
            self.min.y() - margin,
            self.min.z() - margin,
        );
        self.max = Point3::new(
            self.max.x() + margin,
            self.max.y() + margin,
            self.max.z() + margin,
        );
    }

    /// Expand by a vector (different margins per axis)
    #[inline]
    pub fn expand_by_vector(&mut self, v: &Vec3) {
        self.min = Point3::new(
            self.min.x() - v.x(),
            self.min.y() - v.y(),
            self.min.z() - v.z(),
        );
        self.max = Point3::new(
            self.max.x() + v.x(),
            self.max.y() + v.y(),
            self.max.z() + v.z(),
        );
    }

    /// Get the 8 corners of the bounding box
    #[inline]
    pub fn corners(&self) -> [Point3; 8] {
        [
            Point3::new(self.min.x(), self.min.y(), self.min.z()),
            Point3::new(self.max.x(), self.min.y(), self.min.z()),
            Point3::new(self.max.x(), self.max.y(), self.min.z()),
            Point3::new(self.min.x(), self.max.y(), self.min.z()),
            Point3::new(self.min.x(), self.min.y(), self.max.z()),
            Point3::new(self.max.x(), self.min.y(), self.max.z()),
            Point3::new(self.max.x(), self.max.y(), self.max.z()),
            Point3::new(self.min.x(), self.max.y(), self.max.z()),
        ]
    }

    /// Get the closest point on the bounding box to a given point
    #[inline]
    pub fn closest_point(&self, p: &Point3) -> Point3 {
        Point3::new(
            p.x().clamp(self.min.x(), self.max.x()),
            p.y().clamp(self.min.y(), self.max.y()),
            p.z().clamp(self.min.z(), self.max.z()),
        )
    }

    /// Get the squared distance from a point to the bounding box
    #[inline]
    pub fn distance_squared_to_point(&self, p: &Point3) -> f64 {
        let closest = self.closest_point(p);
        p.distance_squared_to(&closest)
    }

    /// Get the distance from a point to the bounding box
    #[inline]
    pub fn distance_to_point(&self, p: &Point3) -> f64 {
        self.distance_squared_to_point(p).sqrt()
    }

    /// Scale the bounding box about its center
    #[inline]
    pub fn scale(&mut self, factor: f64) {
        let center = self.center();
        let half_extents = self.half_extents() * factor;
        *self = Self::from_center_half_extents(&center, &half_extents);
    }

    /// Translate the bounding box
    #[inline]
    pub fn translate(&mut self, offset: &Vec3) {
        self.min = self.min + *offset;
        self.max = self.max + *offset;
    }

    /// Check if approximately equal to another bounding box
    #[inline]
    pub fn approx_eq(&self, other: &BoundingBox3, tol: f64) -> bool {
        self.min.distance_to(&other.min) <= tol &&
        self.max.distance_to(&other.max) <= tol
    }

    /// Get the longest axis index (0=X, 1=Y, 2=Z)
    #[inline]
    pub fn longest_axis(&self) -> usize {
        let e = self.extents();
        if e.x() >= e.y() && e.x() >= e.z() {
            0
        } else if e.y() >= e.z() {
            1
        } else {
            2
        }
    }

    /// Get the longest axis length
    #[inline]
    pub fn longest_axis_length(&self) -> f64 {
        let e = self.extents();
        e.x().max(e.y()).max(e.z())
    }

    /// Get the shortest axis index (0=X, 1=Y, 2=Z)
    #[inline]
    pub fn shortest_axis(&self) -> usize {
        let e = self.extents();
        if e.x() <= e.y() && e.x() <= e.z() {
            0
        } else if e.y() <= e.z() {
            1
        } else {
            2
        }
    }

    /// Get the shortest axis length
    #[inline]
    pub fn shortest_axis_length(&self) -> f64 {
        let e = self.extents();
        e.x().min(e.y()).min(e.z())
    }
}

impl Default for BoundingBox3 {
    fn default() -> Self {
        Self::EMPTY
    }
}

/// 2D axis-aligned bounding box
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox2 {
    pub min: Point2,
    pub max: Point2,
}

impl BoundingBox2 {
    /// Empty bounding box
    pub const EMPTY: Self = Self {
        min: Point2::new(f64::INFINITY, f64::INFINITY),
        max: Point2::new(f64::NEG_INFINITY, f64::NEG_INFINITY),
    };

    /// Create a new bounding box
    #[inline]
    pub fn new(min: Point2, max: Point2) -> Self {
        Self { min, max }
    }

    /// Create from a single point
    #[inline]
    pub fn from_point(p: Point2) -> Self {
        Self { min: p, max: p }
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.min.x() > self.max.x() || self.min.y() > self.max.y()
    }

    /// Get center
    #[inline]
    pub fn center(&self) -> Point2 {
        Point2::new(
            (self.min.x() + self.max.x()) * 0.5,
            (self.min.y() + self.max.y()) * 0.5,
        )
    }

    /// Get extents
    #[inline]
    pub fn extents(&self) -> (f64, f64) {
        (self.max.x() - self.min.x(), self.max.y() - self.min.y())
    }

    /// Get area
    #[inline]
    pub fn area(&self) -> f64 {
        let (dx, dy) = self.extents();
        dx * dy
    }

    /// Check if contains point
    #[inline]
    pub fn contains(&self, p: &Point2) -> bool {
        p.x() >= self.min.x() && p.x() <= self.max.x() &&
        p.y() >= self.min.y() && p.y() <= self.max.y()
    }

    /// Check if intersects another bbox
    #[inline]
    pub fn intersects(&self, other: &BoundingBox2) -> bool {
        self.min.x() <= other.max.x() && self.max.x() >= other.min.x() &&
        self.min.y() <= other.max.y() && self.max.y() >= other.min.y()
    }

    /// Compute union
    #[inline]
    pub fn union(&self, other: &BoundingBox2) -> Self {
        Self {
            min: Point2::new(
                self.min.x().min(other.min.x()),
                self.min.y().min(other.min.y()),
            ),
            max: Point2::new(
                self.max.x().max(other.max.x()),
                self.max.y().max(other.max.y()),
            ),
        }
    }

    /// Expand to include point
    #[inline]
    pub fn expand(&mut self, p: &Point2) {
        self.min = Point2::new(
            self.min.x().min(p.x()),
            self.min.y().min(p.y()),
        );
        self.max = Point2::new(
            self.max.x().max(p.x()),
            self.max.y().max(p.y()),
        );
    }
}

impl Default for BoundingBox2 {
    fn default() -> Self {
        Self::EMPTY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let bbox = BoundingBox3::EMPTY;
        assert!(bbox.is_empty());
    }

    #[test]
    fn test_from_points() {
        let p1 = Point3::new(0.0, 0.0, 0.0);
        let p2 = Point3::new(1.0, 2.0, 3.0);
        let bbox = BoundingBox3::from_points(&p1, &p2);
        assert_eq!(bbox.min.x(), 0.0);
        assert_eq!(bbox.max.x(), 1.0);
        assert_eq!(bbox.max.y(), 2.0);
        assert_eq!(bbox.max.z(), 3.0);
    }

    #[test]
    fn test_center() {
        let bbox = BoundingBox3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(2.0, 4.0, 6.0),
        );
        let center = bbox.center();
        assert_eq!(center.x(), 1.0);
        assert_eq!(center.y(), 2.0);
        assert_eq!(center.z(), 3.0);
    }

    #[test]
    fn test_extents() {
        let bbox = BoundingBox3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(2.0, 4.0, 6.0),
        );
        let e = bbox.extents();
        assert_eq!(e.x(), 2.0);
        assert_eq!(e.y(), 4.0);
        assert_eq!(e.z(), 6.0);
    }

    #[test]
    fn test_volume() {
        let bbox = BoundingBox3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(2.0, 3.0, 4.0),
        );
        assert_eq!(bbox.volume(), 24.0);
    }

    #[test]
    fn test_contains() {
        let bbox = BoundingBox3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 1.0),
        );
        assert!(bbox.contains(&Point3::new(0.5, 0.5, 0.5)));
        assert!(bbox.contains(&Point3::new(0.0, 0.0, 0.0)));
        assert!(bbox.contains(&Point3::new(1.0, 1.0, 1.0)));
        assert!(!bbox.contains(&Point3::new(2.0, 0.5, 0.5)));
    }

    #[test]
    fn test_intersects() {
        let bbox1 = BoundingBox3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(2.0, 2.0, 2.0),
        );
        let bbox2 = BoundingBox3::new(
            Point3::new(1.0, 1.0, 1.0),
            Point3::new(3.0, 3.0, 3.0),
        );
        let bbox3 = BoundingBox3::new(
            Point3::new(3.0, 3.0, 3.0),
            Point3::new(4.0, 4.0, 4.0),
        );
        assert!(bbox1.intersects(&bbox2));
        assert!(!bbox1.intersects(&bbox3));
    }

    #[test]
    fn test_union() {
        let bbox1 = BoundingBox3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 1.0),
        );
        let bbox2 = BoundingBox3::new(
            Point3::new(2.0, 2.0, 2.0),
            Point3::new(3.0, 3.0, 3.0),
        );
        let union = bbox1.union(&bbox2);
        assert_eq!(union.min.x(), 0.0);
        assert_eq!(union.max.x(), 3.0);
    }

    #[test]
    fn test_expand() {
        let mut bbox = BoundingBox3::from_point(Point3::new(0.0, 0.0, 0.0));
        bbox.expand(&Point3::new(2.0, 3.0, 4.0));
        assert_eq!(bbox.max.x(), 2.0);
        assert_eq!(bbox.max.y(), 3.0);
        assert_eq!(bbox.max.z(), 4.0);
    }

    #[test]
    fn test_corners() {
        let bbox = BoundingBox3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 1.0),
        );
        let corners = bbox.corners();
        assert_eq!(corners.len(), 8);
        assert!(corners.contains(&Point3::new(0.0, 0.0, 0.0)));
        assert!(corners.contains(&Point3::new(1.0, 1.0, 1.0)));
    }

    #[test]
    fn test_longest_axis() {
        let bbox = BoundingBox3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 2.0, 3.0),
        );
        assert_eq!(bbox.longest_axis(), 2); // Z axis
    }
}
