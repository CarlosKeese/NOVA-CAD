//! Surface types and implementations

use crate::{GeomResult, GeometryError, UVRange, SurfaceEvaluation, Tessellation, Tessellatable,
            curve::{Curve, Line, CircularArc}};
use nova_math::{Point3, Vec3, Transform3, Plane};

/// Trait for all surface types
pub trait Surface: Send + Sync {
    /// Evaluate the surface at parameters (u, v)
    fn evaluate(&self, u: f64, v: f64) -> Point3;

    /// Evaluate partial derivatives
    fn derivatives(&self, u: f64, v: f64) -> (Vec3, Vec3);

    /// Get the unit normal at (u, v)
    fn normal(&self, u: f64, v: f64) -> Vec3 {
        let (du, dv) = self.derivatives(u, v);
        du.cross(&dv).normalized()
    }

    /// Get principal curvatures at (u, v)
    fn principal_curvatures(&self, u: f64, v: f64) -> (f64, f64);

    /// Get the Gaussian curvature at (u, v)
    fn gaussian_curvature(&self, u: f64, v: f64) -> f64 {
        let (k1, k2) = self.principal_curvatures(u, v);
        k1 * k2
    }

    /// Get the mean curvature at (u, v)
    fn mean_curvature(&self, u: f64, v: f64) -> f64 {
        let (k1, k2) = self.principal_curvatures(u, v);
        (k1 + k2) * 0.5
    }

    /// Get the UV parameter range
    fn uv_range(&self) -> UVRange;

    /// Check if UV is within valid range
    fn contains_uv(&self, u: f64, v: f64) -> bool {
        self.uv_range().contains(u, v)
    }

    /// Get an isocurve at constant u
    fn u_isocurve(&self, u: f64) -> Option<Box<dyn Curve>>;

    /// Get an isocurve at constant v
    fn v_isocurve(&self, v: f64) -> Option<Box<dyn Curve>>;

    /// Project a point to the surface (find closest point)
    fn closest_point(&self, point: &Point3) -> GeomResult<(f64, f64, Point3, f64)>;

    /// Transform the surface
    fn transform(&mut self, transform: &Transform3);

    /// Get surface type
    fn surface_type(&self) -> SurfaceType;

    /// Clone into a boxed surface
    fn clone_box(&self) -> Box<dyn Surface>;
}

/// Surface type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceType {
    /// Planar surface
    Planar,
    /// Cylindrical surface
    Cylindrical,
    /// Conical surface
    Conical,
    /// Spherical surface
    Spherical,
    /// Toroidal surface
    Toroidal,
    /// NURBS surface
    NurbsSurface,
    /// Swept surface
    SweptSurface,
    /// Offset surface
    OffsetSurface,
    /// Ruled surface
    RuledSurface,
}

/// Planar surface
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlanarSurface {
    /// Origin point
    origin: Point3,
    /// U axis (unit vector)
    u_axis: Vec3,
    /// V axis (unit vector)
    v_axis: Vec3,
    /// UV range
    uv_range: UVRange,
}

impl PlanarSurface {
    /// Create a planar surface from origin and two axes
    pub fn new(origin: Point3, u_axis: Vec3, v_axis: Vec3) -> GeomResult<Self> {
        let u_axis = u_axis.normalized();
        let v_axis = v_axis.normalized();
        
        // Check that axes are not parallel
        if u_axis.cross(&v_axis).is_zero(1e-10) {
            return Err(GeometryError::Degenerate(
                "U and V axes cannot be parallel".to_string()
            ));
        }
        
        Ok(Self {
            origin,
            u_axis,
            v_axis,
            uv_range: UVRange::new(f64::NEG_INFINITY, f64::INFINITY, 
                                    f64::NEG_INFINITY, f64::INFINITY),
        })
    }

    /// Create a bounded planar surface
    pub fn bounded(
        origin: Point3,
        u_axis: Vec3,
        v_axis: Vec3,
        u_min: f64,
        u_max: f64,
        v_min: f64,
        v_max: f64,
    ) -> GeomResult<Self> {
        let u_axis = u_axis.normalized();
        let v_axis = v_axis.normalized();
        
        if u_axis.cross(&v_axis).is_zero(1e-10) {
            return Err(GeometryError::Degenerate(
                "U and V axes cannot be parallel".to_string()
            ));
        }
        
        Ok(Self {
            origin,
            u_axis,
            v_axis,
            uv_range: UVRange::new(u_min, u_max, v_min, v_max),
        })
    }

    /// Create from a plane
    pub fn from_plane(plane: &Plane) -> Self {
        let (u_axis, v_axis) = plane.basis_vectors();
        Self {
            origin: plane.origin(),
            u_axis,
            v_axis,
            uv_range: UVRange::new(f64::NEG_INFINITY, f64::INFINITY,
                                    f64::NEG_INFINITY, f64::INFINITY),
        }
    }

    /// Get the plane representation
    pub fn to_plane(&self) -> Plane {
        let normal = self.u_axis.cross(&self.v_axis).normalized();
        Plane::new(self.origin, normal)
    }

    /// Get the origin
    pub fn origin(&self) -> Point3 {
        self.origin
    }

    /// Get the U axis
    pub fn u_axis(&self) -> Vec3 {
        self.u_axis
    }

    /// Get the V axis
    pub fn v_axis(&self) -> Vec3 {
        self.v_axis
    }

    /// Get the normal
    pub fn normal(&self) -> Vec3 {
        self.u_axis.cross(&self.v_axis).normalized()
    }

    /// Convert UV to 3D point
    pub fn uv_to_point(&self, u: f64, v: f64) -> Point3 {
        self.origin + self.u_axis * u + self.v_axis * v
    }

    /// Convert 3D point to UV (projection onto plane)
    pub fn point_to_uv(&self, point: &Point3) -> (f64, f64) {
        let to_point = *point - self.origin;
        let u = to_point.dot(&self.u_axis);
        let v = to_point.dot(&self.v_axis);
        (u, v)
    }

    /// Check if the surface is bounded
    pub fn is_bounded(&self) -> bool {
        let uv = self.uv_range;
        uv.u.start.is_finite() && uv.u.end.is_finite() &&
        uv.v.start.is_finite() && uv.v.end.is_finite()
    }
}

impl Surface for PlanarSurface {
    fn evaluate(&self, u: f64, v: f64) -> Point3 {
        self.uv_to_point(u, v)
    }

    fn derivatives(&self, _u: f64, _v: f64) -> (Vec3, Vec3) {
        (self.u_axis, self.v_axis)
    }

    fn normal(&self, _u: f64, _v: f64) -> Vec3 {
        self.normal()
    }

    fn principal_curvatures(&self, _u: f64, _v: f64) -> (f64, f64) {
        (0.0, 0.0) // Planes have zero curvature
    }

    fn uv_range(&self) -> UVRange {
        self.uv_range
    }

    fn u_isocurve(&self, u: f64) -> Option<Box<dyn Curve>> {
        let start = self.evaluate(u, self.uv_range.v.start);
        let direction = self.v_axis;
        let range = self.uv_range.v;
        Some(Box::new(Line::with_range(start, direction, range).ok()?))
    }

    fn v_isocurve(&self, v: f64) -> Option<Box<dyn Curve>> {
        let start = self.evaluate(self.uv_range.u.start, v);
        let direction = self.u_axis;
        let range = self.uv_range.u;
        Some(Box::new(Line::with_range(start, direction, range).ok()?))
    }

    fn closest_point(&self, point: &Point3) -> GeomResult<(f64, f64, Point3, f64)> {
        let plane = self.to_plane();
        let projected = plane.project_point(point);
        let (u, v) = self.point_to_uv(&projected);
        
        // Clamp to range if bounded
        let uv = self.uv_range;
        let u = if uv.u.start.is_finite() { u.clamp(uv.u.start, uv.u.end) } else { u };
        let v = if uv.v.start.is_finite() { v.clamp(uv.v.start, uv.v.end) } else { v };
        
        let closest = self.evaluate(u, v);
        let dist = point.distance_to(&closest);
        
        Ok((u, v, closest, dist))
    }

    fn transform(&mut self, transform: &Transform3) {
        self.origin = transform.apply_to_point(&self.origin);
        self.u_axis = transform.apply_to_vector(&self.u_axis).normalized();
        self.v_axis = transform.apply_to_vector(&self.v_axis).normalized();
    }

    fn surface_type(&self) -> SurfaceType {
        SurfaceType::Planar
    }

    fn clone_box(&self) -> Box<dyn Surface> {
        Box::new(*self)
    }
}

/// Cylindrical surface
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CylindricalSurface {
    /// Origin (center of base circle)
    origin: Point3,
    /// Axis direction (unit vector)
    axis: Vec3,
    /// Radius
    radius: f64,
    /// Reference direction (unit vector perpendicular to axis)
    ref_direction: Vec3,
    /// Height range along axis
    height_range: (f64, f64),
    /// Angle range (in radians)
    angle_range: (f64, f64),
}

impl CylindricalSurface {
    /// Create a cylindrical surface
    pub fn new(
        origin: Point3,
        axis: Vec3,
        radius: f64,
        ref_direction: Vec3,
    ) -> GeomResult<Self> {
        if radius <= 0.0 {
            return Err(GeometryError::InvalidParameter(
                "Radius must be positive".to_string()
            ));
        }
        
        let axis = axis.normalized();
        let mut ref_direction = ref_direction;
        
        // Project ref_direction to be perpendicular to axis
        ref_direction = ref_direction.reject_from(&axis);
        if ref_direction.is_zero(1e-10) {
            return Err(GeometryError::InvalidParameter(
                "Reference direction cannot be parallel to axis".to_string()
            ));
        }
        ref_direction = ref_direction.normalized();
        
        Ok(Self {
            origin,
            axis,
            radius,
            ref_direction,
            height_range: (f64::NEG_INFINITY, f64::INFINITY),
            angle_range: (0.0, std::f64::consts::TAU),
        })
    }

    /// Create a bounded cylinder
    pub fn bounded(
        origin: Point3,
        axis: Vec3,
        radius: f64,
        ref_direction: Vec3,
        height_min: f64,
        height_max: f64,
        angle_min: f64,
        angle_max: f64,
    ) -> GeomResult<Self> {
        let mut surf = Self::new(origin, axis, radius, ref_direction)?;
        surf.height_range = (height_min, height_max);
        surf.angle_range = (angle_min, angle_max);
        Ok(surf)
    }

    /// Get the origin
    pub fn origin(&self) -> Point3 {
        self.origin
    }

    /// Get the axis
    pub fn axis(&self) -> Vec3 {
        self.axis
    }

    /// Get the radius
    pub fn radius(&self) -> f64 {
        self.radius
    }

    /// Get the perpendicular direction
    pub fn perpendicular_direction(&self) -> Vec3 {
        self.axis.cross(&self.ref_direction)
    }

    /// Check if full cylinder (360 degrees)
    pub fn is_full(&self) -> bool {
        (self.angle_range.1 - self.angle_range.0 - std::f64::consts::TAU).abs() < 1e-10
    }
}

impl Surface for CylindricalSurface {
    fn evaluate(&self, u: f64, v: f64) -> Point3 {
        // u = angle, v = height along axis
        let angle = self.angle_range.0 + u * (self.angle_range.1 - self.angle_range.0);
        let height = self.height_range.0 + v * (self.height_range.1 - self.height_range.0);
        
        let perp = self.perpendicular_direction();
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        
        self.origin 
            + self.ref_direction * (self.radius * cos_a)
            + perp * (self.radius * sin_a)
            + self.axis * height
    }

    fn derivatives(&self, u: f64, v: f64) -> (Vec3, Vec3) {
        let angle = self.angle_range.0 + u * (self.angle_range.1 - self.angle_range.0);
        let angle_scale = self.angle_range.1 - self.angle_range.0;
        let perp = self.perpendicular_direction();
        
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        
        // du: tangent along circle
        let du = (-self.ref_direction * sin_a + perp * cos_a) * (self.radius * angle_scale);
        
        // dv: along axis
        let dv = self.axis * (self.height_range.1 - self.height_range.0);
        
        (du, dv)
    }

    fn principal_curvatures(&self, _u: f64, _v: f64) -> (f64, f64) {
        // Cylinder has one principal curvature (1/r) and one zero
        (1.0 / self.radius, 0.0)
    }

    fn uv_range(&self) -> UVRange {
        UVRange::new(0.0, 1.0, 0.0, 1.0)
    }

    fn u_isocurve(&self, u: f64) -> Option<Box<dyn Curve>> {
        // Vertical line at angle u
        let angle = self.angle_range.0 + u * (self.angle_range.1 - self.angle_range.0);
        let perp = self.perpendicular_direction();
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        
        let base_point = self.origin 
            + self.ref_direction * (self.radius * cos_a)
            + perp * (self.radius * sin_a);
        
        Some(Box::new(Line::with_range(
            base_point,
            self.axis,
            crate::ParamRange::new(self.height_range.0, self.height_range.1),
        ).ok()?))
    }

    fn v_isocurve(&self, v: f64) -> Option<Box<dyn Curve>> {
        // Circle at height v
        let height = self.height_range.0 + v * (self.height_range.1 - self.height_range.0);
        let center = self.origin + self.axis * height;
        
        Some(Box::new(CircularArc::bounded(
            center,
            self.radius,
            self.axis,
            self.ref_direction,
            self.angle_range.0,
            self.angle_range.1,
        ).ok()?))
    }

    fn closest_point(&self, point: &Point3) -> GeomResult<(f64, f64, Point3, f64)> {
        // Project point onto axis
        let to_point = *point - self.origin;
        let height = to_point.dot(&self.axis);
        let height_clamped = height.clamp(self.height_range.0, self.height_range.1);
        
        // Project onto cylinder axis plane
        let projected = *point - self.axis * height;
        let to_proj = projected - self.origin;
        
        // Find angle
        let perp = self.perpendicular_direction();
        let x = to_proj.dot(&self.ref_direction);
        let y = to_proj.dot(&perp);
        let angle = y.atan2(x);
        
        // Normalize angle to [0, 1]
        let angle_normalized = (angle - self.angle_range.0) / (self.angle_range.1 - self.angle_range.0);
        let angle_normalized = angle_normalized.clamp(0.0, 1.0);
        
        let v = (height_clamped - self.height_range.0) / (self.height_range.1 - self.height_range.0);
        
        let closest = self.evaluate(angle_normalized, v);
        let dist = point.distance_to(&closest);
        
        Ok((angle_normalized, v, closest, dist))
    }

    fn transform(&mut self, transform: &Transform3) {
        self.origin = transform.apply_to_point(&self.origin);
        self.axis = transform.apply_to_vector(&self.axis).normalized();
        self.ref_direction = transform.apply_to_vector(&self.ref_direction).normalized();
        // Scale radius
        let scale = transform.to_matrix().rotation_scale()
            .to_nalgebra()
            .determinant()
            .cbrt()
            .abs();
        self.radius *= scale;
    }

    fn surface_type(&self) -> SurfaceType {
        SurfaceType::Cylindrical
    }

    fn clone_box(&self) -> Box<dyn Surface> {
        Box::new(*self)
    }
}

/// Spherical surface
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphericalSurface {
    /// Center
    center: Point3,
    /// Radius
    radius: f64,
    /// North pole direction (Z axis)
    axis: Vec3,
    /// Reference direction (X axis)
    ref_direction: Vec3,
    /// U angle range (azimuth)
    u_range: (f64, f64),
    /// V angle range (polar, from 0 at north pole to PI at south pole)
    v_range: (f64, f64),
}

impl SphericalSurface {
    /// Create a spherical surface
    pub fn new(center: Point3, radius: f64, axis: Vec3, ref_direction: Vec3) -> GeomResult<Self> {
        if radius <= 0.0 {
            return Err(GeometryError::InvalidParameter(
                "Radius must be positive".to_string()
            ));
        }
        
        let axis = axis.normalized();
        let mut ref_direction = ref_direction;
        ref_direction = ref_direction.reject_from(&axis).normalized();
        
        if ref_direction.is_zero(1e-10) {
            return Err(GeometryError::InvalidParameter(
                "Reference direction cannot be parallel to axis".to_string()
            ));
        }
        
        Ok(Self {
            center,
            radius,
            axis,
            ref_direction,
            u_range: (0.0, std::f64::consts::TAU),
            v_range: (0.0, std::f64::consts::PI),
        })
    }

    /// Get the center
    pub fn center(&self) -> Point3 {
        self.center
    }

    /// Get the radius
    pub fn radius(&self) -> f64 {
        self.radius
    }

    /// Check if full sphere
    pub fn is_full_sphere(&self) -> bool {
        (self.u_range.1 - self.u_range.0 - std::f64::consts::TAU).abs() < 1e-10 &&
        (self.v_range.1 - self.v_range.0 - std::f64::consts::PI).abs() < 1e-10
    }
}

impl Surface for SphericalSurface {
    fn evaluate(&self, u: f64, v: f64) -> Point3 {
        // u = azimuth angle, v = polar angle (0 = north pole)
        let azimuth = self.u_range.0 + u * (self.u_range.1 - self.u_range.0);
        let polar = self.v_range.0 + v * (self.v_range.1 - self.v_range.0);
        
        let sin_polar = polar.sin();
        let cos_polar = polar.cos();
        let sin_azimuth = azimuth.sin();
        let cos_azimuth = azimuth.cos();
        
        let perp = self.axis.cross(&self.ref_direction);
        
        self.center
            + self.axis * (self.radius * cos_polar)
            + self.ref_direction * (self.radius * sin_polar * cos_azimuth)
            + perp * (self.radius * sin_polar * sin_azimuth)
    }

    fn derivatives(&self, u: f64, v: f64) -> (Vec3, Vec3) {
        let azimuth = self.u_range.0 + u * (self.u_range.1 - self.u_range.0);
        let polar = self.v_range.0 + v * (self.v_range.1 - self.v_range.0);
        let u_scale = self.u_range.1 - self.u_range.0;
        let v_scale = self.v_range.1 - self.v_range.0;
        
        let sin_polar = polar.sin();
        let cos_polar = polar.cos();
        let sin_azimuth = azimuth.sin();
        let cos_azimuth = azimuth.cos();
        
        let perp = self.axis.cross(&self.ref_direction);
        
        // du: tangent along azimuth
        let du = (-self.ref_direction * sin_azimuth + perp * cos_azimuth) 
                 * (self.radius * sin_polar * u_scale);
        
        // dv: tangent along polar direction
        let dv = (self.axis * (-sin_polar) 
                  + self.ref_direction * (cos_polar * cos_azimuth)
                  + perp * (cos_polar * sin_azimuth))
                 * (self.radius * v_scale);
        
        (du, dv)
    }

    fn principal_curvatures(&self, _u: f64, _v: f64) -> (f64, f64) {
        // Sphere has constant curvature
        (1.0 / self.radius, 1.0 / self.radius)
    }

    fn uv_range(&self) -> UVRange {
        UVRange::new(0.0, 1.0, 0.0, 1.0)
    }

    fn u_isocurve(&self, u: f64) -> Option<Box<dyn Curve>> {
        // Meridian at azimuth u
        let azimuth = self.u_range.0 + u * (self.u_range.1 - self.u_range.0);
        let perp = self.axis.cross(&self.ref_direction);
        let cos_a = azimuth.cos();
        let sin_a = azimuth.sin();
        
        // Direction in the meridian plane
        let meridian_dir = self.ref_direction * cos_a + perp * sin_a;
        
        // Start from north pole
        let start = self.center + self.axis * self.radius;
        
        // This would be a circular arc - simplified for now
        None
    }

    fn v_isocurve(&self, v: f64) -> Option<Box<dyn Curve>> {
        // Parallel circle at polar angle v
        let polar = self.v_range.0 + v * (self.v_range.1 - self.v_range.0);
        let circle_radius = self.radius * polar.sin();
        let circle_center = self.center + self.axis * (self.radius * polar.cos());
        
        Some(Box::new(CircularArc::bounded(
            circle_center,
            circle_radius,
            self.axis,
            self.ref_direction,
            self.u_range.0,
            self.u_range.1,
        ).ok()?))
    }

    fn closest_point(&self, point: &Point3) -> GeomResult<(f64, f64, Point3, f64)> {
        let to_point = *point - self.center;
        let dist = to_point.length();
        
        if dist < 1e-10 {
            return Err(GeometryError::Degenerate(
                "Point is at sphere center".to_string()
            ));
        }
        
        // Normalize and scale to radius
        let on_sphere = self.center + to_point * (self.radius / dist);
        
        // Compute UV from point on sphere
        let to_sphere = on_sphere - self.center;
        let polar = to_sphere.dot(&self.axis).acos() / self.radius;
        
        let perp = self.axis.cross(&self.ref_direction);
        let x = to_sphere.dot(&self.ref_direction);
        let y = to_sphere.dot(&perp);
        let azimuth = y.atan2(x);
        
        let u = (azimuth - self.u_range.0) / (self.u_range.1 - self.u_range.0);
        let v = (polar - self.v_range.0) / (self.v_range.1 - self.v_range.0);
        
        let u = u.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);
        
        let closest = self.evaluate(u, v);
        let dist = point.distance_to(&closest);
        
        Ok((u, v, closest, dist))
    }

    fn transform(&mut self, transform: &Transform3) {
        self.center = transform.apply_to_point(&self.center);
        self.axis = transform.apply_to_vector(&self.axis).normalized();
        self.ref_direction = transform.apply_to_vector(&self.ref_direction).normalized();
        let scale = transform.to_matrix().rotation_scale()
            .to_nalgebra()
            .determinant()
            .cbrt()
            .abs();
        self.radius *= scale;
    }

    fn surface_type(&self) -> SurfaceType {
        SurfaceType::Spherical
    }

    fn clone_box(&self) -> Box<dyn Surface> {
        Box::new(*self)
    }
}

/// Conical surface
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConicalSurface {
    /// Apex point
    apex: Point3,
    /// Axis direction (from apex toward base)
    axis: Vec3,
    /// Half angle (angle between axis and surface)
    half_angle: f64,
    /// Reference direction
    ref_direction: Vec3,
    /// Height range from apex
    height_range: (f64, f64),
    /// Angle range around axis
    angle_range: (f64, f64),
}

impl ConicalSurface {
    /// Create a conical surface
    pub fn new(apex: Point3, axis: Vec3, half_angle: f64, ref_direction: Vec3) -> GeomResult<Self> {
        if half_angle <= 0.0 || half_angle >= std::f64::consts::FRAC_PI_2 {
            return Err(GeometryError::InvalidParameter(
                "Half angle must be between 0 and 90 degrees".to_string()
            ));
        }
        
        let axis = axis.normalized();
        let mut ref_direction = ref_direction;
        ref_direction = ref_direction.reject_from(&axis).normalized();
        
        if ref_direction.is_zero(1e-10) {
            return Err(GeometryError::InvalidParameter(
                "Reference direction cannot be parallel to axis".to_string()
            ));
        }
        
        Ok(Self {
            apex,
            axis,
            half_angle,
            ref_direction,
            height_range: (0.0, f64::INFINITY),
            angle_range: (0.0, std::f64::consts::TAU),
        })
    }

    /// Get the apex
    pub fn apex(&self) -> Point3 {
        self.apex
    }

    /// Get the half angle
    pub fn half_angle(&self) -> f64 {
        self.half_angle
    }

    /// Get radius at a given height from apex
    pub fn radius_at_height(&self, height: f64) -> f64 {
        height * self.half_angle.tan()
    }
}

impl Surface for ConicalSurface {
    fn evaluate(&self, u: f64, v: f64) -> Point3 {
        let angle = self.angle_range.0 + u * (self.angle_range.1 - self.angle_range.0);
        let height = self.height_range.0 + v * (self.height_range.1 - self.height_range.0);
        let radius = self.radius_at_height(height);
        
        let perp = self.axis.cross(&self.ref_direction);
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        
        self.apex
            + self.axis * height
            + self.ref_direction * (radius * cos_a)
            + perp * (radius * sin_a)
    }

    fn derivatives(&self, u: f64, v: f64) -> (Vec3, Vec3) {
        let angle = self.angle_range.0 + u * (self.angle_range.1 - self.angle_range.0);
        let height = self.height_range.0 + v * (self.height_range.1 - self.height_range.0);
        let radius = self.radius_at_height(height);
        let u_scale = self.angle_range.1 - self.angle_range.0;
        let v_scale = self.height_range.1 - self.height_range.0;
        
        let perp = self.axis.cross(&self.ref_direction);
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        
        // du: tangent along circle
        let du = (-self.ref_direction * sin_a + perp * cos_a) * (radius * u_scale);
        
        // dv: along generator line
        let tan_angle = self.half_angle.tan();
        let dv = (self.axis + (self.ref_direction * cos_a + perp * sin_a) * tan_angle) * v_scale;
        
        (du, dv)
    }

    fn principal_curvatures(&self, _u: f64, v: f64) -> (f64, f64) {
        let height = self.height_range.0 + v * (self.height_range.1 - self.height_range.0);
        let radius = self.radius_at_height(height);
        let cos_half = self.half_angle.cos();
        
        // One curvature is zero (along generator)
        // Other is 1 / (radius / cos(half_angle))
        (cos_half / radius, 0.0)
    }

    fn uv_range(&self) -> UVRange {
        UVRange::new(0.0, 1.0, 0.0, 1.0)
    }

    fn u_isocurve(&self, _u: f64) -> Option<Box<dyn Curve>> {
        // Generator line at angle u
        None // Would be a line
    }

    fn v_isocurve(&self, v: f64) -> Option<Box<dyn Curve>> {
        // Circle at height v
        let height = self.height_range.0 + v * (self.height_range.1 - self.height_range.0);
        let radius = self.radius_at_height(height);
        let center = self.apex + self.axis * height;
        
        Some(Box::new(CircularArc::bounded(
            center,
            radius,
            self.axis,
            self.ref_direction,
            self.angle_range.0,
            self.angle_range.1,
        ).ok()?))
    }

    fn closest_point(&self, point: &Point3) -> GeomResult<(f64, f64, Point3, f64)> {
        // Project onto cone surface
        let to_point = *point - self.apex;
        let height = to_point.dot(&self.axis).clamp(self.height_range.0, self.height_range.1);
        
        // Find closest point on circle at this height
        let center = self.apex + self.axis * height;
        let radius = self.radius_at_height(height);
        
        let to_center = *point - center;
        let perp = self.axis.cross(&self.ref_direction);
        let x = to_center.dot(&self.ref_direction);
        let y = to_center.dot(&perp);
        let angle = y.atan2(x);
        
        let u = (angle - self.angle_range.0) / (self.angle_range.1 - self.angle_range.0);
        let v = (height - self.height_range.0) / (self.height_range.1 - self.height_range.0);
        
        let u = u.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);
        
        let closest = self.evaluate(u, v);
        let dist = point.distance_to(&closest);
        
        Ok((u, v, closest, dist))
    }

    fn transform(&mut self, transform: &Transform3) {
        self.apex = transform.apply_to_point(&self.apex);
        self.axis = transform.apply_to_vector(&self.axis).normalized();
        self.ref_direction = transform.apply_to_vector(&self.ref_direction).normalized();
    }

    fn surface_type(&self) -> SurfaceType {
        SurfaceType::Conical
    }

    fn clone_box(&self) -> Box<dyn Surface> {
        Box::new(*self)
    }
}

/// Toroidal surface
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToroidalSurface {
    /// Center of torus
    center: Point3,
    /// Axis of revolution
    axis: Vec3,
    /// Major radius (distance from center to tube center)
    major_radius: f64,
    /// Minor radius (tube radius)
    minor_radius: f64,
    /// Reference direction
    ref_direction: Vec3,
    /// Major angle range (around axis)
    major_range: (f64, f64),
    /// Minor angle range (around tube)
    minor_range: (f64, f64),
}

impl ToroidalSurface {
    /// Create a toroidal surface
    pub fn new(
        center: Point3,
        axis: Vec3,
        major_radius: f64,
        minor_radius: f64,
        ref_direction: Vec3,
    ) -> GeomResult<Self> {
        if major_radius <= 0.0 || minor_radius <= 0.0 {
            return Err(GeometryError::InvalidParameter(
                "Radii must be positive".to_string()
            ));
        }
        
        if minor_radius >= major_radius {
            return Err(GeometryError::InvalidParameter(
                "Minor radius must be less than major radius".to_string()
            ));
        }
        
        let axis = axis.normalized();
        let mut ref_direction = ref_direction;
        ref_direction = ref_direction.reject_from(&axis).normalized();
        
        if ref_direction.is_zero(1e-10) {
            return Err(GeometryError::InvalidParameter(
                "Reference direction cannot be parallel to axis".to_string()
            ));
        }
        
        Ok(Self {
            center,
            axis,
            major_radius,
            minor_radius,
            ref_direction,
            major_range: (0.0, std::f64::consts::TAU),
            minor_range: (0.0, std::f64::consts::TAU),
        })
    }

    /// Get the center
    pub fn center(&self) -> Point3 {
        self.center
    }

    /// Get the major radius
    pub fn major_radius(&self) -> f64 {
        self.major_radius
    }

    /// Get the minor radius
    pub fn minor_radius(&self) -> f64 {
        self.minor_radius
    }

    /// Check if full torus
    pub fn is_full_torus(&self) -> bool {
        (self.major_range.1 - self.major_range.0 - std::f64::consts::TAU).abs() < 1e-10 &&
        (self.minor_range.1 - self.minor_range.0 - std::f64::consts::TAU).abs() < 1e-10
    }
}

impl Surface for ToroidalSurface {
    fn evaluate(&self, u: f64, v: f64) -> Point3 {
        // u = major angle, v = minor angle
        let major_angle = self.major_range.0 + u * (self.major_range.1 - self.major_range.0);
        let minor_angle = self.minor_range.0 + v * (self.minor_range.1 - self.minor_range.0);
        
        let perp = self.axis.cross(&self.ref_direction);
        let cos_major = major_angle.cos();
        let sin_major = major_angle.sin();
        let cos_minor = minor_angle.cos();
        let sin_minor = minor_angle.sin();
        
        // Tube center at this major angle
        let tube_center = self.center
            + self.ref_direction * (self.major_radius * cos_major)
            + perp * (self.major_radius * sin_major);
        
        // Point on tube surface
        let tube_axis = (self.ref_direction * cos_major + perp * sin_major).cross(&self.axis);
        
        tube_center
            + self.axis * (self.minor_radius * cos_minor)
            + (self.ref_direction * cos_major + perp * sin_major) * (self.minor_radius * sin_minor)
    }

    fn derivatives(&self, u: f64, v: f64) -> (Vec3, Vec3) {
        // Complex derivatives for torus
        // Simplified implementation
        let eps = 1e-8;
        let p = self.evaluate(u, v);
        let pu = self.evaluate(u + eps, v);
        let pv = self.evaluate(u, v + eps);
        
        let du = (pu - p) / eps;
        let dv = (pv - p) / eps;
        
        (du, dv)
    }

    fn principal_curvatures(&self, u: f64, v: f64) -> (f64, f64) {
        let minor_angle = self.minor_range.0 + v * (self.minor_range.1 - self.minor_range.0);
        let cos_minor = minor_angle.cos();
        
        // Principal curvatures of torus
        let k1 = cos_minor / (self.major_radius + self.minor_radius * cos_minor);
        let k2 = 1.0 / self.minor_radius;
        
        (k1, k2)
    }

    fn uv_range(&self) -> UVRange {
        UVRange::new(0.0, 1.0, 0.0, 1.0)
    }

    fn u_isocurve(&self, u: f64) -> Option<Box<dyn Curve>> {
        // Circle around tube at major angle u
        let major_angle = self.major_range.0 + u * (self.major_range.1 - self.major_range.0);
        let perp = self.axis.cross(&self.ref_direction);
        let cos_major = major_angle.cos();
        let sin_major = major_angle.sin();
        
        let tube_center = self.center
            + self.ref_direction * (self.major_radius * cos_major)
            + perp * (self.major_radius * sin_major);
        
        let tube_axis = self.ref_direction * cos_major + perp * sin_major;
        
        Some(Box::new(CircularArc::bounded(
            tube_center,
            self.minor_radius,
            tube_axis,
            self.axis,
            self.minor_range.0,
            self.minor_range.1,
        ).ok()?))
    }

    fn v_isocurve(&self, v: f64) -> Option<Box<dyn Curve>> {
        // Circle around axis at minor angle v
        let minor_angle = self.minor_range.0 + v * (self.minor_range.1 - self.minor_range.0);
        let cos_minor = minor_angle.cos();
        let sin_minor = minor_angle.sin();
        
        let circle_radius = self.major_radius + self.minor_radius * sin_minor;
        let circle_center = self.center + self.axis * (self.minor_radius * cos_minor);
        
        Some(Box::new(CircularArc::bounded(
            circle_center,
            circle_radius,
            self.axis,
            self.ref_direction,
            self.major_range.0,
            self.major_range.1,
        ).ok()?))
    }

    fn closest_point(&self, point: &Point3) -> GeomResult<(f64, f64, Point3, f64)> {
        // Iterative closest point for torus
        // Start with projection onto torus plane
        let to_point = *point - self.center;
        let height = to_point.dot(&self.axis);
        
        // Initial guess: project to major circle
        let perp = self.axis.cross(&self.ref_direction);
        let x = to_point.dot(&self.ref_direction);
        let y = to_point.dot(&perp);
        let major_angle = y.atan2(x);
        
        let mut u = (major_angle - self.major_range.0) / (self.major_range.1 - self.major_range.0);
        let mut v = 0.5; // Initial guess for minor angle
        
        // Simple iteration
        for _ in 0..10 {
            let p = self.evaluate(u, v);
            let (du, dv) = self.derivatives(u, v);
            let to_p = *point - p;
            
            // Newton step
            let g11 = du.dot(&du);
            let g12 = du.dot(&dv);
            let g22 = dv.dot(&dv);
            let det = g11 * g22 - g12 * g12;
            
            if det.abs() < 1e-10 {
                break;
            }
            
            let f1 = to_p.dot(&du);
            let f2 = to_p.dot(&dv);
            
            let du_step = (f1 * g22 - f2 * g12) / det;
            let dv_step = (f2 * g11 - f1 * g12) / det;
            
            u = (u + du_step).clamp(0.0, 1.0);
            v = (v + dv_step).clamp(0.0, 1.0);
        }
        
        let closest = self.evaluate(u, v);
        let dist = point.distance_to(&closest);
        
        Ok((u, v, closest, dist))
    }

    fn transform(&mut self, transform: &Transform3) {
        self.center = transform.apply_to_point(&self.center);
        self.axis = transform.apply_to_vector(&self.axis).normalized();
        self.ref_direction = transform.apply_to_vector(&self.ref_direction).normalized();
        let scale = transform.to_matrix().rotation_scale()
            .to_nalgebra()
            .determinant()
            .cbrt()
            .abs();
        self.major_radius *= scale;
        self.minor_radius *= scale;
    }

    fn surface_type(&self) -> SurfaceType {
        SurfaceType::Toroidal
    }

    fn clone_box(&self) -> Box<dyn Surface> {
        Box::new(*self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planar_surface() {
        let plane = PlanarSurface::new(
            Point3::ORIGIN,
            Vec3::X,
            Vec3::Y,
        ).unwrap();
        
        let p = plane.evaluate(2.0, 3.0);
        assert_eq!(p.x(), 2.0);
        assert_eq!(p.y(), 3.0);
        assert_eq!(p.z(), 0.0);
    }

    #[test]
    fn test_cylindrical_surface() {
        let cyl = CylindricalSurface::new(
            Point3::ORIGIN,
            Vec3::Z,
            5.0,
            Vec3::X,
        ).unwrap();
        
        assert_eq!(cyl.radius(), 5.0);
        
        let p = cyl.evaluate(0.0, 0.5); // angle 0, half height
        assert!((p.x() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_spherical_surface() {
        let sphere = SphericalSurface::new(
            Point3::ORIGIN,
            10.0,
            Vec3::Z,
            Vec3::X,
        ).unwrap();
        
        assert_eq!(sphere.radius(), 10.0);
        
        // Point at north pole
        let p = sphere.evaluate(0.0, 0.0);
        assert!((p.z() - 10.0).abs() < 1e-10);
        
        // Point on equator
        let p = sphere.evaluate(0.0, 0.5);
        assert!((p.z()).abs() < 1e-10);
        assert!((p.x() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_surface_curvatures() {
        let plane = PlanarSurface::new(Point3::ORIGIN, Vec3::X, Vec3::Y).unwrap();
        let (k1, k2) = plane.principal_curvatures(0.0, 0.0);
        assert_eq!(k1, 0.0);
        assert_eq!(k2, 0.0);
        
        let sphere = SphericalSurface::new(Point3::ORIGIN, 5.0, Vec3::Z, Vec3::X).unwrap();
        let (k1, k2) = sphere.principal_curvatures(0.0, 0.0);
        assert!((k1 - 0.2).abs() < 1e-10);
        assert!((k2 - 0.2).abs() < 1e-10);
    }
}
