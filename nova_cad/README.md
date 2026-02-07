# Nova Kernel 3D + Nova CAD

Um kernel 3D CAD open-source em Rust com aplicação CAD profissional em C#/AvaloniaUI, inspirado na tecnologia Synchronous do Solid Edge.

## Arquitetura do Projeto

### Nova Kernel 3D (Rust)

O kernel é organizado em camadas modulares:

| Camada | Crate | Descrição |
|--------|-------|-----------|
| L0 | `nova_math` | Fundamentos matemáticos: pontos, vetores, matrizes, tolerâncias |
| L1 | `nova_geom` | Curvas e superfícies: Line, Arc, NURBS, Plane, Cylinder, Sphere, etc. |
| L2 | `nova_topo` | Topologia B-Rep: Vertex, Edge, Coedge, Loop, Face, Shell, Body |
| L3 | `nova_ops` | Operações: Boolean, fillet, chamfer, sweep, loft |
| L4 | `nova_sync` | Edição direta: face move, live rules, reconhecimento geométrico |
| L5 | `nova_tess` | Tesselação: triangulação adaptativa para visualização |
| L6 | `nova_io` | I/O: STEP AP214/AP242, IGES, formato nativo .nova |
| L7 | `nova_check` | Validação: verificação topológica, healing |
| L8 | `nova_ffi` | Interface C-ABI para interoperabilidade |

### Nova CAD (C# / .NET 8 / AvaloniaUI)

A aplicação CAD é organizada em projetos:

| Projeto | Descrição |
|---------|-----------|
| `NovaCad.Core` | Modelos de domínio, interfaces, serviços |
| `NovaCad.Kernel` | Wrapper P/Invoke para o kernel Rust |
| `NovaCad.Viewport` | Renderização 3D com Silk.NET/OpenGL |
| `NovaCad.UI` | Interface do usuário com AvaloniaUI |
| `NovaCad.App` | Aplicação principal |

## Estrutura de Diretórios

```
nova_cad/
├── nova_kernel/           # Kernel Rust
│   ├── Cargo.toml         # Workspace definition
│   └── crates/
│       ├── nova_math/     # Matemática
│       ├── nova_geom/     # Geometria
│       ├── nova_topo/     # Topologia B-Rep
│       ├── nova_ffi/      # Interface C
│       └── ...
├── NovaCAD/               # Aplicação C#
│   ├── NovaCAD.sln        # Solution file
│   └── src/
│       ├── NovaCad.Core/  # Core models
│       ├── NovaCad.Kernel/# P/Invoke wrapper
│       ├── NovaCad.Viewport/ # 3D viewport
│       ├── NovaCad.UI/    # UI components
│       └── NovaCad.App/   # Main application
└── README.md
```

## Compilação

### Kernel Rust

```bash
cd nova_kernel
cargo build --release
```

### Aplicação C#

```bash
cd NovaCAD
dotnet build
dotnet run --project src/NovaCad.App
```

## Funcionalidades Implementadas

### Kernel (Rust)

- [x] Fundamentos matemáticos (Point3, Vec3, Mat4, Transform3, Quaternion)
- [x] Curvas analíticas (Line, CircularArc, EllipseArc)
- [x] Superfícies analíticas (Planar, Cylindrical, Spherical, Conical, Toroidal)
- [x] NURBS curves e surfaces
- [x] Topologia B-Rep completa (Vertex, Edge, Coedge, Loop, Face, Shell, Body)
- [x] Operadores Euler (MVFS, MEV, MEF, KEMR, KFMRH, MEKR)
- [x] Interface C-ABI (nova_ffi)

### CAD Application (C#)

- [x] Estrutura do projeto
- [x] P/Invoke wrapper para o kernel
- [x] Modelos de domínio (Document, Body, Selection)
- [x] Interface básica com AvaloniaUI
- [x] Menu, toolbar, painéis

## Roadmap

### Fase 1 - Fundação (Concluído)
- [x] Matemática e geometria analítica
- [x] Topologia B-Rep
- [x] Interface C-ABI

### Fase 2 - Operações ✅ (Concluído)
- [x] **Boolean operations**: face intersection, point classification, face splitting, result construction
- [x] **Features**: Extrude, Revolve, Sweep, Loft com opções completas
- [x] **Fillets/Chamfers**: constant/variable radius, propagation, análise de edges
- [x] **STEP I/O**: Parser completo AP214/AP242, conversão bidirecional B-Rep
- [x] **STL I/O**: Export ASCII e Binary
- [x] **Operadores Euler Avançados**: extrude_face, revolve_face, create_fillet_face, create_solid_from_faces

### Fase 3 - Edição Direta ✅ (Concluído)
- [x] Face move/rotate/offset com resolução topológica
- [x] Live rules (Parallel, Perpendicular, Concentric, Symmetric, etc.)
- [x] Geometric recognition (Hole, Pad, Pocket, Fillet, Chamfer, etc.)
- [x] Feature handles para manipulação direta
- [x] Steering Wheel (widget de manipulação 3D)
- [x] Topology resolution (Extend, Trim, Blend)

### Fase 4 - Aplicação CAD 🔄 (Em Progresso - 40%)
- [x] Viewport 3D com OpenGL/Silk.NET
- [ ] Integração Steering Wheel com UI
- [ ] Seleção e manipulação
- [ ] Mold tools

## Licenças

- **Nova Kernel (Rust)**: LGPL 2.1+
- **Nova CAD Application**: GPL 3.0
- **NovaSharp (C# Interop)**: MIT

## Referências

- [The NURBS Book](https://www.springer.com/gp/book/9783540615453) - Piegl & Tiller
- [Computational Geometry: Algorithms and Applications](https://link.springer.com/book/10.1007/978-3-540-77974-2) - de Berg et al.
- [Robust Geometric Computation](https://cs.nyu.edu/exact/) - Shewchuk
