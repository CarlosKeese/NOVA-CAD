//! Split Operations - Divide bodies with surfaces or curves

use crate::{OpsError, OpsResult};
use nova_math::ToleranceContext;
use nova_topo::{Body, Face};

/// Split operation options
#[derive(Debug, Clone)]
pub struct SplitOptions {
    /// Keep both sides of the split
    pub keep_both: bool,
}

impl Default for SplitOptions {
    fn default() -> Self {
        Self { keep_both: false }
    }
}

/// Result of splitting a face
#[derive(Debug, Clone)]
pub struct FaceSplit {
    /// The split faces
    pub faces: Vec<Face>,
}

/// Split engine
#[derive(Debug, Clone)]
pub struct SplitEngine;

impl SplitEngine {
    /// Create new split engine
    pub fn new() -> Self {
        Self
    }
    
    /// Split body with surface (stub)
    pub fn split_with_surface(
        &self,
        _body: &Body,
        _surface: &dyn nova_geom::Surface,
        _options: &SplitOptions,
        _tolerance: &ToleranceContext,
    ) -> OpsResult<Vec<Body>> {
        Err(OpsError::NotSupported("Surface split not yet implemented".to_string()))
    }
    
    /// Split face at curves (stub)
    pub fn split_face_at_curves(
        &self,
        _face: &Face,
        _curves: &[Box<dyn nova_geom::Curve>],
        _tolerance: &ToleranceContext,
    ) -> OpsResult<Vec<Face>> {
        Err(OpsError::NotSupported("Face split not yet implemented".to_string()))
    }
}

impl Default for SplitEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Split a body with a surface (convenience function)
pub fn split_body_with_surface(
    _body: &Body,
    _surface: &dyn nova_geom::Surface,
    _tolerance: &ToleranceContext,
) -> OpsResult<Vec<Body>> {
    Err(OpsError::NotSupported("Body splitting not yet implemented".to_string()))
}
