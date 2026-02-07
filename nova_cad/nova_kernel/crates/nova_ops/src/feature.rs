//! Feature Operations - Extrude, Revolve, Sweep, Loft
//!
//! Implements solid modeling features for creating 3D geometry
//! from 2D profiles and paths.

use crate::{OpsError, OpsResult};
use nova_math::{Point3, Vec3, Transform3, ToleranceContext, Plane};
use nova_geom::{Curve, Surface, PlanarSurface, CylindricalSurface, SurfaceEvaluation};
use nova_topo::{Body, Face, Edge, Loop, Vertex, EulerOps, Orientation, Sense, TopologyError};
use std::collections::HashMap;

/// Feature operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureOp {
    /// Extrude a profile along a direction
    Extrude,
    /// Revolve a profile around an axis
    Revolve,
    /// Sweep a profile along a path
    Sweep,
    /// Loft between multiple profiles
    Loft,
}

impl FeatureOp {
    /// Get feature name
    pub fn name(&self) -> &'static str {
        match self {
            FeatureOp::Extrude => "extrude",
            FeatureOp::Revolve => "revolve",
            FeatureOp::Sweep => "sweep",
            FeatureOp::Loft => "loft",
        }
    }
}

/// Options for extrude operation
#[derive(Debug, Clone)]
pub struct ExtrudeOptions {
    /// Direction of extrusion
    pub direction: Vec3,
    /// Distance to extrude (can be negative for opposite direction)
    pub distance: f64,
    /// Whether to create a symmetric extrusion
    pub symmetric: bool,
    /// Draft angle in degrees (0 = no draft)
    pub draft_angle: f64,
    /// Whether operation should add or remove material
    pub operation: FeatureOperationType,
}

impl ExtrudeOptions {
    /// Create default extrude options
    pub fn new(direction: Vec3, distance: f64) -> Self {
        Self {
            direction: direction.normalized(),
            distance,
            symmetric: false,
            draft_angle: 0.0,
            operation: FeatureOperationType::Add,
        }
    }
    
    /// Set symmetric extrusion
    pub fn symmetric(mut self) -> Self {
        self.symmetric = true;
        self
    }
    
    /// Set draft angle
    pub fn with_draft(mut self, angle_degrees: f64) -> Self {
        self.draft_angle = angle_degrees;
        self
    }
    
    /// Set operation type
    pub fn with_operation(mut self, op: FeatureOperationType) -> Self {
        self.operation = op;
        self
    }
}

impl Default for ExtrudeOptions {
    fn default() -> Self {
        Self {
            direction: Vec3::new(0.0, 0.0, 1.0),
            distance: 1.0,
            symmetric: false,
            draft_angle: 0.0,
            operation: FeatureOperationType::Add,
        }
    }
}

/// Options for revolve operation
#[derive(Debug, Clone)]
pub struct RevolveOptions {
    /// Axis origin point
    pub axis_origin: Point3,
    /// Axis direction
    pub axis_direction: Vec3,
    /// Start angle in degrees
    pub start_angle: f64,
    /// End angle in degrees
    pub end_angle: f64,
    /// Whether operation should add or remove material
    pub operation: FeatureOperationType,
}

impl RevolveOptions {
    /// Create default revolve options with full 360 degrees
    pub fn new(axis_origin: Point3, axis_direction: Vec3) -> Self {
        Self {
            axis_origin,
            axis_direction: axis_direction.normalized(),
            start_angle: 0.0,
            end_angle: 360.0,
            operation: FeatureOperationType::Add,
        }
    }
    
    /// Set angle range
    pub fn with_angles(mut self, start: f64, end: f64) -> Self {
        self.start_angle = start;
        self.end_angle = end;
        self
    }
    
    /// Set operation type
    pub fn with_operation(mut self, op: FeatureOperationType) -> Self {
        self.operation = op;
        self
    }
}

/// Options for sweep operation
#[derive(Debug, Clone)]
pub struct SweepOptions {
    /// Path curve to sweep along
    pub path: Box<dyn Curve>,
    /// Whether to keep profile orientation constant
    pub keep_orientation: bool,
    /// Twist angle along path (degrees per unit length)
    pub twist: f64,
    /// Scale factor along path
    pub scale: f64,
    /// Whether operation should add or remove material
    pub operation: FeatureOperationType,
}

impl SweepOptions {
    /// Create sweep options with path
    pub fn new(path: Box<dyn Curve>) -> Self {
        Self {
            path,
            keep_orientation: false,
            twist: 0.0,
            scale: 1.0,
            operation: FeatureOperationType::Add,
        }
    }
    
    /// Set operation type
    pub fn with_operation(mut self, op: FeatureOperationType) -> Self {
        self.operation = op;
        self
    }
}

/// Type of feature operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureOperationType {
    /// Add material (base feature)
    Add,
    /// Remove material (cut)
    Remove,
    /// Intersect with existing material
    Intersect,
}

/// Feature engine for creating solid features
#[derive(Debug, Clone)]
pub struct FeatureEngine;

impl FeatureEngine {
    /// Create a new feature engine
    pub fn new() -> Self {
        Self
    }
    
    /// Extrude a face/profile to create a solid
    pub fn extrude(
        &self,
        profile: &Face,
        options: &ExtrudeOptions,
        tolerance: &ToleranceContext,
    ) -> OpsResult<Body> {
        if options.distance.abs() < tolerance.tolerance() {
            return Err(OpsError::InvalidParameters(
                "Extrude distance too small".to_string()
            ));
        }
        
        let mut euler = EulerOps::new();
        
        // Get profile edges
        let profile_edges = self.get_face_edges(profile);
        if profile_edges.is_empty() {
            return Err(OpsError::InvalidBodies(
                "Profile face has no edges".to_string()
            ));
        }
        
        // Calculate extrusion vector
        let extrude_vec = options.direction * options.distance;
        
        // Create bottom face (profile)
        let bottom_face = profile.clone();
        
        // Create side faces by extruding each edge
        let mut side_faces = Vec::new();
        for edge in &profile_edges {
            let side_face = self.create_extruded_face(
                edge,
                &extrude_vec,
                &mut euler,
                tolerance
            )?;
            side_faces.push(side_face);
        }
        
        // Create top face (translated profile)
        let top_face = self.create_translated_face(profile, &extrude_vec, &mut euler)?;
        
        // Build the solid body
        let body = self.build_solid_from_faces(
            bottom_face,
            top_face,
            side_faces,
            &mut euler
        )?;
        
        Ok(body)
    }
    
    /// Revolve a face/profile around an axis
    pub fn revolve(
        &self,
        profile: &Face,
        options: &RevolveOptions,
        tolerance: &ToleranceContext,
    ) -> OpsResult<Body> {
        let angle_range = options.end_angle - options.start_angle;
        
        if angle_range.abs() < 1e-6 {
            return Err(OpsError::InvalidParameters(
                "Revolve angle too small".to_string()
            ));
        }
        
        // Normalize angles
        let start_rad = options.start_angle.to_radians();
        let end_rad = options.end_angle.to_radians();
        
        let mut euler = EulerOps::new();
        
        // Get profile edges
        let profile_edges = self.get_face_edges(profile);
        
        // Determine number of segments based on angle and tolerance
        let num_segments = self.calculate_revolve_segments(angle_range, tolerance);
        let angle_step = (end_rad - start_rad) / num_segments as f64;
        
        // Create revolved side faces
        let mut side_faces = Vec::new();
        
        for edge in profile_edges {
            for i in 0..num_segments {
                let angle1 = start_rad + i as f64 * angle_step;
                let angle2 = start_rad + (i + 1) as f64 * angle_step;
                
                let side_face = self.create_revolved_face(
                    &edge,
                    options.axis_origin,
                    options.axis_direction,
                    angle1,
                    angle2,
                    &mut euler,
                    tolerance
                )?;
                side_faces.push(side_face);
            }
        }
        
        // Create start and end cap faces if not full 360
        let mut cap_faces = Vec::new();
        if (angle_range - 360.0).abs() > 1e-6 {
            // Start cap
            let start_cap = self.create_revolve_cap(
                profile,
                options.axis_origin,
                options.axis_direction,
                start_rad,
                &mut euler
            )?;
            cap_faces.push(start_cap);
            
            // End cap
            let end_cap = self.create_revolve_cap(
                profile,
                options.axis_origin,
                options.axis_direction,
                end_rad,
                &mut euler
            )?;
            cap_faces.push(end_cap);
        }
        
        // Build solid body
        // TODO: Implement proper body construction from faces
        
        Err(OpsError::NotSupported(
            "Revolve body construction not yet fully implemented".to_string()
        ))
    }
    
    /// Sweep a face/profile along a path
    pub fn sweep(
        &self,
        profile: &Face,
        options: &SweepOptions,
        tolerance: &ToleranceContext,
    ) -> OpsResult<Body> {
        // TODO: Implement sweep operation
        Err(OpsError::NotSupported(
            "Sweep operation not yet implemented".to_string()
        ))
    }
    
    /// Loft between multiple profiles
    pub fn loft(
        &self,
        profiles: &[Face],
        options: &LoftOptions,
        tolerance: &ToleranceContext,
    ) -> OpsResult<Body> {
        if profiles.len() < 2 {
            return Err(OpsError::InvalidParameters(
                "Loft requires at least 2 profiles".to_string()
            ));
        }
        
        // TODO: Implement loft operation
        Err(OpsError::NotSupported(
            "Loft operation not yet implemented".to_string()
        ))
    }
    
    /// Get ordered edges from a face
    fn get_face_edges(&self, face: &Face) -> Vec<Edge> {
        let mut edges = Vec::new();
        let mut visited = std::collections::HashSet::new();
        
        for loop_ in face.loops() {
            for coedge in loop_.coedges() {
                let edge = coedge.edge();
                if visited.insert(edge.id()) {
                    edges.push(edge.clone());
                }
            }
        }
        
        edges
    }
    
    /// Create an extruded face from an edge
    fn create_extruded_face(
        &self,
        edge: &Edge,
        extrude_vec: &Vec3,
        euler: &mut EulerOps,
        tolerance: &ToleranceContext,
    ) -> OpsResult<Face> {
        use nova_geom::Line;
        
        let curve = edge.curve();
        let start = curve.evaluate(curve.parameter_range().start);
        let end = curve.evaluate(curve.parameter_range().end);
        
        // Create extruded surface (ruled surface)
        let bottom_edge = edge.clone();
        let top_start = start.point + *extrude_vec;
        let top_end = end.point + *extrude_vec;
        
        // TODO: Create proper ruled surface
        // For now, create a planar approximation if edges are straight
        
        let surface = PlanarSurface::new(
            Plane::from_three_points(start.point, end.point, top_start)
                .unwrap_or_else(|_| Plane::from_normal(start.point, Vec3::new(0.0, 0.0, 1.0)))
        );
        
        Err(OpsError::NotSupported(
            "Extruded face creation not yet fully implemented".to_string()
        ))
    }
    
    /// Create a translated copy of a face
    fn create_translated_face(
        &self,
        face: &Face,
        translation: &Vec3,
        euler: &mut EulerOps,
    ) -> OpsResult<Face> {
        // TODO: Implement face translation
        // This requires:
        // 1. Transform surface
        // 2. Transform all edges
        // 3. Create new topology
        
        Err(OpsError::NotSupported(
            "Face translation not yet implemented".to_string()
        ))
    }
    
    /// Build a solid body from faces
    fn build_solid_from_faces(
        &self,
        bottom: Face,
        top: Face,
        sides: Vec<Face>,
        euler: &mut EulerOps,
    ) -> OpsResult<Body> {
        // TODO: Implement solid construction using Euler operators
        Err(OpsError::NotSupported(
            "Solid construction not yet implemented".to_string()
        ))
    }
    
    /// Calculate number of segments for revolve
    fn calculate_revolve_segments(&self, angle_degrees: f64, tolerance: &ToleranceContext) -> usize {
        // Use chordal tolerance to determine segments
        // For a circle of radius 1, chordal error = r * (1 - cos(θ/2))
        // Solving for θ: θ = 2 * acos(1 - tolerance/r)
        
        let min_segments = 4; // At least 4 segments
        let max_segments = 128; // At most 128 segments
        
        // Simplified: use 8 segments per 90 degrees
        let segments = ((angle_degrees.abs() / 90.0) * 8.0) as usize;
        
        segments.clamp(min_segments, max_segments)
    }
    
    /// Create a revolved face from an edge
    fn create_revolved_face(
        &self,
        edge: &Edge,
        axis_origin: Point3,
        axis_direction: Vec3,
        angle1: f64,
        angle2: f64,
        euler: &mut EulerOps,
        tolerance: &ToleranceContext,
    ) -> OpsResult<Face> {
        // TODO: Create cylindrical or conical surface from edge
        Err(OpsError::NotSupported(
            "Revolved face creation not yet implemented".to_string()
        ))
    }
    
    /// Create a cap face for revolve
    fn create_revolve_cap(
        &self,
        profile: &Face,
        axis_origin: Point3,
        axis_direction: Vec3,
        angle: f64,
        euler: &mut EulerOps,
    ) -> OpsResult<Face> {
        // TODO: Create rotated copy of profile
        Err(OpsError::NotSupported(
            "Revolve cap creation not yet implemented".to_string()
        ))
    }
}

impl Default for FeatureEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Options for loft operation
#[derive(Debug, Clone)]
pub struct LoftOptions {
    /// Whether to maintain continuity at start
    pub start_continuity: ContinuityType,
    /// Whether to maintain continuity at end
    pub end_continuity: ContinuityType,
    /// Whether operation should add or remove material
    pub operation: FeatureOperationType,
    /// Guide curves for controlling shape
    pub guide_curves: Vec<Box<dyn Curve>>,
}

impl LoftOptions {
    /// Create default loft options
    pub fn new() -> Self {
        Self {
            start_continuity: ContinuityType::Position,
            end_continuity: ContinuityType::Position,
            operation: FeatureOperationType::Add,
            guide_curves: Vec::new(),
        }
    }
}

impl Default for LoftOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Continuity types for loft
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContinuityType {
    /// Position only (C0)
    Position,
    /// Tangent continuity (C1)
    Tangent,
    /// Curvature continuity (C2)
    Curvature,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_op_enum() {
        assert_eq!(FeatureOp::Extrude.name(), "extrude");
        assert_eq!(FeatureOp::Revolve.name(), "revolve");
        assert_eq!(FeatureOp::Sweep.name(), "sweep");
        assert_eq!(FeatureOp::Loft.name(), "loft");
    }

    #[test]
    fn test_extrude_options() {
        let opts = ExtrudeOptions::new(Vec3::new(0.0, 0.0, 1.0), 10.0)
            .symmetric()
            .with_draft(5.0);
        
        assert_eq!(opts.distance, 10.0);
        assert!(opts.symmetric);
        assert_eq!(opts.draft_angle, 5.0);
    }

    #[test]
    fn test_revolve_options() {
        let opts = RevolveOptions::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0)
        ).with_angles(0.0, 180.0);
        
        assert_eq!(opts.start_angle, 0.0);
        assert_eq!(opts.end_angle, 180.0);
    }

    #[test]
    fn test_feature_operation_type() {
        assert!(matches!(FeatureOperationType::Add, FeatureOperationType::Add));
        assert!(matches!(FeatureOperationType::Remove, FeatureOperationType::Remove));
    }

    #[test]
    fn test_continuity_type() {
        assert!(matches!(ContinuityType::Position, ContinuityType::Position));
        assert!(matches!(ContinuityType::Tangent, ContinuityType::Tangent));
        assert!(matches!(ContinuityType::Curvature, ContinuityType::Curvature));
    }
}
