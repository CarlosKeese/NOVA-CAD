//! STEP AP214/AP242 Reader and Writer
//!
//! Implements ISO 10303 (STEP) file format support for CAD data exchange.

use crate::{IoError, IoResult, ImportOptions, ExportOptions};
use nova_topo::{Body, Shell, Face, Loop, Coedge, Edge, Vertex, EulerOps, Sense, Orientation, Entity, new_entity_id};
use nova_math::{Point3, Vec3, Transform3};
use nova_geom::{Curve, Surface, PlanarSurface, CylindricalSurface, SphericalSurface, ConicalSurface, Line, CircularArc};
use nova_math::Plane;
use std::collections::HashMap;
use std::sync::Arc;

/// STEP reader
#[derive(Debug, Clone)]
pub struct StepReader {
    /// Schema version (AP214 or AP242)
    pub schema: StepSchema,
}

/// STEP writer
#[derive(Debug, Clone)]
pub struct StepWriter {
    /// Schema version
    pub schema: StepSchema,
}

/// STEP schema versions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepSchema {
    /// AP214 - Automotive design
    AP214,
    /// AP242 - Managed model based 3D engineering
    AP242,
}

impl StepSchema {
    /// Get schema identifier string
    pub fn identifier(&self) -> &'static str {
        match self {
            StepSchema::AP214 => "AUTOMOTIVE_DESIGN",
            StepSchema::AP242 => "AP242_MANAGED_MODEL_BASED_3D_ENGINEERING",
        }
    }
}

/// STEP error types
#[derive(Debug, thiserror::Error, Clone)]
pub enum StepError {
    /// Syntax error in STEP file
    #[error("STEP syntax error: {0}")]
    SyntaxError(String),
    
    /// Entity not found
    #[error("Entity not found: #{0}")]
    EntityNotFound(u64),
    
    /// Unsupported entity
    #[error("Unsupported entity: {0}")]
    UnsupportedEntity(String),
    
    /// Missing required attribute
    #[error("Missing required attribute: {0}")]
    MissingAttribute(String),
    
    /// Invalid geometry
    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),
}

/// STEP entity representation
#[derive(Debug, Clone)]
pub struct StepEntity {
    /// Entity ID
    pub id: u64,
    /// Entity type name
    pub entity_type: String,
    /// Entity attributes
    pub attributes: Vec<StepAttribute>,
}

/// STEP attribute types
#[derive(Debug, Clone)]
pub enum StepAttribute {
    /// Integer value
    Integer(i64),
    /// Real (floating point) value
    Real(f64),
    /// String value
    String(String),
    /// Boolean value
    Boolean(bool),
    /// Enumeration value
    Enumeration(String),
    /// Entity reference (#123)
    Reference(u64),
    /// List of attributes
    List(Vec<StepAttribute>),
    /// Missing/undefined value
    Undefined,
    /// Derived value
    Derived,
}

/// Parsed STEP file
#[derive(Debug, Clone)]
pub struct StepFile {
    /// Header section
    pub header: StepHeader,
    /// Data section entities
    pub entities: HashMap<u64, StepEntity>,
}

/// STEP header data
#[derive(Debug, Clone, Default)]
pub struct StepHeader {
    /// File description
    pub description: Vec<String>,
    /// Implementation level
    pub implementation_level: String,
    /// File name
    pub name: String,
    /// Time stamp
    pub time_stamp: String,
    /// Author
    pub author: Vec<String>,
    /// Organization
    pub organization: Vec<String>,
    /// Preprocessor version
    pub preprocessor_version: String,
    /// Originating system
    pub originating_system: String,
    /// Authorization
    pub authorization: String,
    /// Schema names
    pub schema_names: Vec<String>,
}

/// Converter from STEP to B-Rep
struct StepToBrepsConverter<'a> {
    step_file: &'a StepFile,
    options: &'a ImportOptions,
    vertex_map: HashMap<u64, Arc<Vertex>>,
    edge_map: HashMap<u64, Arc<Edge>>,
    surface_map: HashMap<u64, Arc<dyn Surface>>,
}

impl<'a> StepToBrepsConverter<'a> {
    fn new(step_file: &'a StepFile, options: &'a ImportOptions) -> Self {
        Self {
            step_file,
            options,
            vertex_map: HashMap::new(),
            edge_map: HashMap::new(),
            surface_map: HashMap::new(),
        }
    }
    
    /// Convert STEP file to bodies
    fn convert(&mut self) -> IoResult<Vec<Body>> {
        let mut bodies = Vec::new();
        
        // Find all MANIFOLD_SOLID_BREP entities
        for (id, entity) in &self.step_file.entities {
            if entity.entity_type == "MANIFOLD_SOLID_BREP" {
                match self.convert_manifold_solid_brep(*id) {
                    Ok(body) => bodies.push(body),
                    Err(e) => eprintln!("Warning: Failed to convert entity #{}: {}", id, e),
                }
            }
        }
        
        if bodies.is_empty() {
            // Try BREP_WITH_VOIDS or other solid representations
            for (id, entity) in &self.step_file.entities {
                if entity.entity_type == "BREP_WITH_VOIDS" {
                    match self.convert_brep_with_voids(*id) {
                        Ok(body) => bodies.push(body),
                        Err(e) => eprintln!("Warning: Failed to convert entity #{}: {}", id, e),
                    }
                }
            }
        }
        
        Ok(bodies)
    }
    
    /// Convert MANIFOLD_SOLID_BREP to Body
    fn convert_manifold_solid_brep(&mut self, id: u64) -> IoResult<Body> {
        let entity = self.get_entity(id)?;
        
        // MANIFOLD_SOLID_BREP(name, outer)
        if entity.attributes.len() < 2 {
            return Err(IoError::StepError("MANIFOLD_SOLID_BREP missing attributes".to_string()));
        }
        
        let outer_ref = entity.attributes[1].as_reference()
            .ok_or_else(|| IoError::StepError("Invalid outer shell reference".to_string()))?;
        
        let shell = self.convert_closed_shell(outer_ref)?;
        
        let mut body = Body::new();
        body.add_shell(shell);
        
        Ok(body)
    }
    
    /// Convert BREP_WITH_VOIDS to Body
    fn convert_brep_with_voids(&mut self, id: u64) -> IoResult<Body> {
        let entity = self.get_entity(id)?;
        
        // BREP_WITH_VOIDS(name, outer, voids)
        if entity.attributes.len() < 3 {
            return Err(IoError::StepError("BREP_WITH_VOIDS missing attributes".to_string()));
        }
        
        let outer_ref = entity.attributes[1].as_reference()
            .ok_or_else(|| IoError::StepError("Invalid outer shell reference".to_string()))?;
        let void_refs = match &entity.attributes[2] {
            StepAttribute::List(refs) => refs.clone(),
            _ => Vec::new(),
        };
        
        let mut body = Body::new();
        
        // Convert outer shell
        let outer_shell = self.convert_closed_shell(outer_ref)?;
        body.add_shell(outer_shell);
        
        // Convert void shells
        for void_ref in void_refs {
            if let StepAttribute::Reference(void_id) = void_ref {
                let void_shell = self.convert_closed_shell(void_id)?;
                body.add_shell(void_shell);
            }
        }
        
        Ok(body)
    }
    
    /// Convert CLOSED_SHELL to Shell
    fn convert_closed_shell(&mut self, id: u64) -> IoResult<Shell> {
        let entity = self.get_entity(id)?;
        
        // CLOSED_SHELL(name, faces)
        if entity.attributes.len() < 2 {
            return Err(IoError::StepError("CLOSED_SHELL missing attributes".to_string()));
        }
        
        let face_refs = match &entity.attributes[1] {
            StepAttribute::List(refs) => refs.clone(),
            _ => Vec::new(),
        };
        
        let mut shell = Shell::new();
        
        for face_ref in face_refs {
            if let StepAttribute::Reference(face_id) = face_ref {
                let face = self.convert_advanced_face(face_id)?;
                shell.add_face(face);
            }
        }
        
        Ok(shell)
    }
    
    /// Convert ADVANCED_FACE to Face
    fn convert_advanced_face(&mut self, id: u64) -> IoResult<Face> {
        let entity = self.get_entity(id)?;
        
        // ADVANCED_FACE(name, bounds, surface, same_sense)
        if entity.attributes.len() < 4 {
            return Err(IoError::StepError("ADVANCED_FACE missing attributes".to_string()));
        }
        
        // Get surface
        let surface_ref = entity.attributes[2].as_reference()
            .ok_or_else(|| IoError::StepError("Invalid surface reference".to_string()))?;
        let bound_refs = match &entity.attributes[1] {
            StepAttribute::List(refs) => refs.clone(),
            _ => Vec::new(),
        };
        
        let surface = self.convert_surface(surface_ref)?;
        
        let mut face = Face::with_surface(surface);
        
        // Get bounds (loops)
        for (i, bound_ref) in bound_refs.iter().enumerate() {
            if let StepAttribute::Reference(bound_id) = bound_ref {
                let loop_ = self.convert_face_bound(*bound_id, i == 0)?;
                face.add_loop(loop_);
            }
        }
        
        Ok(face)
    }
    
    /// Convert FACE_BOUND or FACE_OUTER_BOUND to Loop
    fn convert_face_bound(&mut self, id: u64, is_outer: bool) -> IoResult<Loop> {
        let entity = self.get_entity(id)?;
        
        // FACE_BOUND(name, loop, orientation) or FACE_OUTER_BOUND
        if entity.attributes.len() < 3 {
            return Err(IoError::StepError("FACE_BOUND missing attributes".to_string()));
        }
        
        let loop_ref = entity.attributes[1].as_reference()
            .ok_or_else(|| IoError::StepError("Invalid loop reference".to_string()))?;
        
        let loop_ = self.convert_edge_loop(loop_ref)?;
        
        Ok(loop_)
    }
    
    /// Convert EDGE_LOOP to Loop
    fn convert_edge_loop(&mut self, id: u64) -> IoResult<Loop> {
        let entity = self.get_entity(id)?;
        
        // EDGE_LOOP(name, edge_list)
        if entity.attributes.len() < 2 {
            return Err(IoError::StepError("EDGE_LOOP missing attributes".to_string()));
        }
        
        let edge_refs = match &entity.attributes[1] {
            StepAttribute::List(refs) => refs.clone(),
            _ => Vec::new(),
        };
        
        let mut coedges = Vec::new();
        
        for edge_ref in edge_refs {
            if let StepAttribute::Reference(edge_id) = edge_ref {
                let (edge, sense) = self.convert_oriented_edge(edge_id)?;
                let coedge = Coedge::new(edge, sense);
                coedges.push(coedge);
            }
        }
        
        Ok(Loop::from_coedges(coedges))
    }
    
    /// Convert ORIENTED_EDGE to (Edge, Sense)
    fn convert_oriented_edge(&mut self, id: u64) -> IoResult<(Arc<Edge>, Sense)> {
        let entity = self.get_entity(id)?;
        
        // ORIENTED_EDGE(name, edge_element, orientation)
        if entity.attributes.len() < 3 {
            return Err(IoError::StepError("ORIENTED_EDGE missing attributes".to_string()));
        }
        
        let edge_ref = entity.attributes[1].as_reference()
            .ok_or_else(|| IoError::StepError("Invalid edge reference".to_string()))?;
        
        let orientation = entity.attributes[2].as_bool()
            .unwrap_or(true);
        
        let edge = self.convert_edge_curve(edge_ref)?;
        let sense = if orientation { Sense::Same } else { Sense::Opposite };
        
        Ok((edge, sense))
    }
    
    /// Convert EDGE_CURVE to Edge
    fn convert_edge_curve(&mut self, id: u64) -> IoResult<Arc<Edge>> {
        // Check cache
        if let Some(edge) = self.edge_map.get(&id) {
            return Ok(edge.clone());
        }
        
        let entity = self.get_entity(id)?;
        
        // EDGE_CURVE(name, start_vertex, end_vertex, edge_geometry, same_sense)
        if entity.attributes.len() < 5 {
            return Err(IoError::StepError("EDGE_CURVE missing attributes".to_string()));
        }
        
        let start_ref = entity.attributes[1].as_reference()
            .ok_or_else(|| IoError::StepError("Invalid start vertex reference".to_string()))?;
        let end_ref = entity.attributes[2].as_reference()
            .ok_or_else(|| IoError::StepError("Invalid end vertex reference".to_string()))?;
        let curve_ref = entity.attributes[3].as_reference()
            .ok_or_else(|| IoError::StepError("Invalid curve reference".to_string()))?;
        
        let start = self.convert_vertex_point(start_ref)?;
        let end = self.convert_vertex_point(end_ref)?;
        let curve = self.convert_curve(curve_ref)?;
        
        let edge = Arc::new(Edge::with_curve(start, end, curve));
        self.edge_map.insert(id, edge.clone());
        
        Ok(edge)
    }
    
    /// Convert VERTEX_POINT to Vertex
    fn convert_vertex_point(&mut self, id: u64) -> IoResult<Arc<Vertex>> {
        // Check cache
        if let Some(vertex) = self.vertex_map.get(&id) {
            return Ok(vertex.clone());
        }
        
        let entity = self.get_entity(id)?;
        
        // VERTEX_POINT(name, vertex_geometry)
        if entity.attributes.len() < 2 {
            return Err(IoError::StepError("VERTEX_POINT missing attributes".to_string()));
        }
        
        let geom_ref = entity.attributes[1].as_reference()
            .ok_or_else(|| IoError::StepError("Invalid vertex geometry reference".to_string()))?;
        
        let point = self.convert_cartesian_point(geom_ref)?;
        let vertex = Arc::new(Vertex::new(point));
        
        self.vertex_map.insert(id, vertex.clone());
        Ok(vertex)
    }
    
    /// Convert CARTESIAN_POINT to Point3
    fn convert_cartesian_point(&self, id: u64) -> IoResult<Point3> {
        let entity = self.get_entity(id)?;
        
        // CARTESIAN_POINT(name, coordinates)
        if entity.attributes.len() < 2 {
            return Err(IoError::StepError("CARTESIAN_POINT missing attributes".to_string()));
        }
        
        if let StepAttribute::List(coords) = &entity.attributes[1] {
            let x = coords.get(0).and_then(|c| c.as_real()).unwrap_or(0.0);
            let y = coords.get(1).and_then(|c| c.as_real()).unwrap_or(0.0);
            let z = coords.get(2).and_then(|c| c.as_real()).unwrap_or(0.0);
            
            // Apply unit conversion
            let scale = self.options.target_units.to_mm_factor();
            Ok(Point3::new(x * scale, y * scale, z * scale))
        } else {
            Err(IoError::StepError("Invalid CARTESIAN_POINT coordinates".to_string()))
        }
    }
    
    /// Convert surface entity
    fn convert_surface(&mut self, id: u64) -> IoResult<Arc<dyn Surface>> {
        // Check cache
        if let Some(surface) = self.surface_map.get(&id) {
            return Ok(surface.clone());
        }
        
        let entity = self.get_entity(id)?;
        
        let surface: Arc<dyn Surface> = match entity.entity_type.as_str() {
            "PLANE" => self.convert_plane(id)?,
            "CYLINDRICAL_SURFACE" => self.convert_cylindrical_surface(id)?,
            "SPHERICAL_SURFACE" => self.convert_spherical_surface(id)?,
            "CONICAL_SURFACE" => self.convert_conical_surface(id)?,
            _ => return Err(IoError::StepError(
                format!("Unsupported surface type: {}", entity.entity_type)
            )),
        };
        
        self.surface_map.insert(id, surface.clone());
        Ok(surface)
    }
    
    /// Convert PLANE
    fn convert_plane(&self, id: u64) -> IoResult<Arc<dyn Surface>> {
        let entity = self.get_entity(id)?;
        
        // PLANE(name, position)
        if entity.attributes.len() < 2 {
            return Err(IoError::StepError("PLANE missing attributes".to_string()));
        }
        
        let position_ref = entity.attributes[1].as_reference()
            .ok_or_else(|| IoError::StepError("Invalid position reference".to_string()))?;
        
        // Get axis placement
        let (origin, normal, ref_direction) = self.convert_axis2_placement_3d_with_ref(position_ref)?;
        
        let plane = PlanarSurface::from_plane(&Plane::new(origin, normal));
        Ok(Arc::new(plane))
    }
    
    /// Convert CYLINDRICAL_SURFACE
    fn convert_cylindrical_surface(&self, id: u64) -> IoResult<Arc<dyn Surface>> {
        let entity = self.get_entity(id)?;
        
        // CYLINDRICAL_SURFACE(name, position, radius)
        if entity.attributes.len() < 3 {
            return Err(IoError::StepError("CYLINDRICAL_SURFACE missing attributes".to_string()));
        }
        
        let position_ref = entity.attributes[1].as_reference()
            .ok_or_else(|| IoError::StepError("Invalid position reference".to_string()))?;
        let radius = entity.attributes[2].as_real()
            .ok_or_else(|| IoError::StepError("Invalid radius".to_string()))?;
        
        let (origin, axis, ref_direction) = self.convert_axis2_placement_3d_with_ref(position_ref)?;
        
        let scale = self.options.target_units.to_mm_factor();
        let cylinder = CylindricalSurface::new(origin, axis, radius * scale, ref_direction)?;
        Ok(Arc::new(cylinder))
    }
    
    /// Convert SPHERICAL_SURFACE
    fn convert_spherical_surface(&self, id: u64) -> IoResult<Arc<dyn Surface>> {
        let entity = self.get_entity(id)?;
        
        // SPHERICAL_SURFACE(name, position, radius)
        if entity.attributes.len() < 3 {
            return Err(IoError::StepError("SPHERICAL_SURFACE missing attributes".to_string()));
        }
        
        let position_ref = entity.attributes[1].as_reference()
            .ok_or_else(|| IoError::StepError("Invalid position reference".to_string()))?;
        let radius = entity.attributes[2].as_real()
            .ok_or_else(|| IoError::StepError("Invalid radius".to_string()))?;
        
        let (center, axis, ref_direction) = self.convert_axis2_placement_3d_with_ref(position_ref)?;
        
        let scale = self.options.target_units.to_mm_factor();
        let sphere = SphericalSurface::new(center, radius * scale, axis, ref_direction)?;
        Ok(Arc::new(sphere))
    }
    
    /// Convert CONICAL_SURFACE
    fn convert_conical_surface(&self, id: u64) -> IoResult<Arc<dyn Surface>> {
        let entity = self.get_entity(id)?;
        
        // CONICAL_SURFACE(name, position, radius, semi_angle)
        if entity.attributes.len() < 4 {
            return Err(IoError::StepError("CONICAL_SURFACE missing attributes".to_string()));
        }
        
        let position_ref = entity.attributes[1].as_reference()
            .ok_or_else(|| IoError::StepError("Invalid position reference".to_string()))?;
        let radius = entity.attributes[2].as_real()
            .ok_or_else(|| IoError::StepError("Invalid radius".to_string()))?;
        let semi_angle = entity.attributes[3].as_real()
            .ok_or_else(|| IoError::StepError("Invalid semi_angle".to_string()))?;
        
        let (apex, axis, ref_direction) = self.convert_axis2_placement_3d_with_ref(position_ref)?;
        
        let scale = self.options.target_units.to_mm_factor();
        let cone = ConicalSurface::new(apex, axis, semi_angle, ref_direction)?;
        Ok(Arc::new(cone))
    }
    
    /// Convert AXIS2_PLACEMENT_3D to (origin, z_axis)
    fn convert_axis2_placement_3d(&self, id: u64) -> IoResult<(Point3, Vec3)> {
        let entity = self.get_entity(id)?;
        
        // AXIS2_PLACEMENT_3D(name, location, axis, ref_direction)
        if entity.attributes.len() < 3 {
            return Err(IoError::StepError("AXIS2_PLACEMENT_3D missing attributes".to_string()));
        }
        
        let location_ref = entity.attributes[1].as_reference()
            .ok_or_else(|| IoError::StepError("Invalid location reference".to_string()))?;
        let axis_ref = entity.attributes[2].as_reference()
            .ok_or_else(|| IoError::StepError("Invalid axis reference".to_string()))?;
        
        let origin = self.convert_cartesian_point(location_ref)?;
        let axis = self.convert_direction(axis_ref)?;
        
        Ok((origin, axis))
    }
    
    /// Convert AXIS2_PLACEMENT_3D to (origin, z_axis, ref_direction)
    fn convert_axis2_placement_3d_with_ref(&self, id: u64) -> IoResult<(Point3, Vec3, Vec3)> {
        let entity = self.get_entity(id)?;
        
        // AXIS2_PLACEMENT_3D(name, location, axis, ref_direction)
        if entity.attributes.len() < 4 {
            return Err(IoError::StepError("AXIS2_PLACEMENT_3D missing attributes".to_string()));
        }
        
        let location_ref = entity.attributes[1].as_reference()
            .ok_or_else(|| IoError::StepError("Invalid location reference".to_string()))?;
        let axis_ref = entity.attributes[2].as_reference()
            .ok_or_else(|| IoError::StepError("Invalid axis reference".to_string()))?;
        let ref_dir_ref = entity.attributes[3].as_reference()
            .ok_or_else(|| IoError::StepError("Invalid ref_direction reference".to_string()))?;
        
        let origin = self.convert_cartesian_point(location_ref)?;
        let axis = self.convert_direction(axis_ref)?;
        let ref_direction = self.convert_direction(ref_dir_ref)?;
        
        Ok((origin, axis, ref_direction))
    }
    
    /// Convert DIRECTION to Vec3
    fn convert_direction(&self, id: u64) -> IoResult<Vec3> {
        let entity = self.get_entity(id)?;
        
        // DIRECTION(name, direction_ratios)
        if entity.attributes.len() < 2 {
            return Err(IoError::StepError("DIRECTION missing attributes".to_string()));
        }
        
        if let StepAttribute::List(ratios) = &entity.attributes[1] {
            let x = ratios.get(0).and_then(|r| r.as_real()).unwrap_or(0.0);
            let y = ratios.get(1).and_then(|r| r.as_real()).unwrap_or(0.0);
            let z = ratios.get(2).and_then(|r| r.as_real()).unwrap_or(0.0);
            
            Ok(Vec3::new(x, y, z).normalized())
        } else {
            Err(IoError::StepError("Invalid DIRECTION ratios".to_string()))
        }
    }
    
    /// Convert curve entity
    fn convert_curve(&mut self, id: u64) -> IoResult<Arc<dyn Curve>> {
        let entity = self.get_entity(id)?;
        
        match entity.entity_type.as_str() {
            "LINE" => self.convert_line(id),
            "CIRCLE" => self.convert_circle(id),
            "B_SPLINE_CURVE_WITH_KNOTS" => self.convert_b_spline_curve(id),
            _ => Err(IoError::StepError(
                format!("Unsupported curve type: {}", entity.entity_type)
            )),
        }
    }
    
    /// Convert LINE
    fn convert_line(&self, id: u64) -> IoResult<Arc<dyn Curve>> {
        let entity = self.get_entity(id)?;
        
        // LINE(name, point, direction)
        if entity.attributes.len() < 3 {
            return Err(IoError::StepError("LINE missing attributes".to_string()));
        }
        
        let point_ref = entity.attributes[1].as_reference()
            .ok_or_else(|| IoError::StepError("Invalid point reference".to_string()))?;
        let direction_ref = entity.attributes[2].as_reference()
            .ok_or_else(|| IoError::StepError("Invalid direction reference".to_string()))?;
        
        let point = self.convert_cartesian_point(point_ref)?;
        let direction = self.convert_direction(direction_ref)?;
        
        let line = Line::infinite(point, direction)?;
        Ok(Arc::new(line))
    }
    
    /// Convert CIRCLE
    fn convert_circle(&self, id: u64) -> IoResult<Arc<dyn Curve>> {
        let entity = self.get_entity(id)?;
        
        // CIRCLE(name, position, radius)
        if entity.attributes.len() < 3 {
            return Err(IoError::StepError("CIRCLE missing attributes".to_string()));
        }
        
        let position_ref = entity.attributes[1].as_reference()
            .ok_or_else(|| IoError::StepError("Invalid position reference".to_string()))?;
        let radius = entity.attributes[2].as_real()
            .ok_or_else(|| IoError::StepError("Invalid radius".to_string()))?;
        
        let (center, axis) = self.convert_axis2_placement_3d(position_ref)?;
        
        let scale = self.options.target_units.to_mm_factor();
        let circle = CircularArc::circle(center, radius * scale, axis)?;
        Ok(Arc::new(circle))
    }
    
    /// Convert B_SPLINE_CURVE_WITH_KNOTS
    fn convert_b_spline_curve(&self, id: u64) -> IoResult<Arc<dyn Curve>> {
        // TODO: Implement B-spline curve conversion
        Err(IoError::StepError("B-spline curves not yet implemented".to_string()))
    }
    
    /// Get entity by ID
    fn get_entity(&self, id: u64) -> IoResult<&StepEntity> {
        self.step_file.entities.get(&id)
            .ok_or_else(|| IoError::StepError(format!("Entity #{} not found", id)))
    }
}

// Helper methods for StepAttribute
impl StepAttribute {
    fn as_reference(&self) -> Option<u64> {
        match self {
            StepAttribute::Reference(id) => Some(*id),
            _ => None,
        }
    }
    
    fn as_real(&self) -> Option<f64> {
        match self {
            StepAttribute::Real(r) => Some(*r),
            StepAttribute::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }
    
    fn as_bool(&self) -> Option<bool> {
        match self {
            StepAttribute::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

impl StepReader {
    /// Create a new STEP reader
    pub fn new() -> Self {
        Self {
            schema: StepSchema::AP214,
        }
    }
    
    /// Read STEP file content and return bodies
    pub fn read(&self, content: &str, options: &ImportOptions) -> IoResult<Vec<Body>> {
        let step_file = self.parse(content)?;
        
        let mut converter = StepToBrepsConverter::new(&step_file, options);
        let bodies = converter.convert()?;
        
        Ok(bodies)
    }
    
    /// Parse STEP file content
    fn parse(&self, content: &str) -> IoResult<StepFile> {
        if !content.contains("ISO-10303-21") {
            return Err(IoError::StepError(
                "Not a valid STEP file (missing ISO-10303-21 header)".to_string()
            ));
        }
        
        let mut header = StepHeader::default();
        let mut entities = HashMap::new();
        
        let sections: Vec<&str> = content.split("ENDSEC;").collect();
        
        for section in sections {
            let section = section.trim();
            
            if section.starts_with("HEADER") {
                header = self.parse_header(section)?;
            } else if section.starts_with("DATA") {
                entities = self.parse_data(section)?;
            }
        }
        
        Ok(StepFile { header, entities })
    }
    
    /// Parse HEADER section
    fn parse_header(&self, section: &str) -> IoResult<StepHeader> {
        let mut header = StepHeader::default();
        
        if let Some(start) = section.find("FILE_DESCRIPTION") {
            if let Some(desc_start) = section[start..].find("('") {
                let desc_start = start + desc_start + 1;
                if let Some(desc_end) = section[desc_start..].find("')") {
                    let desc = &section[desc_start..desc_start + desc_end];
                    header.description = desc.split("','")
                        .map(|s| s.trim().trim_matches('\'').to_string())
                        .collect();
                }
            }
        }
        
        if let Some(start) = section.find("FILE_NAME") {
            if let Some(name_start) = section[start..].find("'") {
                let name_start = start + name_start + 1;
                if let Some(name_end) = section[name_start..].find("'") {
                    header.name = section[name_start..name_start + name_end].to_string();
                }
            }
        }
        
        if let Some(start) = section.find("FILE_SCHEMA") {
            if let Some(schema_start) = section[start..].find("'") {
                let schema_start = start + schema_start + 1;
                if let Some(schema_end) = section[schema_start..].find("'") {
                    let schema = &section[schema_start..schema_start + schema_end];
                    header.schema_names = schema.split("','")
                        .map(|s| s.trim().trim_matches('\'').to_string())
                        .collect();
                }
            }
        }
        
        Ok(header)
    }
    
    /// Parse DATA section
    fn parse_data(&self, section: &str) -> IoResult<HashMap<u64, StepEntity>> {
        let mut entities = HashMap::new();
        
        for line in section.lines() {
            let line = line.trim();
            
            if line.starts_with('#') {
                if let Some(eq_pos) = line.find('=') {
                    let id_str = &line[1..eq_pos];
                    if let Ok(id) = id_str.parse::<u64>() {
                        let entity_start = eq_pos + 1;
                        if let Some(entity) = self.parse_entity_line(id, &line[entity_start..]) {
                            entities.insert(id, entity);
                        }
                    }
                }
            }
        }
        
        Ok(entities)
    }
    
    /// Parse a single entity line
    fn parse_entity_line(&self, id: u64, line: &str) -> Option<StepEntity> {
        let line = line.trim().trim_end_matches(';');
        
        if let Some(paren_pos) = line.find('(') {
            let entity_type = line[..paren_pos].trim().to_string();
            let attr_str = &line[paren_pos + 1..line.len() - 1];
            
            let attributes = self.parse_attributes(attr_str);
            
            Some(StepEntity {
                id,
                entity_type,
                attributes,
            })
        } else {
            None
        }
    }
    
    /// Parse entity attributes
    fn parse_attributes(&self, attr_str: &str) -> Vec<StepAttribute> {
        let mut attributes = Vec::new();
        let mut depth = 0;
        let mut current = String::new();
        let mut in_string = false;
        let mut string_char = '"';
        
        for c in attr_str.chars() {
            if in_string {
                current.push(c);
                if c == string_char {
                    in_string = false;
                }
            } else {
                match c {
                    '\'' | '"' => {
                        in_string = true;
                        string_char = c;
                        current.push(c);
                    }
                    '(' => {
                        depth += 1;
                        current.push(c);
                    }
                    ')' => {
                        depth -= 1;
                        current.push(c);
                    }
                    ',' if depth == 0 => {
                        if !current.trim().is_empty() {
                            attributes.push(self.parse_single_attribute(&current));
                        }
                        current.clear();
                    }
                    _ => current.push(c),
                }
            }
        }
        
        if !current.trim().is_empty() {
            attributes.push(self.parse_single_attribute(&current));
        }
        
        attributes
    }
    
    /// Parse a single attribute value
    fn parse_single_attribute(&self, value: &str) -> StepAttribute {
        let value = value.trim();
        
        if value.starts_with('#') {
            if let Ok(id) = value[1..].parse::<u64>() {
                return StepAttribute::Reference(id);
            }
        }
        
        if value.starts_with('"') && value.ends_with('"') {
            return StepAttribute::String(value[1..value.len()-1].to_string());
        }
        if value.starts_with('\'') && value.ends_with('\'') {
            return StepAttribute::String(value[1..value.len()-1].to_string());
        }
        
        if value.starts_with('(') && value.ends_with(')') {
            let inner = &value[1..value.len()-1];
            let items = self.parse_attributes(inner);
            return StepAttribute::List(items);
        }
        
        if value.starts_with('.') && value.ends_with('.') {
            return StepAttribute::Enumeration(value[1..value.len()-1].to_string());
        }
        
        if value == ".T." || value == ".TRUE." {
            return StepAttribute::Boolean(true);
        }
        if value == ".F." || value == ".FALSE." {
            return StepAttribute::Boolean(false);
        }
        
        if value == "$" {
            return StepAttribute::Undefined;
        }
        
        if value == "*" {
            return StepAttribute::Derived;
        }
        
        if let Ok(i) = value.parse::<i64>() {
            return StepAttribute::Integer(i);
        }
        
        if let Ok(f) = value.parse::<f64>() {
            return StepAttribute::Real(f);
        }
        
        StepAttribute::String(value.to_string())
    }
}

impl Default for StepReader {
    fn default() -> Self {
        Self::new()
    }
}

impl StepWriter {
    /// Create a new STEP writer
    pub fn new() -> Self {
        Self {
            schema: StepSchema::AP214,
        }
    }
    
    /// Write bodies to STEP format
    pub fn write(&self, bodies: &[Body], options: &ExportOptions) -> IoResult<String> {
        let mut output = String::new();
        
        self.write_header(&mut output, options)?;
        self.write_data(&mut output, bodies, options)?;
        output.push_str("END-ISO-10303-21;\n");
        
        Ok(output)
    }
    
    /// Write STEP header
    fn write_header(&self, output: &mut String, options: &ExportOptions) -> IoResult<()> {
        output.push_str("ISO-10303-21;\n");
        output.push_str("HEADER;\n");
        
        output.push_str("FILE_DESCRIPTION(('NOVA CAD Model'),'2;1');\n");
        
        let timestamp = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S");
        let author = options.author.as_deref().unwrap_or("Unknown");
        output.push_str(&format!(
            "FILE_NAME('output.step','{}',('{}'),('NOVA CAD'),'NOVA CAD','NOVA CAD','');\n",
            timestamp, author
        ));
        
        output.push_str(&format!("FILE_SCHEMA(('{}'));\n", self.schema.identifier()));
        output.push_str("ENDSEC;\n");
        
        Ok(())
    }
    
    /// Write STEP data section
    fn write_data(&self, output: &mut String, bodies: &[Body], options: &ExportOptions) -> IoResult<()> {
        output.push_str("DATA;\n");
        
        let mut entity_id: u64 = 100;
        let scale = options.units.from_mm_factor();
        
        for body in bodies {
            self.write_body(&mut entity_id, output, body, scale)?;
        }
        
        output.push_str("ENDSEC;\n");
        Ok(())
    }
    
    /// Write a body to STEP
    fn write_body(&self, next_id: &mut u64, output: &mut String, body: &Body, scale: f64) -> IoResult<()> {
        // Write each shell
        let mut shell_ids = Vec::new();
        
        for shell in body.shells() {
            let shell_id = self.write_shell(next_id, output, shell, scale)?;
            shell_ids.push(shell_id);
        }
        
        // Write MANIFOLD_SOLID_BREP or BREP_WITH_VOIDS
        if shell_ids.len() == 1 {
            output.push_str(&format!(
                "#{}=MANIFOLD_SOLID_BREP('Body',#{});\n",
                next_id, shell_ids[0]
            ));
            *next_id += 1;
        } else if shell_ids.len() > 1 {
            let voids: Vec<String> = shell_ids[1..].iter().map(|id| format!("#{}", id)).collect();
            output.push_str(&format!(
                "#{}=BREP_WITH_VOIDS('Body',#{},({}));\n",
                next_id, shell_ids[0], voids.join(",")
            ));
            *next_id += 1;
        }
        
        Ok(())
    }
    
    /// Write a shell to STEP
    fn write_shell(&self, next_id: &mut u64, output: &mut String, shell: &Shell, scale: f64) -> IoResult<u64> {
        let mut face_ids = Vec::new();
        
        for face in shell.faces() {
            let face_id = self.write_face(next_id, output, face, scale)?;
            face_ids.push(face_id);
        }
        
        let shell_id = *next_id;
        let face_refs: Vec<String> = face_ids.iter().map(|id| format!("#{}", id)).collect();
        output.push_str(&format!(
            "#{}=CLOSED_SHELL('Shell',({}));\n",
            shell_id, face_refs.join(","))
        );
        *next_id += 1;
        
        Ok(shell_id)
    }
    
    /// Write a face to STEP
    fn write_face(&self, next_id: &mut u64, output: &mut String, face: &Face, scale: f64) -> IoResult<u64> {
        // Write surface first
        let surface_id = if let Some(surface) = face.surface() {
            self.write_surface(next_id, output, surface.as_ref(), scale)?
        } else {
            return Err(IoError::StepError("Face has no surface".to_string()));
        };
        
        // Write bounds (loops)
        let mut bound_ids = Vec::new();
        let mut is_first = true;
        
        for loop_ in face.loops() {
            let bound_id = self.write_face_bound(next_id, output, loop_, is_first)?;
            bound_ids.push(bound_id);
            is_first = false;
        }
        
        let face_id = *next_id;
        let bound_refs: Vec<String> = bound_ids.iter().map(|id| format!("#{}", id)).collect();
        output.push_str(&format!(
            "#{}=ADVANCED_FACE('Face',({}),#{},.T.);\n",
            face_id, bound_refs.join(","), surface_id)
        );
        *next_id += 1;
        
        Ok(face_id)
    }
    
    /// Write a surface to STEP
    fn write_surface(&self, next_id: &mut u64, output: &mut String, surface: &dyn Surface, scale: f64) -> IoResult<u64> {
        // This would need surface type detection
        // For now, write a placeholder plane
        let surface_id = *next_id;
        
        // Write AXIS2_PLACEMENT_3D
        let axis_id = *next_id + 1;
        let origin_id = *next_id + 2;
        let z_axis_id = *next_id + 3;
        
        output.push_str(&format!("#{}=CARTESIAN_POINT('Origin',(0.0,0.0,0.0));\n", origin_id));
        output.push_str(&format!("#{}=DIRECTION('Z',(0.0,0.0,1.0));\n", z_axis_id));
        output.push_str(&format!("#{}=AXIS2_PLACEMENT_3D('',#{},#{},$);\n", axis_id, origin_id, z_axis_id));
        output.push_str(&format!("#{}=PLANE('',#{});\n", surface_id, axis_id));
        
        *next_id += 4;
        Ok(surface_id)
    }
    
    /// Write a face bound
    fn write_face_bound(&self, next_id: &mut u64, output: &mut String, loop_: &Loop, is_outer: bool) -> IoResult<u64> {
        // Write edge loop
        let loop_id = self.write_edge_loop(next_id, output, loop_)?;
        
        let bound_id = *next_id;
        if is_outer {
            output.push_str(&format!("#{}=FACE_OUTER_BOUND('',#{},.T.);\n", bound_id, loop_id));
        } else {
            output.push_str(&format!("#{}=FACE_BOUND('',#{},.T.);\n", bound_id, loop_id));
        }
        *next_id += 1;
        
        Ok(bound_id)
    }
    
    /// Write an edge loop
    fn write_edge_loop(&self, next_id: &mut u64, output: &mut String, loop_: &Loop) -> IoResult<u64> {
        let mut oriented_edge_ids = Vec::new();
        
        for coedge in loop_.coedges() {
            let edge_id = self.write_oriented_edge(next_id, output, coedge)?;
            oriented_edge_ids.push(edge_id);
        }
        
        let loop_id = *next_id;
        let edge_refs: Vec<String> = oriented_edge_ids.iter().map(|id| format!("#{}", id)).collect();
        output.push_str(&format!(
            "#{}=EDGE_LOOP('',({}));\n",
            loop_id, edge_refs.join(","))
        );
        *next_id += 1;
        
        Ok(loop_id)
    }
    
    /// Write an oriented edge
    fn write_oriented_edge(&self, next_id: &mut u64, output: &mut String, coedge: &Coedge) -> IoResult<u64> {
        let edge_id = self.write_edge_curve(next_id, output, coedge.edge())?;
        
        let oriented_id = *next_id;
        let orientation = if matches!(coedge.sense(), Sense::Same) { ".T." } else { ".F." };
        output.push_str(&format!(
            "#{}=ORIENTED_EDGE('',*,*,#{},{});\n",
            oriented_id, edge_id, orientation)
        );
        *next_id += 1;
        
        Ok(oriented_id)
    }
    
    /// Write an edge curve
    fn write_edge_curve(&self, next_id: &mut u64, output: &mut String, edge: &Edge) -> IoResult<u64> {
        // Write vertices
        let start_id = self.write_vertex_point(next_id, output, edge.start_vertex())?;
        let end_id = self.write_vertex_point(next_id, output, edge.end_vertex())?;
        
        // Write curve (or use line as default)
        let curve_id = if let Some(curve) = edge.curve() {
            self.write_curve(next_id, output, curve.as_ref())?
        } else {
            // Create line between vertices
            let line_id = *next_id;
            // Write line entity
            *next_id += 1;
            line_id
        };
        
        let edge_id = *next_id;
        output.push_str(&format!(
            "#{}=EDGE_CURVE('',#{},#{},#{},.T.);\n",
            edge_id, start_id, end_id, curve_id)
        );
        *next_id += 1;
        
        Ok(edge_id)
    }
    
    /// Write a vertex point
    fn write_vertex_point(&self, next_id: &mut u64, output: &mut String, vertex: &Vertex) -> IoResult<u64> {
        let point_id = self.write_cartesian_point(next_id, output, &vertex.position())?;
        
        let vertex_id = *next_id;
        output.push_str(&format!("#{}=VERTEX_POINT('',#{});\n", vertex_id, point_id));
        *next_id += 1;
        
        Ok(vertex_id)
    }
    
    /// Write a cartesian point
    fn write_cartesian_point(&self, next_id: &mut u64, output: &mut String, point: &Point3) -> IoResult<u64> {
        let point_id = *next_id;
        output.push_str(&format!(
            "#{}=CARTESIAN_POINT('',({:.6},{:.6},{:.6}));\n",
            point_id, point.x(), point.y(), point.z())
        );
        *next_id += 1;
        
        Ok(point_id)
    }
    
    /// Write a curve
    fn write_curve(&self, next_id: &mut u64, output: &mut String, _curve: &dyn Curve) -> IoResult<u64> {
        // Placeholder - would need to detect curve type
        let curve_id = *next_id;
        *next_id += 1;
        Ok(curve_id)
    }
}

impl Default for StepWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_schema() {
        assert_eq!(StepSchema::AP214.identifier(), "AUTOMOTIVE_DESIGN");
        assert_eq!(StepSchema::AP242.identifier(), "AP242_MANAGED_MODEL_BASED_3D_ENGINEERING");
    }

    #[test]
    fn test_step_reader_creation() {
        let reader = StepReader::new();
        assert!(matches!(reader.schema, StepSchema::AP214));
    }

    #[test]
    fn test_step_attribute_parsing() {
        let reader = StepReader::new();
        
        assert!(matches!(
            reader.parse_single_attribute("#123"),
            StepAttribute::Reference(123)
        ));
        
        assert!(matches!(
            reader.parse_single_attribute("42"),
            StepAttribute::Integer(42)
        ));
        
        assert!(matches!(
            reader.parse_single_attribute("3.14"),
            StepAttribute::Real(v) if (v - 3.14).abs() < 1e-6
        ));
    }
}
