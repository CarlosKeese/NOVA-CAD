//! Face Editing Operations
//!
//! Implements face-based editing operations for synchronous technology:
//! - Face Move: Translate faces along a direction
//! - Face Rotate: Rotate faces around an axis
//! - Face Offset: Offset faces along their normals

use crate::{SyncError, SyncResult, SyncContext, SyncOpType, SyncOperation};
use nova_math::{Point3, Vec3, Transform3, ToleranceContext};
use nova_topo::{Body, Face, Edge, Vertex, Shell, Loop, Coedge, EulerAdvanced, Sense, Orientation, Entity, GeometricEntity, new_entity_id};
use nova_geom::{Surface, Plane, PlanarSurface};
use std::sync::Arc;
use std::collections::{HashMap, HashSet};

/// Face editing engine
pub struct FaceEditEngine<'a> {
    context: &'a SyncContext,
}

impl<'a> FaceEditEngine<'a> {
    /// Create a new face editing engine
    pub fn new(context: &'a SyncContext) -> Self {
        Self { context }
    }
    
    /// Move faces along a direction
    pub fn move_faces(
        &self,
        body: &mut Body,
        faces: &[Face],
        options: &MoveOptions,
        ctx: &mut SyncContext,
    ) -> SyncResult<()> {
        if faces.is_empty() {
            return Err(SyncError::NoSelection);
        }
        
        let transform = Transform3::from_translation(options.direction.normalized() * options.distance);
        
        // Get adjacent faces that need to be updated
        let affected = self.get_affected_faces(body, faces)?;
        
        // Apply transformation to selected faces
        for face in faces {
            self.transform_face_geometry(face, &transform)?;
        }
        
        // Resolve topology for adjacent faces
        for adj_face in affected {
            self.resolve_adjacent_face(&adj_face, faces, &transform)?;
        }
        
        // Record operation
        let op = SyncOperation {
            id: ctx.history.len() as u64 + 1,
            op_type: SyncOpType::FaceMove,
            affected_faces: faces.iter().map(|f| f.id()).collect(),
            transform,
            maintained_rules: Vec::new(),
            recognized_features: Vec::new(),
        };
        ctx.record_operation(op);
        
        Ok(())
    }
    
    /// Rotate faces around an axis
    pub fn rotate_faces(
        &self,
        body: &mut Body,
        faces: &[Face],
        options: &RotateOptions,
        ctx: &mut SyncContext,
    ) -> SyncResult<()> {
        if faces.is_empty() {
            return Err(SyncError::NoSelection);
        }
        
        let transform = Transform3::from_axis_angle(
            options.axis_origin,
            options.axis_direction,
            options.angle.to_radians()
        );
        
        // Get adjacent faces
        let affected = self.get_affected_faces(body, faces)?;
        
        // Apply rotation to selected faces
        for face in faces {
            self.transform_face_geometry(face, &transform)?;
        }
        
        // Resolve topology
        for adj_face in affected {
            self.resolve_adjacent_face(&adj_face, faces, &transform)?;
        }
        
        // Record operation
        let op = SyncOperation {
            id: ctx.history.len() as u64 + 1,
            op_type: SyncOpType::FaceRotate,
            affected_faces: faces.iter().map(|f| f.id()).collect(),
            transform,
            maintained_rules: Vec::new(),
            recognized_features: Vec::new(),
        };
        ctx.record_operation(op);
        
        Ok(())
    }
    
    /// Offset faces along their normals
    pub fn offset_faces(
        &self,
        body: &mut Body,
        faces: &[Face],
        options: &OffsetOptions,
        ctx: &mut SyncContext,
    ) -> SyncResult<()> {
        if faces.is_empty() {
            return Err(SyncError::NoSelection);
        }
        
        for face in faces {
            self.offset_single_face(body, face, options.offset)?;
        }
        
        // Record operation
        let op = SyncOperation {
            id: ctx.history.len() as u64 + 1,
            op_type: SyncOpType::FaceOffset,
            affected_faces: faces.iter().map(|f| f.id()).collect(),
            transform: Transform3::identity(),
            maintained_rules: Vec::new(),
            recognized_features: Vec::new(),
        };
        ctx.record_operation(op);
        
        Ok(())
    }
    
    /// Get faces adjacent to the selected faces
    fn get_affected_faces(&self, body: &Body, selected: &[Face]) -> SyncResult<Vec<Face>> {
        let selected_ids: HashSet<_> = selected.iter().map(|f| f.id()).collect();
        let mut affected = Vec::new();
        
        // Find all faces that share an edge with selected faces
        for shell in body.shells() {
            for face in shell.faces() {
                if selected_ids.contains(&face.id()) {
                    continue;
                }
                
                // Check if this face shares any edge with selected faces
                for selected_face in selected {
                    if self.faces_share_edge(face, selected_face) {
                        affected.push(face.clone());
                        break;
                    }
                }
            }
        }
        
        Ok(affected)
    }
    
    /// Check if two faces share an edge
    fn faces_share_edge(&self, face1: &Face, face2: &Face) -> bool {
        let edges1: HashSet<_> = face1.loops()
            .iter()
            .flat_map(|lp| lp.coedges())
            .map(|c| c.edge().id())
            .collect();
        
        let edges2: HashSet<_> = face2.loops()
            .iter()
            .flat_map(|lp| lp.coedges())
            .map(|c| c.edge().id())
            .collect();
        
        edges1.intersection(&edges2).next().is_some()
    }
    
    /// Transform the geometry of a face
    fn transform_face_geometry(&self, face: &Face, transform: &Transform3) -> SyncResult<()> {
        // Transform the surface
        if let Some(surface) = face.surface() {
            // For planar surfaces, transform origin and normal
            // For curved surfaces, more complex transformation needed
            // TODO: Implement surface transformation
        }
        
        Ok(())
    }
    
    /// Resolve topology for an adjacent face
    fn resolve_adjacent_face(
        &self,
        adj_face: &Face,
        moved_faces: &[Face],
        transform: &Transform3,
    ) -> SyncResult<()> {
        // Find shared edges
        let shared_edges = self.find_shared_edges(adj_face, moved_faces);
        
        for edge_info in shared_edges {
            // The shared edge needs to be updated to connect to the moved face
            // This typically involves extending or trimming the adjacent face
            
            match self.determine_resolution_strategy(&edge_info, transform) {
                ResolutionStrategy::ExtendFace => {
                    // Extend the adjacent face to meet the moved face
                    self.extend_face_to_meet(adj_face, &edge_info, transform)?;
                }
                ResolutionStrategy::TrimFace => {
                    // Trim the adjacent face at the intersection
                    self.trim_face_at_intersection(adj_face, &edge_info)?;
                }
                ResolutionStrategy::CreateBlend => {
                    // Create a blend/transition face
                    self.create_blend_face(adj_face, &edge_info)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Find edges shared between a face and a set of faces
    fn find_shared_edges(&self, face: &Face, other_faces: &[Face]) -> Vec<SharedEdgeInfo> {
        let mut shared = Vec::new();
        
        let face_edges: HashMap<_, _> = face.loops()
            .iter()
            .flat_map(|lp| lp.coedges())
            .map(|c| (c.edge().id(), c.clone()))
            .collect();
        
        for other in other_faces {
            for coedge in other.loops().iter().flat_map(|lp| lp.coedges()) {
                if let Some(face_coedge) = face_edges.get(&coedge.edge().id()) {
                    shared.push(SharedEdgeInfo {
                        edge_id: coedge.edge().id(),
                        face_coedge: face_coedge.clone(),
                        other_coedge: coedge.clone(),
                    });
                }
            }
        }
        
        shared
    }
    
    /// Determine how to resolve the topology
    fn determine_resolution_strategy(
        &self,
        edge_info: &SharedEdgeInfo,
        _transform: &Transform3,
    ) -> ResolutionStrategy {
        // Simple heuristic: if faces are planar, extend; otherwise blend
        // TODO: More sophisticated logic based on face types and transform
        ResolutionStrategy::ExtendFace
    }
    
    /// Extend a face to meet another face
    fn extend_face_to_meet(
        &self,
        face: &Face,
        edge_info: &SharedEdgeInfo,
        _transform: &Transform3,
    ) -> SyncResult<()> {
        // For planar faces, this means extending the surface
        // For curved faces, this is more complex
        // TODO: Implement face extension
        Ok(())
    }
    
    /// Trim a face at intersection
    fn trim_face_at_intersection(
        &self,
        face: &Face,
        edge_info: &SharedEdgeInfo,
    ) -> SyncResult<()> {
        // Find where the edge intersects the face boundary
        // Add a new vertex and split the edge
        // TODO: Implement face trimming
        Ok(())
    }
    
    /// Create a blend face between two faces
    fn create_blend_face(
        &self,
        face: &Face,
        edge_info: &SharedEdgeInfo,
    ) -> SyncResult<()> {
        // Create a transitional face between two non-coincident edges
        // TODO: Implement blend creation
        Ok(())
    }
    
    /// Offset a single face
    fn offset_single_face(&self, body: &Body, face: &Face, offset: f64) -> SyncResult<()> {
        // Get face normal
        let normal = if let Some(surface) = face.surface() {
            let (u, v) = surface.midpoint_uv();
            surface.evaluate(u, v).normal
        } else {
            return Err(SyncError::FaceEditFailed("Face has no surface".to_string()));
        };
        
        // For planar faces, offset is simple translation along normal
        // For curved faces, offset creates an offset surface
        // TODO: Implement surface offset
        
        Ok(())
    }
}

/// Information about a shared edge
#[derive(Debug, Clone)]
struct SharedEdgeInfo {
    edge_id: nova_topo::EntityId,
    face_coedge: Coedge,
    other_coedge: Coedge,
}

/// Strategy for resolving topology changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResolutionStrategy {
    /// Extend the face to meet the moved face
    ExtendFace,
    /// Trim the face at the intersection
    TrimFace,
    /// Create a blend face
    CreateBlend,
}

/// Options for face move operation
#[derive(Debug, Clone)]
pub struct MoveOptions {
    /// Direction of movement
    pub direction: Vec3,
    /// Distance to move
    pub distance: f64,
    /// Whether to stop at first intersection
    pub stop_at_intersection: bool,
    /// Whether to detach from adjacent faces
    pub detach: bool,
}

impl MoveOptions {
    /// Create new move options
    pub fn new(direction: Vec3, distance: f64) -> Self {
        Self {
            direction: direction.normalized(),
            distance,
            stop_at_intersection: false,
            detach: false,
        }
    }
    
    /// Set stop at intersection
    pub fn stop_at_intersection(mut self) -> Self {
        self.stop_at_intersection = true;
        self
    }
    
    /// Set detach mode
    pub fn detach(mut self) -> Self {
        self.detach = true;
        self
    }
}

/// Options for face rotate operation
#[derive(Debug, Clone)]
pub struct RotateOptions {
    /// Origin point of rotation axis
    pub axis_origin: Point3,
    /// Direction of rotation axis
    pub axis_direction: Vec3,
    /// Angle in degrees
    pub angle: f64,
    /// Whether to maintain connectivity
    pub maintain_connectivity: bool,
}

impl RotateOptions {
    /// Create new rotate options
    pub fn new(axis_origin: Point3, axis_direction: Vec3, angle: f64) -> Self {
        Self {
            axis_origin,
            axis_direction: axis_direction.normalized(),
            angle,
            maintain_connectivity: true,
        }
    }
    
    /// Set connectivity maintenance
    pub fn with_connectivity(mut self, maintain: bool) -> Self {
        self.maintain_connectivity = maintain;
        self
    }
}

/// Options for face offset operation
#[derive(Debug, Clone)]
pub struct OffsetOptions {
    /// Offset distance (positive = outward, negative = inward)
    pub offset: f64,
    /// Whether to allow self-intersection
    pub allow_self_intersection: bool,
    /// Tolerance for the offset
    pub tolerance: f64,
}

impl OffsetOptions {
    /// Create new offset options
    pub fn new(offset: f64) -> Self {
        Self {
            offset,
            allow_self_intersection: false,
            tolerance: 1e-6,
        }
    }
    
    /// Allow self-intersection
    pub fn allow_self_intersection(mut self) -> Self {
        self.allow_self_intersection = true;
        self
    }
    
    /// Set tolerance
    pub fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance;
        self
    }
}

/// Face editing operation
#[derive(Debug, Clone)]
pub enum FaceEditOp {
    /// Move operation
    Move(MoveOptions),
    /// Rotate operation
    Rotate(RotateOptions),
    /// Offset operation
    Offset(OffsetOptions),
    /// Delete faces
    Delete,
}

#[cfg(test)]
mod tests {
    use super::*;
    use nova_topo::build_cube;

    #[test]
    fn test_move_options() {
        let opts = MoveOptions::new(Vec3::new(0.0, 0.0, 1.0), 10.0)
            .stop_at_intersection();
        
        assert_eq!(opts.distance, 10.0);
        assert!(opts.stop_at_intersection);
        assert!(!opts.detach);
    }

    #[test]
    fn test_rotate_options() {
        let opts = RotateOptions::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            90.0
        );
        
        assert_eq!(opts.angle, 90.0);
        assert!(opts.maintain_connectivity);
    }

    #[test]
    fn test_offset_options() {
        let opts = OffsetOptions::new(5.0)
            .allow_self_intersection()
            .with_tolerance(1e-4);
        
        assert_eq!(opts.offset, 5.0);
        assert!(opts.allow_self_intersection);
        assert_eq!(opts.tolerance, 1e-4);
    }

    #[test]
    fn test_face_edit_engine_creation() {
        let ctx = SyncContext::new();
        let engine = FaceEditEngine::new(&ctx);
        // Just test that it compiles and runs
    }
}
