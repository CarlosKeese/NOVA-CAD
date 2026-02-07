//! Nova Ops - Boolean Operations and Features for Nova Kernel 3D
//!
//! Provides high-level CAD operations:
//! - Boolean operations: unite, subtract, intersect
//! - Features: extrude, revolve, sweep, loft
//! - Fillets and chamfers
//! - Shell operations

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use nova_math::{Point3, Vec3, Transform3, ToleranceContext};
use nova_geom::{Curve, Surface, GeometryError};
use nova_topo::{Body, TopologyError};

pub mod boolean;
pub mod feature;
pub mod fillet;
pub mod error;
pub mod split;

pub use boolean::{BooleanOp, BooleanEngine};
pub use feature::{FeatureOp, ExtrudeOptions, RevolveOptions, SweepOptions};
pub use fillet::{FilletOp, ChamferOp, FilletOptions, ChamferOptions};
pub use error::{OpsError, OpsResult};

/// Operation context for tracking history and dependencies
#[derive(Debug, Clone)]
pub struct OpContext {
    /// Operation ID
    pub id: u64,
    /// Operation type
    pub op_type: OpType,
    /// Parent operation IDs (dependencies)
    pub parents: Vec<u64>,
    /// Tolerance used
    pub tolerance: f64,
}

/// Operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpType {
    /// Boolean unite
    Unite,
    /// Boolean subtract
    Subtract,
    /// Boolean intersect
    Intersect,
    /// Extrude feature
    Extrude,
    /// Revolve feature
    Revolve,
    /// Sweep feature
    Sweep,
    /// Loft feature
    Loft,
    /// Fillet
    Fillet,
    /// Chamfer
    Chamfer,
    /// Shell
    Shell,
    /// Draft
    Draft,
    /// Mirror
    Mirror,
    /// Pattern
    Pattern,
}

/// Trait for operations that can be applied to bodies
pub trait BodyOperation {
    /// Apply the operation to a body or bodies
    fn apply(&self, bodies: &mut [Body], context: &ToleranceContext) -> OpsResult<()>;
    
    /// Get the operation type
    fn op_type(&self) -> OpType;
    
    /// Check if operation is valid for given bodies
    fn validate(&self, bodies: &[Body]) -> OpsResult<()>;
}

/// Engine for CAD operations
pub struct OpsEngine {
    /// Boolean operation engine
    pub boolean: BooleanEngine,
    /// Next operation ID
    next_op_id: std::sync::atomic::AtomicU64,
}

impl OpsEngine {
    /// Create a new operations engine
    pub fn new() -> Self {
        Self {
            boolean: BooleanEngine::new(),
            next_op_id: std::sync::atomic::AtomicU64::new(1),
        }
    }
    
    /// Generate a new operation ID
    pub fn new_op_id(&self) -> u64 {
        self.next_op_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
    
    /// Perform boolean unite
    pub fn unite(&self, body1: &Body, body2: &Body, tolerance: &ToleranceContext) -> OpsResult<Body> {
        self.boolean.unite(body1, body2, tolerance)
    }
    
    /// Perform boolean subtract
    pub fn subtract(&self, body1: &Body, body2: &Body, tolerance: &ToleranceContext) -> OpsResult<Body> {
        self.boolean.subtract(body1, body2, tolerance)
    }
    
    /// Perform boolean intersect
    pub fn intersect(&self, body1: &Body, body2: &Body, tolerance: &ToleranceContext) -> OpsResult<Body> {
        self.boolean.intersect(body1, body2, tolerance)
    }
}

impl Default for OpsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nova_topo::{EulerOps, Face};
    use nova_geom::PlanarSurface;
    use nova_math::{Plane, Point2};

    fn create_test_face() -> Face {
        let mut euler = EulerOps::new();
        let surface = PlanarSurface::new(
            Plane::from_normal(
                Point3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
            )
        );
        
        // Create a simple square face using Euler operators
        // MVFS - Make Vertex Face Shell
        let (v1, face, shell, body) = euler.make_vertex_face_shell(
            Point3::new(0.0, 0.0, 0.0),
            Box::new(surface)
        ).unwrap();
        
        face
    }

    #[test]
    fn test_ops_engine_creation() {
        let engine = OpsEngine::new();
        let id1 = engine.new_op_id();
        let id2 = engine.new_op_id();
        assert_eq!(id2, id1 + 1);
    }

    #[test]
    fn test_op_type_enum() {
        assert_eq!(OpType::Unite as u8, 0);
        assert_eq!(OpType::Extrude as u8, 3);
        assert_eq!(OpType::Fillet as u8, 7);
    }
}
