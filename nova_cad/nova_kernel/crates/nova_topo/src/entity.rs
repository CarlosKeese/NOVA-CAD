//! Entity types and traits for B-Rep topology

use crate::{new_entity_id, Orientation};

/// Unique entity identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntityId(pub u64);

impl EntityId {
    /// Invalid/null entity ID
    pub const NULL: Self = Self(0);
    
    /// Check if valid (non-null)
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::NULL
    }
}

/// Base trait for all entities
pub trait Entity {
    /// Get the entity ID
    fn id(&self) -> EntityId;
    
    /// Get the entity type name
    fn entity_type(&self) -> &'static str;
}

/// Trait for topological entities
pub trait TopologicalEntity: Entity {
    /// Get the body this entity belongs to
    fn body_id(&self) -> EntityId;
    
    /// Get the orientation
    fn orientation(&self) -> Orientation;
    
    /// Set the orientation
    fn set_orientation(&mut self, orientation: Orientation);
    
    /// Reverse the orientation
    fn reverse_orientation(&mut self) {
        let new_orientation = self.orientation().reverse();
        self.set_orientation(new_orientation);
    }
}

/// Trait for geometric entities
pub trait GeometricEntity: Entity {
    /// Type of geometric data
    type GeomType;
    
    /// Get the geometry
    fn geometry(&self) -> Option<&Self::GeomType>;
    
    /// Set the geometry
    fn set_geometry(&mut self, geometry: Option<Self::GeomType>);
    
    /// Check if has geometry
    fn has_geometry(&self) -> bool {
        self.geometry().is_some()
    }
    
    /// Clear the geometry
    fn clear_geometry(&mut self) {
        self.set_geometry(None);
    }
}

/// Entity tag for user-defined attributes
#[derive(Debug, Clone, PartialEq)]
pub struct EntityTag {
    /// Tag name
    pub name: String,
    /// Tag value
    pub value: TagValue,
}

/// Tag value types
#[derive(Debug, Clone, PartialEq)]
pub enum TagValue {
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// String value
    String(String),
    /// Boolean value
    Boolean(bool),
    /// Binary data
    Binary(Vec<u8>),
}
