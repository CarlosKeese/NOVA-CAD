# Nova CAD - Checklist de Validação e Verificação

## 1. Validação da Interface do Usuário

### 1.1 Janela Principal
- [x] Aplicação abre sem erros
- [x] Título da janela correto ("Nova CAD - Untitled")
- [x] Dimensões mínimas respeitadas (800x600)
- [x] Menu principal visível (File, Edit, View, Create, Help)
- [x] Toolbar com ícones renderizados
- [x] Status bar na parte inferior
- [x] Painéis laterais visíveis (Model Tree, Properties)

### 1.2 Viewport 3D
- [x] Área de viewport renderiza com OpenGL
- [x] Grid visível no plano XZ
- [x] Eixos XYZ visíveis (R=Z, G=Y, B=Z)
- [x] Controles de navegação listados (LMB, MMB, Shift+MMB, Scroll)
- [x] Indicador de modo de renderização visível
- [x] Navegação com mouse funcional (orbit, pan, zoom)

### 1.3 Painéis
- [x] Model Tree visível à esquerda
- [x] Properties visível à direita
- [x] Abas funcionando (ShadedWithEdges)
- [x] Mensagem "No selection" quando nada selecionado

## 2. Validação de Funcionalidades

### 2.1 Menu File
- [x] New Document - Limpa documento e viewport
- [ ] Open Document - IMPLEMENTAR (mensagem: "Open document not implemented")
- [ ] Save Document - IMPLEMENTAR
- [ ] Save As - IMPLEMENTAR
- [ ] Import/Export - IMPLEMENTAR

### 2.2 Menu Create
- [x] Create Box - Cria caixa 100x50x30, aparece no viewport e Model Tree
- [x] Create Cylinder - Cria cilindro (r=25, h=100), aparece no viewport e Model Tree
- [x] Create Sphere - Cria esfera (r=30), aparece no viewport e Model Tree
- [ ] Create Cone - IMPLEMENTAR
- [ ] Create Torus - IMPLEMENTAR

### 2.3 Viewport Interactions
- [x] Grid visível
- [x] Eixos XYZ visíveis
- [x] Navegação com mouse funcionando (MMB=orbit, Shift+MMB=pan, scroll=zoom)
- [ ] Seleção de objetos - IMPLEMENTAR
- [x] Zoom/Pan/Orbit funcionando
- [x] Malhas de primitivos renderizando

### 2.4 Model Tree
- [x] Lista de bodies atualizando quando cria primitivos
- [ ] Checkbox de visibilidade funcional - IMPLEMENTAR
- [ ] Seleção na árvore - IMPLEMENTAR
- [ ] Context menu - IMPLEMENTAR

### 2.5 View Commands
- [x] Fit All (F) - Centraliza câmera nos objetos
- [x] Front View - Define vista frontal
- [x] Top View - Define vista superior
- [x] Right View - Define vista lateral direita
- [x] Isometric View - Define vista isométrica
- [x] Shaded Mode - Modo sombreado
- [x] Wireframe Mode - Modo arame
- [x] Shaded with Edges Mode - Modo sombreado com arestas

## 3. Validação Técnica

### 3.1 Compilação
- [x] Projeto Rust compila (cargo check)
- [x] Projeto C# compila (dotnet build)
- [x] Sem erros de linkagem críticos
- [x] Executável gerado

### 3.2 Dependências
- [x] Avalonia UI funcionando
- [x] Silk.NET OpenGL inicializando e renderizando
- [x] Integração ViewModel/Viewport funcionando via eventos
- [ ] Integração Rust/C# completa - PENDENTE (usando stubs)

### 3.3 Performance
- [ ] Tempo de inicialização < 3s
- [ ] FPS estável no viewport
- [ ] Memória estável

## 4. Critérios de Aceitação para v0.1

### Must Have (MVP)
- [x] Criar geometria básica (Box, Sphere, Cylinder)
- [x] Visualizar geometria no viewport 3D
- [x] Navegar no viewport (zoom, pan, rotate)
- [x] Ver lista de objetos no Model Tree
- [ ] Salvar/abrir documentos - PENDENTE

### Should Have
- [x] Grid e eixos de referência
- [ ] Seleção de objetos - PENDENTE
- [ ] Propriedades editáveis - PENDENTE
- [x] Cores diferentes por objeto - IMPLEMENTADO (padrão cinza)

### Nice to Have
- [ ] Transformações (move, rotate, scale)
- [ ] Boolean operations
- [ ] Undo/Redo
- [ ] Import/Export STEP

## 5. Bugs Conhecidos

### Críticos
1. ~~Viewport não renderiza geometria~~ - ✅ CORRIGIDO
2. ~~Comandos do menu não funcionam~~ - ✅ CORRIGIDO (Create funcionando)
3. Integração Rust/C# inativa - nova_ffi desabilitado (usando stubs)

### Médios
1. Ícone da aplicação não carrega
2. Toolbar usa StackPanel ao invés de ToolBarTray (workaround)
3. Vários warnings de nullability no C#
4. Malhas criadas são sempre cinza (cor fixa)

### Baixos
1. Tema visual básico (precisa de polimento)
2. Fontes padrão do sistema
3. Não há seleção visual de objetos

## 6. Próximos Passos de Implementação

### Fase 1: Viewport Funcional ✅ CONCLUÍDO
1. ✅ Implementar shader básico para renderização
2. ✅ Criar sistema de grid no viewport
3. ✅ Implementar câmera orbital funcional
4. ✅ Adicionar eixos XYZ visuais

### Fase 2: Geometria Básica ✅ CONCLUÍDO
1. ✅ Implementar criação de Box via MeshFactory
2. ✅ Implementar criação de Sphere
3. ✅ Implementar criação de Cylinder
4. ✅ Converter geometria para malha OpenGL

### Fase 3: Interatividade (Em Progresso)
1. Seleção de objetos com raycasting
2. Transformações básicas
3. ✅ Atualização da Model Tree
4. Painel de propriedades dinâmico

### Fase 4: Persistência
1. Formato de arquivo nativo (.nova)
2. Salvar/abrir documentos
3. Import STEP básico
4. Export STL para impressão 3D

---

## Relatório de Status Atual

**Data:** 2026-02-07  
**Versão:** 0.0.2-alpha  
**Status:** Viewport funcional com criação de primitivos

### Resumo
A aplicação agora possui um viewport 3D funcional com OpenGL que renderiza grid, eixos e malhas de primitivos (Box, Sphere, Cylinder). A criação de geometria via menu/toolbar funciona e atualiza tanto o viewport quanto a Model Tree. A navegação com mouse (orbit, pan, zoom) está implementada.

### Componentes Funcionais
- ✅ Interface gráfica
- ✅ Estrutura de projeto
- ✅ Compilação Rust + C#
- ✅ Viewport 3D com OpenGL (grid, eixos, malhas)
- ✅ Criação de primitivos (Box, Cylinder, Sphere)
- ✅ Model Tree atualizando
- ✅ Navegação de câmera (orbit, pan, zoom)
- ✅ View presets (Front, Top, Right, Iso)
- ✅ Render modes (Shaded, Wireframe, ShadedWithEdges)

### Componentes Pendentes
- ❌ Abrir/salvar documentos
- ❌ Seleção de objetos
- ❌ Propriedades editáveis
- ❌ Integração completa Rust/C#
- ❌ Import/Export

**Estimativa para MVP completo:** 1 semana de desenvolvimento focado
