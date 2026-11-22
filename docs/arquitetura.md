# Arquitetura do WebSonic

Visao tecnica enxuta do fluxo de dados, dos limites de determinismo e dos diretorios do projeto.

## Mapa do Repositorio

```text
crates/
  core/          Nucleo fiel do Sonic 1 (sem unsafe, sem float, sem HashMap)
  video/         VDP em software (cache 8bpp com filtro de escrita/SAT, tiles 64-bit, fast-path sem sprite/destino vazio, sem unsafe/float)
  audio/         Sintese digital YM2612 (FM em lote de 24 slots) e SN76489 (PSG) em inteiros (fila O(1), sem unsafe)
  audio-saida/   Reamostrador FIR polifase 4-vias sem buffer intermediario (SIMD-friendly, fila de saida O(1), sem unsafe), filtro analogico e controlador P
  console/       Integracao quadro a quadro (DMA em fatia ROM/RAM, divisao preguicosa de audio, LUT RGBA, forbid(unsafe_code))
  rom/           Identificacao SHA-1 e normalizacao .bin/.md/.gen/.smd (16 KiB, forbid(unsafe_code))
  web/           Camada fina wasm-bindgen (forbid(unsafe_code))
  formato-video/ Formato binario de captura e comparacao de video (forbid(unsafe_code))
  recomp/        Gerador e runtime auxiliares de verificacao (forbid(unsafe_code))
  corpus/        [congelado] Leitura de entradas e metadados do corpus
  fidelidade/    [congelado] Relatorio de fidelidade e hashes
site/            Frontend modular estatico (Web Worker, OffscreenCanvas/WebGL2, AudioWorklet, PWA offline completo)
  app.js           Orquestrador de sessao, estado da UI e comunicacao com o Worker
  video.js         Apresentador de video (OffscreenCanvas, fallback WebGL2/2D sem alocacao por quadro, escala inteira, fullscreen)
  preferencias.js  Persistencia validada de preferencias visuais (FPS, filtro CRT e pontos de CRAM)
  gpu.js           Pipeline WebGL2 (R16UI + shader 9-bit, liberacao explicita de recursos GPU)
  audio.js         Controlador de AudioContext, rampa suave no GainNode, AudioWorkletNode e persistencia de volume/mudo
  modal-controles.js  Modal acessivel de remapeamento persistente de teclado
  entrada.js       Leitura de teclado/gamepad e mascara de 8 bits do controle de 3 botoes
  hud.js           Telemetria (media/p95, atraso de audio, FPS) e grafico em canvas
  saves.js         Serializacao binaria deterministica RLE de Save States e IndexedDB
  save-ui.js       Controlador de UI dos slots de Save State e importacao/exportacao .s1sav
  buffers.js       Estruturas puras HistoricoEntradas e PoolBuffers reciclaveis do Worker
  pwa.js           Registro do Service Worker, banner de instalacao e indicador offline
  sw.js            Service Worker offline-first com precache completo de todos os modulos JS e Wasm com hash
  sim-worker.js    Dedicated Web Worker executando Wasm + WebGL2 em OffscreenCanvas
  audio-worklet.js AudioWorkletProcessor com ring buffer potencia de 2, escrita contigua e zero alocacao em process()
  tests/           17 suites unitarias em Node.js nativo (58 testes, 100% de cobertura de linhas nos 12 modulos testaveis de site/)
server.mjs         Servidor HTTP estatico local (porta 3000, cabecalhos de seguranca, bloqueio de path traversal)
package.json       Scripts Node.js sem dependencias externas (`start`, `dev`, `build`, `lint`, `test`, `test:coverage`)
xtask/             Automacao de build (`site`, `verify-js`, `bench`, `anti-rom`, inventarios)
docs/
  requisitos.md    [congelado] Especificacao e regras do projeto
  arquitetura.md   Este documento
  estado.md        Status vivo do projeto
  backlog.md       Pendencias fora do caminho critico
  bugs-conhecidos.md  Lista de verificacao dos 315 itens `FixBugs`
  conhecimento/    Base de conhecimento por tema (indice em `README.md`)
  inventarios/     Inventarios gerados de hardware, memoria, 68000, barramento, interrupcoes e som
  relatorios/      Relatorios de performance e auditorias
  pedidos/         Patches preparados para itens congelados
  arquivo/planos/  Planos historicos de etapas concluidas
scripts/
  sut-reproduz.sh
  arquivo/         Scripts pontuais das etapas 2 a 8
```

## Fluxo de Execucao no Navegador

```text
[app.js (Orquestrador)]
  |-- video.js (OffscreenCanvas / WebGL2 / 2D / escala inteira)
  |-- audio.js (AudioContext / AudioWorklet / volume / bipe)
  |-- entrada.js + modal-controles.js (teclado / gamepad / remap)
  |-- hud.js + saves.js + save-ui.js + pwa.js (telemetria / RLE / offline)
       |  (mascaras de entrada + controle de sessao / Save States)
       v
[sim-worker.js (Dedicated Web Worker)]
  |-- Wasm (`sonic1-web` compilado com `wasm-release` + SIMD128)
  |-- Video zero-copy: `OffscreenCanvas` + WebGL2 direto da memoria Wasm
  |   (com fallback automatico ping-pong Uint16Array para a thread principal)
  |-- Audio direto via `MessageChannel` (sem passar pela thread principal)
       v
[audio-worklet.js (AudioWorkletProcessor)]
  |-- Fila circular potencia de 2 com fast-path estereo via `Float32Array.set`
  |-- Devolve buffers reciclados e ocupacao da fila direto ao Worker
```

## Fronteira de Determinismo

1. **Determinismo bit a bit e 0% `unsafe` em producao**: todas as 9 crates de producao em Rust (`core`, `video`, `audio`, `audio-saida`, `console`, `rom`, `web`, `recomp`, `formato-video`) aplicam `#![forbid(unsafe_code)]`. Alem disso, `sonic1-core`, `sonic1-video` e `sonic1-audio` proibem ponto flutuante, estouro silencioso e colecoes nao deterministicas (`HashMap`/`HashSet`). A mesma ROM + mesma sequencia de botoes gera exatamente os mesmos quadros e amostras digitais em qualquer arquitetura.
2. **Ponto flutuante isolado**: apenas `sonic1-audio-saida` converte a saida inteira dos chips para `f32` na taxa do `AudioContext`, usando convolucao polifase de 4 acumuladores independentes (pronta para auto-vetorizacao SIMD128/NEON/SSE) e buffers pre-alocados sem realocacoes por quadro.
3. **Facade por dominio em `sonic1-core`**: `sonic1_core::jogo` expoe os modulos agrupados por dominio (`sistema`, `telas`, `sonic`, `chefes`, `badniks`, `zonas`), preservando `sonic1_core::abertura` para compatibilidade com comparadores congelados.

## Comandos Principais

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo xtask verify-js
cargo xtask site
cargo xtask bench
npm run lint
npm test
npm run test:coverage
npm start
```

## Recomendacao para `.github/workflows/ci.yml` (arquivo congelado)

Quando descongelar `.github/workflows/ci.yml` localmente, adicione o passo de verificacao JS logo apos os testes nativos (e sincronize `xtask/src/bench.rs` com `package.json` incluindo `gpu.test.mjs`, `modal-controles.test.mjs` e `pwa.test.mjs`):

```yaml
      - name: Testes unitarios do frontend (Node.js)
        run: sh .github/rodar-na-imagem.sh cargo xtask verify-js
```

