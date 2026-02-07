//! Feature Handle System - Direct manipulation of recognized features
//!
//! Provides handles and widgets for manipulating recognized features
//! like holes, pads, pockets, fillets, etc.

use crate::{SyncError, SyncResult, recognition::{RecognizedFeature, FeatureType, FeatureParameters}};
use nova_math::{Point3, Vec3, Transform3};
use nova_topo::{Body, Face, Edge, EntityId, Entity};
use std::collections::HashMap;

/// Feature handle for direct manipulation
#[derive(Debug, Clone)]
pub struct FeatureHandle {
    /// Handle ID
    pub id: u64,
    /// Feature being manipulated
    pub feature: RecognizedFeature,
    /// Handle position
    pub position: Point3,
    /// Handle orientation
    pub orientation: Vec3,
    /// Handle type
    pub handle_type: HandleType,
    /// Whether handle is active
    pub active: bool,
    /// Visual size
    pub size: f64,
}

/// Types of feature handles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleType {
    /// Position handle (move the feature)
    Position,
    /// Size handle (resize)
    Size,
    /// Direction handle (change direction)
    Direction,
    /// Rotation handle
    Rotation,
    /// Depth handle (for holes/pockets)
    Depth,
    /// Radius handle (for fillets)
    Radius,
    /// Angle handle (for drafts/chamfers)
    Angle,
    /// Pattern count handle
    PatternCount,
    /// Pattern spacing handle
    PatternSpacing,
}

/// Feature handle system
pub struct FeatureHandleSystem {
    /// Active handles
    pub handles: Vec<FeatureHandle>,
    /// Selected handle
    pub selected: Option<u64>,
    /// Handle size scale
    pub scale: f64,
}

impl FeatureHandleSystem {
    /// Create a new handle system
    pub fn new() -> Self {
        Self {
            handles: Vec::new(),
            selected: None,
            scale: 1.0,
        }
    }
    
    /// Create handles for a recognized feature
    pub fn create_handles(&mut self, feature: &RecognizedFeature) -> Vec<u64> {
        let mut created = Vec::new();
        
        match &feature.feature_type {
            FeatureType::Hole { radius, depth, .. } => {
                // Position handle at hole center
                if let FeatureParameters::Hole { center, axis, .. } = &feature.parameters {
                    let pos_handle = FeatureHandle {
                        id: self.generate_id(),
                        feature: feature.clone(),
                        position: *center,
                        orientation: *axis,
                        handle_type: HandleType::Position,
                        active: true,
                        size: radius * 0.5,
                    };
                    created.push(pos_handle.id);
                    self.handles.push(pos_handle);
                    
                    // Depth handle
                    let depth_pos = *center + *axis * *depth;
                    let depth_handle = FeatureHandle {
                        id: self.generate_id(),
                        feature: feature.clone(),
                        position: depth_pos,
                        orientation: *axis,
                        handle_type: HandleType::Depth,
                        active: true,
                        size: radius * 0.3,
                    };
                    created.push(depth_handle.id);
                    self.handles.push(depth_handle);
                    
                    // Radius handle
                    let radius_pos = *center + Vec3::new(radius, 0.0, 0.0); // Perpendicular to axis
                    let radius_handle = FeatureHandle {
                        id: self.generate_id(),
                        feature: feature.clone(),
                        position: radius_pos,
                        orientation: (radius_pos - *center).normalized(),
                        handle_type: HandleType::Radius,
                        active: true,
                        size: radius * 0.3,
                    };
                    created.push(radius_handle.id);
                    self.handles.push(radius_handle);
                }
            }
            
            FeatureType::Pad { height } | FeatureType::Pocket { depth: height } => {
                // Height handle
                if let FeatureParameters::Extrusion { direction, distance, .. } = &feature.parameters {
                    // Find center of faces
                    let center = self.calculate_feature_center(feature);
                    let height_pos = center + *direction * *distance;
                    
                    let height_handle = FeatureHandle {
                        id: self.generate_id(),
                        feature: feature.clone(),
                        position: height_pos,
                        orientation: *direction,
                        handle_type: HandleType::Size,
                        active: true,
                        size: distance * 0.2,
                    };
                    created.push(height_handle.id);
                    self.handles.push(height_handle);
                }
            }
            
            FeatureType::Fillet { radius } => {
                // Radius handle
                let center = self.calculate_feature_center(feature);
                let radius_pos = center + Vec3::new(*radius, 0.0, 0.0);
                
                let radius_handle = FeatureHandle {
                    id: self.generate_id(),
                    feature: feature.clone(),
                    position: radius_pos,
                    orientation: Vec3::new(1.0, 0.0, 0.0),
                    handle_type: HandleType::Radius,
                    active: true,
                    size: radius * 0.3,
                };
                created.push(radius_handle.id);
                self.handles.push(radius_handle);
            }
            
            FeatureType::Chamfer { distance, .. } => {
                // Distance handle
                let center = self.calculate_feature_center(feature);
                let dist_pos = center + Vec3::new(*distance, 0.0, 0.0);
                
                let dist_handle = FeatureHandle {
                    id: self.generate_id(),
                    feature: feature.clone(),
                    position: dist_pos,
                    orientation: Vec3::new(1.0, 0.0, 0.0),
                    handle_type: HandleType::Size,
                    active: true,
                    size: distance * 0.3,
                };
                created.push(dist_handle.id);
                self.handles.push(dist_handle);
            }
            
            _ => {
                // Generic position handle
                let center = self.calculate_feature_center(feature);
                let pos_handle = FeatureHandle {
                    id: self.generate_id(),
                    feature: feature.clone(),
                    position: center,
                    orientation: Vec3::new(0.0, 0.0, 1.0),
                    handle_type: HandleType::Position,
                    active: true,
                    size: 5.0,
                };
                created.push(pos_handle.id);
                self.handles.push(pos_handle);
            }
        }
        
        created
    }
    
    /// Remove handles for a feature
    pub fn remove_feature_handles(&mut self, feature_id: u64) {
        self.handles.retain(|h| h.feature.id != feature_id);
    }
    
    /// Get handle by ID
    pub fn get_handle(&self, id: u64) -> Option<&FeatureHandle> {
        self.handles.iter().find(|h| h.id == id)
    }
    
    /// Get mutable handle by ID
    pub fn get_handle_mut(&mut self, id: u64) -> Option<&mut FeatureHandle> {
        self.handles.iter_mut().find(|h| h.id == id)
    }
    
    /// Select a handle
    pub fn select_handle(&mut self, id: u64) -> bool {
        if self.get_handle(id).is_some() {
            self.selected = Some(id);
            true
        } else {
            false
        }
    }
    
    /// Deselect current handle
    pub fn deselect(&mut self) {
        self.selected = None;
    }
    
    /// Drag a handle to a new position
    pub fn drag_handle(
        &mut self,
        handle_id: u64,
        new_position: Point3,
        body: &mut Body,
    ) -> SyncResult<FeatureEdit> {
        let handle = self.get_handle(handle_id)
            .ok_or_else(|| SyncError::InvalidSelection("Handle not found".to_string()))?
            .clone();
        
        let delta = new_position - handle.position;
        
        let edit = match handle.handle_type {
            HandleType::Position => {
                // Move the feature
                FeatureEdit::Move { 
                    feature_id: handle.feature.id,
                    delta,
                }
            }
            
            HandleType::Size | HandleType::Depth | HandleType::Radius => {
                // Calculate new size based on drag direction
                let direction = handle.orientation;
                let projection = delta.dot(direction);
                
                match &handle.feature.feature_type {
                    FeatureType::Hole { radius, .. } => {
                        if handle.handle_type == HandleType::Radius {
                            FeatureEdit::ResizeRadius {
                                feature_id: handle.feature.id,
                                new_radius: (radius + projection).max(0.1),
                            }
                        } else {
                            FeatureEdit::ResizeDepth {
                                feature_id: handle.feature.id,
                                new_depth: projection.max(0.1),
                            }
                        }
                    }
                    FeatureType::Pad { height } | FeatureType::Pocket { depth: height } => {
                        FeatureEdit::ResizeHeight {
                            feature_id: handle.feature.id,
                            new_height: (height + projection).max(0.1),
                        }
                    }
                    FeatureType::Fillet { radius } => {
                        FeatureEdit::ResizeRadius {
                            feature_id: handle.feature.id,
                            new_radius: (radius + projection).max(0.1),
                        }
                    }
                    _ => FeatureEdit::None,
                }
            }
            
            HandleType::Direction => {
                // Change feature direction
                let new_direction = (new_position - handle.feature.faces.iter()
                    .filter_map(|id| self.find_face_center(body, *id))
                    .next()
                    .unwrap_or(Point3::new(0.0, 0.0, 0.0))).normalized();
                
                FeatureEdit::ChangeDirection {
                    feature_id: handle.feature.id,
                    new_direction,
                }
            }
            
            _ => FeatureEdit::None,
        };
        
        // Update handle position
        if let Some(h) = self.get_handle_mut(handle_id) {
            h.position = new_position;
        }
        
        Ok(edit)
    }
    
    /// Apply a feature edit to the body
    pub fn apply_edit(
        &self,
        body: &mut Body,
        edit: &FeatureEdit,
    ) -> SyncResult<()> {
        match edit {
            FeatureEdit::None => Ok(()),
            
            FeatureEdit::Move { feature_id, delta } => {
                // Find the feature faces and move them
                // This would integrate with face_edit module
                Ok(())
            }
            
            FeatureEdit::ResizeRadius { feature_id, new_radius } => {
                // Modify the feature geometry
                Ok(())
            }
            
            FeatureEdit::ResizeDepth { feature_id, new_depth } => {
                Ok(())
            }
            
            FeatureEdit::ResizeHeight { feature_id, new_height } => {
                Ok(())
            }
            
            FeatureEdit::ChangeDirection { feature_id, new_direction } => {
                Ok(())
            }
            
            FeatureEdit::Delete { feature_id } => {
                Ok(())
            }
        }
    }
    
    /// Calculate center of a feature
    fn calculate_feature_center(&self, feature: &RecognizedFeature) -> Point3 {
        // Simple average of face centers
        // In practice, would use bounding box center or centroid
        Point3::new(0.0, 0.0, 0.0)
    }
    
    /// Find center of a face in body
    fn find_face_center(&self, body: &Body, face_id: EntityId) -> Option<Point3> {
        for shell in body.shells() {
            for face in shell.faces() {
                if face.id() == face_id {
                    return Some(face.bounding_box().center());
                }
            }
        }
        None
    }
    
    /// Generate unique handle ID
    fn generate_id(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64 + self.handles.len() as u64
    }
    
    /// Clear all handles
    pub fn clear(&mut self) {
        self.handles.clear();
        self.selected = None;
    }
    
    /// Get handles for a specific feature
    pub fn get_feature_handles(&self, feature_id: u64) -> Vec<&FeatureHandle> {
        self.handles.iter()
            .filter(|h| h.feature.id == feature_id)
            .collect()
    }
    
    /// Update handle positions after feature modification
    pub fn update_handle_positions(&mut self, feature: &RecognizedFeature) {
        for handle in self.handles.iter_mut() {
            if handle.feature.id == feature.id {
                // Recalculate handle position based on new feature parameters
                handle.feature = feature.clone();
                
                // Update position based on handle type
                match handle.handle_type {
                    HandleType::Position => {
                        if let FeatureParameters::Hole { center, .. } = &feature.parameters {
                            handle.position = *center;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

impl Default for FeatureHandleSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Edits that can be applied to features
#[derive(Debug, Clone)]
pub enum FeatureEdit {
    /// No edit
    None,
    /// Move feature
    Move {
        feature_id: u64,
        delta: Vec3,
    },
    /// Resize radius (for holes, fillets)
    ResizeRadius {
        feature_id: u64,
        new_radius: f64,
    },
    /// Resize depth (for holes, pockets)
    ResizeDepth {
        feature_id: u64,
        new_depth: f64,
    },
    /// Resize height (for pads)
    ResizeHeight {
        feature_id: u64,
        new_height: f64,
    },
    /// Change direction
    ChangeDirection {
        feature_id: u64,
        new_direction: Vec3,
    },
    /// Delete feature
    Delete {
        feature_id: u64,
    },
}

/// Feature manipulation widget
#[derive(Debug, Clone)]
pub struct FeatureWidget {
    /// Associated feature
    pub feature: RecognizedFeature,
    /// Widget position
    pub position: Point3,
    /// Widget orientation
    pub orientation: Vec3,
    /// Available actions
    pub actions: Vec<WidgetAction>,
}

/// Actions available on feature widget
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetAction {
    /// Move the feature
    Move,
    /// Resize
    Resize,
    /// Rotate
    Rotate,
    /// Delete
    Delete,
    /// Suppress
    Suppress,
    /// Edit parameters
    EditParams,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_handle_system() {
        let mut system = FeatureHandleSystem::new();
        assert!(system.handles.is_empty());
        
        let feature = RecognizedFeature {
            id: 1,
            feature_type: FeatureType::Hole {
                radius: 5.0,
                depth: 10.0,
                is_through: false,
                is_tapered: false,
                has_counterbore: false,
                has_countersink: false,
            },
            faces: vec![],
            edges: vec![],
            parameters: FeatureParameters::Hole {
                center: Point3::new(0.0, 0.0, 0.0),
                axis: Vec3::new(0.0, 0.0, 1.0),
                radius: 5.0,
                depth: 10.0,
            },
            confidence: 1.0,
            timestamp: std::time::SystemTime::now(),
        };
        
        let handles = system.create_handles(&feature);
        assert_eq!(handles.len(), 3); // Position, depth, radius
    }

    #[test]
    fn test_handle_type() {
        assert!(matches!(HandleType::Position, HandleType::Position));
        assert!(matches!(HandleType::Radius, HandleType::Radius));
    }

    #[test]
    fn test_feature_edit() {
        let edit = FeatureEdit::Move {
            feature_id: 1,
            delta: Vec3::new(10.0, 0.0, 0.0),
        };
        
        assert!(matches!(edit, FeatureEdit::Move { .. }));
    }
}
