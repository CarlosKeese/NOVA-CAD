# Nova CAD - Roadmap de Desenvolvimento

## Fase 1: Viewport 3D Funcional (Semana 1-2)

### Objetivo
Tornar o viewport 3D interativo e capaz de renderizar geometria básica.

### Tarefas
1. **Sistema de Renderização**
   - [ ] Implementar Shader PBR básico
   - [ ] Criar sistema de materiais
   - [ ] Implementar iluminação básica (ambient + directional)

2. **Viewport Interativo**
   - [ ] Implementar grid no chão (XZ plane)
   - [ ] Adicionar eixos XYZ coloridos (R=G=B)
   - [ ] Implementar navegação orbital suave
   - [ ] Adicionar zoom com scroll do mouse

3. **Malhas 3D**
   - [ ] Criar gerador de malha para Box
   - [ ] Criar gerador de malha para Sphere
   - [ ] Criar gerador de malha para Cylinder
   - [ ] Sistema de buffers OpenGL otimizado

### Resultado Esperado
Viewport 3D funcional com grid, eixos e capacidade de renderizar formas básicas.

---

## Fase 2: Criação de Geometria (Semana 3-4)

### Objetivo
Permitir que o usuário crie objetos 3D através da interface.

### Tarefas
1. **Comandos de Criação**
   - [ ] Implementar comando CreateBox
   - [ ] Implementar comando CreateSphere
   - [ ] Implementar comando CreateCylinder
   - [ ] Diálogo de parâmetros para cada forma

2. **Sistema de Documento**
   - [ ] Estrutura de documento em memória
   - [ ] Lista de bodies com metadados
   - [ ] Sistema de undo/redo básico

3. **Model Tree Funcional**
   - [ ] Sincronização com documento
   - [ ] Checkbox de visibilidade funcionando
   - [ ] Seleção na árvore seleciona no viewport
   - [ ] Deletar objetos

### Resultado Esperado
Usuário pode criar múltiplos objetos, vê-los na árvore e no viewport.

---

## Fase 3: Interatividade (Semana 5-6)

### Objetivo
Permitir seleção e manipulação de objetos.

### Tarefas
1. **Seleção**
   - [ ] Raycasting para seleção
   - [ ] Highlight de objetos selecionados
   - [ ] Multi-seleção (Ctrl+Click)
   - [ ] Caixa de seleção (drag)

2. **Transformações**
   - [ ] Gizmo de translação (XYZ)
   - [ ] Gizmo de rotação
   - [ ] Gizmo de escala
   - [ ] Snapping à grid

3. **Propriedades**
   - [ ] Painel de propriedades dinâmico
   - [ ] Editar posição/rotação/escala
   - [ ] Mudar cor do objeto
   - [ ] Renomear objetos

### Resultado Esperado
Usuário pode selecionar, mover, rotacionar e escalar objetos interativamente.

---

## Fase 4: Persistência (Semana 7-8)

### Objetivo
Salvar e carregar trabalho.

### Tarefas
1. **Formato Nativo**
   - [ ] Definir formato .nova (JSON/Binary)
   - [ ] Serialização de geometria
   - [ ] Serialização de estrutura
   - [ ] Salvamento automático (backup)

2. **Import/Export**
   - [ ] Importar STEP (básico)
   - [ ] Exportar STL (para 3D print)
   - [ ] Exportar OBJ (para render)

3. **Gerenciamento de Arquivos**
   - [ ] Recent files
   - [ ] Diálogos de arquivo
   - [ ] Confirmação de save ao sair

### Resultado Esperado
Usuário pode salvar, abrir e exportar seus projetos.

---

## Fase 5: Ferramentas Avançadas (Semana 9-12)

### Objetivo
Adicionar funcionalidades profissionais.

### Tarefas
1. **Modelagem**
   - [ ] Extrude de sketch (2D → 3D)
   - [ ] Revolve (lathe)
   - [ ] Fillet e chamfer
   - [ ] Shell (criar peças finas)

2. **Boolean Operations**
   - [ ] Unite (união)
   - [ ] Subtract (subtração)
   - [ ] Intersect (interseção)

3. **Ferramentas de Moldagem**
   - [ ] Análise de undercuts
   - [ ] Linha de separação
   - [ ] Cavidade e núcleo
   - [ ] Análise de ângulos de saída

4. **Medição**
   - [ ] Distância entre pontos
   - [ ] Ângulo entre faces
   - [ ] Área de superfície
   - [ ] Volume

### Resultado Esperado
Software CAD funcional para modelagem mecânica básica.

---

## Fase 6: Integração Rust/C# (Contínua)

### Objetivo
Implementar integração completa entre kernel Rust e UI C#.

### Tarefas
1. **FFI Completo**
   - [ ] Resolver linkagem MSVC
   - [ ] Criar DLL nativa do kernel
   - [ ] P/Invoke completo
   - [ ] Marshaling eficiente

2. **Performance**
   - [ ] Tesselação em Rust
   - [ ] Caching de malhas
   - [ ] Renderização instanciada
   - [ ] Occlusion culling

3. **Estabilidade**
   - [ ] Tratamento de erros do kernel
   - [ ] Recuperação de falhas
   - [ ] Logging detalhado
   - [ ] Testes automatizados

---

## Métricas de Sucesso

### v0.1 (MVP)
- Criar e visualizar Box, Sphere, Cylinder
- Navegar no viewport
- Salvar/abrir documento

### v0.2 (Usável)
- Transformações (move, rotate, scale)
- Seleção e propriedades
- Import/Export STL

### v0.3 (Profissional)
- Boolean operations
- Extrude e Revolve
- Fillet/Chamfer

### v1.0 (Completo)
- Todas as features do roadmap
- Documentação completa
- Testes automatizados
- Performance otimizada

---

## Notas de Prioridade

**Alta Prioridade:**
1. Viewport funcional com grid/eixos
2. Criação de geometria básica
3. Model tree sincronizada

**Média Prioridade:**
4. Seleção e transformações
5. Salvar/abrir
6. Import/Export

**Baixa Prioridade (pós-MVP):**
7. Boolean operations
8. Ferramentas de moldagem
9. Medição
10. Sketch-based modeling
