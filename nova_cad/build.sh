#!/bin/bash

# Nova CAD Build Script

set -e

echo "======================================"
echo "Nova Kernel 3D + Nova CAD Build"
echo "======================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored messages
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Build Rust kernel
build_kernel() {
    print_info "Building Nova Kernel 3D (Rust)..."
    cd nova_kernel
    
    # Check if cargo is installed
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo not found. Please install Rust: https://rustup.rs/"
        exit 1
    fi
    
    # Build the kernel
    cargo build --release
    
    if [ $? -eq 0 ]; then
        print_info "Kernel built successfully!"
        print_info "Library location: target/release/libnova_ffi.so (Linux)"
        print_info "                  target/release/nova_ffi.dll (Windows)"
        print_info "                  target/release/libnova_ffi.dylib (macOS)"
    else
        print_error "Kernel build failed!"
        exit 1
    fi
    
    cd ..
}

# Build C# application
build_app() {
    print_info "Building Nova CAD Application (C#)..."
    cd NovaCAD
    
    # Check if dotnet is installed
    if ! command -v dotnet &> /dev/null; then
        print_error ".NET SDK not found. Please install .NET 8: https://dotnet.microsoft.com/"
        exit 1
    fi
    
    # Restore packages
    print_info "Restoring NuGet packages..."
    dotnet restore
    
    # Build the solution
    dotnet build
    
    if [ $? -eq 0 ]; then
        print_info "Application built successfully!"
    else
        print_error "Application build failed!"
        exit 1
    fi
    
    cd ..
}

# Run tests
test_kernel() {
    print_info "Running kernel tests..."
    cd nova_kernel
    cargo test
    cd ..
}

# Run application
run_app() {
    print_info "Running Nova CAD..."
    cd NovaCAD
    dotnet run --project src/NovaCad.App
    cd ..
}

# Print usage
usage() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  all       Build everything (kernel + app) [default]"
    echo "  kernel    Build only the Rust kernel"
    echo "  app       Build only the C# application"
    echo "  test      Run kernel tests"
    echo "  run       Build and run the application"
    echo "  clean     Clean build artifacts"
    echo "  help      Show this help message"
}

# Clean build artifacts
clean() {
    print_info "Cleaning build artifacts..."
    cd nova_kernel && cargo clean && cd ..
    cd NovaCAD && dotnet clean && cd ..
    print_info "Clean complete!"
}

# Main script logic
case "${1:-all}" in
    all)
        build_kernel
        build_app
        print_info "Build complete!"
        ;;
    kernel)
        build_kernel
        ;;
    app)
        build_app
        ;;
    test)
        test_kernel
        ;;
    run)
        build_kernel
        build_app
        run_app
        ;;
    clean)
        clean
        ;;
    help|--help|-h)
        usage
        ;;
    *)
        print_error "Unknown command: $1"
        usage
        exit 1
        ;;
esac
