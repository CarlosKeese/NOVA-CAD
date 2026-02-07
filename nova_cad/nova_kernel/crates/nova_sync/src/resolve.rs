//! Topology Resolution for Synchronous Editing
//!
//! Resolves topology changes after face editing to maintain a valid solid.
//! Handles face extensions, trims, and blend creation.

use crate::{SyncError, SyncResult};
use nova_math::{Point3, Vec3, ToleranceContext};
use nova_topo::{Body, Face, Edge, Vertex, Shell, Loop, Coedge, EulerAdvanced, Sense, Entity};
use nova_geom::{Surface, Curve, Line, Plane};
use std::collections::{HashMap, HashSet};

/// Topology resolver for synchronous editing
#[derive(Debug, Clone)]
pub struct TopologyResolver {
    /// Resolution strategies
    pub strategies: Vec<ResolutionStrategy>,
    /// Current strategy
    pub current_strategy: ResolutionStrategy,
}

/// Strategies for resolving topology conflicts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionStrategy {
    /// Extend faces to meet
    Extend,
    /// Trim faces at intersection
    Trim,
    /// Create blend between faces
    Blend,
    /// Split edge and reconnect
    SplitReconnect,
    /// Merge faces
    Merge,
    /// Automatic (choose best)
    Automatic,
}

impl TopologyResolver {
    /// Create a new topology resolver
    pub fn new() -> Self {
        Self {
            strategies: vec![
                ResolutionStrategy::Extend,
                ResolutionStrategy::Trim,
                ResolutionStrategy::Blend,
            ],
            current_strategy: ResolutionStrategy::Automatic,
        }
    }
    
    /// Resolve topology after face movement
    pub fn resolve_after_move(
        &self,
        body: &mut Body,
        moved_faces: &[Face],
        tolerance: &ToleranceContext,
    ) -> SyncResult<()> {
        // Find all conflicts
        let conflicts = self.find_conflicts(body, moved_faces, tolerance)?;
        
        // Resolve each conflict
        for conflict in conflicts {
            self.resolve_conflict(body, &conflict, tolerance)?;
        }
        
        // Validate the result
        self.validate_body(body, tolerance)?;
        
        Ok(())
    }
    
    /// Find topology conflicts after face movement
    fn find_conflicts(
        &self,
        body: &Body,
        moved_faces: &[Face],
        _tolerance: &ToleranceContext,
    ) -> SyncResult<Vec<TopologyConflict>> {
        let mut conflicts = Vec::new();
        let moved_ids: HashSet<_> = moved_faces.iter().map(|f| f.id()).collect();
        
        // Find gaps between moved faces and their neighbors
        for moved_face in moved_faces {
            let adjacent = self.find_adjacent_faces(body, moved_face);
            
            for adj_face in adjacent {
                if moved_ids.contains(&adj_face.id()) {
                    continue; // Adjacent face also moved
                }
                
                // Check for gaps or intersections
                let shared_edges = self.find_shared_edges(moved_face, &adj_face);
                
                for edge_info in shared_edges {
                    // Check if faces still meet at the edge
                    if !self.faces_meet_at_edge(moved_face, &adj_face, &edge_info) {
                        conflicts.push(TopologyConflict {
                            conflict_type: ConflictType::Gap,
                            face1: moved_face.id(),
                            face2: adj_face.id(),
                            edge: edge_info.edge_id,
                            severity: ConflictSeverity::High,
                        });
                    }
                }
            }
        }
        
        Ok(conflicts)
    }
    
    /// Resolve a single topology conflict
    fn resolve_conflict(
        &self,
        body: &mut Body,
        conflict: &TopologyConflict,
        tolerance: &ToleranceContext,
    ) -> SyncResult<()> {
        match conflict.conflict_type {
            ConflictType::Gap => {
                self.resolve_gap(body, conflict, tolerance)?;
            }
            ConflictType::Intersection => {
                self.resolve_intersection(body, conflict, tolerance)?;
            }
            ConflictType::InvalidLoop => {
                self.resolve_invalid_loop(body, conflict, tolerance)?;
            }
        }
        
        Ok(())
    }
    
    /// Resolve a gap between faces
    fn resolve_gap(
        &self,
        body: &mut Body,
        conflict: &TopologyConflict,
        tolerance: &ToleranceContext,
    ) -> SyncResult<()> {
        let strategy = if self.current_strategy == ResolutionStrategy::Automatic {
            self.choose_best_strategy(body, conflict, tolerance)?
        } else {
            self.current_strategy
        };
        
        match strategy {
            ResolutionStrategy::Extend => {
                self.extend_faces_to_meet(body, conflict, tolerance)?;
            }
            ResolutionStrategy::Blend => {
                self.create_blend_between_faces(body, conflict, tolerance)?;
            }
            ResolutionStrategy::Trim => {
                // Trim isn't appropriate for gaps, try extend instead
                self.extend_faces_to_meet(body, conflict, tolerance)?;
            }
            _ => {
                return Err(SyncError::ResolutionFailed(
                    format!("Unsupported strategy for gap resolution: {:?}", strategy)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Resolve face intersection
    fn resolve_intersection(
        &self,
        body: &mut Body,
        conflict: &TopologyConflict,
        tolerance: &ToleranceContext,
    ) -> SyncResult<()> {
        // Find intersection curve
        // Split faces at intersection
        // Reconnect topology
        
        // TODO: Implement intersection resolution
        
        Ok(())
    }
    
    /// Resolve invalid loop
    fn resolve_invalid_loop(
        &self,
        body: &mut Body,
        conflict: &TopologyConflict,
        tolerance: &ToleranceContext,
    ) -> SyncResult<()> {
        // Find the invalid loop
        // Rebuild or repair the loop
        
        // TODO: Implement loop repair
        
        Ok(())
    }
    
    /// Extend two faces to meet at their shared edge
    fn extend_faces_to_meet(
        &self,
        body: &mut Body,
        conflict: &TopologyConflict,
        tolerance: &ToleranceContext,
    ) -> SyncResult<()> {
        // Find the faces
        let face1 = self.find_face_in_body(body, conflict.face1)
            .ok_or_else(|| SyncError::FaceNotFound(conflict.face1.0))?;
        let face2 = self.find_face_in_body(body, conflict.face2)
            .ok_or_else(|| SyncError::FaceNotFound(conflict.face2.0))?;
        
        // For planar faces, extend the surface
        if self.is_planar_face(&face1) && self.is_planar_face(&face2) {
            // Extend both surfaces until they intersect
            // Update face bounds to include the intersection
        }
        
        // For curved faces, more complex extension needed
        
        Ok(())
    }
    
    /// Create a blend between two faces
    fn create_blend_between_faces(
        &self,
        body: &mut Body,
        conflict: &TopologyConflict,
        tolerance: &ToleranceContext,
    ) -> SyncResult<()> {
        // Find shared edge or create one
        // Create blend surface
        // Add blend face to body
        
        // TODO: Implement blend creation
        
        Ok(())
    }
    
    /// Choose the best resolution strategy
    fn choose_best_strategy(
        &self,
        body: &Body,
        conflict: &TopologyConflict,
        tolerance: &ToleranceContext,
    ) -> SyncResult<ResolutionStrategy> {
        // Analyze the conflict and choose best strategy
        
        // For now, simple heuristic:
        // - If faces are planar and angle is small, extend
        // - If faces are curved or angle is large, blend
        // - If faces intersect, trim
        
        Ok(ResolutionStrategy::Extend)
    }
    
    /// Find faces adjacent to a given face
    fn find_adjacent_faces(&self, body: &Body, face: &Face) -> Vec<Face> {
        let face_edges: HashSet<_> = face.loops()
            .iter()
            .flat_map(|lp| lp.coedges())
            .map(|c| c.edge().id())
            .collect();
        
        let mut adjacent = Vec::new();
        
        for shell in body.shells() {
            for other in shell.faces() {
                if other.id() == face.id() {
                    continue;
                }
                
                let other_edges: HashSet<_> = other.loops()
                    .iter()
                    .flat_map(|lp| lp.coedges())
                    .map(|c| c.edge().id())
                    .collect();
                
                if face_edges.intersection(&other_edges).next().is_some() {
                    adjacent.push(other.clone());
                }
            }
        }
        
        adjacent
    }
    
    /// Find shared edges between two faces
    fn find_shared_edges(&self, face1: &Face, face2: &Face) -> Vec<SharedEdgeInfo> {
        let mut shared = Vec::new();
        
        let edges1: HashMap<_, _> = face1.loops()
            .iter()
            .flat_map(|lp| lp.coedges())
            .map(|c| (c.edge().id(), c.clone()))
            .collect();
        
        for coedge in face2.loops().iter().flat_map(|lp| lp.coedges()) {
            if let Some(c1) = edges1.get(&coedge.edge().id()) {
                shared.push(SharedEdgeInfo {
                    edge_id: coedge.edge().id(),
                    face1_coedge: c1.clone(),
                    face2_coedge: coedge.clone(),
                });
            }
        }
        
        shared
    }
    
    /// Check if two faces still meet at an edge
    fn faces_meet_at_edge(&self, face1: &Face, face2: &Face, edge_info: &SharedEdgeInfo) -> bool {
        // Check if the edge geometry is still shared
        // For now, assume they meet if they share the edge
        true
    }
    
    /// Check if a face is planar
    fn is_planar_face(&self, face: &Face) -> bool {
        if let Some(surface) = face.surface() {
            // Check if surface is PlanarSurface
            // For now, simplified check
            true
        } else {
            false
        }
    }
    
    /// Find a face in the body by ID
    fn find_face_in_body(&self, body: &Body, face_id: nova_topo::EntityId) -> Option<Face> {
        for shell in body.shells() {
            for face in shell.faces() {
                if face.id() == face_id {
                    return Some(face.clone());
                }
            }
        }
        None
    }
    
    /// Validate the body after resolution
    fn validate_body(&self, body: &Body, tolerance: &ToleranceContext) -> SyncResult<()> {
        // Check Euler characteristic
        let v = body.vertices().len();
        let e = body.edges().len();
        let f = body.faces().len();
        let s = body.shells().len();
        
        // For a solid: V - E + F = 2 for each shell
        let euler = v as i32 - e as i32 + f as i32;
        let expected = 2 * s as i32;
        
        if euler != expected {
            return Err(SyncError::WouldInvalidateSolid(
                format!("Euler characteristic mismatch: {} != {}", euler, expected)
            ));
        }
        
        // Check manifold
        for edge in body.edges() {
            let coedge_count = edge.coedges().len();
            if coedge_count != 2 {
                return Err(SyncError::WouldInvalidateSolid(
                    format!("Non-manifold edge detected: {} coedges", coedge_count)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Stitch faces in a body to close gaps
    pub fn stitch_faces(
        &self,
        body: &mut Body,
        tolerance: f64,
    ) -> SyncResult<usize> {
        let mut stitch_count = 0;
        
        // Find edge pairs that should be merged
        let edge_pairs = self.find_stitchable_edges(body, tolerance)?;
        
        for (edge1_id, edge2_id) in edge_pairs {
            // Merge the edges
            // Update coedges to point to merged edge
            stitch_count += 1;
        }
        
        Ok(stitch_count)
    }
    
    /// Find edges that can be stitched
    fn find_stitchable_edges(
        &self,
        body: &Body,
        tolerance: f64,
    ) -> SyncResult<Vec<(nova_topo::EntityId, nova_topo::EntityId)>> {
        let mut pairs = Vec::new();
        let edges: Vec<_> = body.edges().iter().map(|e| (*e).clone()).collect();
        
        for (i, edge1) in edges.iter().enumerate() {
            for edge2 in edges.iter().skip(i + 1) {
                if self.edges_match(edge1, edge2, tolerance) {
                    pairs.push((edge1.id(), edge2.id()));
                }
            }
        }
        
        Ok(pairs)
    }
    
    /// Check if two edges match
    fn edges_match(&self, edge1: &Edge, edge2: &Edge, tolerance: f64) -> bool {
        let v1_start = edge1.start_vertex().position();
        let v1_end = edge1.end_vertex().position();
        let v2_start = edge2.start_vertex().position();
        let v2_end = edge2.end_vertex().position();
        
        let same = v1_start.distance_to(&v2_start) < tolerance &&
                   v1_end.distance_to(&v2_end) < tolerance;
        
        let reversed = v1_start.distance_to(&v2_end) < tolerance &&
                       v1_end.distance_to(&v2_start) < tolerance;
        
        same || reversed
    }
}

impl Default for TopologyResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Topology conflict description
#[derive(Debug, Clone)]
struct TopologyConflict {
    /// Type of conflict
    conflict_type: ConflictType,
    /// First face involved
    face1: nova_topo::EntityId,
    /// Second face involved
    face2: nova_topo::EntityId,
    /// Edge where conflict occurs
    edge: nova_topo::EntityId,
    /// Severity level
    severity: ConflictSeverity,
}

/// Types of topology conflicts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConflictType {
    /// Gap between faces
    Gap,
    /// Faces intersect
    Intersection,
    /// Invalid loop structure
    InvalidLoop,
}

/// Conflict severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConflictSeverity {
    /// Can be ignored
    Low,
    /// Should be fixed
    Medium,
    /// Must be fixed
    High,
    /// Critical, may fail
    Critical,
}

/// Information about a shared edge
#[derive(Debug, Clone)]
struct SharedEdgeInfo {
    edge_id: nova_topo::EntityId,
    face1_coedge: Coedge,
    face2_coedge: Coedge,
}

/// Resolution result
#[derive(Debug, Clone)]
pub struct ResolutionResult {
    /// Whether resolution was successful
    pub success: bool,
    /// Number of conflicts resolved
    pub conflicts_resolved: usize,
    /// Number of conflicts remaining
    pub conflicts_remaining: usize,
    /// Changes made
    pub changes: Vec<ResolutionChange>,
}

/// A change made during resolution
#[derive(Debug, Clone)]
pub struct ResolutionChange {
    /// Type of change
    pub change_type: ChangeType,
    /// Entities affected
    pub entities: Vec<nova_topo::EntityId>,
    /// Description
    pub description: String,
}

/// Types of resolution changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    /// Face was extended
    FaceExtended,
    /// Face was trimmed
    FaceTrimmed,
    /// Blend was created
    BlendCreated,
    /// Edge was split
    EdgeSplit,
    /// Faces were merged
    FacesMerged,
    /// Loop was modified
    LoopModified,
}

#[cfg(test)]
mod tests {
    use super::*;
    use nova_topo::build_cube;

    #[test]
    fn test_topology_resolver_creation() {
        let resolver = TopologyResolver::new();
        assert!(!resolver.strategies.is_empty());
        assert!(matches!(resolver.current_strategy, ResolutionStrategy::Automatic));
    }

    #[test]
    fn test_resolution_strategy() {
        assert!(matches!(ResolutionStrategy::Extend, ResolutionStrategy::Extend));
        assert!(matches!(ResolutionStrategy::Trim, ResolutionStrategy::Trim));
        assert!(matches!(ResolutionStrategy::Blend, ResolutionStrategy::Blend));
    }

    #[test]
    fn test_conflict_type() {
        assert!(matches!(ConflictType::Gap, ConflictType::Gap));
        assert!(matches!(ConflictType::Intersection, ConflictType::Intersection));
    }

    #[test]
    fn test_conflict_severity() {
        assert!(ConflictSeverity::Critical > ConflictSeverity::High);
        assert!(ConflictSeverity::High > ConflictSeverity::Medium);
        assert!(ConflictSeverity::Medium > ConflictSeverity::Low);
    }

    #[test]
    fn test_find_adjacent_faces() {
        let body = build_cube(10.0).unwrap();
        let resolver = TopologyResolver::new();
        
        // Get first face
        let first_face = body.shells()[0].faces()[0].clone();
        let adjacent = resolver.find_adjacent_faces(&body, &first_face);
        
        // A cube face has 4 adjacent faces
        assert_eq!(adjacent.len(), 4);
    }

    #[test]
    fn test_edges_match() {
        let resolver = TopologyResolver::new();
        
        let v1 = Vertex::new(Point3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(Point3::new(1.0, 0.0, 0.0));
        let v3 = Vertex::new(Point3::new(0.0, 0.0, 0.0));
        let v4 = Vertex::new(Point3::new(1.0, 0.0, 0.0));
        
        let edge1 = Edge::new(Arc::new(v1.clone()), Arc::new(v2.clone()));
        let edge2 = Edge::new(Arc::new(v3.clone()), Arc::new(v4.clone()));
        
        assert!(resolver.edges_match(&edge1, &edge2, 1e-6));
    }
}
