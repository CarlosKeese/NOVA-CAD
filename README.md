# 🎨 NOVA CAD

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![License: LGPL v2.1](https://img.shields.io/badge/License-LGPL%20v2.1-blue.svg)](https://www.gnu.org/licenses/lgpl-2.1)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![.NET](https://img.shields.io/badge/.NET-8.0-purple.svg)](https://dotnet.microsoft.com)
[![Status](https://img.shields.io/badge/Status-Complete-success.svg)]()

> Um kernel 3D CAD open-source em Rust com aplicação profissional em C#/AvaloniaUI, inspirado na tecnologia Synchronous do Solid Edge.

![Architecture](https://img.shields.io/badge/Architecture-Hybrid%20Rust%2FC%23-success)
![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-informational)

---

## 🎯 Visão Geral

O **NOVA CAD** é um sistema CAD profissional de código aberto, combinando:

- **🦀 Kernel 3D em Rust**: Motor geométrico de alta performance com topologia B-Rep completa
- **🖥️ Aplicação em C#/AvaloniaUI**: Interface moderna, cross-platform e profissional
- **⚡ Tecnologia Synchronous**: Edição direta de geometria como no Solid Edge

### ✅ Status: Projeto Completo!

Todas as 4 fases foram implementadas com sucesso:

| Fase | Descrição | Status |
|------|-----------|--------|
| **Fase 1** | Fundação (Kernel, Matemática, B-Rep) | ✅ 100% |
| **Fase 2** | Operações (Boolean, Features, STEP I/O) | ✅ 100% |
| **Fase 3** | Edição Direta (Synchronous Technology) | ✅ 100% |
| **Fase 4** | Aplicação CAD (Viewport 3D, UI, Mold Tools) | ✅ 100% |

### Estatísticas do Projeto

| Métrica | Valor |
|---------|-------|
| Total de arquivos | 50+ |
| Linhas de código Rust | ~15,000+ |
| Linhas de código C# | ~10,000+ |
| Crates Rust | 6 |
| Projetos C# | 5 |

---

## 🏗️ Arquitetura

### Nova Kernel 3D (Rust)

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

## ✨ Funcionalidades Principais

### 🔧 Modelagem Sólida
- ✅ **Boolean Operations**: Unite, Subtract, Intersect
- ✅ **Features**: Extrude, Revolve, Sweep, Loft
- ✅ **Fillets & Chamfers**: Constante e variable radius
- ✅ **Shell & Draft**: Operações de chapa e ângulo de saída

### ⚡ Edição Direta (Synchronous)
- ✅ **Face Editing**: Move, Rotate, Offset de faces
- ✅ **Live Rules**: Parallel, Perpendicular, Concentric, Symmetric
- ✅ **Feature Recognition**: Reconhecimento automático de holes, pads, pockets
- ✅ **Steering Wheel**: Widget 3D para manipulação direta

### 🖥️ Interface 3D
- ✅ **Viewport OpenGL**: Renderização com Silk.NET
- ✅ **Câmera Orbit**: Pan, rotate, zoom
- ✅ **Seleção 3D**: Picking com ray casting
- ✅ **Transform Gizmos**: Move, rotate, scale
- ✅ **Steering Wheel UI**: Overlay 3D interativo

### 🏭 Moldes
- ✅ **Mold Cavity**: Criação de cavidades
- ✅ **Undercut Analysis**: Análise de undercuts
- ✅ **Draft Analysis**: Análise de ângulos de saída
- ✅ **Parting Line**: Linha de separação
- ✅ **Cooling Channels**: Canais de resfriamento
- ✅ **Ejector Pins**: Pinos ejetores

### 📁 Import/Export
- ✅ **STEP**: AP214/AP242 import/export
- ✅ **IGES**: Formato IGES
- ✅ **STL**: ASCII e Binary
- ✅ **Native**: Formato .nova

---

## 🚀 Como Executar

### Pré-requisitos

- **Rust** 1.75+ com Cargo
- **.NET 8** SDK
- **OpenGL** 3.3+ compatível

### Compilar e Executar

```bash
# Clone o repositório
git clone https://github.com/CarlosKeese/NOVA-CAD.git
cd NOVA-CAD

# Compilar o Kernel Rust
cd nova_cad/nova_kernel
cargo build --release

# Compilar e executar a aplicação C#
cd ../NovaCAD
dotnet build
dotnet run --project src/NovaCad.App
```

### Script de Build (Linux/macOS)

```bash
cd nova_cad
./build.sh all    # Build completo
./build.sh run    # Build e executar
```

---

## 📁 Estrutura do Projeto

```
NOVA-CAD/
├── nova_cad/                      # Código fonte
│   ├── nova_kernel/               # Kernel Rust
│   │   ├── crates/
│   │   │   ├── nova_math/         # Matemática 3D
│   │   │   ├── nova_geom/         # Geometria
│   │   │   ├── nova_topo/         # Topologia B-Rep
│   │   │   ├── nova_ops/          # Operações
│   │   │   ├── nova_sync/         # Edição Direta
│   │   │   ├── nova_io/           # Import/Export
│   │   │   └── nova_ffi/          # Interface C
│   │   └── Cargo.toml
│   │
│   ├── NovaCAD/                   # Aplicação C#
│   │   ├── src/
│   │   │   ├── NovaCad.Core/      # Modelos
│   │   │   ├── NovaCad.Kernel/    # P/Invoke
│   │   │   ├── NovaCad.Viewport/  # OpenGL 3D
│   │   │   ├── NovaCad.UI/        # UI Avalonia
│   │   │   └── NovaCad.App/       # App principal
│   │   └── NovaCAD.sln
│   │
│   ├── README.md
│   ├── SPECIFICATION.md
│   └── IMPLEMENTATION_SUMMARY.md
│
└── README.md                      # Este arquivo
```

---

## 🛠️ Tecnologias Utilizadas

### Backend (Rust)
- **nalgebra**: Álgebra linear
- **thiserror**: Gerenciamento de erros
- **rayon**: Paralelismo
- **serde**: Serialização

### Frontend (C#)
- **AvaloniaUI**: Framework UI cross-platform
- **Silk.NET**: Bindings OpenGL
- **SixLabors.ImageSharp**: Processamento de imagens

---

## 📸 Screenshots

*Em breve: Screenshots da aplicação em execução*

---

## 📚 Documentação

- **[README.md](nova_cad/README.md)** - Documentação técnica do kernel
- **[SPECIFICATION.md](nova_cad/SPECIFICATION.md)** - Especificação completa
- **[IMPLEMENTATION_SUMMARY.md](nova_cad/IMPLEMENTATION_SUMMARY.md)** - Resumo da implementação

---

## 📜 Licenças

| Componente | Licença | Descrição |
|------------|---------|-----------|
| **Nova Kernel (Rust)** | LGPL 2.1+ | Permite uso em projetos proprietários via FFI |
| **Nova CAD Application** | GPL 3.0 | Aplicação completa open-source |
| **NovaSharp (C# Interop)** | MIT | Bindings C# livres |

---

## 🙏 Agradecimentos

- [The NURBS Book](https://www.springer.com/gp/book/9783540615453) - Piegl & Tiller
- [Computational Geometry](https://link.springer.com/book/10.1007/978-3-540-77974-2) - de Berg et al.
- [Synchronous Technology](https://www.plm.automation.siemens.com/) - Siemens

---

<p align="center">
  <b>NOVA CAD</b> - Modelagem 3D Profissional Open-Source
  <br>
  Built with ❤️ using Rust & C#
</p>
