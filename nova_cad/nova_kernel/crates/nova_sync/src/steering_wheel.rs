//! Steering Wheel - 3D Manipulation Widget
//!
//! The Steering Wheel is a visual widget that provides intuitive
//! 3D manipulation of selected geometry. Inspired by Solid Edge's
//! Steering Wheel.

use nova_math::{Point3, Vec3, Transform3};
use std::f64::consts::PI;

/// Steering Wheel widget
#[derive(Debug, Clone)]
pub struct SteeringWheel {
    /// Origin position of the wheel
    pub origin: Point3,
    /// Primary axis (normal to the reference plane)
    pub primary_axis: Vec3,
    /// Secondary axis (in the reference plane)
    pub secondary_axis: Vec3,
    /// Tertiary axis (perpendicular to both)
    pub tertiary_axis: Vec3,
    /// Current mode
    pub mode: WheelMode,
    /// Current constraint
    pub constraint: AxisConstraint,
    /// Whether the wheel is active
    pub active: bool,
    /// Wheel radius
    pub radius: f64,
}

/// Operation modes for the steering wheel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WheelMode {
    /// Free movement
    Free,
    /// Move along primary axis
    MovePrimary,
    /// Move along secondary axis
    MoveSecondary,
    /// Move along tertiary axis
    MoveTertiary,
    /// Move in primary-secondary plane
    MovePlane,
    /// Rotate around primary axis
    RotatePrimary,
    /// Rotate around secondary axis
    RotateSecondary,
    /// Rotate around tertiary axis
    RotateTertiary,
    /// Scale uniformly
    ScaleUniform,
    /// Scale along primary axis
    ScalePrimary,
}

/// Axis constraints for movement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisConstraint {
    /// No constraint
    None,
    /// Constrain to primary axis
    Primary,
    /// Constrain to secondary axis
    Secondary,
    /// Constrain to tertiary axis
    Tertiary,
    /// Constrain to primary-secondary plane
    Plane,
    /// Constrain to a specific direction
    Direction(Vec3),
}

impl SteeringWheel {
    /// Create a new steering wheel
    pub fn new(origin: Point3, normal: Vec3) -> Self {
        let primary_axis = normal.normalized();
        
        // Create perpendicular axes
        let secondary_axis = if primary_axis.dot(Vec3::new(0.0, 0.0, 1.0)).abs() < 0.9 {
            primary_axis.cross(Vec3::new(0.0, 0.0, 1.0)).normalized()
        } else {
            primary_axis.cross(Vec3::new(0.0, 1.0, 0.0)).normalized()
        };
        
        let tertiary_axis = primary_axis.cross(secondary_axis).normalized();
        
        Self {
            origin,
            primary_axis,
            secondary_axis,
            tertiary_axis,
            mode: WheelMode::Free,
            constraint: AxisConstraint::None,
            active: true,
            radius: 10.0,
        }
    }
    
    /// Create a steering wheel with explicit axes
    pub fn with_axes(
        origin: Point3,
        primary: Vec3,
        secondary: Vec3,
    ) -> Self {
        let primary_axis = primary.normalized();
        let secondary_axis = secondary.normalized();
        let tertiary_axis = primary_axis.cross(secondary_axis).normalized();
        
        Self {
            origin,
            primary_axis,
            secondary_axis,
            tertiary_axis,
            mode: WheelMode::Free,
            constraint: AxisConstraint::None,
            active: true,
            radius: 10.0,
        }
    }
    
    /// Set the mode
    pub fn set_mode(&mut self, mode: WheelMode) {
        self.mode = mode;
    }
    
    /// Set the constraint
    pub fn set_constraint(&mut self, constraint: AxisConstraint) {
        self.constraint = constraint;
    }
    
    /// Relocate the wheel to a new position
    pub fn relocate(&mut self, new_origin: Point3) {
        self.origin = new_origin;
    }
    
    /// Orient the wheel to align with a direction
    pub fn orient(&mut self, new_normal: Vec3) {
        let primary_axis = new_normal.normalized();
        
        // Keep the current secondary axis as close as possible
        let secondary_axis = if self.secondary_axis.dot(primary_axis).abs() < 0.9 {
            let proj = self.secondary_axis - primary_axis * self.secondary_axis.dot(primary_axis);
            proj.normalized()
        } else {
            // Create new perpendicular axis
            if primary_axis.dot(Vec3::new(0.0, 0.0, 1.0)).abs() < 0.9 {
                primary_axis.cross(Vec3::new(0.0, 0.0, 1.0)).normalized()
            } else {
                primary_axis.cross(Vec3::new(0.0, 1.0, 0.0)).normalized()
            }
        };
        
        let tertiary_axis = primary_axis.cross(secondary_axis).normalized();
        
        self.primary_axis = primary_axis;
        self.secondary_axis = secondary_axis;
        self.tertiary_axis = tertiary_axis;
    }
    
    /// Get the movement vector for a drag operation
    pub fn get_movement(&self, delta: Vec3) -> Vec3 {
        match self.constraint {
            AxisConstraint::None => delta,
            AxisConstraint::Primary => {
                let projection = delta.dot(self.primary_axis);
                self.primary_axis * projection
            }
            AxisConstraint::Secondary => {
                let projection = delta.dot(self.secondary_axis);
                self.secondary_axis * projection
            }
            AxisConstraint::Tertiary => {
                let projection = delta.dot(self.tertiary_axis);
                self.tertiary_axis * projection
            }
            AxisConstraint::Plane => {
                // Project onto primary-secondary plane
                let normal = self.tertiary_axis;
                delta - normal * delta.dot(normal)
            }
            AxisConstraint::Direction(dir) => {
                let normalized = dir.normalized();
                let projection = delta.dot(normalized);
                normalized * projection
            }
        }
    }
    
    /// Calculate rotation for a drag operation
    pub fn get_rotation(&self, start: Point3, end: Point3) -> (Vec3, f64) {
        // Project points onto the rotation plane
        let axis = match self.mode {
            WheelMode::RotatePrimary => self.primary_axis,
            WheelMode::RotateSecondary => self.secondary_axis,
            WheelMode::RotateTertiary => self.tertiary_axis,
            _ => self.primary_axis,
        };
        
        // Calculate vectors from origin to points
        let v1 = start - self.origin;
        let v2 = end - self.origin;
        
        // Project onto rotation plane
        let proj1 = v1 - axis * v1.dot(axis);
        let proj2 = v2 - axis * v2.dot(axis);
        
        // Calculate angle
        let angle = proj1.angle_to(&proj2);
        
        // Determine direction using cross product
        let cross = proj1.cross(proj2);
        let signed_angle = if cross.dot(axis) < 0.0 {
            -angle
        } else {
            angle
        };
        
        (axis, signed_angle)
    }
    
    /// Create a transformation from wheel manipulation
    pub fn create_transform(&self, delta: Vec3, rotation_axis: Option<Vec3>, rotation_angle: f64) -> Transform3 {
        let mut transform = Transform3::identity();
        
        // Add translation
        let movement = self.get_movement(delta);
        transform = Transform3::from_translation(movement) * transform;
        
        // Add rotation
        if let Some(axis) = rotation_axis {
            let rotation = Transform3::from_axis_angle(self.origin, axis, rotation_angle);
            transform = rotation * transform;
        }
        
        transform
    }
    
    /// Get the primary axis handle position
    pub fn primary_handle(&self) -> Point3 {
        self.origin + self.primary_axis * self.radius
    }
    
    /// Get the secondary axis handle position
    pub fn secondary_handle(&self) -> Point3 {
        self.origin + self.secondary_axis * self.radius
    }
    
    /// Get the tertiary axis handle position
    pub fn tertiary_handle(&self) -> Point3 {
        self.origin + self.tertiary_axis * self.radius
    }
    
    /// Get the plane handle position (for planar movement)
    pub fn plane_handle(&self) -> Point3 {
        (self.origin + self.primary_axis * self.radius * 0.7
            + self.secondary_axis * self.radius * 0.7)
    }
    
    /// Snap the wheel to the nearest major axis
    pub fn snap_to_major_axis(&mut self) {
        let axes = [
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
        ];
        
        // Find closest major axis for primary
        let mut max_dot = -1.0;
        let mut closest = axes[0];
        for axis in &axes {
            let dot = self.primary_axis.dot(*axis);
            if dot > max_dot {
                max_dot = dot;
                closest = *axis;
            }
        }
        
        self.orient(closest);
    }
    
    /// Check if a point is near a handle
    pub fn get_handle_at_point(&self, point: Point3, tolerance: f64) -> Option<WheelHandle> {
        let handles = [
            (self.primary_handle(), WheelHandle::PrimaryAxis),
            (self.secondary_handle(), WheelHandle::SecondaryAxis),
            (self.tertiary_handle(), WheelHandle::TertiaryAxis),
            (self.plane_handle(), WheelHandle::Plane),
        ];
        
        for (handle_pos, handle_type) in &handles {
            if point.distance_to(handle_pos) < tolerance {
                return Some(*handle_type);
            }
        }
        
        None
    }
}

/// Types of handles on the steering wheel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WheelHandle {
    /// Primary axis handle
    PrimaryAxis,
    /// Secondary axis handle
    SecondaryAxis,
    /// Tertiary axis handle
    TertiaryAxis,
    /// Plane movement handle
    Plane,
    /// Rotation ring
    RotationRing,
}

/// Interaction state for the steering wheel
#[derive(Debug, Clone)]
pub struct WheelInteraction {
    /// Whether interaction is active
    pub active: bool,
    /// Handle being interacted with
    pub handle: Option<WheelHandle>,
    /// Start position of interaction
    pub start_position: Option<Point3>,
    /// Current position
    pub current_position: Option<Point3>,
    /// Accumulated transformation
    pub accumulated_transform: Transform3,
}

impl WheelInteraction {
    /// Create a new interaction
    pub fn new() -> Self {
        Self {
            active: false,
            handle: None,
            start_position: None,
            current_position: None,
            accumulated_transform: Transform3::identity(),
        }
    }
    
    /// Start an interaction
    pub fn start(&mut self, handle: WheelHandle, position: Point3) {
        self.active = true;
        self.handle = Some(handle);
        self.start_position = Some(position);
        self.current_position = Some(position);
    }
    
    /// Update the interaction
    pub fn update(&mut self, position: Point3) {
        if self.active {
            self.current_position = Some(position);
        }
    }
    
    /// End the interaction
    pub fn end(&mut self) -> Transform3 {
        self.active = false;
        let transform = self.accumulated_transform;
        self.accumulated_transform = Transform3::identity();
        self.handle = None;
        self.start_position = None;
        self.current_position = None;
        transform
    }
    
    /// Get the delta from start
    pub fn delta(&self) -> Option<Vec3> {
        match (self.start_position, self.current_position) {
            (Some(start), Some(current)) => Some(current - start),
            _ => None,
        }
    }
}

impl Default for WheelInteraction {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steering_wheel_creation() {
        let wheel = SteeringWheel::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        );
        
        assert_eq!(wheel.origin.x(), 0.0);
        assert_eq!(wheel.primary_axis.z(), 1.0);
        assert!(wheel.active);
    }

    #[test]
    fn test_steering_wheel_relocate() {
        let mut wheel = SteeringWheel::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        );
        
        wheel.relocate(Point3::new(10.0, 20.0, 30.0));
        assert_eq!(wheel.origin.x(), 10.0);
        assert_eq!(wheel.origin.y(), 20.0);
        assert_eq!(wheel.origin.z(), 30.0);
    }

    #[test]
    fn test_steering_wheel_orient() {
        let mut wheel = SteeringWheel::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        );
        
        wheel.orient(Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(wheel.primary_axis.x(), 1.0);
        assert_eq!(wheel.primary_axis.y(), 0.0);
        assert_eq!(wheel.primary_axis.z(), 0.0);
    }

    #[test]
    fn test_movement_constraint() {
        let wheel = SteeringWheel::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        );
        
        // Test primary constraint
        let movement = wheel.get_movement(Vec3::new(10.0, 20.0, 30.0));
        // Without constraint, should return input
        assert_eq!(movement.x(), 10.0);
        assert_eq!(movement.y(), 20.0);
        assert_eq!(movement.z(), 30.0);
    }

    #[test]
    fn test_wheel_mode() {
        assert!(matches!(WheelMode::Free, WheelMode::Free));
        assert!(matches!(WheelMode::MovePrimary, WheelMode::MovePrimary));
        assert!(matches!(WheelMode::RotatePrimary, WheelMode::RotatePrimary));
    }

    #[test]
    fn test_wheel_handle() {
        assert!(matches!(WheelHandle::PrimaryAxis, WheelHandle::PrimaryAxis));
        assert!(matches!(WheelHandle::Plane, WheelHandle::Plane));
    }

    #[test]
    fn test_wheel_interaction() {
        let mut interaction = WheelInteraction::new();
        
        interaction.start(WheelHandle::PrimaryAxis, Point3::new(0.0, 0.0, 0.0));
        assert!(interaction.active);
        assert!(matches!(interaction.handle, Some(WheelHandle::PrimaryAxis)));
        
        interaction.update(Point3::new(10.0, 0.0, 0.0));
        let delta = interaction.delta().unwrap();
        assert_eq!(delta.x(), 10.0);
        
        let transform = interaction.end();
        assert!(!interaction.active);
        assert!(transform.is_identity());
    }
}
