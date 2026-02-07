//! STL Writer (ASCII and Binary)
//!
//! Implements StereoLithography format for 3D printing.

use crate::{IoError, IoResult, ExportOptions};
use nova_topo::Body;
use nova_math::{Point3, Vec3};

/// STL format type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StlFormat {
    /// ASCII STL format
    Ascii,
    /// Binary STL format
    Binary,
}

/// STL writer
#[derive(Debug, Clone)]
pub struct StlWriter {
    /// Output format
    pub format: StlFormat,
}

/// Triangle for STL export
#[derive(Debug, Clone, Copy)]
pub struct StlTriangle {
    /// Normal vector
    pub normal: Vec3,
    /// Vertex 1
    pub v1: Point3,
    /// Vertex 2
    pub v2: Point3,
    /// Vertex 3
    pub v3: Point3,
}

impl StlWriter {
    /// Create a new STL writer
    pub fn new(format: StlFormat) -> Self {
        Self { format }
    }
    
    /// Write bodies to STL format
    pub fn write(&self, bodies: &[Body], options: &ExportOptions) -> IoResult<String> {
        // Tessellate bodies to triangles
        let triangles = self.tessellate_bodies(bodies, options)?;
        
        match self.format {
            StlFormat::Ascii => self.write_ascii(&triangles, options),
            StlFormat::Binary => self.write_binary(&triangles, options),
        }
    }
    
    /// Tessellate bodies to triangles
    fn tessellate_bodies(
        &self,
        bodies: &[Body],
        options: &ExportOptions,
    ) -> IoResult<Vec<StlTriangle>> {
        let mut all_triangles = Vec::new();
        
        for body in bodies {
            let triangles = self.tessellate_body(body, options)?;
            all_triangles.extend(triangles);
        }
        
        Ok(all_triangles)
    }
    
    /// Tessellate a single body
    fn tessellate_body(
        &self,
        body: &Body,
        options: &ExportOptions,
    ) -> IoResult<Vec<StlTriangle>> {
        let mut triangles = Vec::new();
        
        for face in body.faces() {
            let face_triangles = self.tessellate_face(face, options)?;
            triangles.extend(face_triangles);
        }
        
        Ok(triangles)
    }
    
    /// Tessellate a face to triangles
    fn tessellate_face(
        &self,
        face: &nova_topo::Face,
        options: &ExportOptions,
    ) -> IoResult<Vec<StlTriangle>> {
        use nova_geom::Tessellatable;
        
        let surface = face.surface();
        let tessellation = surface.tessellate(options.tolerance);
        
        let mut triangles = Vec::new();
        
        if let nova_geom::Tessellation::TriangleMesh { vertices, indices, normals, .. } = tessellation {
            // Convert indices to triangles
            for i in (0..indices.len()).step_by(3) {
                if i + 2 < indices.len() {
                    let i1 = indices[i] as usize;
                    let i2 = indices[i + 1] as usize;
                    let i3 = indices[i + 2] as usize;
                    
                    if i1 < vertices.len() && i2 < vertices.len() && i3 < vertices.len() {
                        // Calculate or use provided normal
                        let normal = if !normals.is_empty() && i1 < normals.len() {
                            normals[i1]
                        } else {
                            // Calculate from vertices
                            let v1 = vertices[i2] - vertices[i1];
                            let v2 = vertices[i3] - vertices[i1];
                            v1.cross(v2).normalized()
                        };
                        
                        triangles.push(StlTriangle {
                            normal,
                            v1: vertices[i1],
                            v2: vertices[i2],
                            v3: vertices[i3],
                        });
                    }
                }
            }
        }
        
        Ok(triangles)
    }
    
    /// Write triangles to ASCII STL
    fn write_ascii(
        &self,
        triangles: &[StlTriangle],
        options: &ExportOptions,
    ) -> IoResult<String> {
        let mut output = String::new();
        
        output.push_str(&format!(
            "solid {}\n",
            options.description.as_deref().unwrap_or("NOVA_CAD_Model")
        ));
        
        for tri in triangles {
            output.push_str(&format!(
                "  facet normal {:.6} {:.6} {:.6}\n",
                tri.normal.x, tri.normal.y, tri.normal.z
            ));
            output.push_str("    outer loop\n");
            output.push_str(&format!(
                "      vertex {:.6} {:.6} {:.6}\n",
                tri.v1.x, tri.v1.y, tri.v1.z
            ));
            output.push_str(&format!(
                "      vertex {:.6} {:.6} {:.6}\n",
                tri.v2.x, tri.v2.y, tri.v2.z
            ));
            output.push_str(&format!(
                "      vertex {:.6} {:.6} {:.6}\n",
                tri.v3.x, tri.v3.y, tri.v3.z
            ));
            output.push_str("    endloop\n");
            output.push_str("  endfacet\n");
        }
        
        output.push_str("endsolid\n");
        
        Ok(output)
    }
    
    /// Write triangles to binary STL
    fn write_binary(
        &self,
        triangles: &[StlTriangle],
        _options: &ExportOptions,
    ) -> IoResult<String> {
        // Binary STL is not text, but we'll return a placeholder
        // In real implementation, this would return Vec<u8>
        Err(IoError::NotSupported(
            "Binary STL output should use Vec<u8>".to_string()
        ))
    }
}

impl Default for StlWriter {
    fn default() -> Self {
        Self::new(StlFormat::Ascii)
    }
}

/// Write binary STL to bytes
pub fn write_binary_stl(bodies: &[Body], options: &ExportOptions) -> IoResult<Vec<u8>> {
    let writer = StlWriter::new(StlFormat::Binary);
    let triangles = writer.tessellate_bodies(bodies, options)?;
    
    let mut bytes = Vec::new();
    
    // 80 byte header
    let header = b"NOVA CAD Binary STL";
    bytes.extend_from_slice(header);
    bytes.resize(80, 0);
    
    // Number of triangles (u32)
    let num_triangles = triangles.len() as u32;
    bytes.extend_from_slice(&num_triangles.to_le_bytes());
    
    // Each triangle: 12 bytes normal + 12 bytes v1 + 12 bytes v2 + 12 bytes v3 + 2 bytes attribute
    for tri in triangles {
        // Normal (3 floats)
        bytes.extend_from_slice(&(tri.normal.x as f32).to_le_bytes());
        bytes.extend_from_slice(&(tri.normal.y as f32).to_le_bytes());
        bytes.extend_from_slice(&(tri.normal.z as f32).to_le_bytes());
        
        // Vertex 1
        bytes.extend_from_slice(&(tri.v1.x as f32).to_le_bytes());
        bytes.extend_from_slice(&(tri.v1.y as f32).to_le_bytes());
        bytes.extend_from_slice(&(tri.v1.z as f32).to_le_bytes());
        
        // Vertex 2
        bytes.extend_from_slice(&(tri.v2.x as f32).to_le_bytes());
        bytes.extend_from_slice(&(tri.v2.y as f32).to_le_bytes());
        bytes.extend_from_slice(&(tri.v2.z as f32).to_le_bytes());
        
        // Vertex 3
        bytes.extend_from_slice(&(tri.v3.x as f32).to_le_bytes());
        bytes.extend_from_slice(&(tri.v3.y as f32).to_le_bytes());
        bytes.extend_from_slice(&(tri.v3.z as f32).to_le_bytes());
        
        // Attribute byte count (usually 0)
        bytes.extend_from_slice(&[0u8, 0]);
    }
    
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stl_writer_creation() {
        let writer = StlWriter::new(StlFormat::Ascii);
        assert!(matches!(writer.format, StlFormat::Ascii));
        
        let writer = StlWriter::new(StlFormat::Binary);
        assert!(matches!(writer.format, StlFormat::Binary));
    }

    #[test]
    fn test_stl_triangle() {
        let tri = StlTriangle {
            normal: Vec3::new(0.0, 0.0, 1.0),
            v1: Point3::new(0.0, 0.0, 0.0),
            v2: Point3::new(1.0, 0.0, 0.0),
            v3: Point3::new(0.0, 1.0, 0.0),
        };
        
        assert_eq!(tri.normal.z, 1.0);
        assert_eq!(tri.v1.x, 0.0);
    }
}
