//! Curve types and implementations

use crate::{GeomResult, GeometryError, ParamRange, CurveEvaluation, Tessellation, Tessellatable};
use nova_math::{Point3, Vec3, Transform3};

/// Trait for all curve types
pub trait Curve: Send + Sync {
    /// Evaluate the curve at parameter t
    fn evaluate(&self, t: f64) -> Point3;

    /// Evaluate the derivative at parameter t
    fn derivative(&self, t: f64, order: u32) -> Vec3;

    /// Get the tangent vector at parameter t
    fn tangent(&self, t: f64) -> Vec3 {
        self.derivative(t, 1)
    }

    /// Get the unit tangent at parameter t
    fn unit_tangent(&self, t: f64) -> Vec3 {
        self.tangent(t).normalized()
    }

    /// Get the curvature at parameter t
    fn curvature(&self, t: f64) -> f64;

    /// Get the parameter range
    fn param_range(&self) -> ParamRange;

    /// Get the start point
    fn start_point(&self) -> Point3 {
        self.evaluate(self.param_range().start)
    }

    /// Get the end point
    fn end_point(&self) -> Point3 {
        self.evaluate(self.param_range().end)
    }

    /// Compute arc length from start to parameter t
    fn arc_length(&self, t: f64) -> f64;

    /// Find parameter for a given arc length
    fn parameter_at_length(&self, length: f64) -> Option<f64>;

    /// Project a point to the curve (find closest point)
    fn closest_point(&self, point: &Point3) -> GeomResult<(f64, Point3, f64)>;

    /// Reverse the curve direction
    fn reverse(&mut self);

    /// Transform the curve
    fn transform(&mut self, transform: &Transform3);

    /// Get curve type
    fn curve_type(&self) -> CurveType;

    /// Clone into a boxed curve
    fn clone_box(&self) -> Box<dyn Curve>;
}

/// Curve type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveType {
    /// Infinite or bounded line
    Line,
    /// Circular arc
    CircularArc,
    /// Elliptical arc
    EllipseArc,
    /// NURBS curve
    NurbsCurve,
    /// Intersection curve
    IntersectionCurve,
    /// Offset curve
    OffsetCurve,
    /// Composite curve
    CompositeCurve,
}

/// 3D line (infinite or bounded)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    /// Origin point
    origin: Point3,
    /// Unit direction vector
    direction: Vec3,
    /// Parameter range (None for infinite line)
    range: Option<ParamRange>,
}

impl Line {
    /// Create an infinite line
    pub fn infinite(origin: Point3, direction: Vec3) -> GeomResult<Self> {
        if direction.is_zero(1e-10) {
            return Err(GeometryError::Degenerate(
                "Line direction cannot be zero".to_string()
            ));
        }
        Ok(Self {
            origin,
            direction: direction.normalized(),
            range: None,
        })
    }

    /// Create a bounded line segment
    pub fn segment(start: Point3, end: Point3) -> GeomResult<Self> {
        let direction = end - start;
        if direction.is_zero(1e-10) {
            return Err(GeometryError::Degenerate(
                "Line segment start and end cannot be the same".to_string()
            ));
        }
        let len = direction.length();
        Ok(Self {
            origin: start,
            direction: direction.normalized(),
            range: Some(ParamRange::new(0.0, len)),
        })
    }

    /// Create from origin and direction with explicit range
    pub fn with_range(origin: Point3, direction: Vec3, range: ParamRange) -> GeomResult<Self> {
        if direction.is_zero(1e-10) {
            return Err(GeometryError::Degenerate(
                "Line direction cannot be zero".to_string()
            ));
        }
        Ok(Self {
            origin,
            direction: direction.normalized(),
            range: Some(range),
        })
    }

    /// Get the origin
    pub fn origin(&self) -> Point3 {
        self.origin
    }

    /// Get the direction
    pub fn direction(&self) -> Vec3 {
        self.direction
    }

    /// Check if the line is infinite
    pub fn is_infinite(&self) -> bool {
        self.range.is_none()
    }

    /// Get the start point (for bounded lines)
    pub fn segment_start(&self) -> Option<Point3> {
        self.range.map(|r| self.evaluate(r.start))
    }

    /// Get the end point (for bounded lines)
    pub fn segment_end(&self) -> Option<Point3> {
        self.range.map(|r| self.evaluate(r.end))
    }

    /// Get the length (for bounded lines)
    pub fn length(&self) -> Option<f64> {
        self.range.map(|r| r.length())
    }

    /// Distance from a point to the infinite line
    pub fn distance_to_point(&self, point: &Point3) -> f64 {
        let to_point = *point - self.origin;
        to_point.reject_from(&self.direction).length()
    }

    /// Project a point onto the infinite line
    pub fn project_point(&self, point: &Point3) -> Point3 {
        let to_point = *point - self.origin;
        let t = to_point.dot(&self.direction);
        self.origin + self.direction * t
    }

    /// Find intersection with another line
    pub fn intersect_line(&self, other: &Line) -> Option<Point3> {
        // For 3D lines, they may be skew (not intersecting and not parallel)
        // We find the closest points and check if they're the same
        let n = self.direction.cross(&other.direction);
        
        if n.is_zero(1e-10) {
            // Lines are parallel
            return None;
        }
        
        let diff = other.origin - self.origin;
        let n_sq = n.length_squared();
        
        // Compute parameters for closest points
        let t = diff.cross(&other.direction).dot(&n) / n_sq;
        let s = diff.cross(&self.direction).dot(&n) / n_sq;
        
        let p1 = self.origin + self.direction * t;
        let p2 = other.origin + other.direction * s;
        
        // Check if closest points are the same (within tolerance)
        if p1.distance_to(&p2) < 1e-6 {
            Some(p1)
        } else {
            None // Lines are skew
        }
    }
}

impl Curve for Line {
    fn evaluate(&self, t: f64) -> Point3 {
        self.origin + self.direction * t
    }

    fn derivative(&self, _t: f64, order: u32) -> Vec3 {
        match order {
            0 => self.origin.to_vector(),
            1 => self.direction,
            _ => Vec3::ZERO,
        }
    }

    fn curvature(&self, _t: f64) -> f64 {
        0.0 // Lines have zero curvature
    }

    fn param_range(&self) -> ParamRange {
        self.range.unwrap_or(ParamRange::new(f64::NEG_INFINITY, f64::INFINITY))
    }

    fn arc_length(&self, t: f64) -> f64 {
        let start = self.param_range().start;
        (t - start).abs()
    }

    fn parameter_at_length(&self, length: f64) -> Option<f64> {
        let range = self.param_range();
        let t = range.start + length;
        if range.contains(t) {
            Some(t)
        } else {
            None
        }
    }

    fn closest_point(&self, point: &Point3) -> GeomResult<(f64, Point3, f64)> {
        let to_point = *point - self.origin;
        let t = to_point.dot(&self.direction);
        
        // Clamp to range if bounded
        let t = if let Some(range) = self.range {
            range.clamp(t)
        } else {
            t
        };
        
        let closest = self.evaluate(t);
        let dist = point.distance_to(&closest);
        Ok((t, closest, dist))
    }

    fn reverse(&mut self) {
        self.direction = -self.direction;
        if let Some(ref mut range) = self.range {
            let start = range.start;
            range.start = -range.end;
            range.end = -start;
        }
    }

    fn transform(&mut self, transform: &Transform3) {
        self.origin = transform.apply_to_point(&self.origin);
        self.direction = transform.apply_to_vector(&self.direction).normalized();
    }

    fn curve_type(&self) -> CurveType {
        CurveType::Line
    }

    fn clone_box(&self) -> Box<dyn Curve> {
        Box::new(*self)
    }
}

impl Tessellatable for Line {
    fn tessellate(&self, _tolerance: f64) -> Tessellation {
        let range = self.param_range();
        Tessellation::Polyline(vec![
            self.evaluate(range.start),
            self.evaluate(range.end),
        ])
    }
}

/// Circular arc in 3D space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CircularArc {
    /// Center of the circle
    center: Point3,
    /// Radius
    radius: f64,
    /// Unit normal to the plane of the circle
    normal: Vec3,
    /// Unit vector from center to start point
    start_vector: Vec3,
    /// Sweep angle in radians (positive = CCW when looking against normal)
    sweep_angle: f64,
}

impl CircularArc {
    /// Create a circular arc
    pub fn new(
        center: Point3,
        radius: f64,
        normal: Vec3,
        start_vector: Vec3,
        sweep_angle: f64,
    ) -> GeomResult<Self> {
        if radius <= 0.0 {
            return Err(GeometryError::InvalidParameter(
                "Radius must be positive".to_string()
            ));
        }
        
        let normal = normal.normalized();
        let mut start_vector = start_vector;
        
        // Project start_vector onto plane perpendicular to normal
        start_vector = start_vector.reject_from(&normal);
        if start_vector.is_zero(1e-10) {
            return Err(GeometryError::InvalidParameter(
                "Start vector cannot be parallel to normal".to_string()
            ));
        }
        start_vector = start_vector.normalized();
        
        Ok(Self {
            center,
            radius,
            normal,
            start_vector,
            sweep_angle,
        })
    }

    /// Create a full circle
    pub fn circle(center: Point3, radius: f64, normal: Vec3) -> GeomResult<Self> {
        // Find a perpendicular vector for start
        let arbitrary = if normal.x().abs() < 0.9 {
            Vec3::X
        } else {
            Vec3::Y
        };
        let start_vector = arbitrary.reject_from(&normal).normalized();
        
        Self::new(center, radius, normal, start_vector, std::f64::consts::TAU)
    }

    /// Create from three points on the arc
    pub fn from_three_points(start: &Point3, mid: &Point3, end: &Point3) -> GeomResult<Self> {
        // Compute center from three points
        let a = *mid - *start;
        let b = *end - *mid;
        
        let normal = a.cross(&b);
        if normal.is_zero(1e-10) {
            return Err(GeometryError::Degenerate(
                "Three points are collinear".to_string()
            ));
        }
        let normal = normal.normalized();
        
        // Find center as intersection of perpendicular bisectors
        let ab_mid = start.midpoint(mid);
        let bc_mid = mid.midpoint(end);
        
        let ab_perp = a.cross(&normal).normalized();
        let bc_perp = b.cross(&normal).normalized();
        
        // Solve for center
        let line1 = Line::infinite(ab_mid, ab_perp)?;
        let line2 = Line::infinite(bc_mid, bc_perp)?;
        
        let center = line1.intersect_line(&line2)
            .ok_or_else(|| GeometryError::Degenerate(
                "Could not compute circle center".to_string()
            ))?;
        
        let radius = center.distance_to(start);
        let start_vector = (*start - center).normalized();
        
        // Compute sweep angle
        let end_vector = (*end - center).normalized();
        let cross = start_vector.cross(&end_vector);
        let dot = start_vector.dot(&end_vector);
        let mut sweep_angle = dot.atan2(cross.dot(&normal).abs());
        
        // Check if mid point is on the arc
        let mid_vector = (*mid - center).normalized();
        let mid_angle = start_vector.dot(&mid_vector).acos();
        if mid_angle > sweep_angle {
            sweep_angle = std::f64::consts::TAU - sweep_angle;
        }
        
        Self::new(center, radius, normal, start_vector, sweep_angle)
    }

    /// Get the center
    pub fn center(&self) -> Point3 {
        self.center
    }

    /// Get the radius
    pub fn radius(&self) -> f64 {
        self.radius
    }

    /// Get the normal
    pub fn normal(&self) -> Vec3 {
        self.normal
    }

    /// Get the sweep angle
    pub fn sweep_angle(&self) -> f64 {
        self.sweep_angle
    }

    /// Check if this is a full circle
    pub fn is_full_circle(&self) -> bool {
        (self.sweep_angle - std::f64::consts::TAU).abs() < 1e-10
    }

    /// Get the arc length
    pub fn arc_length(&self) -> f64 {
        self.radius * self.sweep_angle.abs()
    }

    /// Get a point on the circle at angle (0 = start_vector)
    pub fn point_at_angle(&self, angle: f64) -> Point3 {
        let perp = self.normal.cross(&self.start_vector);
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        self.center + self.start_vector * (self.radius * cos_a) + perp * (self.radius * sin_a)
    }
}

impl Curve for CircularArc {
    fn evaluate(&self, t: f64) -> Point3 {
        let angle = t * self.sweep_angle;
        self.point_at_angle(angle)
    }

    fn derivative(&self, t: f64, order: u32) -> Vec3 {
        let angle = t * self.sweep_angle;
        let perp = self.normal.cross(&self.start_vector);
        
        match order {
            0 => self.point_at_angle(angle).to_vector(),
            1 => {
                // Tangent is perpendicular to radius
                let cos_a = angle.cos();
                let sin_a = angle.sin();
                (perp * cos_a - self.start_vector * sin_a) * self.sweep_angle
            }
            2 => {
                // Second derivative points toward center
                let cos_a = angle.cos();
                let sin_a = angle.sin();
                (-self.start_vector * cos_a - perp * sin_a) * (self.sweep_angle * self.sweep_angle)
            }
            _ => Vec3::ZERO,
        }
    }

    fn curvature(&self, _t: f64) -> f64 {
        1.0 / self.radius // Constant curvature for circles
    }

    fn param_range(&self) -> ParamRange {
        ParamRange::new(0.0, 1.0)
    }

    fn arc_length(&self, t: f64) -> f64 {
        self.radius * self.sweep_angle.abs() * t
    }

    fn parameter_at_length(&self, length: f64) -> Option<f64> {
        let total_length = self.arc_length();
        if length < 0.0 || length > total_length {
            return None;
        }
        Some(length / total_length)
    }

    fn closest_point(&self, point: &Point3) -> GeomResult<(f64, Point3, f64)> {
        // Project point onto plane of circle
        let to_point = *point - self.center;
        let dist_to_plane = to_point.dot(&self.normal);
        let projected = *point - self.normal * dist_to_plane;
        
        // Find closest point on full circle
        let to_proj = projected - self.center;
        let dist_from_center = to_proj.length();
        
        if dist_from_center < 1e-10 {
            // Point is at center - all points on circle are equidistant
            // Return start point
            let closest = self.evaluate(0.0);
            let dist = self.radius.hypot(dist_to_plane);
            return Ok((0.0, closest, dist));
        }
        
        let closest_on_circle = self.center + to_proj * (self.radius / dist_from_center);
        
        // Find parameter for this point
        let to_closest = closest_on_circle - self.center;
        let perp = self.normal.cross(&self.start_vector);
        
        let cos_angle = self.start_vector.dot(&to_closest) / self.radius;
        let sin_angle = perp.dot(&to_closest) / self.radius;
        let mut angle = sin_angle.atan2(cos_angle);
        
        // Normalize angle to [0, sweep_angle]
        if angle < 0.0 {
            angle += std::f64::consts::TAU;
        }
        
        // Check if closest point is within arc
        let t = if angle <= self.sweep_angle {
            angle / self.sweep_angle
        } else if angle - std::f64::consts::TAU >= 0.0 {
            (angle - std::f64::consts::TAU) / self.sweep_angle
        } else {
            // Closest point is outside arc - check endpoints
            let dist_start = point.distance_to(&self.evaluate(0.0));
            let dist_end = point.distance_to(&self.evaluate(1.0));
            if dist_start < dist_end {
                return Ok((0.0, self.evaluate(0.0), dist_start));
            } else {
                return Ok((1.0, self.evaluate(1.0), dist_end));
            }
        };
        
        let closest = self.evaluate(t);
        let dist = point.distance_to(&closest);
        
        Ok((t, closest, dist))
    }

    fn reverse(&mut self) {
        // Reverse sweep direction
        self.sweep_angle = -self.sweep_angle;
        // Update start vector to end point
        self.start_vector = self.evaluate(1.0).to_vector().normalized();
    }

    fn transform(&mut self, transform: &Transform3) {
        self.center = transform.apply_to_point(&self.center);
        self.normal = transform.apply_to_vector(&self.normal).normalized();
        self.start_vector = transform.apply_to_vector(&self.start_vector).normalized();
        // Scale radius
        let scale = transform.to_matrix().rotation_scale()
            .to_nalgebra()
            .determinant()
            .cbrt()
            .abs();
        self.radius *= scale;
    }

    fn curve_type(&self) -> CurveType {
        CurveType::CircularArc
    }

    fn clone_box(&self) -> Box<dyn Curve> {
        Box::new(*self)
    }
}

impl Tessellatable for CircularArc {
    fn tessellate(&self, tolerance: f64) -> Tessellation {
        // Compute number of segments based on tolerance
        // For a circle, chord error = r * (1 - cos(theta/2))
        // Solving for theta: theta = 2 * acos(1 - tolerance/r)
        let max_angle = if tolerance >= self.radius {
            std::f64::consts::FRAC_PI_2
        } else {
            2.0 * (1.0 - tolerance / self.radius).acos()
        };
        
        let num_segments = ((self.sweep_angle.abs() / max_angle).ceil() as usize).max(2);
        
        let points: Vec<Point3> = (0..=num_segments)
            .map(|i| {
                let t = i as f64 / num_segments as f64;
                self.evaluate(t)
            })
            .collect();
        
        Tessellation::Polyline(points)
    }
}

/// Elliptical arc in 3D space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EllipseArc {
    /// Center of the ellipse
    center: Point3,
    /// Major axis radius
    major_radius: f64,
    /// Minor axis radius
    minor_radius: f64,
    /// Unit vector along major axis
    major_axis: Vec3,
    /// Unit normal to the plane
    normal: Vec3,
    /// Start angle (from major axis)
    start_angle: f64,
    /// Sweep angle
    sweep_angle: f64,
}

impl EllipseArc {
    /// Create an elliptical arc
    pub fn new(
        center: Point3,
        major_radius: f64,
        minor_radius: f64,
        major_axis: Vec3,
        normal: Vec3,
        start_angle: f64,
        sweep_angle: f64,
    ) -> GeomResult<Self> {
        if major_radius <= 0.0 || minor_radius <= 0.0 {
            return Err(GeometryError::InvalidParameter(
                "Radii must be positive".to_string()
            ));
        }
        
        let normal = normal.normalized();
        let major_axis = major_axis.reject_from(&normal).normalized();
        
        Ok(Self {
            center,
            major_radius,
            minor_radius,
            major_axis,
            normal,
            start_angle,
            sweep_angle,
        })
    }

    /// Get the minor axis direction
    pub fn minor_axis(&self) -> Vec3 {
        self.normal.cross(&self.major_axis)
    }

    /// Get a point at parameter angle
    pub fn point_at_angle(&self, angle: f64) -> Point3 {
        let minor_axis = self.minor_axis();
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        self.center 
            + self.major_axis * (self.major_radius * cos_a)
            + minor_axis * (self.minor_radius * sin_a)
    }
}

impl Curve for EllipseArc {
    fn evaluate(&self, t: f64) -> Point3 {
        let angle = self.start_angle + t * self.sweep_angle;
        self.point_at_angle(angle)
    }

    fn derivative(&self, t: f64, order: u32) -> Vec3 {
        let angle = self.start_angle + t * self.sweep_angle;
        let minor_axis = self.minor_axis();
        
        match order {
            0 => self.point_at_angle(angle).to_vector(),
            1 => {
                let cos_a = angle.cos();
                let sin_a = angle.sin();
                (-self.major_axis * (self.major_radius * sin_a)
                 + minor_axis * (self.minor_radius * cos_a)) * self.sweep_angle
            }
            2 => {
                let cos_a = angle.cos();
                let sin_a = angle.sin();
                (-self.major_axis * (self.major_radius * cos_a)
                 - minor_axis * (self.minor_radius * sin_a)) * (self.sweep_angle * self.sweep_angle)
            }
            _ => Vec3::ZERO,
        }
    }

    fn curvature(&self, t: f64) -> f64 {
        let angle = self.start_angle + t * self.sweep_angle;
        let a = self.major_radius;
        let b = self.minor_radius;
        let cos_t = angle.cos();
        let sin_t = angle.sin();
        
        // Curvature of ellipse at angle t
        let num = a * b;
        let den = (a * a * sin_t * sin_t + b * b * cos_t * cos_t).powf(1.5);
        num / den
    }

    fn param_range(&self) -> ParamRange {
        ParamRange::new(0.0, 1.0)
    }

    fn arc_length(&self, _t: f64) -> f64 {
        // Ellipse arc length requires elliptic integral
        // For now, approximate
        let a = self.major_radius;
        let b = self.minor_radius;
        let h = ((a - b) / (a + b)).powi(2);
        let approx_circumference = std::f64::consts::PI * (a + b) * (1.0 + 3.0 * h / (10.0 + (4.0 - 3.0 * h).sqrt()));
        approx_circumference * self.sweep_angle.abs() / std::f64::consts::TAU * _t
    }

    fn parameter_at_length(&self, _length: f64) -> Option<f64> {
        // Requires numerical integration
        None
    }

    fn closest_point(&self, point: &Point3) -> GeomResult<(f64, Point3, f64)> {
        // Project to plane and solve 2D closest point on ellipse
        // This is a numerical problem
        let to_point = *point - self.center;
        let dist_to_plane = to_point.dot(&self.normal);
        let projected = *point - self.normal * dist_to_plane;
        let to_proj = projected - self.center;
        
        let minor_axis = self.minor_axis();
        let x = to_proj.dot(&self.major_axis) / self.major_radius;
        let y = to_proj.dot(&minor_axis) / self.minor_radius;
        
        // Approximate closest point using angle
        let angle = y.atan2(x);
        let mut t = (angle - self.start_angle) / self.sweep_angle;
        
        // Clamp to [0, 1]
        t = t.clamp(0.0, 1.0);
        
        let closest = self.evaluate(t);
        let dist = point.distance_to(&closest);
        
        Ok((t, closest, dist))
    }

    fn reverse(&mut self) {
        self.sweep_angle = -self.sweep_angle;
        self.start_angle = self.start_angle + self.sweep_angle;
    }

    fn transform(&mut self, transform: &Transform3) {
        self.center = transform.apply_to_point(&self.center);
        self.normal = transform.apply_to_vector(&self.normal).normalized();
        self.major_axis = transform.apply_to_vector(&self.major_axis).normalized();
        // Scale radii
        let scale = transform.to_matrix().rotation_scale()
            .to_nalgebra()
            .determinant()
            .cbrt()
            .abs();
        self.major_radius *= scale;
        self.minor_radius *= scale;
    }

    fn curve_type(&self) -> CurveType {
        CurveType::EllipseArc
    }

    fn clone_box(&self) -> Box<dyn Curve> {
        Box::new(*self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_new() {
        let line = Line::infinite(Point3::ORIGIN, Vec3::X).unwrap();
        assert_eq!(line.origin(), Point3::ORIGIN);
        assert!(line.direction().dot(&Vec3::X) > 0.99);
    }

    #[test]
    fn test_line_segment() {
        let line = Line::segment(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)).unwrap();
        assert_eq!(line.start_point(), Point3::new(0.0, 0.0, 0.0));
        assert_eq!(line.end_point(), Point3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_line_evaluate() {
        let line = Line::infinite(Point3::ORIGIN, Vec3::X).unwrap();
        assert_eq!(line.evaluate(5.0), Point3::new(5.0, 0.0, 0.0));
    }

    #[test]
    fn test_line_closest_point() {
        let line = Line::infinite(Point3::ORIGIN, Vec3::X).unwrap();
        let point = Point3::new(5.0, 3.0, 0.0);
        let (t, closest, dist) = line.closest_point(&point).unwrap();
        assert_eq!(t, 5.0);
        assert_eq!(closest, Point3::new(5.0, 0.0, 0.0));
        assert_eq!(dist, 3.0);
    }

    #[test]
    fn test_circular_arc_new() {
        let arc = CircularArc::new(
            Point3::ORIGIN,
            5.0,
            Vec3::Z,
            Vec3::X,
            std::f64::consts::PI,
        ).unwrap();
        assert_eq!(arc.radius(), 5.0);
        assert_eq!(arc.sweep_angle(), std::f64::consts::PI);
    }

    #[test]
    fn test_circular_arc_evaluate() {
        let arc = CircularArc::circle(Point3::ORIGIN, 1.0, Vec3::Z).unwrap();
        let p = arc.evaluate(0.0);
        assert!((p.x() - 1.0).abs() < 1e-10);
        assert!(p.y().abs() < 1e-10);
        
        let p = arc.evaluate(0.25); // 90 degrees
        assert!((p.y() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_circular_arc_curvature() {
        let arc = CircularArc::circle(Point3::ORIGIN, 2.0, Vec3::Z).unwrap();
        assert_eq!(arc.curvature(0.0), 0.5);
        assert_eq!(arc.curvature(0.5), 0.5);
    }

    #[test]
    fn test_circular_arc_tessellate() {
        let arc = CircularArc::circle(Point3::ORIGIN, 10.0, Vec3::Z).unwrap();
        let tess = arc.tessellate(0.1);
        if let Tessellation::Polyline(points) = tess {
            assert!(points.len() >= 3);
        } else {
            panic!("Expected Polyline");
        }
    }
}
