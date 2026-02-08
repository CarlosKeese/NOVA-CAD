//! Nova Kernel 3D - Complete CAD Kernel
//!
//! This is the main crate that re-exports all Nova Kernel functionality.
//!
//! ## Modules
//!
//! - `math`: Mathematical foundations (points, vectors, matrices, transforms)
//! - `geom`: Geometric entities (curves, surfaces, NURBS)
//! - `topo`: Topological entities (B-Rep structure)
//! - `ops`: CAD operations (Boolean, features, fillets)
//! - `io`: Import/Export (STEP, IGES, STL)
//! - `tess`: Tessellation for rendering
//! - `check`: Validation and healing
//! - `sync`: Direct editing (Synchronous Technology)
//! - `ffi`: C FFI for interop with other languages (temporarily disabled)

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

// Re-export all sub-crates
pub use nova_math;
pub use nova_geom;
pub use nova_topo;
pub use nova_ops;
pub use nova_io;
pub use nova_tess;
pub use nova_check;
pub use nova_sync;
// pub use nova_ffi;  // Temporarily disabled due to MSVC linking issues

/// Version of the Nova Kernel
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get the kernel version string
pub fn version() -> &'static str {
    VERSION
}

/// Initialize the Nova Kernel
/// 
/// This should be called before using any kernel functionality.
pub fn init() {
    // Initialization code if needed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
}
