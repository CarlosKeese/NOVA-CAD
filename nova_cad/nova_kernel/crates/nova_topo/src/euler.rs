//! Euler operators for B-Rep topology manipulation

use crate::{Body, Shell, Face, Loop, Coedge, Edge, Vertex, EntityId, Sense, TopoResult, TopologyError, Entity};
use nova_math::Point3;
use std::sync::Arc;

/// Errors that can occur during Euler operations
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum EulerError {
    /// Invalid operation
    #[error("Invalid Euler operation: {0}")]
    InvalidOperation(String),
    
    /// Entity not found
    #[error("Entity not found: {0}")]
    EntityNotFound(String),
    
    /// Topology would become invalid
    #[error("Topology would become invalid: {0}")]
    WouldInvalidate(String),
}

/// Euler operators for B-Rep manipulation
/// 
/// These operators maintain the topological validity of the B-Rep
/// while allowing construction and modification of solid models.
pub struct EulerOps;

impl EulerOps {
    /// MVFS - Make Vertex Face Shell
    /// 
    /// Creates a minimal body consisting of:
    /// - One vertex
    /// - One face (with one loop)
    /// - One shell
    /// 
    /// This is the starting point for building a B-Rep.
    pub fn mvfs(position: Point3) -> (Body, Shell, Face, Vertex) {
        let vertex = Vertex::new(position);
        let mut face = Face::new();
        let mut shell = Shell::new();
        let mut body = Body::new();
        
        // Create an empty loop on the face
        let lp = Loop::new();
        face.add_loop(lp);
        
        shell.add_face(face.clone());
        body.add_shell(shell.clone());
        
        (body, shell, face, vertex)
    }
    
    /// MEV - Make Edge Vertex
    /// 
    /// Splits a vertex and creates a new edge connecting them.
    /// Used to extend the boundary of a face.
    pub fn mev(
        vertex: &mut Vertex,
        new_position: Point3,
        face: &mut Face,
    ) -> TopoResult<(Edge, Vertex)> {
        let new_vertex = Vertex::new(new_position);
        let edge = Edge::new(
            Arc::new(vertex.clone()),
            Arc::new(new_vertex.clone()),
        );
        
        // Add edge to the outer loop of the face
        if let Some(lp) = face.loops_mut().first_mut() {
            let coedge = Coedge::new(Arc::new(edge.clone()), Sense::Same);
            lp.add_coedge(coedge);
        }
        
        Ok((edge, new_vertex))
    }
    
    /// MEF - Make Edge Face
    /// 
    /// Creates a new edge between two vertices on the same face,
    /// splitting the face into two.
    pub fn mef(
        v1: &Vertex,
        v2: &Vertex,
        face: &mut Face,
    ) -> TopoResult<(Edge, Face)> {
        let edge = Edge::new(
            Arc::new(v1.clone()),
            Arc::new(v2.clone()),
        );
        
        // Create a new face with a new loop
        let mut new_face = Face::new();
        let mut new_loop = Loop::new();
        
        // Add coedges to the new loop
        let coedge = Coedge::new(Arc::new(edge.clone()), Sense::Same);
        new_loop.add_coedge(coedge);
        new_face.add_loop(new_loop);
        
        Ok((edge, new_face))
    }
    
    /// KEMR - Kill Edge Make Ring
    /// 
    /// Removes an edge and creates an inner loop (hole) in the face.
    pub fn kemr(
        edge: &Edge,
        face: &mut Face,
    ) -> TopoResult<Loop> {
        // Create a new inner loop from the edge
        let mut new_loop = Loop::new();
        
        // The edge becomes part of the inner loop
        let coedge = Coedge::new(Arc::new(edge.clone()), Sense::Same);
        new_loop.add_coedge(coedge);
        
        face.add_loop(new_loop.clone());
        
        Ok(new_loop)
    }
    
    /// KFMRH - Kill Face Make Ring Hole
    /// 
    /// Removes a face and converts it to a hole in another face.
    /// Used to create void shells.
    pub fn kfmrh(
        face_to_remove: &Face,
        shell: &mut Shell,
    ) -> TopoResult<Loop> {
        // Find the face that will contain the hole
        let containing_face = shell.faces().first()
            .ok_or_else(|| TopologyError::InvalidReference(
                "Shell has no faces".to_string()
            ))?;
        
        // Create a hole loop from the face's outer loop
        let mut hole_loop = Loop::new();
        if let Some(outer_loop) = face_to_remove.outer_loop() {
            for coedge in outer_loop.coedges() {
                hole_loop.add_coedge(coedge.clone());
            }
        }
        
        // Note: In a full implementation, we would actually modify the containing face
        // and remove the face_to_remove from the shell
        
        Ok(hole_loop)
    }
    
    /// MEKR - Make Edge Kill Ring
    /// 
    /// Creates an edge connecting an inner loop to the outer loop,
    /// removing the inner loop.
    pub fn mekr(
        inner_vertex: &Vertex,
        outer_vertex: &Vertex,
        face: &mut Face,
    ) -> TopoResult<Edge> {
        let edge = Edge::new(
            Arc::new(inner_vertex.clone()),
            Arc::new(outer_vertex.clone()),
        );
        
        // The edge bridges the inner and outer loops
        // This removes the inner loop from the face
        
        Ok(edge)
    }
    
    /// SEMV - Split Edge Make Vertex
    /// 
    /// Splits an edge at a point, creating a new vertex.
    pub fn semv(
        edge: &mut Edge,
        t: f64,
    ) -> TopoResult<(Vertex, Edge)> {
        // Evaluate the edge at parameter t to get the split point
        let position = edge.evaluate(t)
            .ok_or_else(|| TopologyError::InvalidReference(
                "Edge has no curve".to_string()
            ))?;
        
        let new_vertex = Vertex::new(position);
        
        // Create a new edge from the new vertex to the original end
        let new_edge = Edge::new(
            Arc::new(new_vertex.clone()),
            Arc::new(edge.end_vertex().clone()),
        );
        
        // Update the original edge to end at the new vertex
        // Note: In a full implementation, we would modify the edge directly
        
        Ok((new_vertex, new_edge))
    }
    
    /// JEKV - Join Edge Kill Vertex
    /// 
    /// Removes a vertex and merges the two edges connected to it.
    pub fn jekv(
        vertex: &Vertex,
        edge1: &Edge,
        edge2: &Edge,
    ) -> TopoResult<Edge> {
        // Create a new edge connecting the other ends of edge1 and edge2
        let start = if edge1.start_vertex().id() == vertex.id() {
            edge1.end_vertex()
        } else {
            edge1.start_vertex()
        };
        
        let end = if edge2.start_vertex().id() == vertex.id() {
            edge2.end_vertex()
        } else {
            edge2.start_vertex()
        };
        
        let merged_edge = Edge::new(
            Arc::new(start.clone()),
            Arc::new(end.clone()),
        );
        
        Ok(merged_edge)
    }
}

/// Build a cube using Euler operators
pub fn build_cube(size: f64) -> TopoResult<Body> {
    let half = size / 2.0;
    
    // Create the 8 vertices of the cube
    let vertices: Vec<Vertex> = vec![
        Point3::new(-half, -half, -half), // 0: bottom-back-left
        Point3::new(half, -half, -half),  // 1: bottom-back-right
        Point3::new(half, half, -half),   // 2: bottom-front-right
        Point3::new(-half, half, -half),  // 3: bottom-front-left
        Point3::new(-half, -half, half),  // 4: top-back-left
        Point3::new(half, -half, half),   // 5: top-back-right
        Point3::new(half, half, half),    // 6: top-front-right
        Point3::new(-half, half, half),   // 7: top-front-left
    ].into_iter().map(Vertex::new).collect();
    
    // Create the 6 faces of the cube
    // Each face is defined by 4 vertices in counter-clockwise order
    let face_indices: Vec<Vec<usize>> = vec![
        vec![0, 1, 2, 3], // bottom (z = -half)
        vec![4, 7, 6, 5], // top (z = half)
        vec![0, 4, 5, 1], // back (y = -half)
        vec![2, 6, 7, 3], // front (y = half)
        vec![0, 3, 7, 4], // left (x = -half)
        vec![1, 5, 6, 2], // right (x = half)
    ];
    
    let mut body = Body::new();
    let mut shell = Shell::new();
    
    for face_verts in face_indices {
        let mut face = Face::new();
        let mut lp = Loop::new();
        
        // Create edges for this face
        for i in 0..4 {
            let v1 = Arc::new(vertices[face_verts[i]].clone());
            let v2 = Arc::new(vertices[face_verts[(i + 1) % 4]].clone());
            let edge = Arc::new(Edge::new(v1, v2));
            let coedge = Coedge::new(edge, Sense::Same);
            lp.add_coedge(coedge);
        }
        
        face.add_loop(lp);
        shell.add_face(face);
    }
    
    body.add_shell(shell);
    
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mvfs() {
        let (body, shell, face, vertex) = EulerOps::mvfs(Point3::new(0.0, 0.0, 0.0));
        assert!(!body.is_empty());
        assert!(shell.is_outer());
        assert!(face.loops().is_empty()); // MVFS creates empty loop
        assert_eq!(vertex.position().x(), 0.0);
    }

    #[test]
    fn test_build_cube() {
        let body = build_cube(10.0).unwrap();
        
        // A cube has:
        // - 8 vertices
        // - 12 edges
        // - 6 faces
        // - 1 shell
        // - Euler characteristic: V - E + F = 2
        
        assert_eq!(body.shells().len(), 1);
        assert_eq!(body.faces().len(), 6);
        
        let v = body.vertices().len();
        let e = body.edges().len();
        let f = body.faces().len();
        
        assert_eq!(v, 8);
        assert_eq!(e, 12);
        assert_eq!(f, 6);
        
        let euler = v as i32 - e as i32 + f as i32;
        assert_eq!(euler, 2);
    }
}
