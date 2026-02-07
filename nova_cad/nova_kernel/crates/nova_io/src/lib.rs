//! Nova IO - Import/Export for Nova Kernel 3D
//!
//! Supports file formats:
//! - STEP AP214/AP242 (.step, .stp)
//! - IGES (.igs, .iges)
//! - STL (.stl) - export only
//! - Native (.nova)

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use nova_topo::Body;
use std::path::Path;
use thiserror::Error;

pub mod step;
pub mod iges;
pub mod stl;
pub mod nova_format;

pub use step::{StepReader, StepWriter, StepError};
pub use iges::{IgesReader, IgesWriter};
pub use stl::{StlWriter, StlFormat};
pub use nova_format::{NovaReader, NovaWriter};

/// I/O error types
#[derive(Debug, Error, Clone)]
pub enum IoError {
    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),
    
    /// Write error
    #[error("Write error: {0}")]
    WriteError(String),
    
    /// Unsupported format
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    
    /// Invalid data
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    /// STEP specific error
    #[error("STEP error: {0}")]
    StepError(String),
    
    /// IGES specific error
    #[error("IGES error: {0}")]
    IgesError(String),
}

/// Result type for I/O operations
pub type IoResult<T> = Result<T, IoError>;

/// File format types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    /// STEP AP214
    StepAP214,
    /// STEP AP242
    StepAP242,
    /// IGES
    Iges,
    /// ASCII STL
    StlAscii,
    /// Binary STL
    StlBinary,
    /// Native Nova format
    Nova,
}

impl FileFormat {
    /// Get format from file extension
    pub fn from_path<P: AsRef<Path>>(path: P) -> Option<Self> {
        let ext = path.as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())?;
        
        match ext.as_str() {
            "step" | "stp" => Some(FileFormat::StepAP214),
            "iges" | "igs" => Some(FileFormat::Iges),
            "stl" => Some(FileFormat::StlBinary),
            "nova" => Some(FileFormat::Nova),
            _ => None,
        }
    }
    
    /// Get file extension for format
    pub fn extension(&self) -> &'static str {
        match self {
            FileFormat::StepAP214 | FileFormat::StepAP242 => "step",
            FileFormat::Iges => "igs",
            FileFormat::StlAscii | FileFormat::StlBinary => "stl",
            FileFormat::Nova => "nova",
        }
    }
    
    /// Check if format supports import
    pub fn supports_import(&self) -> bool {
        matches!(self,
            FileFormat::StepAP214 |
            FileFormat::StepAP242 |
            FileFormat::Iges |
            FileFormat::Nova
        )
    }
    
    /// Check if format supports export
    pub fn supports_export(&self) -> bool {
        true // All formats support export
    }
}

/// Import options
#[derive(Debug, Clone)]
pub struct ImportOptions {
    /// Tolerance for healing
    pub tolerance: f64,
    /// Whether to heal the geometry
    pub heal: bool,
    /// Whether to stitch faces into solids
    pub stitch: bool,
    /// Units to convert to (mm, inch, etc.)
    pub target_units: Units,
}

impl ImportOptions {
    /// Create default import options
    pub fn new() -> Self {
        Self {
            tolerance: 1e-6,
            heal: true,
            stitch: true,
            target_units: Units::Millimeters,
        }
    }
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Export options
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Tolerance for tessellation
    pub tolerance: f64,
    /// Units to export in
    pub units: Units,
    /// Author/Company
    pub author: Option<String>,
    /// Description
    pub description: Option<String>,
}

impl ExportOptions {
    /// Create default export options
    pub fn new() -> Self {
        Self {
            tolerance: 1e-3,
            units: Units::Millimeters,
            author: None,
            description: None,
        }
    }
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Units for import/export
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Units {
    /// Millimeters
    Millimeters,
    /// Centimeters
    Centimeters,
    /// Meters
    Meters,
    /// Inches
    Inches,
    /// Feet
    Feet,
}

impl Units {
    /// Get conversion factor to millimeters
    pub fn to_mm_factor(&self) -> f64 {
        match self {
            Units::Millimeters => 1.0,
            Units::Centimeters => 10.0,
            Units::Meters => 1000.0,
            Units::Inches => 25.4,
            Units::Feet => 304.8,
        }
    }
    
    /// Get conversion factor from millimeters
    pub fn from_mm_factor(&self) -> f64 {
        1.0 / self.to_mm_factor()
    }
}

/// Universal importer
pub struct Importer;

impl Importer {
    /// Import a file and return bodies
    pub fn import<P: AsRef<Path>>(
        path: P,
        options: &ImportOptions,
    ) -> IoResult<Vec<Body>> {
        let format = FileFormat::from_path(&path)
            .ok_or_else(|| IoError::UnsupportedFormat(
                "Unknown file extension".to_string()
            ))?;
        
        if !format.supports_import() {
            return Err(IoError::UnsupportedFormat(
                format!("{:?} does not support import", format)
            ));
        }
        
        let content = std::fs::read_to_string(&path)
            .map_err(|e| IoError::FileNotFound(e.to_string()))?;
        
        match format {
            FileFormat::StepAP214 | FileFormat::StepAP242 => {
                let reader = StepReader::new();
                reader.read(&content, options)
            }
            FileFormat::Iges => {
                let reader = IgesReader::new();
                reader.read(&content, options)
            }
            FileFormat::Nova => {
                let reader = NovaReader::new();
                reader.read(&content, options)
            }
            _ => Err(IoError::UnsupportedFormat(
                "Format not supported for import".to_string()
            ))
        }
    }
}

/// Universal exporter
pub struct Exporter;

impl Exporter {
    /// Export bodies to a file
    pub fn export<P: AsRef<Path>>(
        path: P,
        bodies: &[Body],
        options: &ExportOptions,
    ) -> IoResult<()> {
        let format = FileFormat::from_path(&path)
            .ok_or_else(|| IoError::UnsupportedFormat(
                "Unknown file extension".to_string()
            ))?;
        
        if !format.supports_export() {
            return Err(IoError::UnsupportedFormat(
                format!("{:?} does not support export", format)
            ));
        }
        
        let content = match format {
            FileFormat::StepAP214 | FileFormat::StepAP242 => {
                let writer = StepWriter::new();
                writer.write(bodies, options)?
            }
            FileFormat::Iges => {
                let writer = IgesWriter::new();
                writer.write(bodies, options)?
            }
            FileFormat::StlAscii | FileFormat::StlBinary => {
                let format = if matches!(format, FileFormat::StlAscii) {
                    StlFormat::Ascii
                } else {
                    StlFormat::Binary
                };
                let writer = StlWriter::new(format);
                writer.write(bodies, options)?
            }
            FileFormat::Nova => {
                let writer = NovaWriter::new();
                writer.write(bodies, options)?
            }
        };
        
        std::fs::write(&path, content)
            .map_err(|e| IoError::WriteError(e.to_string()))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_format_from_path() {
        assert_eq!(
            FileFormat::from_path("test.step"),
            Some(FileFormat::StepAP214)
        );
        assert_eq!(
            FileFormat::from_path("test.STP"),
            Some(FileFormat::StepAP214)
        );
        assert_eq!(
            FileFormat::from_path("test.igs"),
            Some(FileFormat::Iges)
        );
        assert_eq!(
            FileFormat::from_path("test.stl"),
            Some(FileFormat::StlBinary)
        );
        assert_eq!(
            FileFormat::from_path("test.nova"),
            Some(FileFormat::Nova)
        );
        assert_eq!(
            FileFormat::from_path("test.unknown"),
            None
        );
    }

    #[test]
    fn test_units_conversion() {
        assert_eq!(Units::Millimeters.to_mm_factor(), 1.0);
        assert_eq!(Units::Centimeters.to_mm_factor(), 10.0);
        assert_eq!(Units::Meters.to_mm_factor(), 1000.0);
        assert_eq!(Units::Inches.to_mm_factor(), 25.4);
        assert_eq!(Units::Feet.to_mm_factor(), 304.8);
    }

    #[test]
    fn test_import_export_options() {
        let import_opts = ImportOptions::new();
        assert_eq!(import_opts.tolerance, 1e-6);
        assert!(import_opts.heal);
        
        let export_opts = ExportOptions::new();
        assert_eq!(export_opts.tolerance, 1e-3);
    }
}
