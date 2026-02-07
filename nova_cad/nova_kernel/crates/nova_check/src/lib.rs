//! Nova Check - Validation and Healing for Nova Kernel 3D
//!
//! Provides geometry validation, topology checking, and healing operations.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use nova_topo::Body;

/// Validation error types
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum ValidationError {
    /// Invalid topology
    #[error("Invalid topology: {0}")]
    InvalidTopology(String),
    /// Invalid geometry
    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),
    /// Self-intersection
    #[error("Self-intersection detected")]
    SelfIntersection,
    /// Gap in boundary
    #[error("Gap in boundary")]
    GapInBoundary,
    /// Degenerate geometry
    #[error("Degenerate geometry")]
    DegenerateGeometry,
    /// Orientation error
    #[error("Orientation error")]
    OrientationError,
}

/// Validation warning types
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationWarning {
    /// Small edge
    SmallEdge,
    /// Small face
    SmallFace,
    /// Sharp angle
    SharpAngle,
    /// Near degenerate
    NearDegenerate,
}

/// Validation result
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    /// Errors found
    pub errors: Vec<ValidationError>,
    /// Warnings found
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    /// Create a new empty validation result
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Check if validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
    
    /// Add an error
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }
    
    /// Add a warning
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }
    
    /// Merge another validation result
    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

/// Validation options
#[derive(Debug, Clone, Copy)]
pub struct ValidationOptions {
    /// Tolerance for checks
    pub tolerance: f64,
    /// Check for self-intersections
    pub check_self_intersection: bool,
    /// Check for gaps
    pub check_gaps: bool,
    /// Check orientation
    pub check_orientation: bool,
}

impl Default for ValidationOptions {
    fn default() -> Self {
        Self {
            tolerance: 1e-6,
            check_self_intersection: true,
            check_gaps: true,
            check_orientation: true,
        }
    }
}

/// Validator for B-Rep bodies
pub struct Validator {
    options: ValidationOptions,
}

impl Validator {
    /// Create a new validator with default options
    pub fn new() -> Self {
        Self {
            options: ValidationOptions::default(),
        }
    }
    
    /// Create a new validator with custom options
    pub fn with_options(options: ValidationOptions) -> Self {
        Self { options }
    }
    
    /// Validate a body
    pub fn validate(&self, body: &Body) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // TODO: Implement actual validation
        // For now, return empty result (valid)
        
        result
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a body is valid
pub fn check_body(body: &Body) -> ValidationResult {
    let validator = Validator::new();
    validator.validate(body)
}

/// Check if a body is valid with custom options
pub fn check_body_with_options(body: &Body, options: ValidationOptions) -> ValidationResult {
    let validator = Validator::with_options(options);
    validator.validate(body)
}

/// Healing options
#[derive(Debug, Clone, Copy)]
pub struct HealingOptions {
    /// Tolerance for healing
    pub tolerance: f64,
    /// Fix gaps
    pub fix_gaps: bool,
    /// Fix self-intersections
    pub fix_self_intersections: bool,
    /// Fix orientation
    pub fix_orientation: bool,
    /// Simplify geometry
    pub simplify: bool,
}

impl Default for HealingOptions {
    fn default() -> Self {
        Self {
            tolerance: 1e-6,
            fix_gaps: true,
            fix_self_intersections: true,
            fix_orientation: true,
            simplify: false,
        }
    }
}

/// Healer for B-Rep bodies
pub struct Healer {
    options: HealingOptions,
}

impl Healer {
    /// Create a new healer with default options
    pub fn new() -> Self {
        Self {
            options: HealingOptions::default(),
        }
    }
    
    /// Create a new healer with custom options
    pub fn with_options(options: HealingOptions) -> Self {
        Self { options }
    }
    
    /// Heal a body
    pub fn heal(&self, body: &mut Body) -> Result<(), ValidationError> {
        // TODO: Implement actual healing
        Ok(())
    }
}

impl Default for Healer {
    fn default() -> Self {
        Self::new()
    }
}

/// Heal a body with default options
pub fn heal_body(body: &mut Body) -> Result<(), ValidationError> {
    let healer = Healer::new();
    healer.heal(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::new();
        assert!(result.is_valid());
        
        result.add_error(ValidationError::InvalidTopology("test".to_string()));
        assert!(!result.is_valid());
    }

    #[test]
    fn test_validator() {
        let validator = Validator::new();
        let body = Body::new();
        let result = validator.validate(&body);
        assert!(result.is_valid());
    }
}
