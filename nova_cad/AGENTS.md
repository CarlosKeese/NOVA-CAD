# Nova CAD - Agent Development Guidelines

## Project Overview

Nova CAD is a modern 3D CAD application combining:
- **Rust Geometry Kernel**: High-performance B-Rep modeling
- **C# UI/Viewport**: Windows-optimized interface with OpenGL
- **Python Scripting**: Automation and extensibility

Design inspiration: [Plasticity](https://www.plasticity.xyz/) and [Shapr3D](https://www.shapr3d.com/)

## Architecture Stack

```
┌────────────────────────────────────────┐
│  UI Layer (C# + Avalonia + OpenGL)    │
│  - Viewport3D with hardware accel     │
│  - Ribbon toolbar, model tree         │
│  - Property panels, scripting console │
├────────────────────────────────────────┤
│  Interop Layer (C# P/Invoke)          │
│  - Automatic stub fallback            │
│  - Graceful degradation               │
├────────────────────────────────────────┤
│  Kernel Layer (Rust)                  │
│  - B-Rep geometry                     │
│  - Boolean operations                 │
│  - Tessellation for rendering         │
└────────────────────────────────────────┘
```

## Coding Standards

### C# (UI & Viewport)
- Use `CommunityToolkit.Mvvm` for MVVM pattern
- Use `Avalonia.OpenGL` for viewport (not Silk.NET)
- Implement `IDisposable` for all OpenGL resources
- Use `unsafe` keyword sparingly, prefer `Marshal` for interop
- Log extensively using `ViewportDiagnostics` class

### Rust (Kernel)
- Follow `nova_kernel` crate structure
- Use `#[repr(C)]` for all FFI-exposed types
- Return `Result<T, NovaError>` for all fallible operations
- Keep FFI boundary minimal and well-documented

### Python (Scripting - Future)
- Use `pythonnet` for .NET interop
- Follow Blender/Maya scripting conventions
- Provide both procedural and declarative APIs

## UI Design Principles (Plasticity/Shapr3D Style)

### 1. Direct Modeling First
```csharp
// Good: Immediate visual feedback
public void CreateBox(float w, float h, float d) {
    var mesh = MeshFactory.CreateBox(w, h, d);
    mesh.Initialize(gl); // Immediate OpenGL upload
    viewport.AddMesh(mesh);
    RequestRender();
}
```

### 2. Minimal Chrome, Maximum Viewport
- Toolbar: Icon + text only when space permits
- Model tree: Collapsible, auto-hide option
- Properties: Contextual, appear on selection

### 3. Dark Theme
```csharp
// Standard colors
BackgroundColor = new Color(0.15f, 0.15f, 0.15f, 1.0f);  // #262626
GridColor = new Color(0.3f, 0.3f, 0.3f, 1.0f);            // #4D4D4D
SelectionColor = new Color(0.2f, 0.6f, 1.0f, 1.0f);       // #3399FF
```

### 4. Axis Colors (Industry Standard)
- X (Red): `#FF0000`
- Y (Green): `#00FF00`  
- Z (Blue): `#0000FF`

## Debugging Guidelines

### Viewport Issues
1. Check logs: `%LOCALAPPDATA%\NovaCAD\viewport_logs.txt`
2. Verify OpenGL initialization
3. Confirm ViewModel-Viewport connection
4. Check mesh initialization state

### Kernel Issues
1. Check if using stubs: `NovaKernel.IsUsingStubs`
2. Verify handle validity
3. Check for memory leaks in Rust (use `valgrind`)

### Performance Issues
1. Profile with Visual Studio Performance Profiler
2. Check for unnecessary render calls
3. Verify VSync status
4. Monitor GPU memory usage

## File Organization

```
NovaCAD/src/
├── NovaCad.App/
│   ├── ViewModels/           # One per major view
│   ├── Views/
│   │   ├── MainWindow.axaml  # Root layout
│   │   └── ViewportControl   # OpenGL surface
│   └── App.axaml.cs          # Startup logic
├── NovaCad.Viewport/
│   ├── Viewport3D.cs         # Core rendering
│   ├── ViewportControl.cs    # Avalonia integration
│   ├── Mesh.cs               # OpenGL mesh
│   ├── MeshFactory.cs        # Primitive generators
│   ├── Camera3D.cs           # Navigation
│   └── ViewportDiagnostics.cs # Logging
└── NovaCad.Kernel/
    ├── NovaKernel.cs         # P/Invoke wrapper with stub fallback
    └── NovaKernelStub.cs     # Pure C# implementation
```

## Common Tasks

### Adding a New Primitive
1. Add `nova_make_<primitive>` to Rust kernel
2. Add binding in `NovaKernel.cs`
3. Add mesh generator in `MeshFactory.cs`
4. Add command in `MainWindowViewModel.cs`
5. Add button in `MainWindow.axaml`

### Adding a New Tool
1. Create tool class in `NovaCad.Viewport/Tools/`
2. Implement `ITool` interface
3. Add to `Viewport3D.ToolController`
4. Bind to button/command

### Debugging Render Issues
```csharp
// Enable verbose logging
ViewportDiagnostics.IsEnabled = true;
ViewportDiagnostics.MinimumLevel = LogLevel.Debug;

// Check specific mesh
var mesh = MeshFactory.CreateBox(10, 10, 10);
ViewportDiagnostics.LogMeshInfo(mesh, "DebugBox");
```

## Testing Checklist

Before committing:
- [ ] Application opens without native DLL errors
- [ ] Can create Box, Cylinder, Sphere
- [ ] Viewport shows grid and axes
- [ ] Camera navigation works (MMB, Shift+MMB, Scroll)
- [ ] View presets work (Front, Top, Isometric)
- [ ] Model tree updates correctly
- [ ] No memory leaks in OpenGL resources

## References

### Design Inspiration
- **Plasticity**: https://www.plasticity.xyz/
  - Direct modeling paradigm
  - Minimal UI design
  - Push-pull interactions

- **Shapr3D**: https://www.shapr3d.com/
  - Precision input methods
  - Clean visualization
  - Manufacturing focus

### Technical References
- **OpenGL**: https://www.khronos.org/opengl/
- **Avalonia**: https://avaloniaui.net/
- **Rust FFI**: https://doc.rust-lang.org/nomicon/ffi.html
- **B-Rep Modeling**: "The NURBS Book" by Piegl & Tiller

## Migration Path (Future)

If C# viewport becomes a bottleneck:
1. Move `ViewportControl` to C++/CLI
2. Use raw OpenGL 4.6 instead of ANGLE
3. Keep C# for UI chrome only
4. Interop via C API between C++ viewport and Rust kernel

---

Last updated: 2026-02-08
