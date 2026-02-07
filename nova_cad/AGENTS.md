# Nova Kernel 3D + Nova CAD - Agent Guide

## Visão Geral do Projeto

Este projeto implementa um kernel 3D CAD open-source em Rust com uma aplicação CAD profissional em C#/AvaloniaUI. A arquitetura é dividida em duas partes principais:

1. **Nova Kernel 3D** (Rust) - Kernel computacional de geometria 3D
2. **Nova CAD** (C#/AvaloniaUI) - Aplicação CAD profissional

A inspiração vem da tecnologia Synchronous do Solid Edge, com ênfase em edição direta de modelos.

## Arquitetura

### Kernel Rust (nova_kernel/)

O kernel é organizado em camadas modulares (crates):

| Camada | Crate | Descrição | Status |
|--------|-------|-----------|--------|
| L0 | `nova_math` | Fundamentos matemáticos: pontos, vetores, matrizes, tolerâncias | ✅ Implementado |
| L1 | `nova_geom` | Curvas e superfícies: Line, Arc, NURBS, Plane, Cylinder, Sphere, etc. | ✅ Implementado |
| L2 | `nova_topo` | Topologia B-Rep: Vertex, Edge, Coedge, Loop, Face, Shell, Body | ✅ Implementado |
| L3 | `nova_ops` | Operações: Boolean, fillet, chamfer, sweep, loft | 🔄 Estrutura pronta |
| L4 | `nova_sync` | Edição direta: face move, live rules, reconhecimento geométrico | 🔄 Estrutura pronta |
| L5 | `nova_tess` | Tesselação: triangulação adaptativa para visualização | 🔄 Estrutura pronta |
| L6 | `nova_io` | I/O: STEP AP214/AP242, IGES, formato nativo .nova | 🔄 Estrutura pronta |
| L7 | `nova_check` | Validação: verificação topológica, healing | 🔄 Estrutura pronta |
| L8 | `nova_ffi` | Interface C-ABI para interoperabilidade | ✅ Implementado |

### Aplicação C# (NovaCAD/)

A aplicação é organizada em projetos:

| Projeto | Descrição | Tecnologias |
|---------|-----------|-------------|
| `NovaCad.Core` | Modelos de domínio, interfaces, serviços | .NET 8, CommunityToolkit.Mvvm |
| `NovaCad.Kernel` | Wrapper P/Invoke para o kernel Rust | System.Runtime.InteropServices |
| `NovaCad.Viewport` | Renderização 3D com Silk.NET/OpenGL | Silk.NET 2.21.0 |
| `NovaCad.UI` | Interface do usuário com AvaloniaUI | Avalonia 11.0.7, Dock.Avalonia |
| `NovaCad.App` | Aplicação principal | Avalonia, Serilog |

## Estrutura de Diretórios

```
nova_cad/
├── nova_kernel/              # Kernel Rust
│   ├── Cargo.toml           # Workspace definition
│   └── crates/
│       ├── nova_math/       # Matemática (~1.500 linhas)
│       ├── nova_geom/       # Geometria (~2.000 linhas)
│       ├── nova_topo/       # Topologia B-Rep (~1.500 linhas)
│       ├── nova_ffi/        # Interface C (~800 linhas)
│       └── ...
├── NovaCAD/                 # Aplicação C#
│   ├── NovaCAD.sln          # Solution file
│   └── src/
│       ├── NovaCad.Core/    # Core models
│       ├── NovaCad.Kernel/  # P/Invoke wrapper
│       ├── NovaCad.Viewport/# 3D viewport
│       ├── NovaCad.UI/      # UI components
│       └── NovaCad.App/     # Main application
├── build.sh                 # Build script
├── README.md               # Visão geral
├── SPECIFICATION.md        # Especificação técnica
└── IMPLEMENTATION_SUMMARY.md # Resumo da implementação
```

## Tecnologias e Dependências

### Rust (Kernel)
- **Versão mínima**: 1.75
- **Edition**: 2021
- **Dependências principais**:
  - `nalgebra` 0.33 - Álgebra linear
  - `num-traits` 0.2 - Traits numéricos
  - `thiserror` 2.0 - Erros
  - `serde` 1.0 - Serialização
  - `libc` 0.2 - FFI
  - `once_cell` 1.19 - Inicialização lazy

### C# (Aplicação)
- **Framework**: .NET 8
- **Language Version**: C# 12.0
- **Dependências principais**:
  - `Avalonia` 11.0.7 - UI Framework
  - `CommunityToolkit.Mvvm` 8.2.2 - MVVM Toolkit
  - `Silk.NET` 2.21.0 - OpenGL binding
  - `Serilog` 3.1.1 - Logging
  - `Microsoft.Extensions.DependencyInjection` 8.0.0 - DI

## Comandos de Build

### Script de Build (build.sh)

```bash
# Build completo (kernel + app)
./build.sh all

# Apenas kernel Rust
./build.sh kernel

# Apenas aplicação C#
./build.sh app

# Executar testes do kernel
./build.sh test

# Build e executar
./build.sh run

# Limpar artefatos
./build.sh clean
```

### Build Manual - Kernel Rust

```bash
cd nova_kernel
cargo build --release

# Executar testes
cargo test

# Build com otimizações máximas
# (Configurado em Cargo.toml: opt-level = 3, lto = "thin")
```

**Saída esperada:**
- Linux: `target/release/libnova_ffi.so`
- Windows: `target/release/nova_ffi.dll`
- macOS: `target/release/libnova_ffi.dylib`

### Build Manual - Aplicação C#

```bash
cd NovaCAD

# Restaurar pacotes
dotnet restore

# Build
dotnet build

# Executar
dotnet run --project src/NovaCad.App

# Build de release
dotnet build -c Release
```

## Convenções de Código

### Rust

**Estilo:**
- Siga o rustfmt padrão
- Documentação obrigatória (`#![warn(missing_docs)]`)
- Traits comuns: `Transformable`, `Bounded`, `Evaluable`
- Erros com `thiserror::Error`
- Testes inline em `#[cfg(test)]`

**Padrões de nomenclatura:**
- Tipos: PascalCase (ex: `NurbsCurve`, `BoundingBox3`)
- Funções/variáveis: snake_case (ex: `bounding_box()`, `new_entity_id()`)
- Constantes: SCREAMING_SNAKE_CASE (ex: `DEFAULT_RESABS`)
- Traits: PascalCase com suffixo descritivo quando apropriado

**Exemplo de estrutura de módulo:**
```rust
//! Docstring do módulo

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod submodulo;
pub use submodulo::{Tipo, funcao};

/// Documentação pública
pub struct MinhaStruct {
    campo: Tipo,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_something() {}
}
```

### C#

**Estilo:**
- Convenções Microsoft C#
- MVVM com CommunityToolkit.Mvvm (source generators)
- `partial class` para ViewModels com `[ObservableProperty]`
- Comandos com `[RelayCommand]`
- Records para dados imutáveis

**Padrões de nomenclatura:**
- Classes/structs: PascalCase
- Métodos/propriedades: PascalCase
- Campos privados: _camelCase (com `[ObservableProperty]` gera automaticamente)
- Constantes: PascalCase
- Enums: PascalCase, valores PascalCase

**Exemplo de ViewModel:**
```csharp
public partial class MyViewModel : ObservableObject
{
    [ObservableProperty]
    private string _nome = string.Empty;

    [RelayCommand]
    private void ExecuteAction()
    {
        // Implementação
    }
}
```

## Interop (Rust ↔ C#)

A comunicação entre o kernel Rust e a aplicação C# é feita via C-ABI:

### Convenções FFI

**Rust (nova_ffi):**
```rust
#[no_mangle]
pub extern "C" fn nova_funcao(
    param: NovaReal,
    out_handle: *mut NovaHandle,
) -> NovaResult {
    // Validação de ponteiros nulos
    if out_handle.is_null() {
        return NovaResult::InvalidParameter;
    }
    // ...
}
```

**C# (NovaCad.Kernel):**
```csharp
public static partial class NovaKernel
{
    private const string LibraryName = "nova_ffi";

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    public static extern NovaResult nova_funcao(
        double param,
        out NovaHandle outHandle);
}
```

**Tipos interop correspondentes:**
| Rust | C# | Descrição |
|------|-----|-----------|
| `NovaHandle` (u64) | `NovaHandle` (ulong) | Handle para objetos |
| `NovaPoint3` | `NovaPoint3` | Ponto 3D (x, y, z) |
| `NovaVec3` | `NovaVec3` | Vetor 3D (x, y, z) |
| `NovaMat4` | `NovaMat4` | Matriz 4x4 (row-major) |
| `NovaTransform` | `NovaTransform` | Translação + Quaternion |
| `NovaResult` | `NovaResult` | Códigos de erro |

## Estratégia de Testes

### Rust
- Testes unitários em cada crate (`#[cfg(test)]`)
- Proptest para testes de propriedade (`proptest`)
- Criterion para benchmarks (`criterion`)

```bash
cd nova_kernel
cargo test          # Todos os testes
cargo test --lib    # Apenas testes da lib
cargo bench         # Benchmarks
```

### C#
- Testes unitários (xUnit/MSTest - não configurado ainda)
- Testes de integração para chamadas P/Invoke

## Configurações Importantes

### Cargo.toml (Workspace)
```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
panic = "abort"
```

### .csproj (Propriedades comuns)
```xml
<TargetFramework>net8.0</TargetFramework>
<ImplicitUsings>enable</ImplicitUsings>
<Nullable>enable</Nullable>
<LangVersion>12.0</LangVersion>
```

## Roadmap e Status

### Fase 1 - Fundação ✅ (100%)
- [x] Matemática completa (points, vectors, matrices, transforms)
- [x] Geometria analítica (curves, surfaces)
- [x] Topologia B-Rep (vertex, edge, face, body)
- [x] Operadores Euler
- [x] Interface C-ABI
- [x] Estrutura da aplicação C#

### Fase 2 - Operações 🔄 (Estrutura pronta)
- [ ] Boolean operations (unite, subtract, intersect)
- [ ] Features (extrude, revolve, sweep, loft)
- [ ] Fillets and chamfers
- [ ] STEP import/export

### Fase 3 - Edição Direta 🔄 (Estrutura pronta)
- [ ] Face move/rotate/offset
- [ ] Live rules
- [ ] Geometric recognition

### Fase 4 - Aplicação Completa 🔄 (UI básica pronta)
- [ ] Viewport 3D com OpenGL
- [ ] Steering Wheel
- [ ] Seleção avançada
- [ ] Mold tools

## Considerações de Segurança

1. **FFI Safety**: Sempre validar ponteiros nulos no lado Rust
2. **Handles**: Usar `NovaHandle` (u64) nunca expor ponteiros diretos
3. **Erros**: Retornar códigos de erro, não panics através da FFI
4. **Memory**: O kernel gerencia sua própria memória; liberar com `nova_body_release`
5. **Thread Safety**: O kernel usa `Mutex` para estado global; não thread-safe por padrão

## Licenças

- **Nova Kernel (Rust)**: LGPL 2.1+
- **Nova CAD Application**: GPL 3.0
- **NovaSharp (C# Interop)**: MIT

## Referências

- [The NURBS Book](https://www.springer.com/gp/book/9783540615453) - Piegl & Tiller
- [Computational Geometry](https://link.springer.com/book/10.1007/978-3-540-77974-2) - de Berg et al.
- [Robust Geometric Computation](https://cs.nyu.edu/exact/) - Shewchuk
- [AvaloniaUI Documentation](https://docs.avaloniaui.net/)
- [Silk.NET Documentation](https://dotnet.github.io/Silk.NET/)

## Contato e Contribuição

O projeto está em desenvolvimento ativo. Para contribuir:
1. Mantenha compatibilidade com a arquitetura de camadas
2. Adicione testes para novas funcionalidades
3. Documente APIs públicas
4. Siga as convenções de código existentes
