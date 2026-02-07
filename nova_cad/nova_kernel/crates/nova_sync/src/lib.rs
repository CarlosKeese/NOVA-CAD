//! Nova Sync - Synchronous Technology (Direct Editing) for Nova Kernel 3D
//!
//! Implements Synchronous Technology features inspired by Solid Edge:
//! - Face move/rotate/offset with automatic topology resolution
//! - Live Rules for intelligent editing
//! - Geometric Feature Recognition
//! - Steering Wheel manipulation widget
//!
//! # Core Concepts
//!
//! ## Face-based Editing
//! Unlike history-based CAD, synchronous editing modifies faces directly.
//! When a face is moved, the system automatically resolves the topology
//! to maintain a valid solid.
//!
//! ## Live Rules
//! Live Rules are geometric relationships that are maintained during editing:
//! - Parallelism
//! - Perpendicularity  
//! - Concentricity
//! - Symmetry
//! - Coplanarity
//!
//! ## Steering Wheel
//! The Steering Wheel is a 3D manipulation widget that allows:
//! - Moving faces along primary axes
//! - Rotating faces around axes
//! - Relocating the origin to any point

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use nova_math::{Point3, Vec3, Transform3, ToleranceContext};
use nova_topo::{Body, Face, Edge, Vertex, EntityId};
use std::collections::{HashMap, HashSet};

pub mod face_edit;
pub mod live_rules;
pub mod recognition;
pub mod steering_wheel;
pub mod resolve;
pub mod error;

pub use face_edit::{FaceEditEngine, FaceEditOp, MoveOptions, RotateOptions, OffsetOptions};
pub use live_rules::{LiveRulesEngine, Rule, RuleType, RulePriority};
pub use recognition::{FeatureRecognizer, RecognizedFeature, FeatureType};
pub use steering_wheel::{SteeringWheel, WheelMode, AxisConstraint};
pub use resolve::{TopologyResolver, ResolutionStrategy};
pub use error::{SyncError, SyncResult};

/// Synchronous editing context
#[derive(Debug, Clone)]
pub struct SyncContext {
    /// Global tolerance for geometric operations
    pub tolerance: ToleranceContext,
    /// Live rules engine
    pub live_rules: LiveRulesEngine,
    /// Feature recognizer
    pub recognizer: FeatureRecognizer,
    /// Topology resolver
    pub resolver: TopologyResolver,
    /// Whether to maintain Live Rules automatically
    pub maintain_rules: bool,
    /// Whether to recognize features automatically
    pub auto_recognize: bool,
    /// History of operations for undo/redo
    pub history: Vec<SyncOperation>,
    /// Current position in history
    pub history_position: usize,
}

impl SyncContext {
    /// Create a new synchronous editing context
    pub fn new() -> Self {
        Self {
            tolerance: ToleranceContext::default(),
            live_rules: LiveRulesEngine::new(),
            recognizer: FeatureRecognizer::new(),
            resolver: TopologyResolver::new(),
            maintain_rules: true,
            auto_recognize: true,
            history: Vec::new(),
            history_position: 0,
        }
    }
    
    /// Set tolerance
    pub fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = ToleranceContext::new(tolerance);
        self
    }
    
    /// Enable/disable Live Rules
    pub fn with_live_rules(mut self, enabled: bool) -> Self {
        self.maintain_rules = enabled;
        self
    }
    
    /// Record an operation in history
    pub fn record_operation(&mut self, op: SyncOperation) {
        // Remove any redo history
        self.history.truncate(self.history_position);
        self.history.push(op);
        self.history_position += 1;
    }
    
    /// Undo the last operation
    pub fn undo(&mut self) -> Option<&SyncOperation> {
        if self.history_position > 0 {
            self.history_position -= 1;
            self.history.get(self.history_position)
        } else {
            None
        }
    }
    
    /// Redo the next operation
    pub fn redo(&mut self) -> Option<&SyncOperation> {
        if self.history_position < self.history.len() {
            let op = self.history.get(self.history_position);
            self.history_position += 1;
            op
        } else {
            None
        }
    }
}

impl Default for SyncContext {
    fn default() -> Self {
        Self::new()
    }
}

/// A synchronous editing operation
#[derive(Debug, Clone)]
pub struct SyncOperation {
    /// Operation ID
    pub id: u64,
    /// Operation type
    pub op_type: SyncOpType,
    /// Faces affected
    pub affected_faces: Vec<EntityId>,
    /// Transformation applied
    pub transform: Transform3,
    /// Rules maintained during this operation
    pub maintained_rules: Vec<Rule>,
    /// Features recognized before operation
    pub recognized_features: Vec<RecognizedFeature>,
}

/// Types of synchronous operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncOpType {
    /// Move faces
    FaceMove,
    /// Rotate faces
    FaceRotate,
    /// Offset faces
    FaceOffset,
    /// Delete faces
    FaceDelete,
    /// Replace face
    FaceReplace,
    /// Pattern faces
    FacePattern,
    /// Mirror faces
    FaceMirror,
    /// Recognize feature
    FeatureRecognize,
    /// Apply Live Rule
    LiveRuleApply,
}

/// Synchronous editing engine
pub struct SyncEngine {
    /// Context for editing
    pub context: SyncContext,
    /// Face editing engine
    pub face_edit: FaceEditEngine,
    /// Steering wheel
    pub steering_wheel: Option<SteeringWheel>,
}

impl SyncEngine {
    /// Create a new synchronous editing engine
    pub fn new() -> Self {
        let context = SyncContext::new();
        let face_edit = FaceEditEngine::new(&context);
        
        Self {
            context,
            face_edit,
            steering_wheel: None,
        }
    }
    
    /// Move faces
    pub fn move_faces(
        &mut self,
        body: &mut Body,
        faces: &[Face],
        direction: Vec3,
        distance: f64,
    ) -> SyncResult<()> {
        let options = MoveOptions::new(direction, distance);
        self.face_edit.move_faces(body, faces, &options, &mut self.context)
    }
    
    /// Rotate faces
    pub fn rotate_faces(
        &mut self,
        body: &mut Body,
        faces: &[Face],
        axis_origin: Point3,
        axis_direction: Vec3,
        angle: f64,
    ) -> SyncResult<()> {
        let options = RotateOptions::new(axis_origin, axis_direction, angle);
        self.face_edit.rotate_faces(body, faces, &options, &mut self.context)
    }
    
    /// Offset faces
    pub fn offset_faces(
        &mut self,
        body: &mut Body,
        faces: &[Face],
        offset: f64,
    ) -> SyncResult<()> {
        let options = OffsetOptions::new(offset);
        self.face_edit.offset_faces(body, faces, &options, &mut self.context)
    }
    
    /// Recognize features in a body
    pub fn recognize_features(&self, body: &Body) -> Vec<RecognizedFeature> {
        self.context.recognizer.recognize_all(body, &self.context.tolerance)
    }
    
    /// Find Live Rules for a set of faces
    pub fn find_live_rules(&self, body: &Body, faces: &[Face]) -> Vec<Rule> {
        self.context.live_rules.detect_rules(body, faces, &self.context.tolerance)
    }
    
    /// Show the steering wheel at a position
    pub fn show_steering_wheel(&mut self, position: Point3, normal: Vec3) {
        self.steering_wheel = Some(SteeringWheel::new(position, normal));
    }
    
    /// Hide the steering wheel
    pub fn hide_steering_wheel(&mut self) {
        self.steering_wheel = None;
    }
    
    /// Check if steering wheel is visible
    pub fn is_steering_wheel_visible(&self) -> bool {
        self.steering_wheel.is_some()
    }
}

impl Default for SyncEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Selection set for synchronous editing
#[derive(Debug, Clone, Default)]
pub struct SelectionSet {
    /// Selected faces
    pub faces: HashSet<EntityId>,
    /// Selected edges
    pub edges: HashSet<EntityId>,
    /// Selected vertices
    pub vertices: HashSet<EntityId>,
    /// Primary selection (for manipulation)
    pub primary: Option<EntityId>,
}

impl SelectionSet {
    /// Create a new empty selection
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a face to selection
    pub fn add_face(&mut self, face_id: EntityId) {
        self.faces.insert(face_id);
        if self.primary.is_none() {
            self.primary = Some(face_id);
        }
    }
    
    /// Remove a face from selection
    pub fn remove_face(&mut self, face_id: EntityId) {
        self.faces.remove(&face_id);
        if self.primary == Some(face_id) {
            self.primary = self.faces.iter().next().copied();
        }
    }
    
    /// Clear selection
    pub fn clear(&mut self) {
        self.faces.clear();
        self.edges.clear();
        self.vertices.clear();
        self.primary = None;
    }
    
    /// Check if selection is empty
    pub fn is_empty(&self) -> bool {
        self.faces.is_empty() && self.edges.is_empty() && self.vertices.is_empty()
    }
    
    /// Get number of selected items
    pub fn len(&self) -> usize {
        self.faces.len() + self.edges.len() + self.vertices.len()
    }
}

/// Editing modes for synchronous technology
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditMode {
    /// Move mode
    Move,
    /// Rotate mode
    Rotate,
    /// Offset mode
    Offset,
    /// Delete mode
    Delete,
    /// Dimension mode
    Dimension,
    /// Relation mode
    Relation,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_context_creation() {
        let ctx = SyncContext::new();
        assert!(ctx.maintain_rules);
        assert!(ctx.auto_recognize);
    }

    #[test]
    fn test_sync_context_with_tolerance() {
        let ctx = SyncContext::new().with_tolerance(1e-4);
        assert_eq!(ctx.tolerance.tolerance(), 1e-4);
    }

    #[test]
    fn test_sync_op_type() {
        assert_eq!(SyncOpType::FaceMove as u8, 0);
        assert_eq!(SyncOpType::FaceRotate as u8, 1);
        assert_eq!(SyncOpType::FaceOffset as u8, 2);
    }

    #[test]
    fn test_selection_set() {
        let mut sel = SelectionSet::new();
        let face_id = EntityId(1);
        
        sel.add_face(face_id);
        assert!(sel.faces.contains(&face_id));
        assert_eq!(sel.primary, Some(face_id));
        
        sel.remove_face(face_id);
        assert!(!sel.faces.contains(&face_id));
        assert!(sel.is_empty());
    }

    #[test]
    fn test_edit_mode() {
        assert!(matches!(EditMode::Move, EditMode::Move));
        assert!(matches!(EditMode::Rotate, EditMode::Rotate));
    }
}
