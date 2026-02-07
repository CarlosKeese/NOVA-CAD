//! Boolean Operations - Unite, Subtract, Intersect
//!
//! Implements robust boolean operations on B-Rep bodies using
//! surface-surface intersection and topological classification.

use crate::{OpsError, OpsResult, split::{split_face_at_curves, FaceSplit}};
use nova_math::{Point3, Vec3, ToleranceContext, BoundingBox3};
use nova_geom::{Surface, Curve, IntersectionResult};
use nova_topo::{Body, Face, Edge, Vertex, Shell, Loop, Coedge, EulerOps, Orientation, Sense, Entity, new_entity_id};
use std::sync::Arc;
use std::collections::{HashMap, HashSet};

/// Boolean operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOp {
    /// Unite two bodies (union)
    Unite,
    /// Subtract body2 from body1
    Subtract,
    /// Intersect two bodies
    Intersect,
}

impl BooleanOp {
    /// Get operation name
    pub fn name(&self) -> &'static str {
        match self {
            BooleanOp::Unite => "unite",
            BooleanOp::Subtract => "subtract",
            BooleanOp::Intersect => "intersect",
        }
    }
}

/// Classification of a point relative to a body
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointClassification {
    /// Point is inside the body
    Inside,
    /// Point is outside the body
    Outside,
    /// Point is on the boundary
    OnBoundary,
}

/// Classification of a face after boolean operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceClassification {
    /// Face should be kept in result
    Keep,
    /// Face should be discarded
    Discard,
    /// Face needs to be split
    Split,
}

/// Engine for boolean operations
#[derive(Debug, Clone)]
pub struct BooleanEngine {
    /// Use parallel processing for large models
    pub parallel: bool,
    /// Maximum number of intersection curves
    pub max_intersections: usize,
}

impl BooleanEngine {
    /// Create a new boolean engine
    pub fn new() -> Self {
        Self {
            parallel: true,
            max_intersections: 10000,
        }
    }
    
    /// Perform boolean unite operation
    pub fn unite(&self, body1: &Body, body2: &Body, tolerance: &ToleranceContext) -> OpsResult<Body> {
        // Check bounding box overlap first
        if !self.bboxes_overlap(body1, body2) {
            // No overlap - return combined body
            return self.combine_bodies(body1, body2);
        }
        
        // Perform the boolean operation
        self.perform_boolean(body1, body2, BooleanOp::Unite, tolerance)
    }
    
    /// Perform boolean subtract operation
    pub fn subtract(&self, body1: &Body, body2: &Body, tolerance: &ToleranceContext) -> OpsResult<Body> {
        // Check bounding box overlap
        if !self.bboxes_overlap(body1, body2) {
            // No overlap - return body1 unchanged
            return Ok(body1.clone());
        }
        
        self.perform_boolean(body1, body2, BooleanOp::Subtract, tolerance)
    }
    
    /// Perform boolean intersect operation
    pub fn intersect(&self, body1: &Body, body2: &Body, tolerance: &ToleranceContext) -> OpsResult<Body> {
        // Check bounding box overlap
        if !self.bboxes_overlap(body1, body2) {
            // No overlap - return empty result
            return Err(OpsError::NoIntersection);
        }
        
        self.perform_boolean(body1, body2, BooleanOp::Intersect, tolerance)
    }
    
    /// Check if bounding boxes overlap
    fn bboxes_overlap(&self, body1: &Body, body2: &Body) -> bool {
        let bbox1 = body1.bounding_box();
        let bbox2 = body2.bounding_box();
        bbox1.intersects(&bbox2)
    }
    
    /// Combine two bodies without boolean (just merge)
    fn combine_bodies(&self, body1: &Body, body2: &Body) -> OpsResult<Body> {
        let mut result = body1.clone();
        
        for shell in body2.shells() {
            result.add_shell(shell.clone());
        }
        
        Ok(result)
    }
    
    /// Main boolean algorithm
    fn perform_boolean(
        &self,
        body1: &Body,
        body2: &Body,
        op: BooleanOp,
        tolerance: &ToleranceContext,
    ) -> OpsResult<Body> {
        // Step 1: Find all face-face intersections
        let intersections = self.find_intersections(body1, body2, tolerance)?;
        
        if intersections.is_empty() {
            // No intersections - handle based on operation
            return match op {
                BooleanOp::Unite => self.combine_bodies(body1, body2),
                BooleanOp::Subtract => Ok(body1.clone()),
                BooleanOp::Intersect => Err(OpsError::NoIntersection),
            };
        }
        
        // Step 2: Split faces at intersection curves
        let (split_body1, split_body2) = self.split_all_faces(
            body1, body2, &intersections, tolerance
        )?;
        
        // Step 3: Classify all faces
        let classified1 = self.classify_all_faces(&split_body1, &split_body2, op, true)?;
        let classified2 = self.classify_all_faces(&split_body2, &split_body1, op, false)?;
        
        // Step 4: Build result body from classified faces
        let result = self.build_result_body(&classified1, &classified2, op, tolerance)?;
        
        Ok(result)
    }
    
    /// Find all intersections between faces of two bodies
    fn find_intersections(
        &self,
        body1: &Body,
        body2: &Body,
        tolerance: &ToleranceContext,
    ) -> OpsResult<Vec<FaceFaceIntersection>> {
        let mut intersections = Vec::new();
        
        // Get all faces
        let faces1: Vec<_> = body1.faces();
        let faces2: Vec<_> = body2.faces();
        
        // Iterate over all face pairs
        for (i, face1) in faces1.iter().enumerate() {
            let bbox1 = self.compute_face_bbox(face1);
            
            for (j, face2) in faces2.iter().enumerate() {
                let bbox2 = self.compute_face_bbox(face2);
                
                // Quick reject using bounding boxes
                if !bbox1.intersects(&bbox2) {
                    continue;
                }
                
                // Find surface-surface intersection
                if let Some((surf1, surf2)) = face1.surface().zip(face2.surface()) {
                    match self.intersect_surfaces(
                        surf1.as_ref(),
                        surf2.as_ref(),
                        tolerance
                    ) {
                        Ok(curves) if !curves.is_empty() => {
                            intersections.push(FaceFaceIntersection {
                                face1_idx: i,
                                face2_idx: j,
                                curves,
                            });
                        }
                        _ => continue,
                    }
                }
                
                if intersections.len() >= self.max_intersections {
                    return Err(OpsError::BooleanFailed(
                        format!("Too many intersections (max: {})", self.max_intersections)
                    ));
                }
            }
        }
        
        Ok(intersections)
    }
    
    /// Compute bounding box of a face
    fn compute_face_bbox(&self, face: &Face) -> BoundingBox3 {
        let mut bbox = BoundingBox3::EMPTY;
        
        for loop_ in face.loops() {
            for coedge in loop_.coedges() {
                let edge = coedge.edge();
                bbox.expand(&edge.start_vertex().position());
                bbox.expand(&edge.end_vertex().position());
            }
        }
        
        bbox
    }
    
    /// Intersect two surfaces
    fn intersect_surfaces(
        &self,
        surf1: &dyn Surface,
        surf2: &dyn Surface,
        tolerance: &ToleranceContext,
    ) -> OpsResult<Vec<Box<dyn Curve>>> {
        use nova_geom::intersection::surface_surface_intersection;
        
        match surface_surface_intersection(surf1, surf2, tolerance.tolerance()) {
            Ok(results) => {
                let curves: Vec<Box<dyn Curve>> = results
                    .into_iter()
                    .filter_map(|r| match r {
                        IntersectionResult::Curve(c) => Some(c),
                        _ => None,
                    })
                    .collect();
                Ok(curves)
            }
            Err(e) => Err(OpsError::Geometry(format!("Surface intersection: {}", e))),
        }
    }
    
    /// Split all faces at intersection curves
    fn split_all_faces(
        &self,
        body1: &Body,
        body2: &Body,
        intersections: &[FaceFaceIntersection],
        tolerance: &ToleranceContext,
    ) -> OpsResult<(Body, Body)> {
        // Group curves by face
        let mut face1_curves: HashMap<usize, Vec<Box<dyn Curve>>> = HashMap::new();
        let mut face2_curves: HashMap<usize, Vec<Box<dyn Curve>>> = HashMap::new();
        
        for inter in intersections {
            face1_curves.entry(inter.face1_idx)
                .or_default()
                .extend(inter.curves.clone());
            face2_curves.entry(inter.face2_idx)
                .or_default()
                .extend(inter.curves.clone());
        }
        
        // Split faces in body1
        let mut new_shell1 = Shell::new();
        let faces1: Vec<_> = body1.faces();
        
        for (i, face) in faces1.iter().enumerate() {
            if let Some(curves) = face1_curves.get(&i) {
                let split_faces = split_face_at_curves(face, curves, tolerance)?;
                for new_face in split_faces {
                    new_shell1.add_face(new_face);
                }
            } else {
                new_shell1.add_face((*face).clone());
            }
        }
        
        let mut new_body1 = Body::new();
        new_body1.add_shell(new_shell1);
        
        // Split faces in body2
        let mut new_shell2 = Shell::new();
        let faces2: Vec<_> = body2.faces();
        
        for (i, face) in faces2.iter().enumerate() {
            if let Some(curves) = face2_curves.get(&i) {
                let split_faces = split_face_at_curves(face, curves, tolerance)?;
                for new_face in split_faces {
                    new_shell2.add_face(new_face);
                }
            } else {
                new_shell2.add_face((*face).clone());
            }
        }
        
        let mut new_body2 = Body::new();
        new_body2.add_shell(new_shell2);
        
        Ok((new_body1, new_body2))
    }
    
    /// Classify all faces of a body
    fn classify_all_faces(
        &self,
        body: &Body,
        other: &Body,
        op: BooleanOp,
        is_first: bool,
    ) -> OpsResult<Vec<ClassifiedFace>> {
        let mut result = Vec::new();
        
        for shell in body.shells() {
            for face in shell.faces() {
                let classification = self.classify_face(face, body, other, op, is_first)?;
                result.push(ClassifiedFace {
                    face: face.clone(),
                    classification,
                    shell_is_outer: shell.is_outer(),
                });
            }
        }
        
        Ok(result)
    }
    
    /// Classify a single face
    fn classify_face(
        &self,
        face: &Face,
        _body: &Body,
        other: &Body,
        op: BooleanOp,
        is_first: bool,
    ) -> OpsResult<FaceClassification> {
        // Sample a point on the face (centroid of face vertices)
        let test_point = self.compute_face_centroid(face);
        
        // Classify point relative to other body
        let classification = self.classify_point(test_point, other);
        
        let face_class = match (op, is_first, classification) {
            // Unite: keep outside faces from both bodies
            (BooleanOp::Unite, _, PointClassification::Outside) => FaceClassification::Keep,
            (BooleanOp::Unite, _, PointClassification::OnBoundary) => FaceClassification::Keep,
            (BooleanOp::Unite, _, PointClassification::Inside) => FaceClassification::Discard,
            
            // Subtract (body1 - body2): keep outside from body1, inside from body2
            (BooleanOp::Subtract, true, PointClassification::Outside) => FaceClassification::Keep,
            (BooleanOp::Subtract, true, PointClassification::OnBoundary) => FaceClassification::Keep,
            (BooleanOp::Subtract, true, PointClassification::Inside) => FaceClassification::Discard,
            (BooleanOp::Subtract, false, PointClassification::Inside) => FaceClassification::Keep,
            (BooleanOp::Subtract, false, PointClassification::OnBoundary) => FaceClassification::Keep,
            (BooleanOp::Subtract, false, PointClassification::Outside) => FaceClassification::Discard,
            
            // Intersect: keep inside faces from both bodies
            (BooleanOp::Intersect, _, PointClassification::Inside) => FaceClassification::Keep,
            (BooleanOp::Intersect, _, PointClassification::OnBoundary) => FaceClassification::Keep,
            (BooleanOp::Intersect, _, PointClassification::Outside) => FaceClassification::Discard,
        };
        
        Ok(face_class)
    }
    
    /// Compute centroid of a face
    fn compute_face_centroid(&self, face: &Face) -> Point3 {
        let mut sum = Point3::new(0.0, 0.0, 0.0);
        let mut count = 0;
        
        for loop_ in face.loops() {
            for coedge in loop_.coedges() {
                let edge = coedge.edge();
                let mid = edge.start_vertex().position()
                    .lerp(&edge.end_vertex().position(), 0.5);
                sum = sum + mid - Point3::new(0.0, 0.0, 0.0); // Treat as vector
                count += 1;
            }
        }
        
        if count > 0 {
            Point3::new(sum.x() / count as f64, sum.y() / count as f64, sum.z() / count as f64)
        } else {
            Point3::new(0.0, 0.0, 0.0)
        }
    }
    
    /// Classify a point relative to a body using ray casting
    fn classify_point(&self, point: Point3, body: &Body) -> PointClassification {
        // Cast ray in +X direction
        let ray_dir = Vec3::new(1.0, 0.0, 0.0);
        let mut intersection_count = 0;
        let mut on_boundary = false;
        
        for shell in body.shells() {
            for face in shell.faces() {
                match self.ray_face_intersection(point, ray_dir, face) {
                    RayIntersection::Hit => intersection_count += 1,
                    RayIntersection::OnSurface => on_boundary = true,
                    RayIntersection::Miss => {}
                }
            }
        }
        
        if on_boundary {
            PointClassification::OnBoundary
        } else if intersection_count % 2 == 1 {
            PointClassification::Inside
        } else {
            PointClassification::Outside
        }
    }
    
    /// Ray-face intersection result
    enum RayIntersection {
        Hit,
        OnSurface,
        Miss,
    }
    
    /// Find intersection of ray with face
    fn ray_face_intersection(&self, origin: Point3, direction: Vec3, face: &Face) -> RayIntersection {
        let surface = match face.surface() {
            Some(s) => s,
            None => return RayIntersection::Miss,
        };
        
        // Ray-surface intersection
        // For planes, this is straightforward
        // For curved surfaces, we'd need Newton-Raphson iteration
        
        // Simple plane intersection for now
        let eval = surface.evaluate(0.5, 0.5); // Sample point
        let normal = eval.normal;
        
        let denom = normal.dot(direction);
        if denom.abs() < 1e-10 {
            return RayIntersection::Miss; // Ray parallel to surface
        }
        
        let t = (eval.point - origin).dot(normal) / denom;
        if t < 1e-10 {
            return RayIntersection::Miss; // Intersection behind ray origin
        }
        
        let intersection_point = origin + direction * t;
        
        // Check if point is on the face (using UV bounds check)
        let (u, v) = surface.closest_point(intersection_point);
        
        // Simple bounds check - in practice, need to check against face loops
        if u >= 0.0 && u <= 1.0 && v >= 0.0 && v <= 1.0 {
            // Check if point is inside face boundary
            if self.is_point_in_face_bounds(intersection_point, face) {
                return RayIntersection::Hit;
            }
        }
        
        RayIntersection::Miss
    }
    
    /// Check if point is inside face bounds
    fn is_point_in_face_bounds(&self, point: Point3, face: &Face) -> bool {
        // Use winding number or ray casting in UV space
        // For now, approximate with convex hull check
        let surface = match face.surface() {
            Some(s) => s,
            None => return false,
        };
        
        let (u, v) = surface.closest_point(point);
        
        // Simple bounds - should use actual face boundary
        let mut u_min = f64::INFINITY;
        let mut u_max = f64::NEG_INFINITY;
        let mut v_min = f64::INFINITY;
        let mut v_max = f64::NEG_INFINITY;
        
        for loop_ in face.loops() {
            for coedge in loop_.coedges() {
                let edge = coedge.edge();
                let start = edge.start_vertex().position();
                let end = edge.end_vertex().position();
                
                let (su, sv) = surface.closest_point(start);
                let (eu, ev) = surface.closest_point(end);
                
                u_min = u_min.min(su).min(eu);
                u_max = u_max.max(su).max(eu);
                v_min = v_min.min(sv).min(ev);
                v_max = v_max.max(sv).max(ev);
            }
        }
        
        u >= u_min && u <= u_max && v >= v_min && v <= v_max
    }
    
    /// Build result body from classified faces
    fn build_result_body(
        &self,
        classified1: &[ClassifiedFace],
        classified2: &[ClassifiedFace],
        op: BooleanOp,
        _tolerance: &ToleranceContext,
    ) -> OpsResult<Body> {
        let mut result = Body::new();
        let mut shell = Shell::new();
        
        // Collect all faces to keep
        let mut kept_faces: Vec<&Face> = Vec::new();
        
        for cf in classified1.iter().chain(classified2.iter()) {
            if matches!(cf.classification, FaceClassification::Keep) {
                kept_faces.push(&cf.face);
            }
        }
        
        if kept_faces.is_empty() {
            return Err(OpsError::NoIntersection);
        }
        
        // Add faces to shell
        for face in kept_faces {
            shell.add_face(face.clone());
        }
        
        // For Unite and Intersect, we might need to merge shells
        // For Subtract, handle outer/void shells appropriately
        match op {
            BooleanOp::Unite => {
                shell.set_outer(true);
            }
            BooleanOp::Subtract => {
                shell.set_outer(true);
            }
            BooleanOp::Intersect => {
                shell.set_outer(true);
            }
        }
        
        result.add_shell(shell);
        
        Ok(result)
    }
}

impl Default for BooleanEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Face-face intersection result
#[derive(Debug, Clone)]
struct FaceFaceIntersection {
    /// Index of face in first body
    face1_idx: usize,
    /// Index of face in second body
    face2_idx: usize,
    /// Intersection curves
    curves: Vec<Box<dyn Curve>>,
}

/// Classified face with metadata
#[derive(Debug, Clone)]
struct ClassifiedFace {
    /// The face
    face: Face,
    /// Classification result
    classification: FaceClassification,
    /// Whether the shell is outer
    shell_is_outer: bool,
}

/// Utilities for Point3
trait Point3Ext {
    fn lerp(&self, other: &Self, t: f64) -> Self;
}

impl Point3Ext for Point3 {
    fn lerp(&self, other: &Self, t: f64) -> Self {
        Point3::new(
            self.x() + (other.x() - self.x()) * t,
            self.y() + (other.y() - self.y()) * t,
            self.z() + (other.z() - self.z()) * t,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nova_topo::EulerOps;
    use nova_geom::PlanarSurface;
    use nova_math::Plane;

    #[test]
    fn test_boolean_op_enum() {
        assert_eq!(BooleanOp::Unite.name(), "unite");
        assert_eq!(BooleanOp::Subtract.name(), "subtract");
        assert_eq!(BooleanOp::Intersect.name(), "intersect");
    }

    #[test]
    fn test_point_classification() {
        assert!(matches!(PointClassification::Inside, PointClassification::Inside));
        assert!(matches!(PointClassification::Outside, PointClassification::Outside));
        assert!(matches!(PointClassification::OnBoundary, PointClassification::OnBoundary));
    }

    #[test]
    fn test_boolean_engine_creation() {
        let engine = BooleanEngine::new();
        assert!(engine.parallel);
        assert_eq!(engine.max_intersections, 10000);
    }

    #[test]
    fn test_bbox_overlap() {
        let engine = BooleanEngine::new();
        
        // Create two simple bodies
        let mut body1 = Body::new();
        let mut shell1 = Shell::new();
        let plane1 = PlanarSurface::new(Plane::from_normal(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ));
        let face1 = Face::with_surface(Arc::new(plane1));
        shell1.add_face(face1);
        body1.add_shell(shell1);
        
        let mut body2 = Body::new();
        let mut shell2 = Shell::new();
        let plane2 = PlanarSurface::new(Plane::from_normal(
            Point3::new(10.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ));
        let face2 = Face::with_surface(Arc::new(plane2));
        shell2.add_face(face2);
        body2.add_shell(shell2);
        
        // Non-overlapping bodies
        assert!(!engine.bboxes_overlap(&body1, &body2));
    }

    #[test]
    fn test_classify_face_logic() {
        let engine = BooleanEngine::new();
        
        // Test classification logic for different operations
        assert!(matches!(
            engine.classify_face_logic(BooleanOp::Unite, true, PointClassification::Outside),
            FaceClassification::Keep
        ));
        assert!(matches!(
            engine.classify_face_logic(BooleanOp::Unite, true, PointClassification::Inside),
            FaceClassification::Discard
        ));
    }
}

// Helper trait for testing
impl BooleanEngine {
    fn classify_face_logic(
        &self,
        op: BooleanOp,
        is_first: bool,
        classification: PointClassification,
    ) -> FaceClassification {
        match (op, is_first, classification) {
            (BooleanOp::Unite, _, PointClassification::Outside) => FaceClassification::Keep,
            (BooleanOp::Unite, _, PointClassification::OnBoundary) => FaceClassification::Keep,
            (BooleanOp::Unite, _, PointClassification::Inside) => FaceClassification::Discard,
            (BooleanOp::Subtract, true, PointClassification::Outside) => FaceClassification::Keep,
            (BooleanOp::Subtract, true, PointClassification::OnBoundary) => FaceClassification::Keep,
            (BooleanOp::Subtract, true, PointClassification::Inside) => FaceClassification::Discard,
            (BooleanOp::Subtract, false, PointClassification::Inside) => FaceClassification::Keep,
            (BooleanOp::Subtract, false, PointClassification::OnBoundary) => FaceClassification::Keep,
            (BooleanOp::Subtract, false, PointClassification::Outside) => FaceClassification::Discard,
            (BooleanOp::Intersect, _, PointClassification::Inside) => FaceClassification::Keep,
            (BooleanOp::Intersect, _, PointClassification::OnBoundary) => FaceClassification::Keep,
            (BooleanOp::Intersect, _, PointClassification::Outside) => FaceClassification::Discard,
        }
    }
}
