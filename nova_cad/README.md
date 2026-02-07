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

### Fase 2 - Operações
- [ ] Boolean operations (unite, subtract, intersect)
- [ ] Features (extrude, revolve, sweep, loft)
- [ ] Fillets and chamfers
- [ ] STEP import/export

### Fase 3 - Edição Direta
- [ ] Face move/rotate/offset
- [ ] Live rules
- [ ] Geometric recognition

### Fase 4 - Aplicação CAD
- [ ] Viewport 3D com OpenGL
- [ ] Steering Wheel
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
