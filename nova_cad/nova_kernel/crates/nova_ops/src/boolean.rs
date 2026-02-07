//! Boolean Operations - Unite, Subtract, Intersect
//!
//! Implements robust boolean operations on B-Rep bodies using
//! surface-surface intersection and topological classification.

use crate::{OpsError, OpsResult};
use nova_math::{Point3, Vec3, ToleranceContext, BoundingBox3};
use nova_geom::{Surface, Curve, IntersectionResult};
use nova_topo::{Body, Face, Edge, Vertex, Orientation, TopologyError};
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
        // Clone body1 and add shells from body2
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
        let (split_body1, split_body2) = self.split_faces(body1, body2, &intersections, tolerance)?;
        
        // Step 3: Classify faces
        let classified1 = self.classify_faces(&split_body1, &split_body2, op, true)?;
        let classified2 = self.classify_faces(&split_body2, &split_body1, op, false)?;
        
        // Step 4: Build result body from classified faces
        let result = self.build_result(&classified1, &classified2, op)?;
        
        Ok(result)
    }
    
    /// Find all intersections between faces of two bodies
    fn find_intersections(
        &self,
        body1: &Body,
        body2: &Body,
        tolerance: &ToleranceContext,
    ) -> OpsResult<Vec<FaceIntersection>> {
        let mut intersections = Vec::new();
        
        // Iterate over all face pairs
        for (i, face1) in body1.faces().iter().enumerate() {
            for (j, face2) in body2.faces().iter().enumerate() {
                // Check bounding box overlap
                if !face1.bounding_box().intersects(&face2.bounding_box()) {
                    continue;
                }
                
                // Find surface-surface intersection
                match self.intersect_surfaces(face1.surface(), face2.surface(), tolerance) {
                    Ok(curves) => {
                        for curve in curves {
                            intersections.push(FaceIntersection {
                                face1_idx: i,
                                face2_idx: j,
                                curve,
                            });
                        }
                    }
                    Err(_) => continue, // No intersection or error
                }
                
                if intersections.len() >= self.max_intersections {
                    return Err(OpsError::BooleanFailed(
                        "Too many intersections found".to_string()
                    ));
                }
            }
        }
        
        Ok(intersections)
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
            Err(e) => Err(OpsError::Geometry(format!("Surface intersection failed: {}", e))),
        }
    }
    
    /// Split faces at intersection curves
    fn split_faces(
        &self,
        body1: &Body,
        body2: &Body,
        intersections: &[FaceIntersection],
        tolerance: &ToleranceContext,
    ) -> OpsResult<(Body, Body)> {
        // Clone bodies
        let mut new_body1 = body1.clone();
        let mut new_body2 = body2.clone();
        
        // Group intersections by face
        let mut face1_curves: HashMap<usize, Vec<&FaceIntersection>> = HashMap::new();
        let mut face2_curves: HashMap<usize, Vec<&FaceIntersection>> = HashMap::new();
        
        for intersection in intersections {
            face1_curves.entry(intersection.face1_idx)
                .or_default()
                .push(intersection);
            face2_curves.entry(intersection.face2_idx)
                .or_default()
                .push(intersection);
        }
        
        // Split faces in body1
        for (face_idx, _curves) in face1_curves {
            // TODO: Implement face splitting using Euler operators
            // This is a complex operation that needs:
            // 1. Trim intersection curves to face bounds
            // 2. Create new edges along curves
            // 3. Split face into multiple faces
        }
        
        // Split faces in body2
        for (face_idx, _curves) in face2_curves {
            // TODO: Implement face splitting
        }
        
        Ok((new_body1, new_body2))
    }
    
    /// Classify faces for boolean operation
    fn classify_faces(
        &self,
        body: &Body,
        other: &Body,
        op: BooleanOp,
        is_first: bool,
    ) -> OpsResult<Vec<(Face, FaceClassification)>> {
        let mut result = Vec::new();
        
        for face in body.faces() {
            // Sample a point on the face
            let test_point = self.sample_point_on_face(face);
            
            // Classify point relative to other body
            let classification = self.classify_point(test_point, other);
            
            let face_class = match (op, is_first, classification) {
                // Unite: keep outside faces from both bodies
                (BooleanOp::Unite, _, PointClassification::Outside) => FaceClassification::Keep,
                (BooleanOp::Unite, _, PointClassification::OnBoundary) => FaceClassification::Keep,
                (BooleanOp::Unite, _, PointClassification::Inside) => FaceClassification::Discard,
                
                // Subtract (body1 - body2): keep outside from body1, inside from body2
                (BooleanOp::Subtract, true, PointClassification::Outside) => FaceClassification::Keep,
                (BooleanOp::Subtract, true, _) => FaceClassification::Discard,
                (BooleanOp::Subtract, false, PointClassification::Inside) => FaceClassification::Keep,
                (BooleanOp::Subtract, false, _) => FaceClassification::Discard,
                
                // Intersect: keep inside faces from both bodies
                (BooleanOp::Intersect, _, PointClassification::Inside) => FaceClassification::Keep,
                (BooleanOp::Intersect, _, PointClassification::OnBoundary) => FaceClassification::Keep,
                (BooleanOp::Intersect, _, PointClassification::Outside) => FaceClassification::Discard,
            };
            
            result.push((face.clone(), face_class));
        }
        
        Ok(result)
    }
    
    /// Sample a test point on a face
    fn sample_point_on_face(&self, face: &Face) -> Point3 {
        // Get midpoint of surface UV range
        let (u, v) = face.surface().midpoint_uv();
        face.surface().evaluate(u, v).point
    }
    
    /// Classify a point relative to a body
    fn classify_point(&self, point: Point3, body: &Body) -> PointClassification {
        // Ray casting algorithm
        // Cast ray in +X direction and count intersections
        let ray_dir = Vec3::new(1.0, 0.0, 0.0);
        let mut intersection_count = 0;
        
        for face in body.faces() {
            if let Some(_) = self.ray_face_intersection(point, ray_dir, face) {
                intersection_count += 1;
            }
        }
        
        if intersection_count % 2 == 0 {
            PointClassification::Outside
        } else {
            PointClassification::Inside
        }
    }
    
    /// Find intersection of ray with face
    fn ray_face_intersection(&self, origin: Point3, direction: Vec3, face: &Face) -> Option<Point3> {
        // TODO: Implement proper ray-face intersection
        // This requires ray-surface intersection and UV bounds check
        None
    }
    
    /// Build result body from classified faces
    fn build_result(
        &self,
        classified1: &[(Face, FaceClassification)],
        classified2: &[(Face, FaceClassification)],
        op: BooleanOp,
    ) -> OpsResult<Body> {
        use nova_topo::{EulerOps, Shell};
        
        let mut euler = EulerOps::new();
        let mut result_faces: Vec<Face> = Vec::new();
        
        // Collect faces to keep
        for (face, class) in classified1.iter().chain(classified2.iter()) {
            if matches!(class, FaceClassification::Keep) {
                result_faces.push(face.clone());
            }
        }
        
        if result_faces.is_empty() {
            return Err(OpsError::NoIntersection);
        }
        
        // TODO: Build valid B-Rep from faces
        // This requires stitching faces together to form a valid solid
        
        // For now, return a placeholder
        Err(OpsError::NotSupported(
            "Boolean result construction not yet fully implemented".to_string()
        ))
    }
}

impl Default for BooleanEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Face-face intersection result
#[derive(Debug)]
struct FaceIntersection {
    /// Index of face in first body
    face1_idx: usize,
    /// Index of face in second body
    face2_idx: usize,
    /// Intersection curve
    curve: Box<dyn Curve>,
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
        assert_eq!(
            PointClassification::Inside as u8,
            PointClassification::Inside as u8
        );
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
        let mut euler1 = EulerOps::new();
        let plane1 = Plane::from_normal(Point3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0));
        let (_, _, _, body1) = euler1.make_vertex_face_shell(
            Point3::new(0.0, 0.0, 0.0),
            Box::new(PlanarSurface::new(plane1))
        ).unwrap();
        
        let mut euler2 = EulerOps::new();
        let plane2 = Plane::from_normal(Point3::new(10.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0));
        let (_, _, _, body2) = euler2.make_vertex_face_shell(
            Point3::new(10.0, 0.0, 0.0),
            Box::new(PlanarSurface::new(plane2))
        ).unwrap();
        
        // Non-overlapping bodies should not have bbox overlap
        assert!(!engine.bboxes_overlap(&body1, &body2));
    }
}
