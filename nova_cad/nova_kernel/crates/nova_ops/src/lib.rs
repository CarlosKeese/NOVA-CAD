//! Nova Ops - High-level Operations for Nova Kernel 3D
//!
//! Provides boolean operations, feature creation, splitting, and transformations.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use nova_math::{Point3, Vec3, ToleranceContext};
use nova_topo::Body;
use thiserror::Error;

pub mod boolean;
pub mod feature;
pub mod fillet;
pub mod split;
pub mod transform;

pub use boolean::{BooleanOp, BooleanEngine};
pub use feature::{ExtrudeOptions, RevolveOptions, SweepOptions, LoftOptions, FeatureEngine};
pub use fillet::{FilletOptions, ChamferOptions, FilletEngine};
pub use split::{SplitOptions, SplitEngine, FaceSplit};
pub use transform::{TransformEngine, TransformOptions};

/// Operations error types
#[derive(Debug, Error, Clone)]
pub enum OpsError {
    /// Invalid input bodies
    #[error("Invalid input bodies: {0}")]
    InvalidBodies(String),
    
    /// Invalid parameters
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    
    /// Geometry error
    #[error("Geometry error: {0}")]
    Geometry(String),
    
    /// Topology error
    #[error("Topology error: {0}")]
    Topology(String),
    
    /// Operation not supported
    #[error("Operation not supported: {0}")]
    NotSupported(String),
}

/// Result type for operations
pub type OpsResult<T> = Result<T, OpsError>;

/// Transform body (stub - not fully implemented)
pub fn transform_body(
    body: &Body,
    _transform: &nova_math::Transform3,
    _tolerance: &ToleranceContext,
) -> OpsResult<Body> {
    // TODO: Implement body transformation
    Err(OpsError::NotSupported("Body transformation not yet implemented".to_string()))
}

/// Create extruded body (stub - not fully implemented)
pub fn extrude(
    _profile: &Body,
    _direction: Vec3,
    _distance: f64,
    _tolerance: &ToleranceContext,
) -> OpsResult<Body> {
    Err(OpsError::NotSupported("Extrude not yet implemented".to_string()))
}

/// Create revolved body (stub - not fully implemented)
pub fn revolve(
    _profile: &Body,
    _axis_origin: Point3,
    _axis_direction: Vec3,
    _angle: f64,
    _tolerance: &ToleranceContext,
) -> OpsResult<Body> {
    Err(OpsError::NotSupported("Revolve not yet implemented".to_string()))
}

/// Loft between profiles (stub - not fully implemented)
pub fn loft(
    _profiles: &[Body],
    _tolerance: &ToleranceContext,
) -> OpsResult<Body> {
    Err(OpsError::NotSupported("Loft not yet implemented".to_string()))
}

/// Sweep profile along path (stub - not fully implemented)
pub fn sweep(
    _profile: &Body,
    _path: &dyn nova_geom::Curve,
    _tolerance: &ToleranceContext,
) -> OpsResult<Body> {
    Err(OpsError::NotSupported("Sweep not yet implemented".to_string()))
}

/// Boolean unite (stub - not fully implemented)
pub fn boolean_unite(
    _body1: &Body,
    _body2: &Body,
    _tolerance: &ToleranceContext,
) -> OpsResult<Body> {
    Err(OpsError::NotSupported("Boolean unite not yet implemented".to_string()))
}

/// Boolean subtract (stub - not fully implemented)
pub fn boolean_subtract(
    _body1: &Body,
    _body2: &Body,
    _tolerance: &ToleranceContext,
) -> OpsResult<Body> {
    Err(OpsError::NotSupported("Boolean subtract not yet implemented".to_string()))
}

/// Boolean intersect (stub - not fully implemented)
pub fn boolean_intersect(
    _body1: &Body,
    _body2: &Body,
    _tolerance: &ToleranceContext,
) -> OpsResult<Body> {
    Err(OpsError::NotSupported("Boolean intersect not yet implemented".to_string()))
}

/// Apply fillet to edges (stub - not fully implemented)
pub fn fillet_edges(
    _body: &Body,
    _edges: &[&nova_topo::Edge],
    _radius: f64,
    _tolerance: &ToleranceContext,
) -> OpsResult<Body> {
    Err(OpsError::NotSupported("Fillet not yet implemented".to_string()))
}

/// Apply chamfer to edges (stub - not fully implemented)
pub fn chamfer_edges(
    _body: &Body,
    _edges: &[&nova_topo::Edge],
    _distance: f64,
    _tolerance: &ToleranceContext,
) -> OpsResult<Body> {
    Err(OpsError::NotSupported("Chamfer not yet implemented".to_string()))
}

/// Split body with surface (stub - not fully implemented)
pub fn split_body(
    _body: &Body,
    _surface: &dyn nova_geom::Surface,
    _tolerance: &ToleranceContext,
) -> OpsResult<Vec<Body>> {
    Err(OpsError::NotSupported("Split not yet implemented".to_string()))
}
