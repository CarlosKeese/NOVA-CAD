//! Fillet and Chamfer Operations
//!
//! Implements edge rounding and beveling operations for B-Rep bodies.

use crate::{OpsError, OpsResult};
use nova_math::{Point3, Vec3, ToleranceContext};
use nova_geom::{Curve, Surface, SurfaceEvaluation};
use nova_topo::{Body, Face, Edge, Vertex, EulerOps, Orientation};
use std::collections::{HashMap, HashSet};

/// Fillet operation
#[derive(Debug, Clone)]
pub struct FilletOp {
    /// Edges to fillet
    pub edges: Vec<Edge>,
    /// Radius of the fillet
    pub radius: f64,
    /// Variable radius (if Some, overrides radius)
    pub variable_radius: Option<Vec<(f64, f64)>>, // (position, radius)
}

impl FilletOp {
    /// Create a new fillet operation with constant radius
    pub fn new(edges: Vec<Edge>, radius: f64) -> Self {
        Self {
            edges,
            radius,
            variable_radius: None,
        }
    }
    
    /// Create a fillet with variable radius
    pub fn with_variable_radius(
        edges: Vec<Edge>,
        radius_values: Vec<(f64, f64)>
    ) -> Self {
        Self {
            edges,
            radius: 0.0,
            variable_radius: Some(radius_values),
        }
    }
}

/// Chamfer operation
#[derive(Debug, Clone)]
pub struct ChamferOp {
    /// Edges to chamfer
    pub edges: Vec<Edge>,
    /// Distance along first face
    pub distance1: f64,
    /// Distance along second face (if None, equal to distance1)
    pub distance2: Option<f64>,
    /// Chamfer angle in degrees (alternative to distances)
    pub angle: Option<f64>,
}

impl ChamferOp {
    /// Create a new chamfer operation with equal distances
    pub fn new(edges: Vec<Edge>, distance: f64) -> Self {
        Self {
            edges,
            distance1: distance,
            distance2: None,
            angle: None,
        }
    }
    
    /// Create a chamfer with two distances
    pub fn with_distances(edges: Vec<Edge>, d1: f64, d2: f64) -> Self {
        Self {
            edges,
            distance1: d1,
            distance2: Some(d2),
            angle: None,
        }
    }
    
    /// Create a chamfer with distance and angle
    pub fn with_angle(edges: Vec<Edge>, distance: f64, angle_degrees: f64) -> Self {
        Self {
            edges,
            distance1: distance,
            distance2: None,
            angle: Some(angle_degrees),
        }
    }
}

/// Options for fillet creation
#[derive(Debug, Clone)]
pub struct FilletOptions {
    /// Default radius
    pub default_radius: f64,
    /// Maximum radius allowed
    pub max_radius: f64,
    /// Minimum radius allowed
    pub min_radius: f64,
    /// Whether to propagate to tangent edges
    pub propagate_tangent: bool,
    /// Whether to trim input edges
    pub trim_edges: bool,
}

impl FilletOptions {
    /// Create default fillet options
    pub fn new() -> Self {
        Self {
            default_radius: 1.0,
            max_radius: 1000.0,
            min_radius: 1e-6,
            propagate_tangent: true,
            trim_edges: true,
        }
    }
}

impl Default for FilletOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Options for chamfer creation
#[derive(Debug, Clone)]
pub struct ChamferOptions {
    /// Default distance
    pub default_distance: f64,
    /// Maximum distance allowed
    pub max_distance: f64,
    /// Whether to propagate to tangent edges
    pub propagate_tangent: bool,
}

impl ChamferOptions {
    /// Create default chamfer options
    pub fn new() -> Self {
        Self {
            default_distance: 1.0,
            max_distance: 1000.0,
            propagate_tangent: false,
        }
    }
}

impl Default for ChamferOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Engine for fillet and chamfer operations
#[derive(Debug, Clone)]
pub struct FilletEngine {
    /// Options for fillets
    pub fillet_options: FilletOptions,
    /// Options for chamfers
    pub chamfer_options: ChamferOptions,
}

impl FilletEngine {
    /// Create a new fillet engine
    pub fn new() -> Self {
        Self {
            fillet_options: FilletOptions::default(),
            chamfer_options: ChamferOptions::default(),
        }
    }
    
    /// Apply fillet to edges of a body
    pub fn fillet(
        &self,
        body: &Body,
        edges: &[Edge],
        radius: f64,
        tolerance: &ToleranceContext,
    ) -> OpsResult<Body> {
        // Validate radius
        if radius < self.fillet_options.min_radius {
            return Err(OpsError::InvalidParameters(
                format!("Fillet radius {} too small", radius)
            ));
        }
        if radius > self.fillet_options.max_radius {
            return Err(OpsError::InvalidParameters(
                format!("Fillet radius {} too large", radius)
            ));
        }
        
        if edges.is_empty() {
            return Ok(body.clone());
        }
        
        let mut result_body = body.clone();
        
        // Process each edge
        for edge in edges {
            result_body = self.fillet_single_edge(&result_body, edge, radius, tolerance)?;
        }
        
        Ok(result_body)
    }
    
    /// Apply variable radius fillet
    pub fn variable_fillet(
        &self,
        body: &Body,
        edge: &Edge,
        radius_values: &[(f64, f64)], // (position along edge, radius)
        tolerance: &ToleranceContext,
    ) -> OpsResult<Body> {
        if radius_values.len() < 2 {
            return Err(OpsError::InvalidParameters(
                "Variable fillet needs at least 2 radius values".to_string()
            ));
        }
        
        // TODO: Implement variable radius fillet
        Err(OpsError::NotSupported(
            "Variable radius fillet not yet implemented".to_string()
        ))
    }
    
    /// Apply chamfer to edges
    pub fn chamfer(
        &self,
        body: &Body,
        edges: &[Edge],
        distance: f64,
        tolerance: &ToleranceContext,
    ) -> OpsResult<Body> {
        if distance <= 0.0 || distance > self.chamfer_options.max_distance {
            return Err(OpsError::InvalidParameters(
                format!("Invalid chamfer distance: {}", distance)
            ));
        }
        
        if edges.is_empty() {
            return Ok(body.clone());
        }
        
        let mut result_body = body.clone();
        
        for edge in edges {
            result_body = self.chamfer_single_edge(
                &result_body,
                edge,
                distance,
                distance,
                tolerance
            )?;
        }
        
        Ok(result_body)
    }
    
    /// Apply asymmetric chamfer
    pub fn chamfer_asymmetric(
        &self,
        body: &Body,
        edges: &[Edge],
        distance1: f64,
        distance2: f64,
        tolerance: &ToleranceContext,
    ) -> OpsResult<Body> {
        let mut result_body = body.clone();
        
        for edge in edges {
            result_body = self.chamfer_single_edge(
                &result_body,
                edge,
                distance1,
                distance2,
                tolerance
            )?;
        }
        
        Ok(result_body)
    }
    
    /// Fillet a single edge
    fn fillet_single_edge(
        &self,
        body: &Body,
        edge: &Edge,
        radius: f64,
        tolerance: &ToleranceContext,
    ) -> OpsResult<Body> {
        // Get the two faces meeting at this edge
        let faces = self.get_adjacent_faces(body, edge);
        if faces.len() != 2 {
            return Err(OpsError::FilletFailed(
                "Edge does not have exactly 2 adjacent faces".to_string()
            ));
        }
        
        let (face1, face2) = (&faces[0], &faces[1]);
        
        // Calculate fillet surface (rolling ball surface)
        let fillet_surface = self.create_fillet_surface(
            face1,
            face2,
            edge,
            radius,
            tolerance
        )?;
        
        // Create new topology:
        // 1. Split the original edge into three parts (trimmed ends + middle)
        // 2. Create fillet face connecting the trimmed edges
        // 3. Modify adjacent faces to meet the fillet
        
        // TODO: Implement complete fillet topology modification
        
        Err(OpsError::NotSupported(
            "Fillet edge modification not yet fully implemented".to_string()
        ))
    }
    
    /// Chamfer a single edge
    fn chamfer_single_edge(
        &self,
        body: &Body,
        edge: &Edge,
        distance1: f64,
        distance2: f64,
        tolerance: &ToleranceContext,
    ) -> OpsResult<Body> {
        // Get adjacent faces
        let faces = self.get_adjacent_faces(body, edge);
        if faces.len() != 2 {
            return Err(OpsError::FilletFailed(
                "Edge does not have exactly 2 adjacent faces".to_string()
            ));
        }
        
        // Calculate chamfer plane/surface
        let chamfer_surface = self.create_chamfer_surface(
            &faces[0],
            &faces[1],
            edge,
            distance1,
            distance2,
            tolerance
        )?;
        
        // TODO: Implement chamfer topology modification
        
        Err(OpsError::NotSupported(
            "Chamfer edge modification not yet fully implemented".to_string()
        ))
    }
    
    /// Get faces adjacent to an edge
    fn get_adjacent_faces(&self, body: &Body, edge: &Edge) -> Vec<Face> {
        let mut faces = Vec::new();
        let mut seen = HashSet::new();
        
        // Find all coedges of this edge
        for coedge in edge.coedges() {
            // Get the loop containing this coedge
            if let Some(loop_) = coedge.loop_() {
                // Get the face containing this loop
                if let Some(face) = loop_.face() {
                    if seen.insert(face.id()) {
                        faces.push(face.clone());
                    }
                }
            }
        }
        
        faces
    }
    
    /// Create fillet surface between two faces
    fn create_fillet_surface(
        &self,
        face1: &Face,
        face2: &Face,
        edge: &Edge,
        radius: f64,
        tolerance: &ToleranceContext,
    ) -> OpsResult<Box<dyn Surface>> {
        use nova_geom::{CylindricalSurface, Surface};
        
        // Get surface evaluations at edge midpoint
        let edge_curve = edge.curve();
        let mid_t = (edge_curve.parameter_range().start + edge_curve.parameter_range().end) / 2.0;
        let edge_point = edge_curve.evaluate(mid_t);
        
        let surf1 = face1.surface();
        let surf2 = face2.surface();
        
        // Get surface normals (pointing away from material)
        // TODO: Determine correct UV coordinates on surfaces
        let uv1 = surf1.closest_point(edge_point.point);
        let uv2 = surf2.closest_point(edge_point.point);
        
        let eval1 = surf1.evaluate(uv1.0, uv1.1);
        let eval2 = surf2.evaluate(uv2.0, uv2.1);
        
        // Calculate fillet axis (cross product of face normals)
        let axis = eval1.normal.cross(eval2.normal).normalized();
        
        // Calculate fillet center
        let bisector = (eval1.normal + eval2.normal).normalized();
        let fillet_center = edge_point.point + bisector * radius;
        
        // Create cylindrical surface for the fillet
        let cylinder = CylindricalSurface::new(
            fillet_center,
            axis,
            radius,
        );
        
        Ok(Box::new(cylinder))
    }
    
    /// Create chamfer surface between two faces
    fn create_chamfer_surface(
        &self,
        face1: &Face,
        face2: &Face,
        edge: &Edge,
        distance1: f64,
        distance2: f64,
        tolerance: &ToleranceContext,
    ) -> OpsResult<Box<dyn Surface>> {
        use nova_geom::PlanarSurface;
        use nova_math::Plane;
        
        // Get edge curve
        let edge_curve = edge.curve();
        let mid_t = (edge_curve.parameter_range().start + edge_curve.parameter_range().end) / 2.0;
        let edge_point = edge_curve.evaluate(mid_t);
        
        // Get face normals
        let surf1 = face1.surface();
        let surf2 = face2.surface();
        
        let uv1 = surf1.closest_point(edge_point.point);
        let uv2 = surf2.closest_point(edge_point.point);
        
        let eval1 = surf1.evaluate(uv1.0, uv1.1);
        let eval2 = surf2.evaluate(uv2.0, uv2.1);
        
        // Calculate chamfer plane normal (bisector of face normals)
        let normal = (eval1.normal * distance2 + eval2.normal * distance1).normalized();
        
        // Create chamfer plane
        let plane = PlanarSurface::new(Plane::from_normal(edge_point.point, normal));
        
        Ok(Box::new(plane))
    }
    
    /// Propagate edge selection to tangent edges
    pub fn propagate_tangent_edges(
        &self,
        body: &Body,
        seed_edges: &[Edge],
    ) -> Vec<Edge> {
        let mut result: HashSet<_> = seed_edges.iter().map(|e| e.id()).collect();
        let mut to_process: Vec<_> = seed_edges.to_vec();
        
        while let Some(edge) = to_process.pop() {
            // Find tangent edges at vertices
            for vertex in [edge.start_vertex(), edge.end_vertex()] {
                for adjacent_edge in vertex.edges() {
                    if result.contains(&adjacent_edge.id()) {
                        continue;
                    }
                    
                    // Check if tangent
                    if self.are_edges_tangent(&edge, &adjacent_edge) {
                        result.insert(adjacent_edge.id());
                        to_process.push(adjacent_edge.clone());
                    }
                }
            }
        }
        
        // Convert IDs back to edges
        body.edges()
            .into_iter()
            .filter(|e| result.contains(&e.id()))
            .collect()
    }
    
    /// Check if two edges are tangent at their common vertex
    fn are_edges_tangent(&self, edge1: &Edge, edge2: &Edge) -> bool {
        // Find common vertex
        let common_vertex = if edge1.start_vertex().id() == edge2.start_vertex().id() {
            Some(edge1.start_vertex())
        } else if edge1.start_vertex().id() == edge2.end_vertex().id() {
            Some(edge1.start_vertex())
        } else if edge1.end_vertex().id() == edge2.start_vertex().id() {
            Some(edge1.end_vertex())
        } else if edge1.end_vertex().id() == edge2.end_vertex().id() {
            Some(edge1.end_vertex())
        } else {
            None
        };
        
        let vertex = match common_vertex {
            Some(v) => v,
            None => return false,
        };
        
        // Get tangent directions at vertex
        let curve1 = edge1.curve();
        let curve2 = edge2.curve();
        
        let t1 = if edge1.start_vertex().id() == vertex.id() {
            curve1.parameter_range().start
        } else {
            curve1.parameter_range().end
        };
        
        let t2 = if edge2.start_vertex().id() == vertex.id() {
            curve2.parameter_range().start
        } else {
            curve2.parameter_range().end
        };
        
        let eval1 = curve1.evaluate(t1);
        let eval2 = curve2.evaluate(t2);
        
        // Check if tangents are parallel (cross product near zero)
        let cross = eval1.tangent.cross(eval2.tangent);
        cross.magnitude() < 1e-6
    }
}

impl Default for FilletEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of fillet analysis
#[derive(Debug, Clone)]
pub struct FilletAnalysis {
    /// Edges that can be filleted
    pub valid_edges: Vec<Edge>,
    /// Edges that would fail
    pub invalid_edges: Vec<(Edge, String)>,
    /// Estimated number of new faces
    pub new_face_count: usize,
    /// Maximum possible radius for each edge
    pub max_radius: HashMap<nova_topo::EntityId, f64>,
}

/// Analyze edges for fillet feasibility
pub fn analyze_fillet_edges(
    body: &Body,
    edges: &[Edge],
    _tolerance: &ToleranceContext,
) -> FilletAnalysis {
    let mut valid_edges = Vec::new();
    let mut invalid_edges = Vec::new();
    let mut max_radius = HashMap::new();
    
    for edge in edges {
        // Check if edge has exactly 2 adjacent faces
        let face_count = edge.coedges().len();
        if face_count != 2 {
            invalid_edges.push((
                edge.clone(),
                format!("Edge has {} adjacent faces (expected 2)", face_count)
            ));
            continue;
        }
        
        // Calculate maximum possible radius
        // This is limited by the shortest edge connected to either vertex
        let max_r = estimate_max_fillet_radius(body, edge);
        max_radius.insert(edge.id(), max_r);
        
        valid_edges.push(edge.clone());
    }
    
    FilletAnalysis {
        valid_edges,
        invalid_edges,
        new_face_count: valid_edges.len(),
        max_radius,
    }
}

/// Estimate maximum fillet radius for an edge
fn estimate_max_fillet_radius(body: &Body, edge: &Edge) -> f64 {
    let mut min_dist = f64::MAX;
    
    // Check edges connected to start vertex
    for other_edge in edge.start_vertex().edges() {
        if other_edge.id() == edge.id() {
            continue;
        }
        let len = other_edge.curve().length();
        min_dist = min_dist.min(len);
    }
    
    // Check edges connected to end vertex
    for other_edge in edge.end_vertex().edges() {
        if other_edge.id() == edge.id() {
            continue;
        }
        let len = other_edge.curve().length();
        min_dist = min_dist.min(len);
    }
    
    // Max radius is about 1/3 of shortest adjacent edge
    min_dist / 3.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fillet_op_creation() {
        let edges = Vec::new();
        let op = FilletOp::new(edges, 5.0);
        assert_eq!(op.radius, 5.0);
        assert!(op.variable_radius.is_none());
    }

    #[test]
    fn test_chamfer_op_creation() {
        let edges = Vec::new();
        let op = ChamferOp::new(edges, 2.0);
        assert_eq!(op.distance1, 2.0);
        assert!(op.distance2.is_none());
    }

    #[test]
    fn test_chamfer_with_distances() {
        let edges = Vec::new();
        let op = ChamferOp::with_distances(edges, 2.0, 3.0);
        assert_eq!(op.distance1, 2.0);
        assert_eq!(op.distance2, Some(3.0));
    }

    #[test]
    fn test_fillet_options() {
        let opts = FilletOptions::new();
        assert_eq!(opts.default_radius, 1.0);
        assert!(opts.propagate_tangent);
        assert!(opts.trim_edges);
    }

    #[test]
    fn test_chamfer_options() {
        let opts = ChamferOptions::new();
        assert_eq!(opts.default_distance, 1.0);
        assert!(!opts.propagate_tangent);
    }
}
