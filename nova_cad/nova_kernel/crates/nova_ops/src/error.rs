//! Error types for operations

use thiserror::Error;
use nova_topo::TopologyError;
use nova_geom::GeometryError;

/// Result type for operations
pub type OpsResult<T> = Result<T, OpsError>;

/// Errors that can occur during CAD operations
#[derive(Debug, Error, Clone, PartialEq)]
pub enum OpsError {
    /// Topology error
    #[error("Topology error: {0}")]
    Topology(String),
    
    /// Geometry error
    #[error("Geometry error: {0}")]
    Geometry(String),
    
    /// Boolean operation failed
    #[error("Boolean operation failed: {0}")]
    BooleanFailed(String),
    
    /// Feature creation failed
    #[error("Feature creation failed: {0}")]
    FeatureFailed(String),
    
    /// Fillet/Chamfer failed
    #[error("Fillet/Chamfer failed: {0}")]
    FilletFailed(String),
    
    /// Invalid input bodies
    #[error("Invalid input bodies: {0}")]
    InvalidBodies(String),
    
    /// No intersection found
    #[error("No intersection found between bodies")]
    NoIntersection,
    
    /// Non-manifold result
    #[error("Operation would create non-manifold geometry: {0}")]
    NonManifold(String),
    
    /// Self-intersection detected
    #[error("Self-intersection detected: {0}")]
    SelfIntersection(String),
    
    /// Invalid parameters
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    
    /// Operation not supported
    #[error("Operation not supported: {0}")]
    NotSupported(String),
    
    /// Numerical error
    #[error("Numerical error: {0}")]
    Numerical(String),
}

impl From<TopologyError> for OpsError {
    fn from(err: TopologyError) -> Self {
        OpsError::Topology(err.to_string())
    }
}

impl From<GeometryError> for OpsError {
    fn from(err: GeometryError) -> Self {
        OpsError::Geometry(err.to_string())
    }
}

/// Error context for detailed error reporting
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Operation being performed
    pub operation: String,
    /// Entity involved
    pub entity: Option<String>,
    /// Additional context
    pub details: Vec<String>,
}

impl ErrorContext {
    /// Create new error context
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            entity: None,
            details: Vec::new(),
        }
    }
    
    /// Add entity context
    pub fn with_entity(mut self, entity: impl Into<String>) -> Self {
        self.entity = Some(entity.into());
        self
    }
    
    /// Add detail
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.details.push(detail.into());
        self
    }
}
