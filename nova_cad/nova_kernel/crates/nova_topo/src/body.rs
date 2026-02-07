//! B-Rep body structures: Body, Shell, Face, Loop, Coedge, Edge, Vertex

use crate::{EntityId, Entity, TopologicalEntity, GeometricEntity, Orientation, Sense, new_entity_id};
use nova_math::{Point3, Vec3, Transform3, BoundingBox3};
use nova_geom::{Curve, Surface};
use std::sync::Arc;

/// A solid body composed of shells
#[derive(Debug, Clone)]
pub struct Body {
    id: EntityId,
    shells: Vec<Shell>,
    transforms: Vec<Transform3>,
}

impl Body {
    /// Create a new empty body
    pub fn new() -> Self {
        Self {
            id: new_entity_id(),
            shells: Vec::new(),
            transforms: Vec::new(),
        }
    }
    
    /// Get all shells
    pub fn shells(&self) -> &[Shell] {
        &self.shells
    }
    
    /// Get mutable shells
    pub fn shells_mut(&mut self) -> &mut Vec<Shell> {
        &mut self.shells
    }
    
    /// Add a shell
    pub fn add_shell(&mut self, shell: Shell) {
        self.shells.push(shell);
    }
    
    /// Get all faces
    pub fn faces(&self) -> Vec<&Face> {
        let mut faces = Vec::new();
        for shell in &self.shells {
            faces.extend(shell.faces());
        }
        faces
    }
    
    /// Get all loops
    pub fn loops(&self) -> Vec<&Loop> {
        let mut loops = Vec::new();
        for shell in &self.shells {
            for face in shell.faces() {
                loops.extend(face.loops());
            }
        }
        loops
    }
    
    /// Get all coedges
    pub fn coedges(&self) -> Vec<&Coedge> {
        let mut coedges = Vec::new();
        for shell in &self.shells {
            for face in shell.faces() {
                for lp in face.loops() {
                    coedges.extend(lp.coedges());
                }
            }
        }
        coedges
    }
    
    /// Get all edges
    pub fn edges(&self) -> Vec<&Edge> {
        let mut edges = Vec::new();
        for shell in &self.shells {
            for face in shell.faces() {
                for lp in face.loops() {
                    for coedge in lp.coedges() {
                        if !edges.iter().any(|e: &&Edge| e.id() == coedge.edge().id()) {
                            edges.push(coedge.edge());
                        }
                    }
                }
            }
        }
        edges
    }
    
    /// Get all vertices
    pub fn vertices(&self) -> Vec<&Vertex> {
        let mut vertices = Vec::new();
        for edge in self.edges() {
            let start = edge.start_vertex();
            let end = edge.end_vertex();
            if !vertices.iter().any(|v: &&Vertex| v.id() == start.id()) {
                vertices.push(start);
            }
            if !vertices.iter().any(|v: &&Vertex| v.id() == end.id()) {
                vertices.push(end);
            }
        }
        vertices
    }
    
    /// Check if the body is a solid (has outer shell)
    pub fn is_solid(&self) -> bool {
        !self.shells.is_empty()
    }
    
    /// Check if the body is empty
    pub fn is_empty(&self) -> bool {
        self.shells.is_empty()
    }
    
    /// Transform the body
    pub fn transform(&mut self, transform: &Transform3) {
        self.transforms.push(*transform);
        // Note: Actual transformation of geometry would be applied lazily
    }
    
    /// Compute bounding box
    pub fn bounding_box(&self) -> BoundingBox3 {
        let mut bbox = BoundingBox3::empty();
        for vertex in self.vertices() {
            bbox.expand(&vertex.position());
        }
        bbox
    }
}

impl Default for Body {
    fn default() -> Self {
        Self::new()
    }
}

impl Entity for Body {
    fn id(&self) -> EntityId {
        self.id
    }
    
    fn entity_type(&self) -> &'static str {
        "Body"
    }
}

/// A shell is a connected set of faces
#[derive(Debug, Clone)]
pub struct Shell {
    id: EntityId,
    faces: Vec<Face>,
    is_outer: bool,
}

impl Shell {
    /// Create a new shell
    pub fn new() -> Self {
        Self {
            id: new_entity_id(),
            faces: Vec::new(),
            is_outer: true,
        }
    }
    
    /// Create an outer shell
    pub fn outer() -> Self {
        Self::new()
    }
    
    /// Create a void shell
    pub fn void() -> Self {
        let mut shell = Self::new();
        shell.is_outer = false;
        shell
    }
    
    /// Get all faces
    pub fn faces(&self) -> &[Face] {
        &self.faces
    }
    
    /// Get mutable faces
    pub fn faces_mut(&mut self) -> &mut Vec<Face> {
        &mut self.faces
    }
    
    /// Add a face
    pub fn add_face(&mut self, face: Face) {
        self.faces.push(face);
    }
    
    /// Check if this is the outer shell
    pub fn is_outer(&self) -> bool {
        self.is_outer
    }
    
    /// Set whether this is the outer shell
    pub fn set_outer(&mut self, outer: bool) {
        self.is_outer = outer;
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}

impl Entity for Shell {
    fn id(&self) -> EntityId {
        self.id
    }
    
    fn entity_type(&self) -> &'static str {
        "Shell"
    }
}

/// A face is a bounded region on a surface
pub struct Face {
    id: EntityId,
    surface: Option<Arc<dyn Surface>>,
    loops: Vec<Loop>,
    orientation: Orientation,
}

impl std::fmt::Debug for Face {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Face")
            .field("id", &self.id)
            .field("surface", &self.surface.is_some())
            .field("loops", &self.loops)
            .field("orientation", &self.orientation)
            .finish()
    }
}

impl Clone for Face {
    fn clone(&self) -> Self {
        Self {
            id: new_entity_id(),
            surface: None, // Cannot clone dyn Surface
            loops: self.loops.clone(),
            orientation: self.orientation,
        }
    }
}

impl Face {
    /// Create a new face
    pub fn new() -> Self {
        Self {
            id: new_entity_id(),
            surface: None,
            loops: Vec::new(),
            orientation: Orientation::Forward,
        }
    }
    
    /// Create a face with surface
    pub fn with_surface(surface: Arc<dyn Surface>) -> Self {
        let mut face = Self::new();
        face.surface = Some(surface);
        face
    }
    
    /// Get all loops
    pub fn loops(&self) -> &[Loop] {
        &self.loops
    }
    
    /// Get mutable loops
    pub fn loops_mut(&mut self) -> &mut Vec<Loop> {
        &mut self.loops
    }
    
    /// Add a loop
    pub fn add_loop(&mut self, lp: Loop) {
        self.loops.push(lp);
    }
    
    /// Get the outer loop (first loop)
    pub fn outer_loop(&self) -> Option<&Loop> {
        self.loops.first()
    }
    
    /// Get inner loops (all loops except first)
    pub fn inner_loops(&self) -> &[Loop] {
        if self.loops.len() > 1 {
            &self.loops[1..]
        } else {
            &[]
        }
    }
    
    /// Get the surface
    pub fn surface(&self) -> Option<&Arc<dyn Surface>> {
        self.surface.as_ref()
    }
    
    /// Set the surface
    pub fn set_surface(&mut self, surface: Option<Arc<dyn Surface>>) {
        self.surface = surface;
    }
    
    /// Check if the face has a surface
    pub fn has_surface(&self) -> bool {
        self.surface.is_some()
    }
    
    /// Get the normal at a point on the face
    pub fn normal(&self, u: f64, v: f64) -> Option<Vec3> {
        self.surface.as_ref().map(|s| s.normal(u, v))
    }
}

impl Default for Face {
    fn default() -> Self {
        Self::new()
    }
}

impl Entity for Face {
    fn id(&self) -> EntityId {
        self.id
    }
    
    fn entity_type(&self) -> &'static str {
        "Face"
    }
}

impl TopologicalEntity for Face {
    fn body_id(&self) -> EntityId {
        EntityId::NULL // Would be set by parent
    }
    
    fn orientation(&self) -> Orientation {
        self.orientation
    }
    
    fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }
}

impl GeometricEntity for Face {
    type GeomType = Arc<dyn Surface>;
    
    fn geometry(&self) -> Option<&Self::GeomType> {
        self.surface.as_ref()
    }
    
    fn set_geometry(&mut self, geometry: Option<Self::GeomType>) {
        self.surface = geometry;
    }
}

/// A loop is a closed sequence of coedges
#[derive(Debug, Clone)]
pub struct Loop {
    id: EntityId,
    coedges: Vec<Coedge>,
}

impl Loop {
    /// Create a new empty loop
    pub fn new() -> Self {
        Self {
            id: new_entity_id(),
            coedges: Vec::new(),
        }
    }
    
    /// Create a loop from coedges
    pub fn from_coedges(coedges: Vec<Coedge>) -> Self {
        Self {
            id: new_entity_id(),
            coedges,
        }
    }
    
    /// Get all coedges
    pub fn coedges(&self) -> &[Coedge] {
        &self.coedges
    }
    
    /// Get mutable coedges
    pub fn coedges_mut(&mut self) -> &mut Vec<Coedge> {
        &mut self.coedges
    }
    
    /// Add a coedge
    pub fn add_coedge(&mut self, coedge: Coedge) {
        self.coedges.push(coedge);
    }
    
    /// Check if the loop is closed
    pub fn is_closed(&self) -> bool {
        if self.coedges.is_empty() {
            return false;
        }
        
        // Check that end of each coedge connects to start of next
        for i in 0..self.coedges.len() {
            let curr = &self.coedges[i];
            let next = &self.coedges[(i + 1) % self.coedges.len()];
            
            let curr_end = if curr.sense().is_same() {
                curr.edge().end_vertex().position()
            } else {
                curr.edge().start_vertex().position()
            };
            
            let next_start = if next.sense().is_same() {
                next.edge().start_vertex().position()
            } else {
                next.edge().end_vertex().position()
            };
            
            if curr_end.distance_to(&next_start) > 1e-6 {
                return false;
            }
        }
        
        true
    }
    
    /// Get the number of coedges
    pub fn len(&self) -> usize {
        self.coedges.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.coedges.is_empty()
    }
}

impl Default for Loop {
    fn default() -> Self {
        Self::new()
    }
}

impl Entity for Loop {
    fn id(&self) -> EntityId {
        self.id
    }
    
    fn entity_type(&self) -> &'static str {
        "Loop"
    }
}

/// A coedge is an oriented use of an edge within a loop
#[derive(Debug, Clone)]
pub struct Coedge {
    id: EntityId,
    edge: Arc<Edge>,
    sense: Sense,
}

impl Coedge {
    /// Create a new coedge
    pub fn new(edge: Arc<Edge>, sense: Sense) -> Self {
        Self {
            id: new_entity_id(),
            edge,
            sense,
        }
    }
    
    /// Get the edge
    pub fn edge(&self) -> &Edge {
        &self.edge
    }
    
    /// Get the sense
    pub fn sense(&self) -> Sense {
        self.sense
    }
    
    /// Set the sense
    pub fn set_sense(&mut self, sense: Sense) {
        self.sense = sense;
    }
    
    /// Reverse the sense
    pub fn reverse_sense(&mut self) {
        self.sense = self.sense.reverse();
    }
    
    /// Get the start vertex of this coedge
    pub fn start_vertex(&self) -> &Vertex {
        if self.sense.is_same() {
            self.edge.start_vertex()
        } else {
            self.edge.end_vertex()
        }
    }
    
    /// Get the end vertex of this coedge
    pub fn end_vertex(&self) -> &Vertex {
        if self.sense.is_same() {
            self.edge.end_vertex()
        } else {
            self.edge.start_vertex()
        }
    }
}

impl Entity for Coedge {
    fn id(&self) -> EntityId {
        self.id
    }
    
    fn entity_type(&self) -> &'static str {
        "Coedge"
    }
}

/// An edge connects two vertices and has an associated curve
pub struct Edge {
    id: EntityId,
    start_vertex: Arc<Vertex>,
    end_vertex: Arc<Vertex>,
    curve: Option<Arc<dyn Curve>>,
    tolerance: f64,
    coedges: Vec<EntityId>, // References to coedges using this edge
}

impl std::fmt::Debug for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Edge")
            .field("id", &self.id)
            .field("start_vertex", &self.start_vertex)
            .field("end_vertex", &self.end_vertex)
            .field("curve", &self.curve.is_some())
            .field("tolerance", &self.tolerance)
            .field("coedges", &self.coedges)
            .finish()
    }
}

impl Clone for Edge {
    fn clone(&self) -> Self {
        Self {
            id: new_entity_id(),
            start_vertex: self.start_vertex.clone(),
            end_vertex: self.end_vertex.clone(),
            curve: None, // Cannot clone dyn Curve
            tolerance: self.tolerance,
            coedges: self.coedges.clone(),
        }
    }
}

impl Edge {
    /// Create a new edge
    pub fn new(start: Arc<Vertex>, end: Arc<Vertex>) -> Self {
        Self {
            id: new_entity_id(),
            start_vertex: start,
            end_vertex: end,
            curve: None,
            tolerance: 1e-6,
            coedges: Vec::new(),
        }
    }
    
    /// Create an edge with curve
    pub fn with_curve(start: Arc<Vertex>, end: Arc<Vertex>, curve: Arc<dyn Curve>) -> Self {
        let mut edge = Self::new(start, end);
        edge.curve = Some(curve);
        edge
    }
    
    /// Get the start vertex
    pub fn start_vertex(&self) -> &Vertex {
        &self.start_vertex
    }
    
    /// Get the end vertex
    pub fn end_vertex(&self) -> &Vertex {
        &self.end_vertex
    }
    
    /// Get the curve
    pub fn curve(&self) -> Option<&Arc<dyn Curve>> {
        self.curve.as_ref()
    }
    
    /// Set the curve
    pub fn set_curve(&mut self, curve: Option<Arc<dyn Curve>>) {
        self.curve = curve;
    }
    
    /// Get the tolerance
    pub fn tolerance(&self) -> f64 {
        self.tolerance
    }
    
    /// Set the tolerance
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.tolerance = tolerance;
    }
    
    /// Get coedges using this edge
    pub fn coedges(&self) -> &[EntityId] {
        &self.coedges
    }
    
    /// Add a coedge reference
    pub fn add_coedge(&mut self, coedge_id: EntityId) {
        if !self.coedges.contains(&coedge_id) {
            self.coedges.push(coedge_id);
        }
    }
    
    /// Remove a coedge reference
    pub fn remove_coedge(&mut self, coedge_id: EntityId) {
        self.coedges.retain(|&id| id != coedge_id);
    }
    
    /// Get the length of the edge
    pub fn length(&self) -> f64 {
        if let Some(curve) = &self.curve {
            let range = curve.param_range();
            curve.arc_length(range.end)
        } else {
            self.start_vertex.position().distance_to(&self.end_vertex.position())
        }
    }
    
    /// Evaluate at parameter t
    pub fn evaluate(&self, t: f64) -> Option<Point3> {
        self.curve.as_ref().map(|c| c.evaluate(t))
    }
    
    /// Check if the edge is degenerate (zero length)
    pub fn is_degenerate(&self) -> bool {
        self.start_vertex.position().distance_to(&self.end_vertex.position()) < self.tolerance
    }
}

impl Entity for Edge {
    fn id(&self) -> EntityId {
        self.id
    }
    
    fn entity_type(&self) -> &'static str {
        "Edge"
    }
}

impl GeometricEntity for Edge {
    type GeomType = Arc<dyn Curve>;
    
    fn geometry(&self) -> Option<&Self::GeomType> {
        self.curve.as_ref()
    }
    
    fn set_geometry(&mut self, geometry: Option<Self::GeomType>) {
        self.curve = geometry;
    }
}

/// A vertex is a point in 3D space
#[derive(Debug, Clone)]
pub struct Vertex {
    id: EntityId,
    position: Point3,
    tolerance: f64,
    edges: Vec<EntityId>, // References to edges connected to this vertex
}

impl Vertex {
    /// Create a new vertex
    pub fn new(position: Point3) -> Self {
        Self {
            id: new_entity_id(),
            position,
            tolerance: 1e-6,
            edges: Vec::new(),
        }
    }
    
    /// Get the position
    pub fn position(&self) -> Point3 {
        self.position
    }
    
    /// Set the position
    pub fn set_position(&mut self, position: Point3) {
        self.position = position;
    }
    
    /// Get the tolerance
    pub fn tolerance(&self) -> f64 {
        self.tolerance
    }
    
    /// Set the tolerance
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.tolerance = tolerance;
    }
    
    /// Get connected edges
    pub fn edges(&self) -> &[EntityId] {
        &self.edges
    }
    
    /// Add an edge reference
    pub fn add_edge(&mut self, edge_id: EntityId) {
        if !self.edges.contains(&edge_id) {
            self.edges.push(edge_id);
        }
    }
    
    /// Remove an edge reference
    pub fn remove_edge(&mut self, edge_id: EntityId) {
        self.edges.retain(|&id| id != edge_id);
    }
    
    /// Get the degree (number of connected edges)
    pub fn degree(&self) -> usize {
        self.edges.len()
    }
    
    /// Check if this vertex coincides with another
    pub fn coincides_with(&self, other: &Vertex) -> bool {
        self.position.distance_to(&other.position) < self.tolerance.max(other.tolerance)
    }
}

impl Entity for Vertex {
    fn id(&self) -> EntityId {
        self.id
    }
    
    fn entity_type(&self) -> &'static str {
        "Vertex"
    }
}

impl GeometricEntity for Vertex {
    type GeomType = Point3;
    
    fn geometry(&self) -> Option<&Self::GeomType> {
        Some(&self.position)
    }
    
    fn set_geometry(&mut self, geometry: Option<Self::GeomType>) {
        if let Some(pos) = geometry {
            self.position = pos;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex() {
        let v = Vertex::new(Point3::new(1.0, 2.0, 3.0));
        assert_eq!(v.position().x(), 1.0);
        assert_eq!(v.position().y(), 2.0);
        assert_eq!(v.position().z(), 3.0);
    }

    #[test]
    fn test_edge() {
        let v1 = Arc::new(Vertex::new(Point3::new(0.0, 0.0, 0.0)));
        let v2 = Arc::new(Vertex::new(Point3::new(1.0, 0.0, 0.0)));
        let edge = Edge::new(v1, v2);
        assert_eq!(edge.length(), 1.0);
    }

    #[test]
    fn test_loop() {
        let v1 = Arc::new(Vertex::new(Point3::new(0.0, 0.0, 0.0)));
        let v2 = Arc::new(Vertex::new(Point3::new(1.0, 0.0, 0.0)));
        let v3 = Arc::new(Vertex::new(Point3::new(1.0, 1.0, 0.0)));
        let v4 = Arc::new(Vertex::new(Point3::new(0.0, 1.0, 0.0)));
        
        let e1 = Arc::new(Edge::new(v1.clone(), v2.clone()));
        let e2 = Arc::new(Edge::new(v2.clone(), v3.clone()));
        let e3 = Arc::new(Edge::new(v3.clone(), v4.clone()));
        let e4 = Arc::new(Edge::new(v4.clone(), v1.clone()));
        
        let c1 = Coedge::new(e1, Sense::Same);
        let c2 = Coedge::new(e2, Sense::Same);
        let c3 = Coedge::new(e3, Sense::Same);
        let c4 = Coedge::new(e4, Sense::Same);
        
        let lp = Loop::from_coedges(vec![c1, c2, c3, c4]);
        assert!(lp.is_closed());
    }

    #[test]
    fn test_body() {
        let mut body = Body::new();
        let shell = Shell::new();
        body.add_shell(shell);
        assert!(body.is_solid());
        assert!(!body.is_empty());
    }
}
