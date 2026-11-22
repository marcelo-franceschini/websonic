# Desempenho Web e Remediação de Gargalos

Documento de conhecimento técnico sobre o perfil de desempenho, otimizações e arquitetura do caminho WebAssembly e navegador do WebSonic (novembro de 2026).

## 1. Visão Geral e Contexto

Em 17/11/2026, uma auditoria no ambiente de execução real (Chrome 157, Intel Core i5-1334U limitado a 1.300 MHz) revelou que a simulação WebAssembly não sustentava 60 quadros por segundo, apresentando 52,4 quadros/s, faltas frequentes no AudioWorklet e picos de tempo de quadro. O plano de remediação executou 31 itens técnicos cobrindo pipeline de vídeo, alocações de memória, síntese de áudio, temporização de quadros e custos de DOM.

Todas as afirmações neste documento são sustentadas por medições e testes do repositório.

## 2. Pipeline de Vídeo e Compositor WebGL2 (Itens V1a e V1b)

### Diagnóstico inicial
O modelo anterior utilizava um compositor de candidatos que delegava à GPU a seleção de prioridades de planos e sprites. Esse formato exigia a extração e envio de até 20 candidatos por pixel a cada linha, transferindo em média 1.486.964 bytes por quadro para a GPU e consumindo 72% do tempo de CPU em Wasm nas funções `fontes_planos`, `fontes_sprites` e `QuadroGpu::ponto`.

### Arquitetura implementada
1. Remoção do caminho de candidatos e do modo widescreen 16:9 no frontend.
2. Restauração do desenho completo na CPU via renderizador em software fiel da VDP.
3. Envio direto do buffer final de 320x224 pontos em 9 bits (71.680 palavras `u16`, totalizando 143.360 bytes por quadro).
4. Textura WebGL2 de formato `R16UI` com amostragem direta no shader de fragmento e decodificação dos 3 canais de cor de 3 bits.
5. Remoção de todo o código morto associado (`QuadroGpu`, `FonteDeTile`, `fontes_planos`, `fontes_sprites`).

### Evidência e Medições
- Transferência para GPU por quadro: reduzida de 1.486.964 bytes para 143.360 bytes fixos.
- Chamadas `texImage2D` após carregamento: reduzidas de 136 por minuto para zero, utilizando apenas `texSubImage2D` sobre buffer pré-alocado.
- Medição em Wasm (`partida-ghz1`, via `SONIC1_ROM=... node --test site/tests/desempenho-wasm.test.mjs`):
  - Média de tempo por quadro: 6,70 ms (meta: <= 8,0 ms; linha de base: 15,46 ms).
  - Percentil 95: 11,56 ms (meta: <= 13,0 ms; linha de base: 23,50 ms).
  - Quadros acima de 16,67 ms: 0,6% (meta: <= 2,0%; linha de base: 27,2%).

## 3. Eliminação de Alocações Dinâmicas (Itens V2, V3, V4, C1, K1, C2, A1)

### Diagnóstico inicial
A simulação executava 1.250,9 alocações e 119.701 bytes alocados por quadro na partida da GHZ1 (quadros 1000 a 1599). As principais fontes eram descartes de buffers de DMA, saídas de eventos do núcleo, listas de saídas de som e vetores temporários no misturador.

### Arquitetura implementada
1. Padrão de drenagem e reutilização de capacidade:
   - `Memoria::drenar_saidas_em` e `Memoria::descartar_saidas` no crate `core`.
   - Reutilização de `palavras_dma` e campos de reserva em `Console` e `Vdp`.
   - Preservação da capacidade dos vetores `saida_fm`, `saida_psg`, `amostras_mono` e `amostras_psg` no `MisturadorSaida`.
2. Preservação de determinismo e integridade: nenhum cálculo de ponto flutuante foi alterado e nenhuma regra do `core` foi violada.

### Evidência e Medições
- Teste de alocações (`SONIC1_ROM=... cargo test --locked --offline --release -p sonic1-console --test alocacoes_rom -- --ignored --nocapture`):
  - Alocações por quadro (quadros 1000 a 1599): 0,1 por quadro (meta: <= 2; linha de base: 1.250,9).
- Guarda de fidelidade da ROM (`SONIC1_ROM=... cargo test --locked --offline --release -p sonic1-console --test guarda_rom -- --ignored`):
  - 17.509 quadros comparados bit a bit: resultado 100% idêntico até `beb3175`. Depois da revisão R6 (`fe49e48`, anéis perdidos fiéis ao disassembly), os hashes esperados mudaram com aprovação do Marcelo e estão no próprio teste.
- Guarda de hash do console (`cargo run --locked --offline --release -q -p sonic1-console --bin console_hashes -- ... 7200 | sha1sum`):
  - Hash obtido: `b8d4a450a5baf18634d191411896fd5caad98d6e`, idêntico à auditoria até `beb3175`. Depois da R6, o valor esperado é `28299e8d4df6fd706ceb0fc8faecd61d382966fd`.

## 4. Síntese de Áudio e Canal Web (Itens S1, S2, A2, W2, A5, A6)

### Diagnóstico inicial
A rotina de sintetizador iterava ciclo a ciclo conferindo filas de escrita em 81.069 ciclos mestre por quadro. Além disso, cada amostra FM gerada copiava estruturas de 104 bytes, a reamostragem do PSG mono calculava duas convoluções redundantes e a transferência para o AudioWorklet gerava cópias e mensagens em excesso.

### Arquitetura implementada
1. Avanço em laço interno até a próxima escrita de FM e PSG no sintetizador.
2. Adição de `avancar_ate_compacto`, entregando fatias compactas `(i16, i16)` e `u16`.
3. Função `processar_mono` no reamostrador sinc, reduzindo para uma convolução a conversão do PSG e duplicando o sinal para estéreo na mistura.
4. Pool de buffers `Float32Array(4096)` reutilizáveis no JavaScript e devolução de buffers pelo AudioWorklet via canal `postMessage`.
5. Frequência de atualização de status do worklet reduzida para 1 em cada 24 blocos (cerca de 5 vezes por segundo).

### Evidência e Medições
- Benchmark de áudio (`WEBSONIC_BENCH=1 cargo test --locked --offline -p sonic1-audio-saida --release --test desempenho -- --nocapture`):
  - Mediana: 2.539 us (meta: <= 3.000 us; linha de base: 2.786 us).
  - Percentil 95: 4.753 us; máximo: 5.008 us.
- Teste de carga do worklet (`node --test site/tests/audio-worklet-carga.test.cjs`):
  - Aprovado com taxa de mensagens inferior a 16 mensagens por segundo.

## 5. Temporização e Custos de Interface (Itens R1, J1, J2, D1, D2, D3)

### Diagnóstico inicial
O laço web usava frequência de 59,9227 Hz da VDP do Mega Drive enquanto o navegador operava com base nominal de 60,000 Hz, provocando repetição periódica de quadros a cada 13 segundos (4,7 repetições por minuto). Elementos de debug no DOM causavam repaints contínuos e `getGamepads` gerava chamadas desnecessárias a cada quadro.

### Arquitetura implementada
1. Relógio a 60 Hz exatos (`1000 / 60 ms` por quadro, correspondendo a 16,6666 ms).
2. Ajuste na razão de reamostragem para `taxa_dispositivo / 60.0`.
3. Verificação de gamepad apenas após evento `gamepadconnected`.
4. Atualização de indicadores de áudio do painel restrita a 1 a cada 6 quadros, sem uso de `innerHTML` e sem transições CSS com recálculo de layout.
5. Supressão de eventos de teclado com `e.repeat` e cache de elementos por código de tecla.
6. Ocultação da camada de filtro CRT via `display: none` quando desativada.

### Evidência e Medições
- Testes de ritmo e relógio:
  - `node --test site/tests/ritmo-60hz.test.mjs`: aprovado com zero quadros repetidos ou descartados a 60 Hz.
  - `node --test site/tests/relogio.test.mjs`: aprovado em 60 Hz, 120 Hz, 144 Hz e 240 Hz.

## 6. Tabela Comparativa de Resultados

| Métrica | Antes | Depois | Meta | Comando de Validação |
| --- | --- | --- | --- | --- |
| Tempo por quadro Wasm (média) | 15,46 ms | 6,70 ms | <= 8,0 ms | `SONIC1_ROM=... node --test site/tests/desempenho-wasm.test.mjs` |
| Tempo por quadro Wasm (p95) | 23,50 ms | 11,56 ms | <= 13,0 ms | `SONIC1_ROM=... node --test site/tests/desempenho-wasm.test.mjs` |
| Quadros Wasm acima de 16,67 ms | 27,2% | 0,6% | <= 2,0% | `SONIC1_ROM=... node --test site/tests/desempenho-wasm.test.mjs` |
| Alocações por quadro (gameplay) | 1.250,9 | 0,1 | <= 2,0 | `SONIC1_ROM=... cargo test -p sonic1-console --test alocacoes_rom -- --ignored` |
| Dados enviados à GPU por quadro | 1.486.964 B | 143.360 B | 143.360 B | Constante WebGL2 R16UI (320x224x2) |
| Benchmark síntese de áudio (mediana) | 2.786 us | 2.539 us | <= 3.000 us | `WEBSONIC_BENCH=1 cargo test -p sonic1-audio-saida --release --test desempenho` |
| Tamanho binário Wasm (`sonic1_web_bg.wasm`) | 593.170 B (`opt-level = "s"`) | 685.335 B (`opt-level = 3`, +15,5%) | <= 760.000 B | `ls -l dist/pkg/sonic1_web_bg.wasm` |
| Fidelidade 7.200 quadros (`console_hashes`) | `b8d4a450a5...` | `28299e8d4d...` depois da R6 | Igual ao valor esperado | `cargo run -p sonic1-console --bin console_hashes -- ... 7200` |
| Fidelidade 17.509 quadros (`guarda_rom`) | Hashes da auditoria | Hashes da R6 | Iguais aos do teste | `SONIC1_ROM=... cargo test -p sonic1-console --test guarda_rom -- --ignored` |

## 7. Desempenho Web, Rodada 2 (19/11/2026)

Fatos técnicos e medições da auditoria e execução da rodada 2 (`docs/relatorios/relatorio-performance-2.md`):

1. **DevTools e compilação do V8:** com o DevTools aberto, o V8 roda o Wasm 2,4 vezes mais devagar (13,84 contra 5,79 ms na microbancada). Medições de desempenho web devem ser feitas com o DevTools fechado (`docs/relatorios/relatorio-performance-2.md`, seção 3.1).
2. **Topologia de núcleos e frequência:** o processador i5-1334U possui núcleos P e E. Com o turbo desligado, o Wasm consome 6,20 ms no núcleo P e 10,79 ms no núcleo E (`docs/relatorios/relatorio-performance-2.md`, seção 3.1). Com turbo ligado, o executor mediu na base média de 3,47 ms no núcleo P e 5,09 ms no núcleo E.
3. **Pausas do compositor no navegador:** o Chrome suspende chamadas de `requestAnimationFrame` por 50 a 242 ms periodicamente, mesmo sem sobrecarga da CPU. Temporizadores via `setInterval` continuam sendo atendidos durante essas pausas (`docs/relatorios/relatorio-performance-2.md`, seção 3.1). O vigia implementado em `site/app.js` utiliza esse mecanismo para manter a cadência de simulação e evitar faltas no buffer do AudioWorklet.
4. **Divergência de libm no áudio Wasm:** o áudio gerado pelo Wasm diverge do nativo em amostras fracionárias devido a implementações distintas de `f32::sin` e `f32::cos` na libm. A renderização visual é idêntica bit a bit. A integridade do Wasm é garantida pela guarda própria `site/tests/guarda-wasm.test.mjs` (`docs/relatorios/relatorio-performance-2.md`, seção 3.1).
5. **Otimizações descartadas e medidas:**
   - Desenrolar FM: +0,45 ms no Wasm e +52 KB no binário.
   - Tabela de índices no FM: sem ganho comprovado no Wasm (razão de 0,97 a 1,15 no nativo).
   - Composição por tabela: +0,17 ms no Wasm.
   - Saída antecipada da VDP em FIFO vazia: razão de 0,99 a 1,05 no nativo.
   - Flags do compilador (`opt-level = 2`, `codegen-units = 1`, `+simd128`): sem ganho de tempo de execução.
   - Otimizador `wasm-opt` (-O3 e -O4): ganho no ruído (-1% a -3%) e piora severa de aquecimento (6,5 ms contra 4,0 ms nos primeiros 60 quadros).
6. **Detector de vazamento da ROM:** o script detector foi validado com controle positivo (código de saída 1) e negativo (código de saída 0), operando sobre o índice git antes de cada commit (`docs/relatorios/relatorio-performance-2.md`, seção 0.7).
7. **Resultados medidos pelo executor após a rodada 2 (com turbo ativo):**
   - Quadro Wasm `partida-ghz1`, núcleo P (média intercalada, 3 rodadas): 1,51 ms (base 1,80 ms, razão de 0,84x, comando: `taskset -c 2 $(command -v node) /tmp/websonic-exec/bancada-wasm.mjs dist/pkg`).
   - Quadro Wasm, núcleo E: média 2,34 ms, p95 2,55 ms, zero quadros acima de 16,67 ms (comando: `taskset -c 6 $(command -v node) site/tests/desempenho-wasm.test.mjs`).
   - Alocações dinâmicas por quadro: 0,1 alocação por quadro (comando: `cargo test --locked --offline --release -p sonic1-console --test alocacoes_rom -- --ignored --nocapture`).
   - Tamanho do artefato Wasm: 688.379 bytes (comando: `cargo xtask site && ls -l dist/pkg/sonic1_web_bg.wasm`).
   - Linhas de código removidas (K1 a K5): 613 linhas de código e testes não utilizados eliminadas do repositório.
