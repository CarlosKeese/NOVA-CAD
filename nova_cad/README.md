# Nova CAD

A modern, lightweight 3D CAD application inspired by [Plasticity](https://www.plasticity.xyz/) and [Shapr3D](https://www.shapr3d.com/), featuring a Rust-based geometry kernel, C# UI with hardware-accelerated OpenGL viewport, and Python scripting support.

## 🎯 Vision

Nova CAD aims to bring the intuitive, direct modeling experience of Plasticity and the precision of Shapr3D to a cross-platform, open-source CAD solution. Unlike traditional parametric CAD systems, Nova CAD focuses on:

- **Direct Modeling**: Push-pull, drag, and manipulate geometry directly
- **Modern UI**: Clean, distraction-free interface with dark theme
- **Performance**: Hardware-accelerated rendering with OpenGL
- **Extensibility**: Python scripting for automation and custom workflows
- **Cross-Platform**: Windows, macOS, and Linux support (future)

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    USER INTERFACE (C#)                       │
│              Avalonia UI + OpenGL Viewport                  │
│  ┌─────────┐  ┌──────────────┐  ┌─────────────────────┐    │
│  │  Menu   │  │   Toolbar    │  │   Property Panel    │    │
│  │  Bar    │  │   (Ribbon)   │  │                     │    │
│  └─────────┘  └──────────────┘  └─────────────────────┘    │
│  ┌─────────┐  ┌──────────────┐  ┌─────────────────────┐    │
│  │ Model   │  │   Viewport   │  │   Python Console    │    │
│  │ Tree    │  │   (OpenGL)   │  │   (Scripting)       │    │
│  └─────────┘  └──────────────┘  └─────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                 GEOMETRY KERNEL (Rust)                       │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │  B-Rep      │  │   Primitives │  │   Booleans       │   │
│  │  (Boundary  │  │   (Box,      │  │   (Union,        │   │
│  │  Rep)       │  │   Sphere...) │  │   Subtract...)   │   │
│  └─────────────┘  └──────────────┘  └──────────────────┘   │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │  Curves     │  │   Surfaces   │  │   Tessellation   │   │
│  │  (NURBS)    │  │   (NURBS)    │  │   (Mesh Gen)     │   │
│  └─────────────┘  └──────────────┘  └──────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   SCRIPTING (Python)                         │
│              Automation, Macros, Extensions                 │
└─────────────────────────────────────────────────────────────┘
```

### Why This Stack?

| Component | Language | Rationale |
|-----------|----------|-----------|
| **Geometry Kernel** | Rust | Memory safety, performance, robust B-Rep algorithms |
| **UI & Viewport** | C# / Avalonia | Excellent Windows integration, memory management, rich visual capabilities |
| **Scripting** | Python | Industry standard, easy learning curve, vast ecosystem |
| **Rendering** | OpenGL | Cross-platform, hardware-accelerated, proven CAD viewport solution |

> **Note on C#**: While C# provides superior memory management and UI capabilities on Windows, we're open to migrating critical viewport code to C++ with raw OpenGL if performance demands it.

## ✨ Features

### Current (MVP)
- [x] Modern dark-themed UI (Avalonia)
- [x] Hardware-accelerated OpenGL viewport
- [x] Interactive camera (orbit, pan, zoom)
- [x] Grid and reference axes (XYZ)
- [x] Primitive creation (Box, Cylinder, Sphere)
- [x] Model tree with visibility controls
- [x] View presets (Front, Top, Right, Isometric)
- [x] Render modes (Shaded, Wireframe, Shaded with Edges)

### In Development
- [ ] Selection and transformation tools
- [ ] Edge/face fillets and chamfers
- [ ] Boolean operations (union, subtract, intersect)
- [ ] Python scripting console
- [ ] STEP/IGES import/export
- [ ] STL export for 3D printing

### Future Roadmap
- [ ] Sketch-based modeling (2D profiles → 3D)
- [ ] Assembly design
- [ ] Constraints and parametric relationships
- [ ] Version control integration (Git for CAD)
- [ ] Cloud sync and collaboration
- [ ] macOS and Linux support

## 🎨 UI Philosophy (Inspired by Plasticity & Shapr3D)

### From Plasticity:
- **Direct manipulation**: Click and drag to modify geometry
- **Minimal UI chrome**: Maximum space for the viewport
- **Contextual tools**: Tools appear based on selection
- **Smooth interactions**: 60fps viewport, responsive controls

### From Shapr3D:
- **Precision input**: Numeric input alongside gestures
- **Clean iconography**: Intuitive, consistent icons
- **Layer/Body organization**: Clear hierarchy in model tree
- **Professional output**: Manufacturing-ready geometry

### Nova CAD Design Principles:
1. **Zero-click workflow**: Common actions should be immediately accessible
2. **Visual feedback**: Highlight, preview, and ghost states for all operations
3. **Keyboard-centric**: Full hotkey support for power users
4. **Dark mode first**: Easy on the eyes for long modeling sessions

## 🚀 Getting Started

### Prerequisites
- Windows 10/11
- .NET 8.0 SDK
- GPU with OpenGL ES 3.0+ support

### Build & Run
```bash
cd nova_cad/NovaCAD
dotnet build
dotnet run --project src/NovaCad.App/NovaCad.App.csproj
```

### Usage
1. **Navigate**: Middle-click + drag to orbit, Shift + Middle-click to pan, Scroll to zoom
2. **Create Geometry**: Use Create menu or toolbar buttons (Box, Cylinder, Sphere)
3. **Change Views**: Use View menu or toolbar (Front, Top, Isometric)
4. **Fit to Screen**: Press 'F' or click Fit All button

## 🛠️ Development

### Project Structure
```
nova_cad/
├── NovaCAD/
│   ├── src/
│   │   ├── NovaCad.App/          # Main application (C#)
│   │   ├── NovaCad.Core/         # Core models and logic (C#)
│   │   ├── NovaCad.Kernel/       # Rust kernel bindings (C#)
│   │   ├── NovaCad.Viewport/     # OpenGL viewport (C#)
│   │   └── NovaCad.UI/           # UI controls and themes
│   └── NovaCAD.sln
└── nova_kernel/                   # Rust geometry kernel
    ├── nova_geom/                 # Geometry primitives
    ├── nova_topo/                 # B-Rep topology
    ├── nova_io/                   # File I/O (STEP, STL)
    └── nova_ffi/                  # C FFI bindings
```

### Rust Kernel Components
| Crate | Purpose |
|-------|---------|
| `nova_math` | Vectors, matrices, numerical utilities |
| `nova_geom` | Curves, surfaces, NURBS |
| `nova_topo` | B-Rep: solids, shells, faces, edges, vertices |
| `nova_io` | Import/export formats |
| `nova_ffi` | C-compatible interface for C# interop |

## 🤝 Contributing

Contributions are welcome! Areas where help is needed:
- OpenGL viewport optimizations
- Additional geometry primitives
- Import/export format support
- Documentation and tutorials
- Bug testing and reports

## 📄 License

MIT License - See LICENSE file for details

## 🙏 Acknowledgments

- **Plasticity** by Indro Software - For reimagining CAD interaction
- **Shapr3D** - For proving CAD can be both simple and powerful
- **Open CASCADE** - For B-Rep algorithm inspiration
- **Avalonia UI** - For cross-platform .NET UI framework
- **Rust Community** - For exceptional geometry crates

---

**Status**: Alpha - Active Development  
**Last Updated**: 2026-02-08
