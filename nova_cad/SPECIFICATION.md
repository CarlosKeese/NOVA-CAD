# Nova Kernel 3D + Nova CAD - Especificação Técnica

## Visão Geral

Este projeto implementa um kernel 3D CAD open-source em Rust com uma aplicação CAD profissional em C#/AvaloniaUI, seguindo a especificação do documento fornecido.

## Arquitetura do Kernel (Rust)

### Camadas

```
L0 - nova_math     : Fundamentos matemáticos
L1 - nova_geom     : Curvas e superfícies
L2 - nova_topo     : Topologia B-Rep
L3 - nova_ops      : Operações (Boolean, features)
L4 - nova_sync     : Edição direta (Synchronous Technology)
L5 - nova_tess     : Tesselação
L6 - nova_io       : Import/export (STEP, IGES)
L7 - nova_check    : Validação e healing
L8 - nova_ffi      : Interface C-ABI
```

### Módulos Implementados

#### nova_math
- `Point2`, `Point3`, `Point4` - Pontos em 2D, 3D, 4D (homogêneo)
- `Vec2`, `Vec3`, `Vec4` - Vetores
- `Mat3`, `Mat4` - Matrizes de transformação
- `Transform3` - Transformações rígidas (rotação + translação)
- `Quaternion` - Quaternions para rotações
- `BoundingBox2`, `BoundingBox3` - Bounding boxes
- `Interval` - Aritmética de intervalos
- `ToleranceContext` - Contexto de tolerância hierárquico
- `Plane` - Planos 3D
- `predicates` - Predicados geométricos robustos (orient2d, orient3d, incircle, insphere)

#### nova_geom
- `Curve` trait - Interface para todas as curvas
- `Line` - Linhas infinitas e segmentos
- `CircularArc` - Arcos circulares e círculos completos
- `EllipseArc` - Arcos elípticos
- `NurbsCurve` - Curvas NURBS
- `Surface` trait - Interface para todas as superfícies
- `PlanarSurface` - Superfícies planas
- `CylindricalSurface` - Superfícies cilíndricas
- `SphericalSurface` - Superfícies esféricas
- `ConicalSurface` - Superfícies cônicas
- `ToroidalSurface` - Superfícies toroidais
- `NurbsSurface` - Superfícies NURBS
- `intersection` - Algoritmos de interseção

#### nova_topo
- `EntityId` - Identificadores únicos de entidades
- `Vertex` - Vértices com posição e tolerância
- `Edge` - Arestas com curva geométrica
- `Coedge` - Uso orientado de aresta em um loop
- `Loop` - Sequência fechada de coedges
- `Face` - Face com superfície e loops
- `Shell` - Conjunto conexo de faces
- `Body` - Corpo sólido completo
- `EulerOps` - Operadores Euler (MVFS, MEV, MEF, KEMR, KFMRH, MEKR)
- `build_cube` - Construção de cubo usando operadores Euler

#### nova_ffi
- Interface C-ABI completa
- Tipos: `NovaHandle`, `NovaPoint3`, `NovaVec3`, `NovaMat4`, `NovaTransform`, `NovaBBox3`, `NovaMesh`
- Funções de inicialização: `nova_init`, `nova_shutdown`, `nova_version`
- Criação de primitivas: `nova_make_box`, `nova_make_cylinder`, `nova_make_sphere`, `nova_make_cone`, `nova_make_torus`
- Operações em bodies: `nova_body_release`, `nova_body_transform`, `nova_body_bounding_box`, `nova_body_copy`
- Boolean: `nova_boolean_unite`, `nova_boolean_subtract`, `nova_boolean_intersect`
- Features: `nova_fillet`, `nova_chamfer`, `nova_shell`
- Tesselação: `nova_tessellate_body`, `nova_mesh_free`
- I/O: `nova_import_step`, `nova_export_step`, `nova_export_stl`

## Arquitetura da Aplicação CAD (C#)

### Projetos

```
NovaCad.Core      : Modelos de domínio, interfaces
NovaCad.Kernel    : P/Invoke wrapper para o kernel Rust
NovaCad.Viewport  : Renderização 3D com Silk.NET/OpenGL
NovaCad.UI        : Componentes de UI com AvaloniaUI
NovaCad.App       : Aplicação principal
```

### Componentes Implementados

#### NovaCad.Core
- `NovaDocument` - Documento CAD com bodies, seleção, undo/redo
- `NovaBodyRef` - Referência a body do kernel
- `SelectionSet` - Conjunto de entidades selecionadas
- `ViewState` - Estado da visualização (câmera, modo de render)
- `CameraState` - Posição, target, FOV da câmera
- `MaterialLibrary` - Biblioteca de materiais
- `NovaColor` - Cores com RGBA
- `BoundingBox3` - Bounding box 3D

#### NovaCad.Kernel
- `NovaKernel` - Classe estática com P/Invoke
- `NovaHandle` - Handle para objetos do kernel
- `NovaResult` - Códigos de resultado
- `NovaPoint3`, `NovaVec3`, `NovaMat4`, `NovaTransform`, `NovaBBox3` - Estruturas interop
- `NovaMesh`, `NovaMeshVertex` - Estruturas de mesh
- `NovaKernelException` - Exceção para erros do kernel

#### NovaCad.App
- `App.axaml` - Recursos e estilos da aplicação
- `App.axaml.cs` - Inicialização, DI container
- `Program.cs` - Entry point
- `MainWindow.axaml` - Janela principal com menu, toolbar, painéis
- `MainWindowViewModel` - ViewModel principal
- `ViewportViewModel` - ViewModel do viewport
- `ModelTreeViewModel` - ViewModel da árvore de modelo
- `PropertyPanelViewModel` - ViewModel do painel de propriedades
- `RibbonViewModel` - ViewModel da ribbon

## Funcionalidades

### Kernel
- [x] Fundamentos matemáticos completos
- [x] Curvas analíticas (linha, arco, elipse, NURBS)
- [x] Superfícies analíticas (plano, cilindro, esfera, cone, toro, NURBS)
- [x] Topologia B-Rep completa
- [x] Operadores Euler
- [x] Interface C-ABI

### CAD Application
- [x] Estrutura do projeto
- [x] P/Invoke wrapper
- [x] Modelos de domínio
- [x] Interface básica (menu, toolbar, painéis)
- [x] Comandos para criar primitivas (box, cylinder, sphere)
- [x] Controles de visualização

## Compilação

### Requisitos
- Rust 1.75+ (com Cargo)
- .NET 8 SDK
- (Opcional) AvaloniaUI templates

### Build
```bash
# Build completo
./build.sh

# Apenas kernel
./build.sh kernel

# Apenas aplicação
./build.sh app

# Testes
./build.sh test

# Executar
./build.sh run
```

## Roadmap

### Fase 1 - Fundação ✓
- [x] Matemática e geometria analítica
- [x] Topologia B-Rep
- [x] Interface C-ABI
- [x] Estrutura da aplicação C#

### Fase 2 - Operações 🔄 (Em Progresso - 75%)
- [x] Crate `nova_ops` criado (boolean, feature, fillet, split, error)
- [x] Boolean operations: intersection, classification, face splitting, construction
- [x] Features: Extrude, Revolve, Sweep, Loft com opções completas
- [x] Fillets/Chamfers: analysis, propagation, variable radius
- [x] STEP I/O: AP214/AP242 parser completo, conversão B-Rep bidirecional
- [x] STL I/O: Export ASCII e Binary
- [ ] Finalização com operadores Euler completos

### Fase 3 - Edição Direta
- [ ] Face move/rotate/offset
- [ ] Live rules
- [ ] Geometric recognition

### Fase 4 - Aplicação Completa
- [ ] Viewport 3D com OpenGL
- [ ] Steering Wheel
- [ ] Seleção e manipulação
- [ ] Mold tools

## Licenças

- **Nova Kernel (Rust)**: LGPL 2.1+
- **Nova CAD Application**: GPL 3.0
- **NovaSharp (C# Interop)**: MIT

## Referências

- [The NURBS Book](https://www.springer.com/gp/book/9783540615453) - Piegl & Tiller (1997)
- [Computational Geometry](https://link.springer.com/book/10.1007/978-3-540-77974-2) - de Berg et al.
- [Robust Geometric Computation](https://cs.nyu.edu/exact/) - Shewchuk (1997)
- [Synchronous Technology](https://www.plm.automation.siemens.com/global/en/products/nx/synchronous-technology.html) - Siemens
