# Guia de Diagnóstico - Nova CAD Viewport

## 📍 Localização dos Logs

Os logs são salvos em:
```
%LOCALAPPDATA%\NovaCAD\viewport_logs.txt
```

Ou execute no PowerShell:
```powershell
Get-Content "$env:LOCALAPPDATA\NovaCAD\viewport_logs.txt" -Wait
```

---

## 📋 Questionário de Diagnóstico

Por favor, responda às seguintes perguntas para ajudar a identificar o problema:

### 1. Sistema Operacional
- [ ] Windows 10
- [X] Windows 11
- [ ] Outro: _______

### 2. Placa de Vídeo
- Qual sua placa de vídeo? (NVIDIA/AMD/Intel)
- Modelo: Nvidia RTX 3060
- Drivers atualizados? [X] Sim [ ] Não

### 3. Comportamento da Aplicação
Abre exibindo algumas opções mas não funcionam e exibem respostas de não implementação, não é possível visualizar nada e nem um ambiente 3D renderizado

#### Ao abrir:
- [X] A janela principal aparece normalmente
- [ ] A janela aparece mas fica preta/branca
- [ ] A janela não aparece
- [ ] Erro ao iniciar

#### Área do Viewport:
- [ ] Vê uma área cinza escura (fundo 3D)
- [X] Vê uma área preta
- [ ] Vê uma área branca
- [ ] Vê o texto "3D Viewport" (placeholder antigo)
- [ ] Área do viewport não aparece

#### Ao criar um Box (Create > Box):
- [X] Nada acontece visualmente
- [ ] Aparece mensagem "Box created" na barra de status
- [ ] Aparece item na Model Tree (painel esquerdo)
- [ ] Erro é exibido

### 4. Console/Terminal

Ao executar, aparecem mensagens como estas?

```
[VIEWPORT] [Info] ViewportControl constructor called
[VIEWPORT] [Info] ViewportControl attached to visual tree
[VIEWPORT] [Info] OnOpenGlInit called
[VIEWPORT] [Info] OpenGL Version: 4.x.x
```

- [ ] Sim, vejo mensagens de log
- [ ] Não, não vejo mensagens
- [X] Vejo mensagens de erro

### 5. Conteúdo do Log

Cole aqui as últimas 20 linhas do arquivo de log:

```
(cole o conteúdo aqui)
```

---

## 🖼️ Referência Visual Esperada

Baseado no Solid Edge e outros CADs profissionais, o viewport deve ter esta aparência:

### Layout da Interface

```
┌─────────────────────────────────────────────────────────────┐
│  Nova CAD - Untitled                              [_][□][X] │
├─────────────────────────────────────────────────────────────┤
│ File  Edit  View  Create  Help                              │
├─────┬───────────────────────────────────────────┬───────────┤
│     │  [Box][Cyl][Sph]  [Fit][Fr][To][Iso]      │           │
│     ├───────────────────────────────────────────┤           │
│  M  │                                           │  P        │
│  o  │         V I E W P O R T  3 D              │  r        │
│  d  │                                           │  o        │
│  e  │    ┌─────────────────────────┐            │  p        │
│  l  │    │      GRID (chão)        │            │  e        │
│     │    │         │ Y (verde)      │            │  r        │
│  T  │    │         │                │            │  t        │
│  r  │    └─────────┼────────────────┘            │  i        │
│  e  │    Z (azul)──┼──X (vermelho)  │            │  e        │
│  e  │              │                 │            │  s        │
│     │    [Caixas 3D aparecem aqui]  │            │           │
│     │                                           │           │
│     │  [ShadedWithEdges]     LMB: Select       │           │
│     │                          MMB: Orbit      │           │
│     └───────────────────────────────────────────┘           │
│  1 body  mm  Ready                                          │
└─────────────────────────────────────────────────────────────┘
```

### Elementos Visuais do Viewport

1. **Fundo**: Cinza escuro (RGB: 38, 38, 38)
2. **Grid**: Linhas cinzas no plano XZ (chão)
3. **Eixos XYZ**:
   - X (Vermelho) - Horizontal direita
   - Y (Verde) - Vertical para cima
   - Z (Azul) - Profundidade
4. **Malhas 3D**: Cinza claro com shading

---

## 🔍 Problemas Comuns

### Problema: Viewport fica preto
**Causas possíveis:**
- OpenGL não inicializou
- Contexto gráfico não criado
- Erro nos shaders

**Para verificar:**
1. Abra o log
2. Procure por "OnOpenGlInit called"
3. Verifique se há mensagens de erro após isso

### Problema: Geometria não aparece
**Causas possíveis:**
- Malha não foi inicializada com OpenGL
- ViewModel não está conectado ao ViewportControl
- Coordenadas fora do campo de visão

**Para verificar:**
1. Clique em Create > Box
2. Verifique se aparece "Box created" na barra de status
3. Verifique se aparece "Box" na Model Tree (esquerda)
4. Veja o log por "VisualObjectCreated"

### Problema: Viewport não responde ao mouse
**Causas possíveis:**
- Controle não tem foco
- Eventos não estão sendo processados

**Para verificar:**
1. Clique no viewport
2. Tente MMB (botão do meio) para orbitar
3. Verifique log por "MouseDown" messages

---

## 🛠️ Testes Rápidos

Execute estes testes e me informe os resultados:

### Teste 1: Verificar OpenGL
```powershell
cd nova_cad/NovaCAD
dotnet run 2>&1 | Select-String "OpenGL"
```

**Resultado esperado:**
```
[VIEWPORT] [Info] OpenGL Version: 4.x.x NVIDIA/AMD/Intel
[VIEWPORT] [Info] OpenGL Vendor: NVIDIA/AMD/Intel
[VIEWPORT] [Info] OpenGL Renderer: GTX/RTX/Radeon
```

### Teste 2: Criar Box
1. Execute a aplicação
2. Clique em Create > Box
3. Verifique o log:
```powershell
Get-Content "$env:LOCALAPPDATA\NovaCAD\viewport_logs.txt" | Select-String "Box"
```

**Resultado esperado:**
```
CreateBox command executed
nova_make_box result: Success
Box created
VisualObjectCreated: Box
Mesh Box added to viewport
```

---

## 📸 Capturas de Tela de Referência

Se possível, envie prints mostrando:

1. **Aplicação completa** (janela inteira)
2. **Viewport em detalhe** (região central)
3. **Model Tree** (painel esquerdo após criar um Box)
4. **Barra de status** (parte inferior)

### Exemplo do Solid Edge (referência):

O viewport do Solid Edge mostra:
- **Canto inferior esquerdo**: Indicador de orientação (cube view)
- **Centro**: Grid sutil no plano XY
- **Canto inferior direito**: Controles de zoom/navegação
- **Borda**: Eixos X,Y,Z coloridos
- **Fundo**: Gradiente ou cor sólida escura

---

## ✅ Checklist Final

Antes de enviar feedback, verifique:

- [ ] Anexe o arquivo de log completo
- [ ] Respondeu todas as perguntas do questionário
- [ ] Anexou prints da tela (se possível)
- [ ] Descreveu o comportamento esperado vs atual

---

## 📧 Como Enviar Feedback

1. Execute a aplicação
2. Tente criar um Box (Create > Box)
3. Feche a aplicação
4. Cole o conteúdo do log aqui
5. Responda o questionário acima
6. Anexe prints se possível

**Arquivo de log:**
```
%LOCALAPPDATA%\NovaCAD\viewport_logs.txt
```
