//! Nova Geom - Geometry Engine for Nova Kernel 3D
//! 
//! Provides curve and surface types with full geometric interrogation:
//! - Analytic curves: Line, Arc, Ellipse, NURBS
//! - Analytic surfaces: Plane, Cylinder, Cone, Sphere, Torus, NURBS
//! - Intersection algorithms
//! - Closest point projection
//! - Tessellation support

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod curve;
pub mod surface;
pub mod nurbs;
pub mod intersection;

pub use curve::{Curve, CurveType, Line, CircularArc, EllipseArc};
pub use surface::{Surface, SurfaceType, PlanarSurface, CylindricalSurface, 
                  ConicalSurface, SphericalSurface, ToroidalSurface};

use nova_math::{Point3, Vec3};
use thiserror::Error;

/// Geometry-related errors
#[derive(Debug, Error, Clone, PartialEq)]
pub enum GeometryError {
    /// Invalid parameter value
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    /// Degenerate geometry
    #[error("Degenerate geometry: {0}")]
    Degenerate(String),
    
    /// Intersection failed
    #[error("Intersection failed: {0}")]
    IntersectionFailed(String),
    
    /// Closest point computation failed
    #[error("Closest point failed: {0}")]
    ClosestPointFailed(String),
    
    /// Unsupported operation
    #[error("Unsupported operation: {0}")]
    Unsupported(String),
    
    /// Numerical error
    #[error("Numerical error: {0}")]
    Numerical(String),
}

/// Result type for geometry operations
pub type GeomResult<T> = Result<T, GeometryError>;

/// Curve evaluation result
#[derive(Debug, Clone, Copy)]
pub struct CurveEvaluation {
    /// Point on curve
    pub point: Point3,
    /// First derivative (tangent direction)
    pub tangent: Vec3,
    /// Curvature
    pub curvature: f64,
}

/// Surface evaluation result
#[derive(Debug, Clone, Copy)]
pub struct SurfaceEvaluation {
    /// Point on surface
    pub point: Point3,
    /// Partial derivative in U direction
    pub du: Vec3,
    /// Partial derivative in V direction
    pub dv: Vec3,
    /// Unit normal
    pub normal: Vec3,
    /// Principal curvatures
    pub curvature: (f64, f64),
}

/// Intersection result type
pub enum IntersectionResult {
    /// Point intersection
    Point(Point3),
    /// Curve intersection
    Curve(Box<dyn Curve>),
    /// No intersection
    None,
}

impl std::fmt::Debug for IntersectionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntersectionResult::Point(p) => f.debug_tuple("Point").field(p).finish(),
            IntersectionResult::Curve(_) => f.debug_tuple("Curve").field(&"<dyn Curve>").finish(),
            IntersectionResult::None => f.debug_struct("None").finish(),
        }
    }
}

impl Clone for IntersectionResult {
    fn clone(&self) -> Self {
        match self {
            IntersectionResult::Point(p) => IntersectionResult::Point(*p),
            IntersectionResult::Curve(_) => IntersectionResult::None, // Cannot clone dyn Curve
            IntersectionResult::None => IntersectionResult::None,
        }
    }
}

/// Trait for geometric entities that can be tessellated
pub trait Tessellatable {
    /// Tessellate with given tolerance
    fn tessellate(&self, tolerance: f64) -> Tessellation;
}

/// Tessellation output
#[derive(Debug, Clone)]
pub enum Tessellation {
    /// Polyline for curves
    Polyline(Vec<Point3>),
    /// Triangle mesh for surfaces
    TriangleMesh {
        /// Vertices
        vertices: Vec<Point3>,
        /// Triangle indices (triplets)
        indices: Vec<u32>,
        /// Normals per vertex
        normals: Vec<Vec3>,
        /// UV coordinates per vertex
        uvs: Vec<(f64, f64)>,
    },
}

/// Parameter range for curves
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParamRange {
    /// Start parameter
    pub start: f64,
    /// End parameter
    pub end: f64,
}

impl ParamRange {
    /// Create a new parameter range
    pub fn new(start: f64, end: f64) -> Self {
        Self { start, end }
    }

    /// Check if parameter is within range
    pub fn contains(&self, t: f64) -> bool {
        t >= self.start && t <= self.end
    }

    /// Clamp parameter to range
    pub fn clamp(&self, t: f64) -> f64 {
        t.clamp(self.start, self.end)
    }

    /// Length of the parameter range
    pub fn length(&self) -> f64 {
        self.end - self.start
    }

    /// Check if range is valid (start <= end)
    pub fn is_valid(&self) -> bool {
        self.start <= self.end
    }

    /// Normalize parameter to [0, 1]
    pub fn normalize(&self, t: f64) -> f64 {
        (t - self.start) / self.length()
    }

    /// Denormalize parameter from [0, 1]
    pub fn denormalize(&self, t: f64) -> f64 {
        self.start + t * self.length()
    }
}

/// UV parameter range for surfaces
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UVRange {
    /// U parameter range
    pub u: ParamRange,
    /// V parameter range
    pub v: ParamRange,
}

impl UVRange {
    /// Create a new UV range
    pub fn new(u_start: f64, u_end: f64, v_start: f64, v_end: f64) -> Self {
        Self {
            u: ParamRange::new(u_start, u_end),
            v: ParamRange::new(v_start, v_end),
        }
    }

    /// Check if UV is within range
    pub fn contains(&self, u: f64, v: f64) -> bool {
        self.u.contains(u) && self.v.contains(v)
    }
}

/// Continuity types for curves and surfaces
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Continuity {
    /// Position only (C0)
    C0,
    /// Position + tangent (C1)
    C1,
    /// Position + tangent + curvature (C2)
    C2,
    /// Geometric continuity (G1)
    G1,
    /// Geometric continuity (G2)
    G2,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_range() {
        let range = ParamRange::new(0.0, 1.0);
        assert!(range.contains(0.5));
        assert!(!range.contains(1.5));
        assert_eq!(range.clamp(1.5), 1.0);
        assert_eq!(range.length(), 1.0);
    }

    #[test]
    fn test_uv_range() {
        let range = UVRange::new(0.0, 1.0, 0.0, 2.0);
        assert!(range.contains(0.5, 1.0));
        assert!(!range.contains(1.5, 1.0));
    }
}
