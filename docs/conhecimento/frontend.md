# Frontend Modular, UI, PWA e Cobertura de Testes

Registro técnico da arquitetura do frontend em `site/`, do servidor local `server.mjs`, das decisões de interface, do empacotamento PWA offline e da suíte de testes unitários em Node.js (22/11/2026).

## 1. Arquitetura Modular de `site/`

O frontend opera sem frameworks nem bundler JS externo, mantendo módulos ES puros com responsabilidades isoladas:

- `site/app.js`: orquestrador de sessão, estado da UI, ligação entre módulos e mensagens com o Web Worker.
- `site/sim-worker.js`: Dedicated Web Worker que executa `sonic1-web` (Wasm), gerencia o loop de simulação com `Relogio`, histórico de entradas (`HistoricoEntradas`), pool de buffers (`PoolBuffers`) e renderização WebGL2 em `OffscreenCanvas`. A rotina `prepararNovaSessao(msg, manterPausado)` centraliza o reset de sessão entre `iniciar` e `carregar-save`.
- `site/video.js` e `site/gpu.js`: apresentação em 320x224 com textura WebGL2 `R16UI` (cor de 9 bits), recuperação de `webglcontextlost`/`webglcontextrestored`, fallback automático para Canvas 2D com LUT de 512 cores, escala inteira e tela cheia. Em `gpu.js`, falha na compilação do fragment shader libera explicitamente o vertex shader compilado (`gl.deleteShader(vs)`).
- `site/audio.js` e `site/audio-worklet.js`: `AudioContext`, `GainNode` com rampa suave, persistência de volume/mudo, bipe de teste e canal direto `MessageChannel` entre Worker e `AudioWorkletProcessor` (fila circular potência de 2, zero alocação em `process()`).
- `site/entrada.js` e `site/modal-controles.js`: leitura de teclado (com remapeamento persistente e troca automática em conflito), gamepad (padrão e eixos DirectInput 4/5), toque opcional, cancelamento de direções opostas e modal acessível com restauração de foco.
- `site/saves.js` e `site/save-ui.js`: Save States determinísticos por log de entradas comprimido em RLE (`.s1sav`), validação por SHA-1 da ROM e persistência em 3 slots via IndexedDB com fallback em memória. `criarGerenciadorSaves({ idb })` aceita injeção opcional da fábrica IndexedDB para testes unitários isolados.
- `site/buffers.js`: `HistoricoEntradas` (crescimento geométrico) e `PoolBuffers` reciclável com busca de trás para frente e *swap-pop* O(1), preservando buffers de tamanhos distintos sem descarte indevido.
- `site/preferencias.js`: validação e persistência de preferências visuais (FPS 60/desbloqueado, filtro CRT, pontos de CRAM).
- `site/hud.js`: telemetria em tempo real (FPS, Hz de simulação, média e p95 sem alocação no laço quente, gráfico de frametime em canvas) e diagnóstico de áudio.
- `site/pwa.js` e `site/sw.js`: registro do Service Worker offline-first, indicador de status offline, fluxo `beforeinstallprompt` (Chrome/Edge) e guia de instalação no iOS Safari.

## 2. Servidor Local e Empacotamento Estático

- `server.mjs`: servidor HTTP local na porta 3000 (`npm start` / `npm run dev`), priorizando `dist/index.html` e fazendo fallback para `site/`. Bloqueia URLs malformadas (`400`), bytes nulos (`400`), *path traversal* (`403`) e métodos fora de `GET`/`HEAD` (`405`), além de enviar `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer` e `Cache-Control: no-store`.
- `xtask/src/site.rs`: monta `dist/` compilando `sonic1-web` com perfil `wasm-release`, `+simd128,+bulk-memory,+nontrapping-fptoint`, `wasm-opt` (quando disponível), cópia de todos os módulos/ícones PWA e reescrita topológica de query strings `?h=<hash>` em `index.html`, módulos JS e `sw.js`.

## 3. Interface e Responsividade

- Layout redesenhado em `site/index.html` e `site/style.css` com hierarquia clara entre palco do console (proporção 320x224 em escala inteira), barra de ações primárias, painel retrátil de diagnóstico, modal de remapeamento de teclas e controles responsivos para telas estreitas.
- Evidência (22/11/2026, Chromium headless via `playwright-cli` em `http://localhost:3000`):
  - Desktop `1440x900`: `overflowX: false`, 0 erros de console, 0 avisos.
  - Mobile `390x844`: `overflowX: false`, 0 erros de console, 0 avisos.

## 4. Suíte de Testes Unitários e Cobertura

Execução com Node.js nativo (`node --test` e `--experimental-test-coverage`), sem dependências npm externas:

```sh
npm run lint
npm test
npm run test:coverage
```

### Evidência (22/11/2026, Node v22.23.2)

- `npm run lint` (`node --check` em `server.mjs` e nos 15 módulos de `site/`): 0 erros.
- `npm run test:coverage`: **58 de 58 testes passando** em 17 arquivos de teste (`site/tests/`), com **99,78% de cobertura de linhas**, **93,96% de branches** e **97,56% de funções**:
  - `site/buffers.js`, `site/gpu.js`, `site/preferencias.js`, `site/relogio.js`: 100,00% linhas, 100,00% branches, 100,00% funções.
  - `site/saves.js`: 100,00% linhas, 97,89% branches, 100,00% funções.
  - `site/modal-controles.js`: 100,00% linhas, 94,12% branches, 100,00% funções.
  - `site/video.js`: 100,00% linhas, 92,73% branches, 90,91% funções.
  - `site/audio.js`: 100,00% linhas, 92,31% branches, 92,86% funções.
  - `site/hud.js`: 100,00% linhas, 90,38% branches, 100,00% funções.
  - `site/entrada.js`: 100,00% linhas, 88,15% branches, 100,00% funções.
  - `site/pwa.js`: 100,00% linhas, 85,00% branches, 100,00% funções.
  - `site/save-ui.js`: 100,00% linhas, 80,00% branches, 100,00% funções.
  - `server.mjs`: 93,23% linhas (100% das funções exportadas testadas via HTTP real GET/HEAD/POST e casos 400/403/404/405).

## 5. Pendência para Sincronização no Host (Rust)

- Em `xtask/src/bench.rs`, a função `verificar_js()` ainda lista 14 arquivos de teste; ao rodar no host com `cargo`, incluir `site/tests/gpu.test.mjs`, `site/tests/modal-controles.test.mjs` e `site/tests/pwa.test.mjs` para manter `cargo xtask verify-js` alinhado com `npm test`.
