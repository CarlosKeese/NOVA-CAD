//! Error types for synchronous editing

use thiserror::Error;
use nova_topo::TopologyError;
use nova_geom::GeometryError;

/// Result type for synchronous operations
pub type SyncResult<T> = Result<T, SyncError>;

/// Errors that can occur during synchronous editing
#[derive(Debug, Error, Clone, PartialEq)]
pub enum SyncError {
    /// Topology error
    #[error("Topology error: {0}")]
    Topology(String),
    
    /// Geometry error
    #[error("Geometry error: {0}")]
    Geometry(String),
    
    /// Face editing failed
    #[error("Face editing failed: {0}")]
    FaceEditFailed(String),
    
    /// Live Rule violation
    #[error("Live Rule violation: {0}")]
    RuleViolation(String),
    
    /// Feature recognition failed
    #[error("Feature recognition failed: {0}")]
    RecognitionFailed(String),
    
    /// Topology resolution failed
    #[error("Topology resolution failed: {0}")]
    ResolutionFailed(String),
    
    /// Invalid selection
    #[error("Invalid selection: {0}")]
    InvalidSelection(String),
    
    /// No faces selected
    #[error("No faces selected")]
    NoSelection,
    
    /// Face not found
    #[error("Face not found: {0}")]
    FaceNotFound(u64),
    
    /// Cannot resolve topology
    #[error("Cannot resolve topology: {0}")]
    UnresolvableTopology(String),
    
    /// Would create invalid solid
    #[error("Operation would create invalid solid: {0}")]
    WouldInvalidateSolid(String),
    
    /// Degenerate geometry
    #[error("Degenerate geometry: {0}")]
    DegenerateGeometry(String),
    
    /// Self-intersection detected
    #[error("Self-intersection detected: {0}")]
    SelfIntersection(String),
    
    /// Operation not supported
    #[error("Operation not supported: {0}")]
    NotSupported(String),
    
    /// Feature not implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

impl From<TopologyError> for SyncError {
    fn from(err: TopologyError) -> Self {
        SyncError::Topology(err.to_string())
    }
}

impl From<GeometryError> for SyncError {
    fn from(err: GeometryError) -> Self {
        SyncError::Geometry(err.to_string())
    }
}

/// Error context for detailed error reporting
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Operation being performed
    pub operation: String,
    /// Entity involved
    pub entity_id: Option<u64>,
    /// Additional context
    pub details: Vec<String>,
}

impl ErrorContext {
    /// Create new error context
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            entity_id: None,
            details: Vec::new(),
        }
    }
    
    /// Add entity ID
    pub fn with_entity(mut self, entity_id: u64) -> Self {
        self.entity_id = Some(entity_id);
        self
    }
    
    /// Add detail
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.details.push(detail.into());
        self
    }
}
