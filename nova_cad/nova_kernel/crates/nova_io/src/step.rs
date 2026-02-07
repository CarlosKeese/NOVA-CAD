//! STEP AP214/AP242 Reader and Writer
//!
//! Implements ISO 10303 (STEP) file format support for CAD data exchange.

use crate::{IoError, IoResult, ImportOptions, ExportOptions};
use nova_topo::Body;
use std::collections::HashMap;

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

impl StepReader {
    /// Create a new STEP reader
    pub fn new() -> Self {
        Self {
            schema: StepSchema::AP214,
        }
    }
    
    /// Read STEP file content and return bodies
    pub fn read(&self, content: &str, options: &ImportOptions) -> IoResult<Vec<Body>> {
        // Parse the STEP file
        let step_file = self.parse(content)?;
        
        // Convert STEP entities to bodies
        let bodies = self.convert_to_bodies(&step_file, options)?;
        
        Ok(bodies)
    }
    
    /// Parse STEP file content
    fn parse(&self, content: &str) -> IoResult<StepFile> {
        // Basic structure validation
        if !content.contains("ISO-10303-21") {
            return Err(IoError::StepError(
                "Not a valid STEP file (missing ISO-10303-21 header)".to_string()
            ));
        }
        
        let mut header = StepHeader::default();
        let mut entities = HashMap::new();
        
        // Simple parser - split into sections
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
        
        // Extract file_description
        if let Some(start) = section.find("FILE_DESCRIPTION") {
            if let Some(desc_start) = section[start..].find("('") {
                let desc_start = start + desc_start + 1;
                if let Some(desc_end) = section[desc_start..].find("')") {
                    let desc = &section[desc_start..desc_start + desc_end];
                    header.description = desc.split("','")
                        .map(|s| s.trim().to_string())
                        .collect();
                }
            }
        }
        
        // Extract file_name
        if let Some(start) = section.find("FILE_NAME") {
            if let Some(name_start) = section[start..].find("'") {
                let name_start = start + name_start + 1;
                if let Some(name_end) = section[name_start..].find("'") {
                    header.name = section[name_start..name_start + name_end].to_string();
                }
            }
        }
        
        // Extract file_schema
        if let Some(start) = section.find("FILE_SCHEMA") {
            if let Some(schema_start) = section[start..].find("'") {
                let schema_start = start + schema_start + 1;
                if let Some(schema_end) = section[schema_start..].find("'") {
                    let schema = &section[schema_start..schema_start + schema_end];
                    header.schema_names = schema.split("','")
                        .map(|s| s.trim().to_string())
                        .collect();
                }
            }
        }
        
        Ok(header)
    }
    
    /// Parse DATA section
    fn parse_data(&self, section: &str) -> IoResult<HashMap<u64, StepEntity>> {
        let mut entities = HashMap::new();
        
        // Parse each entity line
        for line in section.lines() {
            let line = line.trim();
            
            // Look for entity definitions: #123=ENTITY_TYPE(...);
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
        // Extract entity type and attributes
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
        
        for c in attr_str.chars() {
            match c {
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
        
        // Add last attribute
        if !current.trim().is_empty() {
            attributes.push(self.parse_single_attribute(&current));
        }
        
        attributes
    }
    
    /// Parse a single attribute value
    fn parse_single_attribute(&self, value: &str) -> StepAttribute {
        let value = value.trim();
        
        // Reference: #123
        if value.starts_with('#') {
            if let Ok(id) = value[1..].parse::<u64>() {
                return StepAttribute::Reference(id);
            }
        }
        
        // String: 'value'
        if value.starts_with('"') && value.ends_with('"') {
            return StepAttribute::String(
                value[1..value.len()-1].to_string()
            );
        }
        if value.starts_with('\'') && value.ends_with('\'') {
            return StepAttribute::String(
                value[1..value.len()-1].to_string()
            );
        }
        
        // List: (a,b,c)
        if value.starts_with('(') && value.ends_with(')') {
            let inner = &value[1..value.len()-1];
            let items = self.parse_attributes(inner);
            return StepAttribute::List(items);
        }
        
        // Enumeration: .VALUE.
        if value.starts_with('.') && value.ends_with('.') {
            return StepAttribute::Enumeration(
                value[1..value.len()-1].to_string()
            );
        }
        
        // Boolean
        if value == ".T." || value == ".TRUE." {
            return StepAttribute::Boolean(true);
        }
        if value == ".F." || value == ".FALSE." {
            return StepAttribute::Boolean(false);
        }
        
        // Undefined
        if value == "$" {
            return StepAttribute::Undefined;
        }
        
        // Derived
        if value == "*" {
            return StepAttribute::Derived;
        }
        
        // Integer
        if let Ok(i) = value.parse::<i64>() {
            return StepAttribute::Integer(i);
        }
        
        // Real
        if let Ok(f) = value.parse::<f64>() {
            return StepAttribute::Real(f);
        }
        
        // Default to string
        StepAttribute::String(value.to_string())
    }
    
    /// Convert STEP entities to Bodies
    fn convert_to_bodies(
        &self,
        step_file: &StepFile,
        _options: &ImportOptions,
    ) -> IoResult<Vec<Body>> {
        // TODO: Implement full STEP to B-Rep conversion
        // This requires:
        // 1. Find all MANIFOLD_SOLID_BREP entities
        // 2. Convert each to Body
        // 3. Handle units and transformations
        
        let mut bodies = Vec::new();
        
        // Look for manifold solid brep entities
        for (id, entity) in &step_file.entities {
            if entity.entity_type == "MANIFOLD_SOLID_BREP" {
                match self.convert_manifold_solid_brep(*id, step_file) {
                    Ok(body) => bodies.push(body),
                    Err(e) => eprintln!("Warning: Failed to convert entity #{}: {}", id, e),
                }
            }
        }
        
        if bodies.is_empty() {
            return Err(IoError::StepError(
                "No valid MANIFOLD_SOLID_BREP entities found".to_string()
            ));
        }
        
        Ok(bodies)
    }
    
    /// Convert MANIFOLD_SOLID_BREP entity to Body
    fn convert_manifold_solid_brep(
        &self,
        id: u64,
        step_file: &StepFile,
    ) -> IoResult<Body> {
        let entity = step_file.entities.get(&id)
            .ok_or_else(|| IoError::StepError(
                format!("Entity #{} not found", id)
            ))?;
        
        // Get the outer reference
        if let Some(StepAttribute::Reference(outer_id)) = entity.attributes.first() {
            // Get the SHELL entity
            if let Some(shell_entity) = step_file.entities.get(outer_id) {
                if shell_entity.entity_type == "CLOSED_SHELL" {
                    return self.convert_closed_shell(*outer_id, step_file);
                }
            }
        }
        
        Err(IoError::StepError(
            format!("Invalid MANIFOLD_SOLID_BREP structure at #{}" , id)
        ))
    }
    
    /// Convert CLOSED_SHELL entity to Body
    fn convert_closed_shell(
        &self,
        id: u64,
        step_file: &StepFile,
    ) -> IoResult<Body> {
        // TODO: Implement shell to body conversion
        // This requires converting all faces in the shell
        
        Err(IoError::NotSupported(
            "CLOSED_SHELL to Body conversion not yet implemented".to_string()
        ))
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
        
        // Write header
        self.write_header(&mut output, options)?;
        
        // Write data section
        self.write_data(&mut output, bodies, options)?;
        
        // Write trailer
        output.push_str("END-ISO-10303-21;\n");
        
        Ok(output)
    }
    
    /// Write STEP header
    fn write_header(&self, output: &mut String, options: &ExportOptions) -> IoResult<()> {
        output.push_str("ISO-10303-21;\n");
        output.push_str("HEADER;\n");
        
        // FILE_DESCRIPTION
        output.push_str("FILE_DESCRIPTION(('NOVA CAD Model'),'2;1');\n");
        
        // FILE_NAME
        let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S");
        output.push_str(&format!(
            "FILE_NAME('output.step','{}',('{}'),('NOVA CAD'),'NOVA CAD','NOVA CAD','');\n",
            timestamp,
            options.author.as_deref().unwrap_or("Unknown")
        ));
        
        // FILE_SCHEMA
        output.push_str(&format!(
            "FILE_SCHEMA(('{}'));\n",
            self.schema.identifier()
        ));
        
        output.push_str("ENDSEC;\n");
        
        Ok(())
    }
    
    /// Write STEP data section
    fn write_data(
        &self,
        output: &mut String,
        bodies: &[Body],
        _options: &ExportOptions,
    ) -> IoResult<()> {
        output.push_str("DATA;\n");
        
        let mut entity_id: u64 = 100;
        
        for body in bodies {
            self.write_body(&mut entity_id, output, body)?;
        }
        
        output.push_str("ENDSEC;\n");
        
        Ok(())
    }
    
    /// Write a body to STEP
    fn write_body(
        &self,
        next_id: &mut u64,
        output: &mut String,
        _body: &Body,
    ) -> IoResult<()> {
        // TODO: Implement full Body to STEP conversion
        // This requires:
        // 1. Write all surfaces
        // 2. Write all curves
        // 3. Write all topological entities
        // 4. Write the manifold solid brep
        
        // Placeholder
        let shell_id = *next_id;
        *next_id += 1;
        
        output.push_str(&format!(
            "#{}=MANIFOLD_SOLID_BREP('Body',#{});\n",
            shell_id,
            shell_id + 1
        ));
        
        Ok(())
    }
}

impl Default for StepWriter {
    fn default() -> Self {
        Self::new()
    }
}

// Add chrono dependency for timestamps
use chrono;

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
    fn test_step_writer_creation() {
        let writer = StepWriter::new();
        assert!(matches!(writer.schema, StepSchema::AP214));
    }

    #[test]
    fn test_parse_step_attribute() {
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
        
        assert!(matches!(
            reader.parse_single_attribute("'.TRUE.'"),
            StepAttribute::String(s) if s == ".TRUE."
        ));
        
        assert!(matches!(
            reader.parse_single_attribute(".ENUM."),
            StepAttribute::Enumeration(s) if s == "ENUM"
        ));
    }

    #[test]
    fn test_parse_simple_step() {
        let step_content = r#"ISO-10303-21;
HEADER;
FILE_DESCRIPTION(('Test'),'2;1');
FILE_NAME('test.step','2024-01-01T00:00:00',('Author'),('Org'),'System','System','');
FILE_SCHEMA(('AUTOMOTIVE_DESIGN'));
ENDSEC;
DATA;
#100=CARTESIAN_POINT('Origin',(0.0,0.0,0.0));
#101=DIRECTION('Z Axis',(0.0,0.0,1.0));
#102=DIRECTION('X Axis',(1.0,0.0,0.0));
ENDSEC;
END-ISO-10303-21;"#;

        let reader = StepReader::new();
        let result = reader.parse(step_content);
        assert!(result.is_ok());
        
        let step_file = result.unwrap();
        assert_eq!(step_file.header.schema_names.len(), 1);
        assert_eq!(step_file.header.schema_names[0], "AUTOMOTIVE_DESIGN");
        assert!(step_file.entities.contains_key(&100));
    }
}
