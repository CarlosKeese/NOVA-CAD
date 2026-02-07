//! Nova FFI - C-ABI FFI Layer for Nova Kernel 3D
//! 
//! This crate exposes the Nova Kernel API through a C-compatible ABI,
//! allowing the kernel to be used from C#, Python, C++, and other languages.

use nova_math::{Point3, Vec3, Transform3};
use std::ffi::{c_char, c_double, c_void, CStr};
use std::sync::Mutex;
use once_cell::sync::Lazy;

// Re-export types for C interface
pub type NovaReal = c_double;
pub type NovaHandle = u64;

/// Invalid/null handle
pub const NOVA_NULL_HANDLE: NovaHandle = 0;

/// Result codes for Nova operations
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NovaResult {
    /// Success
    Success = 0,
    /// Invalid handle
    InvalidHandle = 1,
    /// Invalid parameter
    InvalidParameter = 2,
    /// Out of memory
    OutOfMemory = 3,
    /// Geometry error
    GeometryError = 4,
    /// Topology error
    TopologyError = 5,
    /// Not implemented
    NotImplemented = 6,
    /// Unknown error
    UnknownError = 7,
}

/// 3D point structure for C interface
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NovaPoint3 {
    pub x: NovaReal,
    pub y: NovaReal,
    pub z: NovaReal,
}

impl NovaPoint3 {
    /// Create from Point3
    pub fn from_point3(p: &Point3) -> Self {
        Self {
            x: p.x(),
            y: p.y(),
            z: p.z(),
        }
    }
    
    /// Convert to Point3
    pub fn to_point3(&self) -> Point3 {
        Point3::new(self.x, self.y, self.z)
    }
}

impl Default for NovaPoint3 {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
}

/// 3D vector structure for C interface
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NovaVec3 {
    pub x: NovaReal,
    pub y: NovaReal,
    pub z: NovaReal,
}

impl NovaVec3 {
    /// Create from Vec3
    pub fn from_vec3(v: &Vec3) -> Self {
        Self {
            x: v.x(),
            y: v.y(),
            z: v.z(),
        }
    }
    
    /// Convert to Vec3
    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl Default for NovaVec3 {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
}

/// 4x4 matrix for C interface (row-major)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NovaMat4 {
    pub m: [NovaReal; 16],
}

impl Default for NovaMat4 {
    fn default() -> Self {
        let mut m = [0.0; 16];
        m[0] = 1.0;
        m[5] = 1.0;
        m[10] = 1.0;
        m[15] = 1.0;
        Self { m }
    }
}

/// Transform structure for C interface
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NovaTransform {
    pub translation: NovaPoint3,
    pub rotation: [NovaReal; 4], // Quaternion (w, x, y, z)
}

impl Default for NovaTransform {
    fn default() -> Self {
        Self {
            translation: NovaPoint3::default(),
            rotation: [1.0, 0.0, 0.0, 0.0],
        }
    }
}

/// Bounding box for C interface
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NovaBBox3 {
    pub min: NovaPoint3,
    pub max: NovaPoint3,
}

impl Default for NovaBBox3 {
    fn default() -> Self {
        Self {
            min: NovaPoint3 { x: f64::INFINITY, y: f64::INFINITY, z: f64::INFINITY },
            max: NovaPoint3 { x: f64::NEG_INFINITY, y: f64::NEG_INFINITY, z: f64::NEG_INFINITY },
        }
    }
}

/// Mesh vertex structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NovaMeshVertex {
    pub position: NovaPoint3,
    pub normal: NovaVec3,
    pub u: NovaReal,
    pub v: NovaReal,
}

/// Mesh structure
#[repr(C)]
pub struct NovaMesh {
    pub vertices: *mut NovaMeshVertex,
    pub vertex_count: u32,
    pub indices: *mut u32,
    pub index_count: u32,
}

// Global context for the kernel
static NOVA_CONTEXT: Lazy<Mutex<NovaContext>> = Lazy::new(|| {
    Mutex::new(NovaContext::new())
});

/// Context for Nova kernel operations
pub struct NovaContext {
    tolerance: f64,
    next_handle: NovaHandle,
    bodies: std::collections::HashMap<NovaHandle, ()>, // Placeholder
}

impl NovaContext {
    fn new() -> Self {
        Self {
            tolerance: 1e-6,
            next_handle: 1,
            bodies: std::collections::HashMap::new(),
        }
    }
    
    fn allocate_handle(&mut self) -> NovaHandle {
        let handle = self.next_handle;
        self.next_handle += 1;
        handle
    }
}

/// Initialize the Nova kernel
/// 
/// Must be called before any other Nova functions.
#[no_mangle]
pub extern "C" fn nova_init() -> NovaResult {
    NovaResult::Success
}

/// Shutdown the Nova kernel
/// 
/// Releases all resources.
#[no_mangle]
pub extern "C" fn nova_shutdown() -> NovaResult {
    NovaResult::Success
}

/// Get the version string
#[no_mangle]
pub extern "C" fn nova_version() -> *const c_char {
    const VERSION: &[u8] = b"Nova Kernel 3D v0.1.0\0";
    VERSION.as_ptr() as *const c_char
}

/// Set global tolerance
#[no_mangle]
pub extern "C" fn nova_set_tolerance(tolerance: NovaReal) -> NovaResult {
    if let Ok(mut ctx) = NOVA_CONTEXT.lock() {
        ctx.tolerance = tolerance;
        NovaResult::Success
    } else {
        NovaResult::UnknownError
    }
}

/// Get global tolerance
#[no_mangle]
pub extern "C" fn nova_get_tolerance() -> NovaReal {
    if let Ok(ctx) = NOVA_CONTEXT.lock() {
        ctx.tolerance
    } else {
        1e-6
    }
}

// ============================================================================
// Primitive Creation
// ============================================================================

/// Create a box primitive
/// 
/// Returns a handle to the created body.
#[no_mangle]
pub extern "C" fn nova_make_box(
    width: NovaReal,
    height: NovaReal,
    depth: NovaReal,
    out_handle: *mut NovaHandle,
) -> NovaResult {
    if out_handle.is_null() {
        return NovaResult::InvalidParameter;
    }
    
    if let Ok(mut ctx) = NOVA_CONTEXT.lock() {
        let handle = ctx.allocate_handle();
        // In a full implementation, create the actual body
        unsafe {
            *out_handle = handle;
        }
        NovaResult::Success
    } else {
        NovaResult::UnknownError
    }
}

/// Create a cylinder primitive
#[no_mangle]
pub extern "C" fn nova_make_cylinder(
    radius: NovaReal,
    height: NovaReal,
    out_handle: *mut NovaHandle,
) -> NovaResult {
    if out_handle.is_null() {
        return NovaResult::InvalidParameter;
    }
    
    if let Ok(mut ctx) = NOVA_CONTEXT.lock() {
        let handle = ctx.allocate_handle();
        unsafe {
            *out_handle = handle;
        }
        NovaResult::Success
    } else {
        NovaResult::UnknownError
    }
}

/// Create a sphere primitive
#[no_mangle]
pub extern "C" fn nova_make_sphere(
    radius: NovaReal,
    out_handle: *mut NovaHandle,
) -> NovaResult {
    if out_handle.is_null() {
        return NovaResult::InvalidParameter;
    }
    
    if let Ok(mut ctx) = NOVA_CONTEXT.lock() {
        let handle = ctx.allocate_handle();
        unsafe {
            *out_handle = handle;
        }
        NovaResult::Success
    } else {
        NovaResult::UnknownError
    }
}

/// Create a cone primitive
#[no_mangle]
pub extern "C" fn nova_make_cone(
    base_radius: NovaReal,
    top_radius: NovaReal,
    height: NovaReal,
    out_handle: *mut NovaHandle,
) -> NovaResult {
    if out_handle.is_null() {
        return NovaResult::InvalidParameter;
    }
    
    if let Ok(mut ctx) = NOVA_CONTEXT.lock() {
        let handle = ctx.allocate_handle();
        unsafe {
            *out_handle = handle;
        }
        NovaResult::Success
    } else {
        NovaResult::UnknownError
    }
}

/// Create a torus primitive
#[no_mangle]
pub extern "C" fn nova_make_torus(
    major_radius: NovaReal,
    minor_radius: NovaReal,
    out_handle: *mut NovaHandle,
) -> NovaResult {
    if out_handle.is_null() {
        return NovaResult::InvalidParameter;
    }
    
    if let Ok(mut ctx) = NOVA_CONTEXT.lock() {
        let handle = ctx.allocate_handle();
        unsafe {
            *out_handle = handle;
        }
        NovaResult::Success
    } else {
        NovaResult::UnknownError
    }
}

// ============================================================================
// Body Operations
// ============================================================================

/// Release a body handle
#[no_mangle]
pub extern "C" fn nova_body_release(handle: NovaHandle) -> NovaResult {
    if handle == NOVA_NULL_HANDLE {
        return NovaResult::InvalidHandle;
    }
    
    if let Ok(mut ctx) = NOVA_CONTEXT.lock() {
        ctx.bodies.remove(&handle);
        NovaResult::Success
    } else {
        NovaResult::UnknownError
    }
}

/// Transform a body
#[no_mangle]
pub extern "C" fn nova_body_transform(
    handle: NovaHandle,
    transform: *const NovaTransform,
) -> NovaResult {
    if handle == NOVA_NULL_HANDLE {
        return NovaResult::InvalidHandle;
    }
    
    if transform.is_null() {
        return NovaResult::InvalidParameter;
    }
    
    // In a full implementation, apply the transform
    NovaResult::NotImplemented
}

/// Get body bounding box
#[no_mangle]
pub extern "C" fn nova_body_bounding_box(
    handle: NovaHandle,
    out_bbox: *mut NovaBBox3,
) -> NovaResult {
    if handle == NOVA_NULL_HANDLE {
        return NovaResult::InvalidHandle;
    }
    
    if out_bbox.is_null() {
        return NovaResult::InvalidParameter;
    }
    
    unsafe {
        *out_bbox = NovaBBox3::default();
    }
    
    NovaResult::NotImplemented
}

/// Copy a body
#[no_mangle]
pub extern "C" fn nova_body_copy(
    handle: NovaHandle,
    out_handle: *mut NovaHandle,
) -> NovaResult {
    if handle == NOVA_NULL_HANDLE {
        return NovaResult::InvalidHandle;
    }
    
    if out_handle.is_null() {
        return NovaResult::InvalidParameter;
    }
    
    NovaResult::NotImplemented
}

// ============================================================================
// Boolean Operations
// ============================================================================

/// Boolean unite (union)
#[no_mangle]
pub extern "C" fn nova_boolean_unite(
    body_a: NovaHandle,
    body_b: NovaHandle,
    out_result: *mut NovaHandle,
) -> NovaResult {
    if body_a == NOVA_NULL_HANDLE || body_b == NOVA_NULL_HANDLE {
        return NovaResult::InvalidHandle;
    }
    
    if out_result.is_null() {
        return NovaResult::InvalidParameter;
    }
    
    NovaResult::NotImplemented
}

/// Boolean subtract
#[no_mangle]
pub extern "C" fn nova_boolean_subtract(
    body_a: NovaHandle,
    body_b: NovaHandle,
    out_result: *mut NovaHandle,
) -> NovaResult {
    if body_a == NOVA_NULL_HANDLE || body_b == NOVA_NULL_HANDLE {
        return NovaResult::InvalidHandle;
    }
    
    if out_result.is_null() {
        return NovaResult::InvalidParameter;
    }
    
    NovaResult::NotImplemented
}

/// Boolean intersect
#[no_mangle]
pub extern "C" fn nova_boolean_intersect(
    body_a: NovaHandle,
    body_b: NovaHandle,
    out_result: *mut NovaHandle,
) -> NovaResult {
    if body_a == NOVA_NULL_HANDLE || body_b == NOVA_NULL_HANDLE {
        return NovaResult::InvalidHandle;
    }
    
    if out_result.is_null() {
        return NovaResult::InvalidParameter;
    }
    
    NovaResult::NotImplemented
}

// ============================================================================
// Feature Operations
// ============================================================================

/// Fillet edges
#[no_mangle]
pub extern "C" fn nova_fillet(
    body: NovaHandle,
    edges: *const NovaHandle,
    edge_count: u32,
    radius: NovaReal,
    out_result: *mut NovaHandle,
) -> NovaResult {
    if body == NOVA_NULL_HANDLE {
        return NovaResult::InvalidHandle;
    }
    
    if edges.is_null() || out_result.is_null() {
        return NovaResult::InvalidParameter;
    }
    
    NovaResult::NotImplemented
}

/// Chamfer edges
#[no_mangle]
pub extern "C" fn nova_chamfer(
    body: NovaHandle,
    edges: *const NovaHandle,
    edge_count: u32,
    distance1: NovaReal,
    distance2: NovaReal,
    out_result: *mut NovaHandle,
) -> NovaResult {
    if body == NOVA_NULL_HANDLE {
        return NovaResult::InvalidHandle;
    }
    
    if edges.is_null() || out_result.is_null() {
        return NovaResult::InvalidParameter;
    }
    
    NovaResult::NotImplemented
}

/// Shell a body
#[no_mangle]
pub extern "C" fn nova_shell(
    body: NovaHandle,
    faces: *const NovaHandle,
    face_count: u32,
    thickness: NovaReal,
    out_result: *mut NovaHandle,
) -> NovaResult {
    if body == NOVA_NULL_HANDLE {
        return NovaResult::InvalidHandle;
    }
    
    if out_result.is_null() {
        return NovaResult::InvalidParameter;
    }
    
    NovaResult::NotImplemented
}

// ============================================================================
// Tessellation
// ============================================================================

/// Tessellate a body
#[no_mangle]
pub extern "C" fn nova_tessellate_body(
    body: NovaHandle,
    chord_tolerance: NovaReal,
    angle_tolerance: NovaReal,
    out_mesh: *mut NovaMesh,
) -> NovaResult {
    if body == NOVA_NULL_HANDLE {
        return NovaResult::InvalidHandle;
    }
    
    if out_mesh.is_null() {
        return NovaResult::InvalidParameter;
    }
    
    NovaResult::NotImplemented
}

/// Free a mesh
#[no_mangle]
pub extern "C" fn nova_mesh_free(mesh: *mut NovaMesh) -> NovaResult {
    if mesh.is_null() {
        return NovaResult::InvalidParameter;
    }
    
    // In a full implementation, free the mesh data
    NovaResult::NotImplemented
}

// ============================================================================
// File I/O
// ============================================================================

/// Import STEP file
#[no_mangle]
pub extern "C" fn nova_import_step(
    filepath: *const c_char,
    out_handle: *mut NovaHandle,
) -> NovaResult {
    if filepath.is_null() || out_handle.is_null() {
        return NovaResult::InvalidParameter;
    }
    
    let path = unsafe {
        match CStr::from_ptr(filepath).to_str() {
            Ok(s) => s,
            Err(_) => return NovaResult::InvalidParameter,
        }
    };
    
    // In a full implementation, parse the STEP file
    NovaResult::NotImplemented
}

/// Export STEP file
#[no_mangle]
pub extern "C" fn nova_export_step(
    body: NovaHandle,
    filepath: *const c_char,
) -> NovaResult {
    if body == NOVA_NULL_HANDLE {
        return NovaResult::InvalidHandle;
    }
    
    if filepath.is_null() {
        return NovaResult::InvalidParameter;
    }
    
    NovaResult::NotImplemented
}

/// Export STL file
#[no_mangle]
pub extern "C" fn nova_export_stl(
    body: NovaHandle,
    filepath: *const c_char,
) -> NovaResult {
    if body == NOVA_NULL_HANDLE {
        return NovaResult::InvalidHandle;
    }
    
    if filepath.is_null() {
        return NovaResult::InvalidParameter;
    }
    
    NovaResult::NotImplemented
}

// ============================================================================
// Error Handling
// ============================================================================

/// Get the last error message
#[no_mangle]
pub extern "C" fn nova_last_error() -> *const c_char {
    const NO_ERROR: &[u8] = b"No error\0";
    NO_ERROR.as_ptr() as *const c_char
}

/// Clear the last error
#[no_mangle]
pub extern "C" fn nova_clear_error() {
    // Clear the last error
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nova_init() {
        assert_eq!(nova_init(), NovaResult::Success);
    }

    #[test]
    fn test_nova_tolerance() {
        nova_set_tolerance(1e-4);
        assert!((nova_get_tolerance() - 1e-4).abs() < 1e-10);
    }

    #[test]
    fn test_nova_make_box() {
        let mut handle: NovaHandle = 0;
        let result = nova_make_box(10.0, 20.0, 30.0, &mut handle);
        assert_eq!(result, NovaResult::Success);
        assert_ne!(handle, NOVA_NULL_HANDLE);
    }
}
