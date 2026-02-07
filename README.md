# 🎨 NOVA CAD

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![License: LGPL v2.1](https://img.shields.io/badge/License-LGPL%20v2.1-blue.svg)](https://www.gnu.org/licenses/lgpl-2.1)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![.NET](https://img.shields.io/badge/.NET-8.0-purple.svg)](https://dotnet.microsoft.com)

> Um kernel 3D CAD open-source em Rust com aplicação profissional em C#/AvaloniaUI, inspirado na tecnologia Synchronous do Solid Edge.

![Architecture](https://img.shields.io/badge/Architecture-Hybrid%20Rust%2FC%23-success)
![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-informational)

---

## 📋 Índice

- [Visão Geral](#-visão-geral)
- [Arquitetura](#-arquitetura)
- [Componentes](#-componentes)
- [Funcionalidades](#-funcionalidades)
- [Roadmap](#-roadmap)
- [Compilação](#-compilação)
- [Licenças](#-licenças)

---

## 🎯 Visão Geral

O **NOVA CAD** é um projeto ambicioso que visa criar um sistema CAD profissional de código aberto, combinando:

- **🦀 Kernel 3D em Rust**: Motor geométrico de alta performance com topologia B-Rep completa
- **🖥️ Aplicação em C#/AvaloniaUI**: Interface moderna, cross-platform e profissional
- **⚡ Tecnologia Synchronous**: Edição direta de geometria como no Solid Edge

### Estatísticas do Projeto

| Métrica | Valor |
|---------|-------|
| Total de arquivos | 66+ |
| Linhas de código Rust | ~6,000+ |
| Linhas de código C# | ~2,000+ |
| Crates Rust | 4+ |
| Projetos C# | 5 |

---

## 🏗️ Arquitetura

### Nova Kernel 3D (Rust)

Kernel modular organizado em camadas:

```
┌─────────────────────────────────────────────────────────────┐
│  L8 - nova_ffi      │  Interface C-ABI para interoperabilidade│
├─────────────────────┼─────────────────────────────────────────┤
│  L7 - nova_check    │  Validação e healing geométrico         │
├─────────────────────┼─────────────────────────────────────────┤
│  L6 - nova_io       │  Import/Export (STEP, IGES, .nova)      │
├─────────────────────┼─────────────────────────────────────────┤
│  L5 - nova_tess     │  Tesselação para visualização           │
├─────────────────────┼─────────────────────────────────────────┤
│  L4 - nova_sync     │  Edição direta (Synchronous Tech)       │
├─────────────────────┼─────────────────────────────────────────┤
│  L3 - nova_ops      │  Operações: Boolean, fillet, sweep      │
├─────────────────────┼─────────────────────────────────────────┤
│  L2 - nova_topo     │  Topologia B-Rep completa               │
├─────────────────────┼─────────────────────────────────────────┤
│  L1 - nova_geom     │  Curvas e superfícies NURBS             │
├─────────────────────┼─────────────────────────────────────────┤
│  L0 - nova_math     │  Fundamentos matemáticos                │
└─────────────────────────────────────────────────────────────┘
```

### Nova CAD Application (C# / .NET 8 / AvaloniaUI)

```
┌─────────────────────────────────────────────────────────────┐
│                    NovaCad.App                              │
│              (Aplicação Principal)                          │
├─────────────────────────────────────────────────────────────┤
│  NovaCad.UI  │  NovaCad.Viewport  │  NovaCad.Kernel        │
│  (Interface) │  (Renderização 3D) │  (P/Invoke Wrapper)    │
├─────────────────────────────────────────────────────────────┤
│                    NovaCad.Core                             │
│              (Modelos de Domínio)                           │
├─────────────────────────────────────────────────────────────┤
│                    Nova Kernel (Rust)                       │
│              (via C-ABI / FFI)                              │
└─────────────────────────────────────────────────────────────┘
```

---

## 🧩 Componentes

### 1️⃣ Nova Math (Fundamentos Matemáticos)

Base numérica robusta para computação geométrica:

| Componente | Descrição |
|------------|-----------|
| `Point2/3/4` | Pontos em 2D, 3D e coordenadas homogêneas |
| `Vec2/3/4` | Vetores com operações completas |
| `Mat3/4` | Matrizes de transformação |
| `Transform3` | Transformações rígidas (rotação + translação) |
| `Quaternion` | Quaternions para rotações com slerp |
| `BoundingBox2/3` | Bounding boxes |
| `Interval` | Aritmética de intervalos |
| `ToleranceContext` | Tolerância hierárquica (SPAresabs equivalente) |
| `Plane` | Planos 3D com projeções |
| `predicates` | Predicados geométricos robustos (orient2d, orient3d, incircle, insphere) |

### 2️⃣ Nova Geom (Geometria)

Curvas e superfícies analíticas e NURBS:

**Curvas:**
- ✅ `Line` - Linhas infinitas e segmentos
- ✅ `CircularArc` - Arcos circulares e círculos
- ✅ `EllipseArc` - Arcos elípticos
- ✅ `NurbsCurve` - Curvas NURBS com algoritmo de Boor

**Superfícies:**
- ✅ `PlanarSurface` - Superfícies planas
- ✅ `CylindricalSurface` - Cilindros
- ✅ `SphericalSurface` - Esferas
- ✅ `ConicalSurface` - Cones
- ✅ `ToroidalSurface` - Toros
- ✅ `NurbsSurface` - Superfícies NURBS

**Algoritmos:**
- ✅ Interseções curva-curva
- ✅ Interseções curva-superfície
- ✅ Interseções superfície-superfície

### 3️⃣ Nova Topo (Topologia B-Rep)

Estrutura de dados B-Rep completa:

```
Body (Sólido)
 └── Shell (Concha)
      └── Face (Face)
           ├── Surface (Superfície geométrica)
           └── Loop (Loop)
                └── Coedge (Coedge)
                     ├── Edge (Aresta)
                     │    ├── Curve (Curva geométrica)
                     │    ├── Vertex (Vértice inicial)
                     │    └── Vertex (Vértice final)
                     └── Orientation (Orientação)
```

**Operadores Euler Implementados:**
- ✅ `MVFS` - Make Vertex Face Shell
- ✅ `MEV` - Make Edge Vertex
- ✅ `MEF` - Make Edge Face
- ✅ `KEMR` - Kill Edge Make Ring
- ✅ `KFMRH` - Kill Face Make Ring Hole
- ✅ `MEKR` - Make Edge Kill Ring

### 4️⃣ Nova FFI (Interface C)

Interface C-ABI completa para interoperabilidade:

**Tipos:**
- `NovaHandle`, `NovaPoint3`, `NovaVec3`, `NovaMat4`
- `NovaTransform`, `NovaBBox3`, `NovaMesh`

**Funções:**
- Inicialização: `nova_init`, `nova_shutdown`, `nova_version`
- Primitivas: `nova_make_box`, `nova_make_cylinder`, `nova_make_sphere`
- Operações: `nova_boolean_unite`, `nova_boolean_subtract`, `nova_fillet`
- I/O: `nova_import_step`, `nova_export_step`, `nova_export_stl`
- Tesselação: `nova_tessellate_body`

### 5️⃣ Nova CAD Application (C#)

Aplicação desktop profissional:

| Projeto | Responsabilidade |
|---------|------------------|
| `NovaCad.Core` | Modelos de domínio, documento, seleção, undo/redo |
| `NovaCad.Kernel` | Wrapper P/Invoke para o kernel Rust |
| `NovaCad.Viewport` | Renderização 3D com Silk.NET/OpenGL |
| `NovaCad.UI` | Componentes de UI com AvaloniaUI |
| `NovaCad.App` | Aplicação principal, ViewModels, comandos |

---

## ✅ Funcionalidades

### Fase 1 - Fundação ✅ (100% Concluído)

- [x] Fundamentos matemáticos completos (points, vectors, matrices, transforms)
- [x] Curvas analíticas (Line, CircularArc, EllipseArc, NURBS)
- [x] Superfícies analíticas (Planar, Cylindrical, Spherical, Conical, Toroidal, NURBS)
- [x] Topologia B-Rep completa (Vertex, Edge, Face, Shell, Body)
- [x] Operadores Euler para manipulação topológica
- [x] Interface C-ABI para interoperabilidade
- [x] Estrutura da aplicação C# com AvaloniaUI
- [x] P/Invoke wrapper completo
- [x] Interface básica (menu, toolbar, painéis)

### Fase 2 - Operações 🔄 (Estrutura Pronta)

- [x] Estrutura para Boolean operations (unite, subtract, intersect)
- [x] Estrutura para Features (extrude, revolve, sweep, loft)
- [x] Estrutura para Fillets e Chamfers
- [x] Estrutura para STEP import/export
- [ ] Implementação completa das operações Boolean
- [ ] Implementação completa das features
- [ ] Parser STEP completo

### Fase 3 - Edição Direta 🔄 (Planejada)

- [ ] Face move/rotate/offset
- [ ] Live rules (regras de edição síncrona)
- [ ] Geometric recognition (reconhecimento de features)
- [ ] Steering Wheel (widget de manipulação)

### Fase 4 - Aplicação Completa 🔄 (Em Desenvolvimento)

- [x] Interface básica com AvaloniaUI
- [x] Menu, toolbar, painéis
- [x] Comandos para criar primitivas
- [ ] Viewport 3D com OpenGL/Silk.NET
- [ ] Renderização de malhas trianguladas
- [ ] Seleção e manipulação 3D
- [ ] Mold tools (ferramentas de moldagem)

---

## 🗺️ Roadmap

```
2024 Q1-Q2                    2024 Q3-Q4                    2025
   │                              │                           │
   ▼                              ▼                           ▼
┌──────────────┐           ┌──────────────┐           ┌──────────────┐
│   FASE 1     │           │   FASE 2     │           │   FASE 3     │
│ Fundação     │──────────▶│ Operações    │──────────▶│ Synchronous  │
│ ✅ Concluído │           │ 🔄 Em breve  │           │ 📋 Futuro    │
└──────────────┘           └──────────────┘           └──────────────┘
       │                          │                          │
       ▼                          ▼                          ▼
┌──────────────┐           ┌──────────────┐           ┌──────────────┐
│ • Matemática │           │ • Boolean    │           │ • Face move  │
│ • Geometria  │           │ • Features   │           │ • Live rules │
│ • Topologia  │           │ • STEP I/O   │           │ • Steering   │
│ • FFI        │           │ • Fillet     │           │   Wheel      │
│ • UI Básica  │           │ • Chamfer    │           │              │
└──────────────┘           └──────────────┘           └──────────────┘
```

---

## 🚀 Compilação

### Requisitos

- **Rust** 1.75+ (com Cargo)
- **.NET 8** SDK
- **AvaloniaUI** templates (opcional)

### Compilar Tudo

```bash
# Clone o repositório
git clone https://github.com/CarlosKeese/NOVA-CAD.git
cd NOVA-CAD

# Build completo (kernel + aplicação)
cd nova_cad
./build.sh all

# Ou build e execute
./build.sh run
```

### Compilar Separadamente

**Kernel Rust:**
```bash
cd nova_cad/nova_kernel
cargo build --release
```

**Aplicação C#:**
```bash
cd nova_cad/NovaCAD
dotnet build
dotnet run --project src/NovaCad.App
```

---

## 📁 Estrutura de Diretórios

```
NOVA-CAD/
├── nova_cad/                      # Código fonte principal
│   ├── nova_kernel/               # Kernel Rust
│   │   ├── Cargo.toml             # Workspace definition
│   │   └── crates/
│   │       ├── nova_math/         # Matemática
│   │       ├── nova_geom/         # Geometria
│   │       ├── nova_topo/         # Topologia B-Rep
│   │       └── nova_ffi/          # Interface C
│   ├── NovaCAD/                   # Aplicação C#
│   │   ├── NovaCAD.sln            # Solution file
│   │   └── src/
│   │       ├── NovaCad.Core/      # Core models
│   │       ├── NovaCad.Kernel/    # P/Invoke wrapper
│   │       ├── NovaCad.Viewport/  # 3D viewport
│   │       ├── NovaCad.UI/        # UI components
│   │       └── NovaCad.App/       # Main application
│   ├── README.md                  # Documentação interna
│   ├── SPECIFICATION.md           # Especificação técnica
│   ├── IMPLEMENTATION_SUMMARY.md  # Resumo de implementação
│   └── build.sh                   # Script de build
└── README.md                      # Este arquivo
```

---

## 📚 Documentação

- **[README.md](nova_cad/README.md)** - Visão geral do projeto
- **[SPECIFICATION.md](nova_cad/SPECIFICATION.md)** - Especificação técnica detalhada
- **[IMPLEMENTATION_SUMMARY.md](nova_cad/IMPLEMENTATION_SUMMARY.md)** - Resumo da implementação

---

## 📜 Licenças

Este projeto utiliza uma estratégia de licenciamento em camadas:

| Componente | Licença | Descrição |
|------------|---------|-----------|
| **Nova Kernel (Rust)** | LGPL 2.1+ | Permite uso em projetos proprietários via FFI |
| **Nova CAD Application** | GPL 3.0 | Aplicação completa open-source |
| **NovaSharp (C# Interop)** | MIT | Bindings C# livres para qualquer uso |

---

## 📖 Referências

- [The NURBS Book](https://www.springer.com/gp/book/9783540615453) - Piegl & Tiller (1997)
- [Computational Geometry: Algorithms and Applications](https://link.springer.com/book/10.1007/978-3-540-77974-2) - de Berg et al.
- [Robust Geometric Computation](https://cs.nyu.edu/exact/) - Shewchuk (1997)
- [Synchronous Technology](https://www.plm.automation.siemens.com/global/en/products/nx/synchronous-technology.html) - Siemens

---

## 🤝 Contribuição

Contribuições são bem-vindas! Este é um projeto em desenvolvimento ativo.

---

<p align="center">
  <b>NOVA CAD</b> - Modelagem 3D Profissional Open-Source
  <br>
  Built with ❤️ using Rust & C#
</p>
