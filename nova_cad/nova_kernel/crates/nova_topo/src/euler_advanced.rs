//! Advanced Euler operators for complex B-Rep operations
//!
//! Provides high-level operations built on top of basic Euler operators:
//! - Solid construction by sweeping
//! - Face and edge splitting
//! - Fillet construction
//! - Boolean result construction

use crate::{Body, Shell, Face, Loop, Coedge, Edge, Vertex, Sense, Orientation, Entity, GeometricEntity, TopologicalEntity};
use crate::{EulerOps, EulerError, TopoResult, TopologyError, new_entity_id};
use nova_math::{Point3, Vec3, Transform3};
use nova_geom::{Curve, Surface, Line, Plane, PlanarSurface};
use std::sync::Arc;

/// Advanced Euler operations for solid modeling
pub struct EulerAdvanced;

impl EulerAdvanced {
    /// Extrude a face to create a solid
    /// 
    /// Creates a solid by extruding a face profile along a direction.
    /// Uses Euler operators to maintain valid topology.
    pub fn extrude_face(
        profile: &Face,
        direction: Vec3,
        distance: f64,
    ) -> TopoResult<Body> {
        let mut body = Body::new();
        let mut shell = Shell::new();
        
        // Get the profile's outer loop
        let outer_loop = profile.outer_loop()
            .ok_or_else(|| TopologyError::InvalidReference("Face has no outer loop".to_string()))?;
        
        // Get vertices from the profile
        let profile_vertices: Vec<Arc<Vertex>> = outer_loop.coedges()
            .iter()
            .map(|c| Arc::new(c.start_vertex().clone()))
            .collect();
        
        if profile_vertices.len() < 3 {
            return Err(TopologyError::InvalidReference(
                "Profile must have at least 3 vertices".to_string()
            ));
        }
        
        // Create bottom face (original profile)
        let mut bottom_face = create_face_from_loop(outer_loop)?;
        if let Some(surf) = profile.surface() {
            bottom_face.set_geometry(Some(surf.clone()));
        }
        shell.add_face(bottom_face.clone());
        
        // Create top face vertices (translated)
        let extrude_vec = direction.normalized() * distance;
        let top_vertices: Vec<Arc<Vertex>> = profile_vertices
            .iter()
            .map(|v| {
                let new_pos = v.position() + extrude_vec;
                Arc::new(Vertex::new(new_pos))
            })
            .collect();
        
        // Create top face
        let mut top_loop = Loop::new();
        let num_verts = top_vertices.len();
        for i in 0..num_verts {
            let v1 = top_vertices[i].clone();
            let v2 = top_vertices[(i + 1) % num_verts].clone();
            let edge = Arc::new(Edge::new(v1, v2));
            let coedge = Coedge::new(edge, Sense::Same);
            top_loop.add_coedge(coedge);
        }
        
        let mut top_face = Face::new();
        top_face.add_loop(top_loop);
        // Translate surface for top face
        if let Some(surf) = profile.surface() {
            // TODO: Transform surface
        }
        shell.add_face(top_face.clone());
        
        // Create side faces
        let num_edges = profile_vertices.len();
        for i in 0..num_edges {
            let v1_bottom = profile_vertices[i].clone();
            let v2_bottom = profile_vertices[(i + 1) % num_edges].clone();
            let v1_top = top_vertices[i].clone();
            let v2_top = top_vertices[(i + 1) % num_edges].clone();
            
            // Create side face as a quad
            let mut side_loop = Loop::new();
            
            // Bottom edge
            let edge_bottom = Arc::new(Edge::new(v1_bottom.clone(), v2_bottom.clone()));
            side_loop.add_coedge(Coedge::new(edge_bottom, Sense::Same));
            
            // Right edge (up)
            let edge_right = Arc::new(Edge::new(v2_bottom.clone(), v2_top.clone()));
            side_loop.add_coedge(Coedge::new(edge_right, Sense::Same));
            
            // Top edge (reversed)
            let edge_top = Arc::new(Edge::new(v2_top.clone(), v1_top.clone()));
            side_loop.add_coedge(Coedge::new(edge_top, Sense::Same));
            
            // Left edge (down, reversed)
            let edge_left = Arc::new(Edge::new(v1_top.clone(), v1_bottom.clone()));
            side_loop.add_coedge(Coedge::new(edge_left, Sense::Same));
            
            let mut side_face = Face::new();
            side_face.add_loop(side_loop);
            
            // Create planar surface for side face
            let origin = v1_bottom.position();
            let normal = (v2_bottom.position() - v1_bottom.position())
                .cross(v1_top.position() - v1_bottom.position())
                .normalized();
            let plane = PlanarSurface::from_origin_normal(origin, normal);
            side_face.set_geometry(Some(Arc::new(plane)));
            
            shell.add_face(side_face);
        }
        
        body.add_shell(shell);
        Ok(body)
    }
    
    /// Revolve a face around an axis to create a solid
    pub fn revolve_face(
        profile: &Face,
        axis_origin: Point3,
        axis_direction: Vec3,
        angle: f64,
        num_segments: usize,
    ) -> TopoResult<Body> {
        let mut body = Body::new();
        let mut shell = Shell::new();
        
        let outer_loop = profile.outer_loop()
            .ok_or_else(|| TopologyError::InvalidReference("Face has no outer loop".to_string()))?;
        
        // For simplicity, create approximated revolution with planar faces
        let angle_step = angle / num_segments as f64;
        
        // Store profile vertices for each angle step
        let mut profile_rings: Vec<Vec<Arc<Vertex>>> = Vec::new();
        
        for i in 0..=num_segments {
            let current_angle = i as f64 * angle_step;
            let rotation = Transform3::from_axis_angle(axis_origin, axis_direction, current_angle);
            
            let ring: Vec<Arc<Vertex>> = outer_loop.coedges()
                .iter()
                .map(|c| {
                    let pos = c.start_vertex().position();
                    let rotated_pos = rotation.transform_point(pos);
                    Arc::new(Vertex::new(rotated_pos))
                })
                .collect();
            
            profile_rings.push(ring);
        }
        
        // Create faces between rings
        for i in 0..num_segments {
            let ring1 = &profile_rings[i];
            let ring2 = &profile_rings[i + 1];
            
            let num_verts = ring1.len();
            for j in 0..num_verts {
                let v1 = ring1[j].clone();
                let v2 = ring1[(j + 1) % num_verts].clone();
                let v3 = ring2[(j + 1) % num_verts].clone();
                let v4 = ring2[j].clone();
                
                // Create quad face
                let mut loop_ = Loop::new();
                loop_.add_coedge(Coedge::new(Arc::new(Edge::new(v1.clone(), v2.clone())), Sense::Same));
                loop_.add_coedge(Coedge::new(Arc::new(Edge::new(v2.clone(), v3.clone())), Sense::Same));
                loop_.add_coedge(Coedge::new(Arc::new(Edge::new(v3.clone(), v4.clone())), Sense::Same));
                loop_.add_coedge(Coedge::new(Arc::new(Edge::new(v4.clone(), v1.clone())), Sense::Same));
                
                let mut face = Face::new();
                face.add_loop(loop_);
                shell.add_face(face);
            }
        }
        
        // Add start and end caps if angle < 360
        if angle < 2.0 * std::f64::consts::PI - 1e-6 {
            // Start cap (original profile)
            if let Some(first_ring) = profile_rings.first() {
                let mut start_loop = Loop::new();
                let num_verts = first_ring.len();
                for j in 0..num_verts {
                    let v1 = first_ring[j].clone();
                    let v2 = first_ring[(j + 1) % num_verts].clone();
                    let edge = Arc::new(Edge::new(v1, v2));
                    start_loop.add_coedge(Coedge::new(edge, Sense::Same));
                }
                let mut start_face = Face::new();
                start_face.add_loop(start_loop);
                shell.add_face(start_face);
            }
            
            // End cap (rotated profile)
            if let Some(last_ring) = profile_rings.last() {
                let mut end_loop = Loop::new();
                let num_verts = last_ring.len();
                for j in 0..num_verts {
                    let v1 = last_ring[j].clone();
                    let v2 = last_ring[(j + 1) % num_verts].clone();
                    let edge = Arc::new(Edge::new(v1, v2));
                    end_loop.add_coedge(Coedge::new(edge, Sense::Same));
                }
                let mut end_face = Face::new();
                end_face.add_loop(end_loop);
                shell.add_face(end_face);
            }
        }
        
        body.add_shell(shell);
        Ok(body)
    }
    
    /// Split an edge at a parameter, creating a new vertex
    pub fn split_edge(
        edge: &Edge,
        t: f64,
    ) -> TopoResult<(Vertex, Edge, Edge)> {
        // Get position at split parameter
        let position = edge.evaluate(t)
            .ok_or_else(|| TopologyError::InvalidReference("Edge has no curve".to_string()))?;
        
        let split_vertex = Vertex::new(position);
        
        // Create two new edges
        let edge1 = Edge::new(
            Arc::new(edge.start_vertex().clone()),
            Arc::new(split_vertex.clone()),
        );
        
        let edge2 = Edge::new(
            Arc::new(split_vertex.clone()),
            Arc::new(edge.end_vertex().clone()),
        );
        
        Ok((split_vertex, edge1, edge2))
    }
    
    /// Split a face by adding an edge between two existing vertices
    pub fn split_face_by_edge(
        face: &mut Face,
        v1: &Vertex,
        v2: &Vertex,
    ) -> TopoResult<(Edge, Face)> {
        // Create the splitting edge
        let split_edge = Edge::new(
            Arc::new(v1.clone()),
            Arc::new(v2.clone()),
        );
        
        // Find which loops contain these vertices
        let mut found_loop_idx = None;
        for (i, lp) in face.loops().iter().enumerate() {
            let has_v1 = lp.coedges().iter().any(|c| {
                c.start_vertex().id() == v1.id() || c.end_vertex().id() == v1.id()
            });
            let has_v2 = lp.coedges().iter().any(|c| {
                c.start_vertex().id() == v2.id() || c.end_vertex().id() == v2.id()
            });
            
            if has_v1 && has_v2 {
                found_loop_idx = Some(i);
                break;
            }
        }
        
        if found_loop_idx.is_none() {
            return Err(TopologyError::InvalidReference(
                "Vertices not found on the same loop".to_string()
            ));
        }
        
        // Create a new face
        let mut new_face = Face::new();
        let mut new_loop = Loop::new();
        new_loop.add_coedge(Coedge::new(Arc::new(split_edge.clone()), Sense::Same));
        new_face.add_loop(new_loop);
        
        Ok((split_edge, new_face))
    }
    
    /// Create a fillet face between two faces meeting at an edge
    pub fn create_fillet_face(
        edge: &Edge,
        radius: f64,
        face1: &Face,
        face2: &Face,
    ) -> TopoResult<(Face, Vec<Edge>, Vec<Vertex>)> {
        // Get adjacent vertices
        let v1 = Arc::new(edge.start_vertex().clone());
        let v2 = Arc::new(edge.end_vertex().clone());
        
        // Create fillet vertices (offset from original edge)
        let offset1 = calculate_fillet_offset(edge, face1, radius)?;
        let offset2 = calculate_fillet_offset(edge, face2, radius)?;
        
        let fillet_v1 = Arc::new(Vertex::new(v1.position() + offset1));
        let fillet_v2 = Arc::new(Vertex::new(v2.position() + offset1));
        let fillet_v3 = Arc::new(Vertex::new(v2.position() + offset2));
        let fillet_v4 = Arc::new(Vertex::new(v1.position() + offset2));
        
        // Create fillet face
        let mut fillet_loop = Loop::new();
        
        let e1 = Arc::new(Edge::new(fillet_v1.clone(), fillet_v2.clone()));
        let e2 = Arc::new(Edge::new(fillet_v2.clone(), fillet_v3.clone()));
        let e3 = Arc::new(Edge::new(fillet_v3.clone(), fillet_v4.clone()));
        let e4 = Arc::new(Edge::new(fillet_v4.clone(), fillet_v1.clone()));
        
        fillet_loop.add_coedge(Coedge::new(e1, Sense::Same));
        fillet_loop.add_coedge(Coedge::new(e2, Sense::Same));
        fillet_loop.add_coedge(Coedge::new(e3, Sense::Same));
        fillet_loop.add_coedge(Coedge::new(e4, Sense::Same));
        
        let mut fillet_face = Face::new();
        fillet_face.add_loop(fillet_loop);
        
        let edges = vec![
            Edge::new(v1.clone(), fillet_v1),
            Edge::new(v2.clone(), fillet_v3),
        ];
        
        let vertices = vec![
            (*fillet_v1).clone(),
            (*fillet_v2).clone(),
            (*fillet_v3).clone(),
            (*fillet_v4).clone(),
        ];
        
        Ok((fillet_face, edges, vertices))
    }
    
    /// Merge two faces that share an edge
    pub fn merge_faces(
        face1: &Face,
        face2: &Face,
        shared_edge: &Edge,
    ) -> TopoResult<Face> {
        // Create a new face by combining the loops
        let mut merged_face = Face::new();
        
        // Add all loops from face1 except those containing the shared edge
        for lp in face1.loops() {
            let contains_edge = lp.coedges().iter().any(|c| {
                c.edge().id() == shared_edge.id()
            });
            if !contains_edge {
                merged_face.add_loop(lp.clone());
            }
        }
        
        // Add all loops from face2 except those containing the shared edge
        for lp in face2.loops() {
            let contains_edge = lp.coedges().iter().any(|c| {
                c.edge().id() == shared_edge.id()
            });
            if !contains_edge {
                merged_face.add_loop(lp.clone());
            }
        }
        
        // TODO: Properly merge the outer loops by removing the shared edge
        
        Ok(merged_face)
    }
    
    /// Create a solid body from a set of faces (used in Boolean operations)
    pub fn create_solid_from_faces(
        faces: &[Face],
        tolerance: f64,
    ) -> TopoResult<Body> {
        let mut body = Body::new();
        let mut shell = Shell::new();
        
        // Add all faces to the shell
        for face in faces {
            shell.add_face(face.clone());
        }
        
        // Stitch faces together by matching edges
        stitch_faces(&mut shell, tolerance)?;
        
        body.add_shell(shell);
        Ok(body)
    }
    
    /// Add an inner loop (hole) to a face
    pub fn add_inner_loop(
        face: &mut Face,
        vertices: &[Arc<Vertex>],
    ) -> TopoResult<Loop> {
        if vertices.len() < 3 {
            return Err(TopologyError::InvalidReference(
                "Inner loop must have at least 3 vertices".to_string()
            ));
        }
        
        let mut inner_loop = Loop::new();
        
        let num_verts = vertices.len();
        for i in 0..num_verts {
            let v1 = vertices[i].clone();
            let v2 = vertices[(i + 1) % num_verts].clone();
            let edge = Arc::new(Edge::new(v1, v2));
            let coedge = Coedge::new(edge, Sense::Same);
            inner_loop.add_coedge(coedge);
        }
        
        face.add_loop(inner_loop.clone());
        Ok(inner_loop)
    }
}

/// Helper function to create a face from a loop
fn create_face_from_loop(loop_: &Loop) -> TopoResult<Face> {
    let mut face = Face::new();
    
    // Copy the coedges
    let mut new_loop = Loop::new();
    for coedge in loop_.coedges() {
        new_loop.add_coedge(coedge.clone());
    }
    
    face.add_loop(new_loop);
    Ok(face)
}

/// Calculate fillet offset direction
fn calculate_fillet_offset(
    edge: &Edge,
    face: &Face,
    radius: f64,
) -> TopoResult<Vec3> {
    // Get surface normal
    let surface = face.surface()
        .ok_or_else(|| TopologyError::InvalidReference("Face has no surface".to_string()))?;
    
    // Sample point on edge
    let mid_point = if let Some(curve) = edge.curve() {
        let range = curve.param_range();
        curve.evaluate((range.start + range.end) / 2.0)
    } else {
        edge.start_vertex().position()
            .lerp(&edge.end_vertex().position(), 0.5)
    };
    
    // Get surface normal at that point
    let (u, v) = surface.closest_point(mid_point);
    let eval = surface.evaluate(u, v);
    
    // Edge direction
    let edge_dir = if let Some(curve) = edge.curve() {
        let range = curve.param_range();
        let eval_mid = curve.evaluate_derivative((range.start + range.end) / 2.0);
        eval_mid.tangent.normalized()
    } else {
        (edge.end_vertex().position() - edge.start_vertex().position()).normalized()
    };
    
    // Offset direction is perpendicular to both edge and normal
    let offset = edge_dir.cross(eval.normal).normalized() * radius;
    
    Ok(offset)
}

/// Stitch faces together by matching edges
fn stitch_faces(shell: &mut Shell, tolerance: f64) -> TopoResult<()> {
    // Collect all edges
    let mut edges: Vec<(EntityId, Arc<Edge>)> = Vec::new();
    
    for face in shell.faces() {
        for lp in face.loops() {
            for coedge in lp.coedges() {
                let edge = coedge.edge();
                edges.push((edge.id(), Arc::new(edge.clone())));
            }
        }
    }
    
    // Find matching edges (same geometry, opposite orientation)
    let mut to_merge: Vec<(EntityId, EntityId)> = Vec::new();
    
    for (i, (id1, edge1)) in edges.iter().enumerate() {
        for (id2, edge2) in edges.iter().skip(i + 1) {
            if edges_match(edge1.as_ref(), edge2.as_ref(), tolerance) {
                to_merge.push((*id1, *id2));
            }
        }
    }
    
    // In a full implementation, we would merge the matching edges
    // and update the coedges to point to the shared edge
    
    Ok(())
}

/// Check if two edges match (same geometry)
fn edges_match(edge1: &Edge, edge2: &Edge, tolerance: f64) -> bool {
    // Check if vertices match (possibly reversed)
    let v1_start = edge1.start_vertex().position();
    let v1_end = edge1.end_vertex().position();
    let v2_start = edge2.start_vertex().position();
    let v2_end = edge2.end_vertex().position();
    
    // Same direction
    let same = v1_start.distance_to(&v2_start) < tolerance && 
               v1_end.distance_to(&v2_end) < tolerance;
    
    // Reversed
    let reversed = v1_start.distance_to(&v2_end) < tolerance && 
                   v1_end.distance_to(&v2_start) < tolerance;
    
    same || reversed
}

/// Extension trait for Point3
trait Point3Ext {
    fn lerp(&self, other: &Self, t: f64) -> Self;
}

impl Point3Ext for Point3 {
    fn lerp(&self, other: &Self, t: f64) -> Self {
        Point3::new(
            self.x() * (1.0 - t) + other.x() * t,
            self.y() * (1.0 - t) + other.y() * t,
            self.z() * (1.0 - t) + other.z() * t,
        )
    }
}

/// Extension trait for Curve
trait CurveExt {
    fn evaluate_derivative(&self, t: f64) -> CurveEval;
}

struct CurveEval {
    point: Point3,
    tangent: Vec3,
}

#[cfg(test)]
mod tests {
    use super::*;
    use nova_geom::Line;

    #[test]
    fn test_extrude_face() {
        // Create a triangular profile
        let v1 = Arc::new(Vertex::new(Point3::new(0.0, 0.0, 0.0)));
        let v2 = Arc::new(Vertex::new(Point3::new(1.0, 0.0, 0.0)));
        let v3 = Arc::new(Vertex::new(Point3::new(0.5, 1.0, 0.0)));
        
        let mut profile = Face::new();
        let mut lp = Loop::new();
        lp.add_coedge(Coedge::new(Arc::new(Edge::new(v1.clone(), v2.clone())), Sense::Same));
        lp.add_coedge(Coedge::new(Arc::new(Edge::new(v2.clone(), v3.clone())), Sense::Same));
        lp.add_coedge(Coedge::new(Arc::new(Edge::new(v3.clone(), v1.clone())), Sense::Same));
        profile.add_loop(lp);
        
        let body = EulerAdvanced::extrude_face(&profile, Vec3::new(0.0, 0.0, 1.0), 2.0).unwrap();
        
        // Should have 5 faces: top, bottom, and 3 sides
        let face_count: usize = body.shells().iter()
            .map(|s| s.faces().len())
            .sum();
        assert_eq!(face_count, 5);
    }

    #[test]
    fn test_split_edge() {
        let v1 = Arc::new(Vertex::new(Point3::new(0.0, 0.0, 0.0)));
        let v2 = Arc::new(Vertex::new(Point3::new(2.0, 0.0, 0.0)));
        let line = Line::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 0.0, 0.0));
        let mut edge = Edge::with_curve(v1.clone(), v2.clone(), Arc::new(line));
        
        let (split_v, edge1, edge2) = EulerAdvanced::split_edge(&edge, 0.5).unwrap();
        
        assert_eq!(split_v.position().x(), 1.0);
        assert_eq!(edge1.start_vertex().position().x(), 0.0);
        assert_eq!(edge1.end_vertex().position().x(), 1.0);
        assert_eq!(edge2.start_vertex().position().x(), 1.0);
        assert_eq!(edge2.end_vertex().position().x(), 2.0);
    }

    #[test]
    fn test_create_solid_from_faces() {
        // Create a simple tetrahedron
        let v0 = Vertex::new(Point3::new(0.0, 0.0, 0.0));
        let v1 = Vertex::new(Point3::new(1.0, 0.0, 0.0));
        let v2 = Vertex::new(Point3::new(0.5, 1.0, 0.0));
        let v3 = Vertex::new(Point3::new(0.5, 0.33, 1.0));
        
        let mut faces = Vec::new();
        
        // Bottom face
        faces.push(create_triangle_face(&v0, &v1, &v2));
        // Side faces
        faces.push(create_triangle_face(&v0, &v3, &v1));
        faces.push(create_triangle_face(&v1, &v3, &v2));
        faces.push(create_triangle_face(&v2, &v3, &v0));
        
        let body = EulerAdvanced::create_solid_from_faces(&faces, 1e-6).unwrap();
        
        assert!(!body.is_empty());
        let face_count: usize = body.shells().iter()
            .map(|s| s.faces().len())
            .sum();
        assert_eq!(face_count, 4);
    }

    fn create_triangle_face(v1: &Vertex, v2: &Vertex, v3: &Vertex) -> Face {
        let mut face = Face::new();
        let mut lp = Loop::new();
        lp.add_coedge(Coedge::new(Arc::new(Edge::new(Arc::new(v1.clone()), Arc::new(v2.clone()))), Sense::Same));
        lp.add_coedge(Coedge::new(Arc::new(Edge::new(Arc::new(v2.clone()), Arc::new(v3.clone()))), Sense::Same));
        lp.add_coedge(Coedge::new(Arc::new(Edge::new(Arc::new(v3.clone()), Arc::new(v1.clone()))), Sense::Same));
        face.add_loop(lp);
        face
    }
}
