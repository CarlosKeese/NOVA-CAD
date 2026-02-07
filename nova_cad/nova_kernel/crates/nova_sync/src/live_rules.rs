//! Live Rules - Intelligent geometric relationships for synchronous editing
//!
//! Live Rules maintain geometric relationships during face editing:
//! - Parallel: Faces remain parallel
//! - Perpendicular: Faces remain perpendicular
//! - Concentric: Cylindrical faces remain concentric
//! - Symmetric: Faces maintain symmetry
//! - Coplanar: Faces remain coplanar
//! - Tangent: Faces remain tangent

use crate::{SyncError, SyncResult};
use nova_math::{Point3, Vec3, ToleranceContext};
use nova_topo::{Body, Face, Edge, Entity};
use std::collections::{HashMap, HashSet};

/// Live Rules engine
#[derive(Debug, Clone)]
pub struct LiveRulesEngine {
    /// Active rules
    pub rules: Vec<Rule>,
    /// Whether to auto-detect rules
    pub auto_detect: bool,
    /// Rule priority weights
    pub priorities: HashMap<RuleType, RulePriority>,
}

impl LiveRulesEngine {
    /// Create a new Live Rules engine
    pub fn new() -> Self {
        let mut priorities = HashMap::new();
        priorities.insert(RuleType::Coincident, RulePriority::Highest);
        priorities.insert(RuleType::Concentric, RulePriority::High);
        priorities.insert(RuleType::Parallel, RulePriority::High);
        priorities.insert(RuleType::Perpendicular, RulePriority::High);
        priorities.insert(RuleType::Symmetric, RulePriority::Medium);
        priorities.insert(RuleType::Coplanar, RulePriority::Medium);
        priorities.insert(RuleType::Tangent, RulePriority::Medium);
        priorities.insert(RuleType::EqualRadius, RulePriority::Low);
        priorities.insert(RuleType::EqualDistance, RulePriority::Low);
        
        Self {
            rules: Vec::new(),
            auto_detect: true,
            priorities,
        }
    }
    
    /// Detect rules between faces
    pub fn detect_rules(
        &self,
        body: &Body,
        faces: &[Face],
        tolerance: &ToleranceContext,
    ) -> Vec<Rule> {
        let mut detected = Vec::new();
        
        // Check all pairs of faces
        for (i, face1) in faces.iter().enumerate() {
            for face2 in faces.iter().skip(i + 1) {
                if let Some(rule) = self.detect_rule_between_faces(face1, face2, tolerance) {
                    detected.push(rule);
                }
            }
        }
        
        // Also check relationships with other faces in the body
        for face in faces {
            let related = self.find_related_faces(body, face);
            for other in related {
                if !faces.contains(&other) {
                    if let Some(rule) = self.detect_rule_between_faces(face, &other, tolerance) {
                        detected.push(rule);
                    }
                }
            }
        }
        
        // Sort by priority
        detected.sort_by_key(|r| self.priorities.get(&r.rule_type).copied().unwrap_or(RulePriority::Low));
        detected.reverse();
        
        detected
    }
    
    /// Detect a rule between two faces
    fn detect_rule_between_faces(
        &self,
        face1: &Face,
        face2: &Face,
        tolerance: &ToleranceContext,
    ) -> Option<Rule> {
        // Get surface normals
        let normal1 = self.get_face_normal(face1)?;
        let normal2 = self.get_face_normal(face2)?;
        
        // Check for parallelism
        let dot = normal1.dot(normal2).abs();
        if (1.0 - dot) < tolerance.tolerance() {
            return Some(Rule {
                id: self.generate_rule_id(),
                rule_type: RuleType::Parallel,
                faces: vec![face1.id(), face2.id()],
                entities: vec![],
                strength: 1.0,
                active: true,
            });
        }
        
        // Check for perpendicularity
        if dot < tolerance.tolerance() {
            return Some(Rule {
                id: self.generate_rule_id(),
                rule_type: RuleType::Perpendicular,
                faces: vec![face1.id(), face2.id()],
                entities: vec![],
                strength: 1.0,
                active: true,
            });
        }
        
        // Check for coplanarity
        if self.are_faces_coplanar(face1, face2, tolerance) {
            return Some(Rule {
                id: self.generate_rule_id(),
                rule_type: RuleType::Coplanar,
                faces: vec![face1.id(), face2.id()],
                entities: vec![],
                strength: 1.0,
                active: true,
            });
        }
        
        // Check for concentricity (for cylindrical faces)
        if self.are_faces_concentric(face1, face2, tolerance) {
            return Some(Rule {
                id: self.generate_rule_id(),
                rule_type: RuleType::Concentric,
                faces: vec![face1.id(), face2.id()],
                entities: vec![],
                strength: 1.0,
                active: true,
            });
        }
        
        None
    }
    
    /// Get the normal of a face
    fn get_face_normal(&self, face: &Face) -> Option<Vec3> {
        if let Some(surface) = face.surface() {
            let (u, v) = surface.midpoint_uv();
            Some(surface.evaluate(u, v).normal)
        } else {
            None
        }
    }
    
    /// Check if two faces are coplanar
    fn are_faces_coplanar(&self, face1: &Face, face2: &Face, tolerance: &ToleranceContext) -> bool {
        // Get normals
        let Some(normal1) = self.get_face_normal(face1) else { return false };
        let Some(normal2) = self.get_face_normal(face2) else { return false };
        
        // Normals should be parallel
        if (1.0 - normal1.dot(normal2).abs()) > tolerance.tolerance() {
            return false;
        }
        
        // Get a point from each face
        let point1 = self.get_face_point(face1);
        let point2 = self.get_face_point(face2);
        
        // Distance between planes should be small
        let distance = (point2 - point1).dot(normal1).abs();
        distance < tolerance.tolerance()
    }
    
    /// Check if two faces are concentric (for cylindrical surfaces)
    fn are_faces_concentric(&self, face1: &Face, face2: &Face, tolerance: &ToleranceContext) -> bool {
        // This requires the surfaces to be cylindrical
        // For now, simplified check based on bounding boxes
        let bbox1 = face1.bounding_box();
        let bbox2 = face2.bounding_box();
        
        // Centers should coincide
        let center1 = bbox1.center();
        let center2 = bbox2.center();
        
        center1.distance_to(&center2) < tolerance.tolerance()
    }
    
    /// Get a representative point from a face
    fn get_face_point(&self, face: &Face) -> Point3 {
        if let Some(loop_) = face.outer_loop() {
            if let Some(coedge) = loop_.coedges().first() {
                return coedge.start_vertex().position();
            }
        }
        Point3::new(0.0, 0.0, 0.0)
    }
    
    /// Find faces related to a given face in the body
    fn find_related_faces(&self, body: &Body, face: &Face) -> Vec<Face> {
        let mut related = Vec::new();
        let face_edges: HashSet<_> = face.loops()
            .iter()
            .flat_map(|lp| lp.coedges())
            .map(|c| c.edge().id())
            .collect();
        
        for shell in body.shells() {
            for other in shell.faces() {
                if other.id() == face.id() {
                    continue;
                }
                
                let other_edges: HashSet<_> = other.loops()
                    .iter()
                    .flat_map(|lp| lp.coedges())
                    .map(|c| c.edge().id())
                    .collect();
                
                if face_edges.intersection(&other_edges).next().is_some() {
                    related.push(other.clone());
                }
            }
        }
        
        related
    }
    
    /// Add a rule manually
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }
    
    /// Remove a rule
    pub fn remove_rule(&mut self, rule_id: u64) {
        self.rules.retain(|r| r.id != rule_id);
    }
    
    /// Enable a rule
    pub fn enable_rule(&mut self, rule_id: u64) {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.id == rule_id) {
            rule.active = true;
        }
    }
    
    /// Disable a rule
    pub fn disable_rule(&mut self, rule_id: u64) {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.id == rule_id) {
            rule.active = false;
        }
    }
    
    /// Get active rules
    pub fn active_rules(&self) -> Vec<&Rule> {
        self.rules.iter().filter(|r| r.active).collect()
    }
    
    /// Apply rules after an edit
    pub fn apply_rules(
        &self,
        body: &mut Body,
        edited_faces: &[Face],
    ) -> SyncResult<()> {
        for rule in &self.rules {
            if !rule.active {
                continue;
            }
            
            // Check if this rule involves edited faces
            let involves_edited = rule.faces.iter()
                .any(|id| edited_faces.iter().any(|f| f.id() == *id));
            
            if involves_edited {
                self.enforce_rule(body, rule)?;
            }
        }
        
        Ok(())
    }
    
    /// Enforce a specific rule
    fn enforce_rule(&self, _body: &mut Body, rule: &Rule) -> SyncResult<()> {
        match rule.rule_type {
            RuleType::Parallel => {
                // Ensure faces remain parallel
                // TODO: Implement
            }
            RuleType::Perpendicular => {
                // Ensure faces remain perpendicular
                // TODO: Implement
            }
            RuleType::Concentric => {
                // Ensure faces remain concentric
                // TODO: Implement
            }
            RuleType::Coplanar => {
                // Ensure faces remain coplanar
                // TODO: Implement
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Generate a unique rule ID
    fn generate_rule_id(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64 + self.rules.len() as u64
    }
}

impl Default for LiveRulesEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// A geometric rule
#[derive(Debug, Clone)]
pub struct Rule {
    /// Rule ID
    pub id: u64,
    /// Type of rule
    pub rule_type: RuleType,
    /// Faces involved
    pub faces: Vec<nova_topo::EntityId>,
    /// Other entities involved (edges, vertices)
    pub entities: Vec<nova_topo::EntityId>,
    /// Rule strength (0.0 to 1.0)
    pub strength: f64,
    /// Whether the rule is active
    pub active: bool,
}

/// Types of Live Rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleType {
    /// Faces remain parallel
    Parallel,
    /// Faces remain perpendicular
    Perpendicular,
    /// Faces remain concentric
    Concentric,
    /// Faces remain symmetric
    Symmetric,
    /// Faces remain coplanar
    Coplanar,
    /// Faces remain tangent
    Tangent,
    /// Equal radius
    EqualRadius,
    /// Equal distance
    EqualDistance,
    /// Coincident (same position)
    Coincident,
    /// Angle between faces
    Angle(f64),
    /// Distance between faces
    Distance(f64),
}

/// Rule priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RulePriority {
    /// Lowest priority
    Lowest = 0,
    /// Low priority
    Low = 1,
    /// Medium priority
    Medium = 2,
    /// High priority
    High = 3,
    /// Highest priority
    Highest = 4,
}

/// Rule application result
#[derive(Debug, Clone)]
pub struct RuleApplication {
    /// Rule that was applied
    pub rule: Rule,
    /// Whether the rule was satisfied
    pub satisfied: bool,
    /// Error measure (0.0 = perfect)
    pub error: f64,
    /// Adjustments made
    pub adjustments: Vec<Adjustment>,
}

/// Adjustment made to satisfy a rule
#[derive(Debug, Clone)]
pub struct Adjustment {
    /// Entity that was adjusted
    pub entity_id: nova_topo::EntityId,
    /// Type of adjustment
    pub adjustment_type: AdjustmentType,
    /// Magnitude of adjustment
    pub magnitude: f64,
}

/// Types of adjustments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdjustmentType {
    /// Translation
    Translate,
    /// Rotation
    Rotate,
    /// Scaling
    Scale,
    /// Offset
    Offset,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_live_rules_engine_creation() {
        let engine = LiveRulesEngine::new();
        assert!(engine.auto_detect);
        assert!(engine.rules.is_empty());
    }

    #[test]
    fn test_rule_priority() {
        assert!(RulePriority::Highest > RulePriority::High);
        assert!(RulePriority::High > RulePriority::Medium);
        assert!(RulePriority::Medium > RulePriority::Low);
        assert!(RulePriority::Low > RulePriority::Lowest);
    }

    #[test]
    fn test_rule_type() {
        assert!(matches!(RuleType::Parallel, RuleType::Parallel));
        assert!(matches!(RuleType::Perpendicular, RuleType::Perpendicular));
        assert!(matches!(RuleType::Concentric, RuleType::Concentric));
    }

    #[test]
    fn test_add_and_remove_rule() {
        let mut engine = LiveRulesEngine::new();
        
        let rule = Rule {
            id: 1,
            rule_type: RuleType::Parallel,
            faces: vec![],
            entities: vec![],
            strength: 1.0,
            active: true,
        };
        
        engine.add_rule(rule);
        assert_eq!(engine.rules.len(), 1);
        
        engine.remove_rule(1);
        assert!(engine.rules.is_empty());
    }

    #[test]
    fn test_enable_disable_rule() {
        let mut engine = LiveRulesEngine::new();
        
        let rule = Rule {
            id: 1,
            rule_type: RuleType::Parallel,
            faces: vec![],
            entities: vec![],
            strength: 1.0,
            active: true,
        };
        
        engine.add_rule(rule);
        engine.disable_rule(1);
        
        assert_eq!(engine.active_rules().len(), 0);
        
        engine.enable_rule(1);
        assert_eq!(engine.active_rules().len(), 1);
    }
}
