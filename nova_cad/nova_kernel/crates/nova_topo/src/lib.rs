//! Nova Topo - B-Rep Topology Engine for Nova Kernel 3D
//! 
//! Provides boundary representation topology:
//! - Vertex, Edge, Coedge, Loop, Face, Shell, Body
//! - Euler operators for topology manipulation
//! - Persistent entity IDs

#![warn(missing_docs)]

use nova_math::{Point3, Vec3, Transform3, BoundingBox3, ToleranceContext};
use nova_geom::{Curve, Surface, GeometryError};
use std::sync::atomic::{AtomicU64, Ordering};

mod entity;
mod body;
mod euler;
mod euler_advanced;

pub use entity::{EntityId, Entity, TopologicalEntity, GeometricEntity};
pub use body::{Body, Shell, Face, Loop, Coedge, Edge, Vertex};
pub use euler::{EulerOps, EulerError};
pub use euler_advanced::EulerAdvanced;

/// Global entity ID counter
static ENTITY_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generate a new unique entity ID
pub fn new_entity_id() -> EntityId {
    EntityId(ENTITY_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
}

/// Topology-related errors
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum TopologyError {
    /// Invalid entity reference
    #[error("Invalid entity reference: {0}")]
    InvalidReference(String),
    
    /// Topology inconsistency
    #[error("Topology inconsistency: {0}")]
    Inconsistency(String),
    
    /// Euler operator failed
    #[error("Euler operator failed: {0}")]
    EulerFailed(String),
    
    /// Geometry error
    #[error("Geometry error: {0}")]
    Geometry(#[from] GeometryError),
}

/// Result type for topology operations
pub type TopoResult<T> = Result<T, TopologyError>;

/// Orientation of a topological entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Orientation {
    /// Forward orientation
    Forward,
    /// Reversed orientation
    Reversed,
}

impl Orientation {
    /// Check if forward
    pub fn is_forward(&self) -> bool {
        matches!(self, Orientation::Forward)
    }
    
    /// Check if reversed
    pub fn is_reversed(&self) -> bool {
        matches!(self, Orientation::Reversed)
    }
    
    /// Reverse the orientation
    pub fn reverse(&self) -> Self {
        match self {
            Orientation::Forward => Orientation::Reversed,
            Orientation::Reversed => Orientation::Forward,
        }
    }
    
    /// Apply orientation to a value (multiply by +1 or -1)
    pub fn apply(&self, value: f64) -> f64 {
        match self {
            Orientation::Forward => value,
            Orientation::Reversed => -value,
        }
    }
}

impl Default for Orientation {
    fn default() -> Self {
        Orientation::Forward
    }
}

/// Sense flag for coedges (direction within a loop)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sense {
    /// Same direction as edge
    Same,
    /// Opposite direction to edge
    Opposite,
}

impl Sense {
    /// Check if same as edge
    pub fn is_same(&self) -> bool {
        matches!(self, Sense::Same)
    }
    
    /// Check if opposite to edge
    pub fn is_opposite(&self) -> bool {
        matches!(self, Sense::Opposite)
    }
    
    /// Reverse the sense
    pub fn reverse(&self) -> Self {
        match self {
            Sense::Same => Sense::Opposite,
            Sense::Opposite => Sense::Same,
        }
    }
}

impl Default for Sense {
    fn default() -> Self {
        Sense::Same
    }
}

/// Check if a body is valid (manifold, closed, consistent)
pub fn validate_body(body: &Body, tolerance: &ToleranceContext) -> TopoResult<Vec<String>> {
    let mut issues = Vec::new();
    
    // Check Euler-Poincaré formula
    let v = body.vertices().len();
    let e = body.edges().len();
    let f = body.faces().len();
    let s = body.shells().len();
    
    // For a solid body: V - E + F = 2 (for each shell)
    let euler = v as i32 - e as i32 + f as i32;
    if s > 0 && euler != 2 * s as i32 {
        issues.push(format!(
            "Euler characteristic mismatch: V-E+F={}, expected {} for {} shell(s)",
            euler, 2 * s, s
        ));
    }
    
    // Check that all edges have two coedges (manifold)
    for edge in body.edges() {
        let coedge_count = edge.coedges().len();
        if coedge_count != 2 {
            issues.push(format!(
                "Edge {:?} has {} coedges (expected 2 for manifold)",
                edge.id(), coedge_count
            ));
        }
    }
    
    // Check that all faces have valid loops
    for face in body.faces() {
        if face.loops().is_empty() {
            issues.push(format!("Face {:?} has no loops", face.id()));
        }
    }
    
    // Check vertex-edge consistency
    for vertex in body.vertices() {
        if vertex.edges().is_empty() {
            issues.push(format!("Vertex {:?} has no edges", vertex.id()));
        }
    }
    
    Ok(issues)
}

/// Compute bounding box of a body
pub fn body_bounding_box(body: &Body) -> BoundingBox3 {
    let mut bbox = BoundingBox3::EMPTY;
    
    for vertex in body.vertices() {
        bbox.expand(&vertex.position());
    }
    
    bbox
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_id() {
        let id1 = new_entity_id();
        let id2 = new_entity_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_orientation() {
        let fwd = Orientation::Forward;
        let rev = Orientation::Reversed;
        
        assert!(fwd.is_forward());
        assert!(rev.is_reversed());
        assert_eq!(fwd.reverse(), rev);
        assert_eq!(fwd.apply(5.0), 5.0);
        assert_eq!(rev.apply(5.0), -5.0);
    }

    #[test]
    fn test_sense() {
        let same = Sense::Same;
        let opp = Sense::Opposite;
        
        assert!(same.is_same());
        assert!(opp.is_opposite());
        assert_eq!(same.reverse(), opp);
    }
}
