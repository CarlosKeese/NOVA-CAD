# Análise de Consistência e Sugestões de Melhorias - NOVA-CAD

## 🔍 Verificação de Consistência do Projeto

### Estrutura Esperada do Repositório

Verifique se a seguinte estrutura está presente no GitHub:

```
NOVA-CAD/
├── nova_kernel/              # Kernel Rust
│   ├── Cargo.toml            # Workspace root
│   └── crates/
│       ├── nova_math/        # Fundamentos matemáticos (L0-L1)
│       ├── nova_geom/        # Geometria analítica e NURBS (L2)
│       ├── nova_topo/        # Topologia B-Rep (L3)
│       ├── nova_ops/         # Operações Boolean e features (L4) ⚠️
│       ├── nova_sync/        # Edição direta/Synchronous (L5) ⚠️
│       ├── nova_tess/        # Tesselação para visualização (L6) ⚠️
│       ├── nova_io/          # Import/export STEP/IGES/STL (L7) ⚠️
│       ├── nova_check/       # Validação e healing (L8) ⚠️
│       ├── nova_ffi/         # Interface C-ABI
│       ├── nova_kernel/      # Crate principal (facade)
│       ├── nova_bench/       # Benchmarks
│       └── nova_test/        # Testes de integração
├── NovaCAD/                  # Aplicação C# AvaloniaUI
│   ├── NovaCAD.sln
│   └── src/
│       ├── NovaCad.Core/     # Modelos de domínio e documento
│       ├── NovaCad.Kernel/   # P/Invoke wrapper para nova_ffi
│       ├── NovaCad.Viewport/ # Renderização 3D OpenGL
│       ├── NovaCad.UI/       # Componentes UI reutilizáveis
│       └── NovaCad.App/      # Aplicação principal
├── README.md
├── SPECIFICATION.md          # Especificação original
├── IMPLEMENTATION_SUMMARY.md # Resumo da implementação
├── build.sh / build.ps1      # Scripts de build
├── .gitignore
└── .github/                  # CI/CD workflows
    └── workflows/
        ├── ci.yml
        └── release.yml
```

---

## ⚠️ Problemas Críticos para Verificação

### 1. Código Placeholder (Stubs não implementados)

**Arquivo**: `nova_kernel/crates/nova_ffi/src/lib.rs`

Verifique se estas funções ainda retornam `NovaResult::NotImplemented`:

```rust
// === Boolean Operations (nova_ops) ===
#[no_mangle]
pub extern "C" fn nova_boolean_unite(...) -> NovaResult  // ⚠️ Stub?
#[no_mangle]
pub extern "C" fn nova_boolean_subtract(...) -> NovaResult  // ⚠️ Stub?
#[no_mangle]
pub extern "C" fn nova_boolean_intersect(...) -> NovaResult  // ⚠️ Stub?

// === Feature Operations (nova_ops) ===
#[no_mangle]
pub extern "C" fn nova_fillet(...) -> NovaResult  // ⚠️ Stub?
#[no_mangle]
pub extern "C" fn nova_chamfer(...) -> NovaResult  // ⚠️ Stub?
#[no_mangle]
pub extern "C" fn nova_shell(...) -> NovaResult  // ⚠️ Stub?
#[no_mangle]
pub extern "C" fn nova_draft(...) -> NovaResult  // ⚠️ Stub?
#[no_mangle]
pub extern "C" fn nova_hole(...) -> NovaResult  // ⚠️ Stub?

// === Tessellation (nova_tess) ===
#[no_mangle]
pub extern "C" fn nova_tessellate_body(...) -> NovaResult  // ⚠️ Stub?
#[no_mangle]
pub extern "C" fn nova_mesh_free(...)  // ⚠️ Stub?

// === File I/O (nova_io) ===
#[no_mangle]
pub extern "C" fn nova_import_step(...) -> NovaResult  // ⚠️ Stub?
#[no_mangle]
pub extern "C" fn nova_export_step(...) -> NovaResult  // ⚠️ Stub?
#[no_mangle]
pub extern "C" fn nova_import_iges(...) -> NovaResult  // ⚠️ Stub?
#[no_mangle]
pub extern "C" fn nova_export_iges(...) -> NovaResult  // ⚠️ Stub?
#[no_mangle]
pub extern "C" fn nova_export_stl(...) -> NovaResult  // ⚠️ Stub?

// === Synchronous Technology (nova_sync) ===
#[no_mangle]
pub extern "C" fn nova_sync_begin_edit(...) -> NovaResult  // ⚠️ Stub?
#[no_mangle]
pub extern "C" fn nova_sync_apply_dimension(...) -> NovaResult  // ⚠️ Stub?
#[no_mangle]
pub extern "C" fn nova_sync_solve(...) -> NovaResult  // ⚠️ Stub?
#[no_mangle]
pub extern "C" fn nova_sync_end_edit(...) -> NovaResult  // ⚠️ Stub?

// === Validation (nova_check) ===
#[no_mangle]
pub extern "C" fn nova_validate_body(...) -> NovaResult  // ⚠️ Stub?
#[no_mangle]
pub extern "C" fn nova_heal_body(...) -> NovaResult  // ⚠️ Stub?
```

**Ação Requerida**: 
- Se forem stubs: Criar crates `nova_ops`, `nova_tess`, `nova_io`, `nova_sync`, `nova_check`
- Implementar versões mínimas ou remover do workspace temporariamente

---

### 2. Crates Potencialmente Vazios ou Incompletos

Verifique o conteúdo destes crates:

| Crate | Status Esperado | Verificação |
|-------|----------------|-------------|
| `nova_ops` | ⚠️ Vazio ou stub | Deve conter Boolean, features |
| `nova_sync` | ⚠️ Vazio ou stub | Deve conter edição direta |
| `nova_tess` | ⚠️ Vazio ou stub | Deve conter triangulação |
| `nova_io` | ⚠️ Vazio ou stub | Deve conter STEP/IGES/STL |
| `nova_check` | ⚠️ Vazio ou stub | Deve conter validação |
| `nova_bench` | ⚠️ Vazio | Deve conter benchmarks Criterion |
| `nova_test` | ⚠️ Vazio | Deve conter testes de integração |

**Comando para verificar**:
```bash
cd nova_kernel
cargo tree --duplicates  # Ver dependências
cargo build --all  # Ver se todos compilam
```

---

### 3. Cobertura de Testes

**Verifique testes em cada crate**:

```bash
# Executar todos os testes
cargo test --workspace --lib

# Verificar cobertura (instalar cargo-tarpaulin se necessário)
cargo tarpaulin --workspace --out Html
```

**Mínimo esperado**:
- [ ] `nova_math`: 80%+ cobertura (pontos, vetores, matrizes, quaternions)
- [ ] `nova_geom`: 70%+ cobertura (curvas, superfícies, NURBS)
- [ ] `nova_topo`: 70%+ cobertura (B-Rep, operadores Euler)
- [ ] `nova_ffi`: Testes de integração C# ↔ Rust

**Testes ausentes comuns**:
- Predicados geométricos robustos (orient2d, orient3d, incircle)
- Interseções curva-curva e curva-superfície
- Operadores Euler em sequências complexas
- Serialização/deserialização de documentos

---

### 4. Documentação do Código

**Verifique no Rust**:
```bash
cargo doc --workspace --no-deps 2>&1 | grep -E "(warning|missing)"
```

**Esperado**:
- [ ] `#![warn(missing_docs)]` em todos os crates
- [ ] Todos os itens `pub` têm docstrings
- [ ] Exemplos de código nas docstrings principais

**Verifique no C#**:
- [ ] XML documentation em classes públicas
- [ ] Comentários em métodos P/Invoke

---

### 5. Qualidade de Código

**Execute no Rust**:
```bash
# Linter
cargo clippy --workspace -- -D warnings 2>&1 | tee clippy.log

# Formatação
cargo fmt -- --check

# Segurança
cargo audit  # requer cargo-audit
```

**Execute no C#**:
```bash
dotnet build --verbosity normal 2>&1 | tee build.log
dotnet format --verify-no-changes
```

**Problemas comuns a verificar**:
- [ ] Warnings de unused imports/variables
- [ ] Unsafe code sem documentação de segurança
- [ ] unwrap() / expect() em código de produção
- [ ] unwrap() em código de teste é OK

---

### 6. CI/CD Pipeline

**Verifique se existe** `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  rust-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-action@stable
      - run: cd nova_kernel && cargo build --release
      - run: cd nova_kernel && cargo test --workspace
      - run: cd nova_kernel && cargo clippy --workspace -- -D warnings
      - run: cd nova_kernel && cargo fmt -- --check
      - run: cd nova_kernel && cargo doc --workspace --no-deps

  csharp-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-dotnet@v4
        with:
          dotnet-version: '8.0'
      - run: cd NovaCAD && dotnet restore
      - run: cd NovaCAD && dotnet build --configuration Release
      - run: cd NovaCAD && dotnet test --verbosity normal
```

---

## 🚀 Melhorias por Prioridade

### 🔴 Alta Prioridade (Bloqueantes para MVP)

#### 1. Implementar Operações Boolean Básicas

Criar `nova_kernel/crates/nova_ops/src/boolean.rs`:

```rust
//! Operações Boolean: Unite, Subtract, Intersect
//! 
//! Algoritmo geral:
//! 1. Encontrar interseções entre faces dos dois sólidos
//! 2. Classificar cada face como dentro/fora do outro sólido
//! 3. Reconstruir B-Rep com faces classificadas apropriadamente

use nova_topo::{Body, Face, Edge, Vertex, TopoResult};
use nova_geom::surface::Surface;

/// Une dois sólidos em um único sólido
pub fn boolean_unite(body_a: &Body, body_b: &Body) -> TopoResult<Body> {
    // TODO: Implementar algoritmo de subdivisão de faces
    // TODO: Classificar faces com ray casting
    // TODO: Reconstruir shell exterior
    todo!("Boolean unite not yet implemented")
}

/// Subtrai body_b de body_a
pub fn boolean_subtract(body_a: &Body, body_b: &Body) -> TopoResult<Body> {
    todo!("Boolean subtract not yet implemented")
}

/// Intersecta dois sólidos
pub fn boolean_intersect(body_a: &Body, body_b: &Body) -> TopoResult<Body> {
    todo!("Boolean intersect not yet implemented")
}

/// Encontra interseções entre duas faces
fn face_face_intersection(face_a: &Face, face_b: &Face) -> Vec<Edge> {
    // Interseção superfície-superfície
    // Trimar com limites das faces
    todo!()
}
```

**Tarefas**:
- [ ] Implementar interseção superfície-superfície
- [ ] Implementar trimagem de curvas com loops de face
- [ ] Implementar classificação ponto-em-sólido
- [ ] Implementar reconstrução B-Rep

---

#### 2. Implementar Tesselação para Visualização

Criar `nova_kernel/crates/nova_tess/src/lib.rs`:

```rust
//! Tesselação adaptativa de superfícies para visualização

use nova_topo::{Face, Body};
use nova_geom::surface::Surface;

/// Mesh triangular de uma face
#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<[f64; 3]>,
    pub normals: Vec<[f64; 3]>,
    pub uvs: Vec<[f64; 2]>,
    pub indices: Vec<u32>,
}

/// Tessela uma face com controle de erro de corda
pub fn tessellate_face(face: &Face, chordal_tolerance: f64) -> Mesh {
    match face.surface() {
        Surface::Planar(plane) => tessellate_planar(face, plane),
        Surface::Cylindrical(cyl) => tessellate_cylindrical(face, cyl, chordal_tolerance),
        Surface::Spherical(sph) => tessellate_spherical(face, sph, chordal_tolerance),
        Surface::Nurbs(nurbs) => tessellate_nurbs(face, nurbs, chordal_tolerance),
        _ => todo!("Tessellation for surface type not implemented"),
    }
}

/// Tesselação de superfície planar (simples)
fn tessellate_planar(face: &Face, plane: &PlanarSurface) -> Mesh {
    // Usar ear-clipping ou Delaunay para polígono 2D
    // Mapear para 3D via parametrização
    todo!()
}

/// Tesselação adaptativa de superfície curva
fn tessellate_cylindrical(face: &Face, cyl: &CylindricalSurface, tolerance: f64) -> Mesh {
    // Subdividir baseado na curvatura
    // Garantir erro de corda < tolerance
    todo!()
}
```

**Tarefas**:
- [ ] Implementar triangulação de polígonos 2D (ear-clipping)
- [ ] Implementar subdivisão adaptativa baseada em curvatura
- [ ] Implementar trimagem de malha com loops de borda
- [ ] Otimizar para compartilhamento de vértices

---

#### 3. Implementar Parser STEP Básico

Criar `nova_kernel/crates/nova_io/src/step/parser.rs`:

```rust
//! Parser para arquivos STEP (ISO 10303-21)

use std::path::Path;
use nova_topo::Body;

/// Parser de arquivo STEP
pub struct StepParser;

impl StepParser {
    /// Parse de arquivo STEP completo
    pub fn parse_file(path: &Path) -> Result<Vec<Body>, StepError> {
        let content = std::fs::read_to_string(path)?;
        Self::parse_str(&content)
    }
    
    /// Parse de string STEP
    pub fn parse_str(content: &str) -> Result<Vec<Body>, StepError> {
        // 1. Parse HEADER section
        // 2. Parse DATA section
        // 3. Resolver referências entre entidades
        // 4. Converter para B-Rep
        todo!()
    }
}

/// Entidades STEP suportadas
#[derive(Debug)]
enum StepEntity {
    // B-Rep shape representation
    AdvancedBrepShapeRepresentation { context: i64, items: Vec<i64> },
    // Geometria
    CartesianPoint { name: String, coordinates: Vec<f64> },
    Direction { name: String, ratios: Vec<f64> },
    Vector { name: String, orientation: i64, magnitude: f64 },
    // Topologia
    VertexPoint { name: String, vertex_geometry: i64 },
    EdgeCurve { name: String, edge_start: i64, edge_end: i64, edge_geometry: i64, same_sense: bool },
    FaceSurface { name: String, bounds: Vec<i64>, face_geometry: i64, same_sense: bool },
    ClosedShell { name: String, cfs_faces: Vec<i64> },
    ManifoldSolidBrep { name: String, outer: i64 },
}
```

**Tarefas**:
- [ ] Implementar lexer para formato STEP físico
- [ ] Implementar parser para entidades comuns
- [ ] Implementar resolução de referências forward
- [ ] Mapear entidades STEP para B-Rep interno

---

### 🟡 Média Prioridade (Funcionalidade Completa)

#### 4. Viewport 3D Funcional com OpenGL

Em `NovaCAD/src/NovaCad.Viewport/OpenGLViewport.cs`:

```csharp
using System;
using Avalonia;
using Avalonia.Controls;
using Avalonia.OpenGL;
using Avalonia.OpenGL.Controls;
using NovaCad.Core.Models;

namespace NovaCad.Viewport
{
    /// <summary>
    /// Viewport 3D usando OpenGL para renderização
    /// </summary>
    public class OpenGLViewport : OpenGlControlBase
    {
        private Shader _shader;
        private VertexArrayObject _vao;
        private BufferObject _vbo;
        private BufferObject _ebo;
        
        // Dados da cena
        private List<MeshData> _meshes = new();
        private Camera _camera;
        
        protected override void OnOpenGlInit(GlInterface gl)
        {
            base.OnOpenGlInit(gl);
            
            // Compilar shaders
            _shader = new Shader(gl, 
                vertexSource: ShaderSources.VertexShader,
                fragmentSource: ShaderSources.FragmentShader);
            
            // Criar VAO/VBO
            _vao = new VertexArrayObject(gl);
            _vbo = new BufferObject(gl, BufferTarget.ArrayBuffer);
            _ebo = new BufferObject(gl, BufferTarget.ElementArrayBuffer);
            
            // Configurar atributos de vértice
            // position (3), normal (3), uv (2)
            _vao.ConfigureAttribute(0, 3, VertexAttribPointerType.Float, false, 8 * sizeof(float), 0);
            _vao.ConfigureAttribute(1, 3, VertexAttribPointerType.Float, false, 8 * sizeof(float), 3 * sizeof(float));
            _vao.ConfigureAttribute(2, 2, VertexAttribPointerType.Float, false, 8 * sizeof(float), 6 * sizeof(float));
        }
        
        protected override void OnOpenGlRender(GlInterface gl, int fb)
        {
            gl.ClearColor(0.2f, 0.2f, 0.2f, 1.0f);
            gl.Clear(ClearBufferMask.ColorBufferBit | ClearBufferMask.DepthBufferBit);
            gl.Enable(EnableCap.DepthTest);
            
            _shader.Use();
            
            // Configurar uniforms (matrizes, luzes)
            _shader.SetMatrix4("u_view", _camera.ViewMatrix);
            _shader.SetMatrix4("u_projection", _camera.ProjectionMatrix);
            _shader.SetVector3("u_lightPos", new Vector3(10, 10, 10));
            
            // Renderizar cada mesh
            foreach (var mesh in _meshes)
            {
                _shader.SetMatrix4("u_model", mesh.Transform);
                _vao.Bind();
                gl.DrawElements(PrimitiveType.Triangles, mesh.IndexCount, DrawElementsType.UnsignedInt, IntPtr.Zero);
            }
        }
        
        /// <summary>
        /// Atualiza meshes a partir de um documento CAD
        /// </summary>
        public void LoadDocument(NovaDocument document)
        {
            _meshes.Clear();
            foreach (var body in document.Bodies)
            {
                var meshData = TessellateBody(body);
                _meshes.Add(meshData);
            }
            RequestNextFrameRendering();
        }
    }
}
```

**Tarefas**:
- [ ] Implementar classe Shader (compilação e linking)
- [ ] Implementar classes BufferObject e VertexArrayObject
- [ ] Implementar classe Camera (orbit, pan, zoom)
- [ ] Implementar sistema de materiais básico
- [ ] Implementar iluminação Phong simples

---

#### 5. Sistema de Undo/Redo

Em `NovaCAD/src/NovaCad.Core/Commands/CommandHistory.cs`:

```csharp
using System;
using System.Collections.Generic;

namespace NovaCad.Core.Commands
{
    /// <summary>
    /// Interface para comandos que podem ser desfeitos
    /// </summary>
    public interface ICommand
    {
        string Name { get; }
        void Execute();
        void Undo();
        void Redo() => Execute();
    }
    
    /// <summary>
    /// Histórico de comandos com suporte a undo/redo
    /// </summary>
    public class CommandHistory
    {
        private readonly Stack<ICommand> _undoStack = new();
        private readonly Stack<ICommand> _redoStack = new();
        private readonly int _maxHistory;
        
        public event Action HistoryChanged;
        
        public bool CanUndo => _undoStack.Count > 0;
        public bool CanRedo => _redoStack.Count > 0;
        
        public CommandHistory(int maxHistory = 100)
        {
            _maxHistory = maxHistory;
        }
        
        /// <summary>
        /// Executa um novo comando e adiciona ao histórico
        /// </summary>
        public void Execute(ICommand command)
        {
            command.Execute();
            _undoStack.Push(command);
            _redoStack.Clear(); // Novo comando limpa redo
            
            // Limitar tamanho do histórico
            if (_undoStack.Count > _maxHistory)
            {
                // TODO: Implementar truncamento
            }
            
            HistoryChanged?.Invoke();
        }
        
        /// <summary>
        /// Desfaz o último comando
        /// </summary>
        public void Undo()
        {
            if (!CanUndo) return;
            
            var command = _undoStack.Pop();
            command.Undo();
            _redoStack.Push(command);
            HistoryChanged?.Invoke();
        }
        
        /// <summary>
        /// Refaz o último comando desfeito
        /// </summary>
        public void Redo()
        {
            if (!CanRedo) return;
            
            var command = _redoStack.Pop();
            command.Redo();
            _undoStack.Push(command);
            HistoryChanged?.Invoke();
        }
        
        /// <summary>
        /// Limpa todo o histórico
        /// </summary>
        public void Clear()
        {
            _undoStack.Clear();
            _redoStack.Clear();
            HistoryChanged?.Invoke();
        }
    }
}
```

**Tarefas**:
- [ ] Criar comandos para cada operação (CreateBody, DeleteBody, TransformBody)
- [ ] Integrar com ViewModel principal
- [ ] Adicionar atalhos de teclado (Ctrl+Z, Ctrl+Y)
- [ ] Implementar persistência de histórico (opcional)

---

#### 6. Seleção de Entidades (Ray Picking)

Em `NovaCAD/src/NovaCad.Viewport/Picking/RayPicker.cs`:

```csharp
using System;
using System.Collections.Generic;
using System.Linq;
using NovaCad.Core.Models;
using NovaCad.Core.Math;

namespace NovaCad.Viewport.Picking
{
    /// <summary>
    /// Sistema de seleção por ray casting
    /// </summary>
    public class RayPicker
    {
        /// <summary>
        /// Seleciona a entidade mais próxima sob o cursor
        /// </summary>
        public PickResult? Pick(
            Point2 screenPos, 
            Camera camera, 
            IEnumerable<Body> bodies,
            PickFilter filter = PickFilter.All)
        {
            // 1. Criar ray a partir do cursor
            var ray = camera.ScreenPointToRay(screenPos);
            
            // 2. Testar interseção com bounding boxes primeiro
            var candidates = bodies
                .Where(b => filter.Allows(PickFilter.Body))
                .Where(b => ray.Intersects(b.BoundingBox))
                .ToList();
            
            // 3. Testar interseção com geometria
            PickResult? closest = null;
            float closestDistance = float.MaxValue;
            
            foreach (var body in candidates)
            {
                foreach (var face in body.Faces)
                {
                    if (!filter.Allows(PickFilter.Face)) continue;
                    
                    var intersection = ray.IntersectFace(face);
                    if (intersection.HasValue && intersection.Value.Distance < closestDistance)
                    {
                        closestDistance = intersection.Value.Distance;
                        closest = new PickResult 
                        { 
                            Entity = face, 
                            Point = intersection.Value.Point,
                            Distance = intersection.Value.Distance
                        };
                    }
                }
                
                // Similar para edges e vertices
            }
            
            return closest;
        }
    }
    
    public class PickResult
    {
        public Entity Entity { get; set; }
        public Point3 Point { get; set; }
        public float Distance { get; set; }
    }
    
    [Flags]
    public enum PickFilter
    {
        None = 0,
        Vertex = 1,
        Edge = 2,
        Face = 4,
        Body = 8,
        All = Vertex | Edge | Face | Body
    }
}
```

**Tarefas**:
- [ ] Implementar ScreenPointToRay na Camera
- [ ] Implementar Ray-Box intersection
- [ ] Implementar Ray-Face intersection (usar tesselação ou analítico)
- [ ] Visualização de entidades selecionadas (highlight)

---

### 🟢 Baixa Prioridade (Otimizações e Polish)

#### 7. Otimizações de Performance

**Spatial Indexing**:
```rust
// nova_kernel/crates/nova_geom/src/spatial.rs

/// Bounding Volume Hierarchy para acelerar interseções
pub struct BVH {
    root: BVHNode,
}

enum BVHNode {
    Leaf { bounds: BoundingBox, primitives: Vec<Primitive> },
    Internal { bounds: BoundingBox, left: Box<BVHNode>, right: Box<BVHNode> },
}

impl BVH {
    pub fn build(primitives: &[Primitive]) -> Self {
        // Construir usando SAH (Surface Area Heuristic)
        todo!()
    }
    
    pub fn intersect_ray(&self, ray: &Ray) -> Option<Intersection> {
        self.root.intersect_ray(ray)
    }
}
```

**Paralelização com Rayon**:
```rust
// Em operações que processam múltiplas faces/edges
use rayon::prelude::*;

faces.par_iter().map(|face| {
    process_face(face)
}).collect()
```

**SIMD para operações vetoriais**:
```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub fn dot_product_simd(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    unsafe {
        let va = _mm256_loadu_pd(a.as_ptr());
        let vb = _mm256_loadu_pd(b.as_ptr());
        let prod = _mm256_mul_pd(va, vb);
        // Horizontal sum
        let sum = _mm256_hadd_pd(prod, prod);
        // Extrair resultado
    }
}
```

---

#### 8. Ferramentas de Desenvolvimento

**Configuração rustfmt** (`.rustfmt.toml`):
```toml
edition = "2021"
max_width = 100
tab_spaces = 4
use_small_heuristics = "Default"
reorder_imports = true
reorder_modules = true
remove_nested_parens = true
```

**Configuração Clippy** (`.clippy.toml`):
```toml
cognitive-complexity-threshold = 30
too-many-arguments-threshold = 8
type-complexity-threshold = 500
```

**Pre-commit hooks** (`.pre-commit-config.yaml`):
```yaml
repos:
  - repo: local
    hooks:
      - id: rust-fmt
        name: Rust fmt
        entry: cargo fmt -- --check
        language: system
        files: \\.rs$
        pass_filenames: false
      
      - id: rust-clippy
        name: Rust clippy
        entry: cargo clippy --workspace -- -D warnings
        language: system
        files: \\.rs$
        pass_filenames: false
      
      - id: rust-test
        name: Rust test
        entry: cargo test --workspace
        language: system
        files: \\.rs$
        pass_filenames: false
```

---

## 📋 Checklist de Verificação Completa

Execute estes comandos no repositório clonado:

### Rust Kernel

```bash
cd nova_kernel

# 1. Build
printf "\n=== BUILD ===\n"
cargo build --release 2>&1 | tee build.log

# 2. Testes
printf "\n=== TESTS ===\n"
cargo test --workspace 2>&1 | tee test.log

# 3. Linter
printf "\n=== CLIPPY ===\n"
cargo clippy --workspace -- -D warnings 2>&1 | tee clippy.log

# 4. Formatação
printf "\n=== FMT ===\n"
cargo fmt -- --check 2>&1 | tee fmt.log

# 5. Documentação
printf "\n=== DOC ===\n"
cargo doc --workspace --no-deps 2>&1 | tee doc.log

# 6. Verificar crates não utilizados
printf "\n=== UNUSED ===\n"
cargo +nightly udeps 2>&1 | tee udeps.log || echo "cargo-udeps não instalado"

# 7. Verificar segurança
printf "\n=== AUDIT ===\n"
cargo audit 2>&1 | tee audit.log || echo "cargo-audit não instalado"
```

### C# Application

```bash
cd NovaCAD

# 1. Restore
printf "\n=== RESTORE ===\n"
dotnet restore 2>&1 | tee restore.log

# 2. Build
printf "\n=== BUILD ===\n"
dotnet build --configuration Release 2>&1 | tee build.log

# 3. Testes
printf "\n=== TEST ===\n"
dotnet test --verbosity normal 2>&1 | tee test.log

# 4. Formatação
printf "\n=== FORMAT ===\n"
dotnet format --verify-no-changes 2>&1 | tee format.log || echo "dotnet-format não instalado"
```

---

## 🎯 Próximos Passos Recomendados (Roadmap)

### Sprint 1 (Semanas 1-2): Fundamentos
- [ ] Implementar `nova_ops` com Boolean básico (unite apenas)
- [ ] Implementar `nova_tess` com triangulação planar
- [ ] Criar testes unitários para Boolean
- [ ] Criar CI/CD básico

### Sprint 2 (Semanas 3-4): Visualização
- [ ] Implementar viewport OpenGL funcional
- [ ] Integrar tesselação com viewport
- [ ] Implementar navegação de câmera (orbit, pan, zoom)
- [ ] Adicionar grid e eixos de referência

### Sprint 3 (Semanas 5-6): Interatividade
- [ ] Implementar sistema de seleção (ray picking)
- [ ] Implementar undo/redo
- [ ] Adicionar manipuladores de transformação (move, rotate, scale)
- [ ] Implementar feedback visual de seleção

### Sprint 4 (Semanas 7-8): Import/Export
- [ ] Implementar parser STEP básico (B-Rep apenas)
- [ ] Implementar export STL
- [ ] Testar com arquivos STEP reais
- [ ] Adicionar suporte a IGES (opcional)

### Sprint 5+ (Semanas 9+): Features Avançadas
- [ ] Implementar Boolean completo (subtract, intersect)
- [ ] Implementar features (fillet, chamfer, shell)
- [ ] Implementar Synchronous Technology básica
- [ ] Otimizações de performance (BVH, paralelização)

---

## 📞 Relatando Problemas

Se encontrar problemas durante a verificação, crie uma issue no GitHub incluindo:

1. **Descrição do problema**
2. **Comando que falhou**
3. **Logs completos** (stdout e stderr)
4. **Ambiente**:
   - OS: (Windows/Linux/macOS)
   - Versão Rust: `rustc --version`
   - Versão .NET: `dotnet --version`
5. **Passos para reproduzir**

---

## 📚 Recursos Úteis

### Documentação
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [AvaloniaUI Documentation](https://docs.avaloniaui.net/)
- [OpenGL Tutorial](https://learnopengl.com/)

### Referências CAD
- [OpenCASCADE Documentation](https://dev.opencascade.org/doc/overview/html/)
- [STEP File Format](https://www.steptools.com/stds/step/IS_final_p21e3.html)
- [NURBS Book](https://link.springer.com/book/10.1007/978-3-642-97385-7)

### Ferramentas
- `cargo install cargo-tarpaulin` - Cobertura de testes
- `cargo install cargo-audit` - Verificação de segurança
- `cargo install cargo-udeps` - Detecção de dependências não utilizadas
