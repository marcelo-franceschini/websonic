# Sintetizador (Etapa 8)

## Saída no navegador, evolução de 13/11/2026

- O AudioWorklet espera aproximadamente três quadros de amostras antes de iniciar e conta separadamente faltas e amostras descartadas. `node --test site/tests/audio-worklet.test.cjs` verifica preenchimento, falta, reinício e fila cheia.
- O controlador recebe a ocupação da fila do navegador. Com a fila cheia, aumenta o passo do reamostrador e reduz a produção. Prova: `cargo test -p sonic1-console --test audio fila_cheia_reduz_a_quantidade_de_audio_produzido`.
- `cargo test -p sonic1-audio-saida --test fluxo_real` simula 600 quadros NTSC com consumo fracionário a 44,1 e 48 kHz e atualização periódica da ocupação, sem esvaziar nem saturar a fila simulada. Não substitui uma partida real.
- O misturador preserva sobras de FM e PSG entre lotes, inclusive quando chegam separados. Prova: `cargo test -p sonic1-audio-saida --test lotes_defasados`, também executado em `wasm32-wasip1`.
- A tabela sinc ajusta o corte para a menor taxa entre entrada e saída. Prova: `cargo test -p sonic1-audio-saida --test antialias`, também em `wasm32-wasip1`, compara atenuação de um sinal acima do Nyquist com um sinal audível.
- No Chrome headless com a ROM REV01, AudioWorklet ficou ativo e informou zero faltas durante os primeiros 468 quadros observados. O navegador usou SwiftShader. Isso não comprova fidelidade da temporização dos eventos de som nem uma partida completa.
- Medida nativa após as mudanças: `WEBSONIC_BENCH=1 cargo test --locked --offline -p sonic1-audio-saida --release --test desempenho -- --nocapture`, mediana 879 µs, p95 2473 µs e máximo 3284 µs, com outras tarefas em execução. Essa medida não representa desempenho Wasm.

## Otimizações de síntese e reamostragem na rodada 2 (19/11/2026)

- **PSG por amostra estavel (item S1):** quando os divisores de frequencia e canais mantem sua atenuacao e nao ha escritas pendentes, o laco de sintese em `crates/audio/src/sintetizador.rs` avanca 16 ciclos mestre de uma vez atraves de `Psg::amostra_estavel`. A fidelidade amostra a amostra contra `Psg::ciclo` e validada pelo teste `crates/audio/tests/psg_amostra_estavel.rs`. O tempo de quadro Wasm caiu cerca de 0,65 ms (razao de 0,88 a 0,90 contra a base no nucleo P).
- **Alimentacao mono sem copia redundante (item A1):** em `crates/audio-saida/src/reamostrador.rs`, `alimentar_mono` alimenta exclusivamente `buffer_in_l`. O buffer do canal direito nao recebe copias desnecessarias e `processar_mono` drena apenas o canal esquerdo, economizando cerca de 15 us por quadro.
- **Guarda de sintese nativa (item G1):** o teste `crates/console/tests/guarda_sintese.rs` afere as amostras nativas pre-reamostragem de FM e PSG e as taxas do site (47.938,19 Hz e 44.043,22 Hz), conferindo os 5 hashes de referencia do Anexo A.2 do relatorio da rodada 2.

Plano: `docs/arquivo/planos/plano-etapa-8.md`. Cada afirmação cita o teste ou o comando. Nada aqui tem dado da ROM.

## Licenças e fontes (8.0, bloco 1 do Marcelo, 09/11/2026)

| Referência | Commit | Licença | SHA-1 do `LICENSE` |
| --- | --- | --- | --- |
| Nuked-OPN2 (`ym3438.c`, `ym3438.h`) | `335747d78cb0abbc3b55b004e62dad9763140115` | LGPL 2.1 ou superior (cabeçalho e `LICENSE`) | `b386b371ce94933e63ced1052aa72a60da5485ff` |
| Nuked-PSG (`ympsg.c`, `ympsg.h`) | `d15a168c676f4669e23660be9225b34ad7c1764e` | GPL 2 ou superior (cabeçalho e `LICENSE`) | `4cc77b90af91e615a64ae04893fdffa7939db84c` |

Comando: `sh scripts/arquivo/etapa8-bloco1.sh` (commits conferidos iguais aos fixados; `cc` GCC 16.2.1; Rust 1.99.1; o host não tem `rustup` nem `wasmtime`, então o alvo wasm e os testes wasm rodam na imagem). Uso permitido: comparação local, no workspace `diferencial-sintese/` (GPLv3), fora do `core`, do `crates/audio` e do site. O fonte dos chips fica em `$SONIC1_EXT_DIR/trabalho/nuked`, fora do git.

## Decisões

- **D8.2 (origem do código, com divulgação).** O `crates/audio` não copia código do Nuked. Ao desenhar o formato de comparação do PSG, a IA **leu o `ympsg.c` inteiro** (9 KB), porque o `YMPSG_GetOutput` devolve `float` e o formato digital só se define vendo o estado interno. O PSG do `crates/audio` foi escrito depois, em estrutura própria (nomes, funções e decomposição diferentes), mas segue a mesma semântica de ritmo de 4 ciclos, porque é isso que a identidade amostra a amostra exige. Quem revisar deve tratar o PSG como **derivado do comportamento do Nuked-PSG**, não como escrito só de documentação pública. Para o FM, a IA **não abriu o corpo do `ym3438.c`**: só o cabeçalho público (`ym3438.h`, nomes dos campos) e o comportamento medido por sondas sintéticas (`diferencial-sintese/examples/sonda*.rs`).
- **D8.7 (quem roda o quê).** O `D1` (execução com a ROM só pelo Marcelo) não proíbe rodar o Nuked com **escritas sintéticas**. A IA roda a bateria sintética núcleo contra Nuked para calibrar. O veredito da etapa (8.5) vem das escritas reais do corpus e roda só pelo Marcelo.
- **D8.8 (amostra do FM).** No modo YM2612 o pino do Nuked-OPN2 é **multiplexado**: cada canal aparece em um ciclo próprio dentro dos 24 ciclos de uma amostra, e os outros ciclos mostram o valor de repouso. Logo a "amostra do FM" para comparar é o fluxo de 24 pares (esquerda, direita) por amostra, não uma soma. Ordem do pico por canal, medida (`examples/sonda3.rs`): ciclo 3 canal 1, 7 canal 5, 11 canal 3, 15 canal 0, 19 canal 4, 23 canal 2 (contando a partir de `cycles == 0`). A mistura para o navegador soma os 6 picos.
- **D8.9 (amostra do PSG).** A saída digital do Nuked-PSG é o que `YMPSG_Read` devolve: 4 níveis de 4 bits (atenuação de cada canal, 15 = mudo), empacotados em 16 bits, lidos a cada 16 ciclos do PSG (mestre/15). O `float` do `YMPSG_GetOutput` é só a conversão para mistura e não entra na comparação.
- **D8.10 (reamostragem polifásica em fase).** Reamostrador sinc janelado (Blackman-Harris) com 256 fases e 32 taps implementado em `crates/audio-saida/src/reamostrador.rs`. FM (~53 267 Hz) e PSG (~223 722 Hz) reamostrados independentemente para a taxa do dispositivo (48 kHz ou 44,1 kHz).
- **D8.11 (controle fino de taxa contra deriva).** Controlador proporcional de baixa banda que varia a razão em até ±0,5% baseado na ocupação da fila circular de áudio, compensando diferenças entre a taxa de vídeo (60 Hz nominal) e o relógio de áudio do sistema operacional sem cortes nem estalos.
- **D8.12 (cadeia analógica aproximada).** Filtro passa-baixa RC de primeira ordem (~3386 Hz, Mega Drive Modelo 1) em `crates/audio-saida/src/analogico.rs`, marcado como `APROXIMADO`. Fica fora da prova amostra a amostra.
- **D8.13 (AudioWorklet web sem SharedArrayBuffer).** Processador `AudioWorkletProcessor` em `site/audio-worklet.js` usando fila circular com preenchimento em blocos de 128 amostras e transferência via `port.postMessage` para garantir funcionamento em hospedagens estáticas (GitHub Pages) sem isolamento de origem cruzada.
- **D8.14 (orçamento de desempenho).** Medido com `WEBSONIC_BENCH=1` em 600 quadros de pior caso (todas as vozes ativas, DAC e modulações densas). Mediana em modo release nativo de ~1,8 ms (orçamento de 3 ms para WebAssembly).

## PSG: prova (8.1 e 8.2)

- `diferencial-sintese/tests/psg.rs` (4 testes, `cargo test` no workspace, rodado pela IA com escritas sintéticas): silêncio depois do reset; tom simples; tons mais ruído; 40 sementes de 300 escritas aleatórias em ciclos aleatórios (6000 amostras cada). Resultado em 09/11/2026: **iguais ao Nuked-PSG em todas as amostras**, com checagem de que a saída varia (não é silêncio nos dois lados).
- Unidade: um ciclo do PSG = mestre/15 (15 mclk); uma amostra = 16 ciclos (240 mclk). O reset (16 ciclos com IC alto) é o estado de partida de `Psg::novo()`.
- Falta: modelo independente simples e propriedades (determinismo, nunca panica) no `crates/audio`.

## FM: o que foi medido do Nuked-OPN2 (8.3, sondas sintéticas)

- **Saída do pino (modo YM2612):** `pino = 3 * saida_do_canal + (saida >= 0 ? 3 : 0)`, com a saída do canal em 9 bits com sinal (-256 a 255). Canal com o lado desligado no panning mostra o repouso `3`. `examples/sonda2.rs` e `sonda3.rs`.
- **Processamento por slot:** 24 slots; o slot de índice `i` (grupo `i / 6`, canal `i % 6`) é processado no ciclo `(5 + i) mod 24` (contagem do campo `cycles` antes do ciclo). Grupo 0 é o operador 1 (endereço `+0`). A ordem dos grupos 1 a 3 ainda não foi medida.
- **Operador:** a saída `fm_out` usa a fase **antes** do incremento da amostra. O canal latcha a soma dos operadores no ciclo `6 + canal`, mas com a saída do grupo 0 da amostra **anterior**. Resultado: fase no instante T, saída do canal em T+2 amostras. `diag_latencia_tecla` em `diferencial-sintese/tests/fm.rs`.
- **Tecla (key-on):** o nível do envelope vai a 0 cerca de 49 ciclos depois da escrita do dado e a fase zera 52 ciclos depois (canal 0, grupo 0). A fase corre livre antes disso.
- **Latência de escrita:** o efeito de uma escrita depende do ciclo em que ela chega, em degraus de 24 ciclos. Registradores de operador (multiplicador, detune, TL) são lidos num ponto do ciclo e o registrador de F-number baixo seis ciclos antes (`examples/sonda4.rs`: limiar j=11 contra j=17). Isto indica que cada registrador é gravado quando o seu slot passa.
- **Tabelas log-seno e exponencial:** formula publicada das ROMs do OPL2 (`diferencial-sintese/examples/tabelas.rs`); a operação `exp(x)` indexa `EXP[x & 255]` direto e desloca por `x >> 8`.

## FM: máquina de estados e prova (8.3)

- **Identidade:** idêntico ciclo a ciclo e amostra a amostra ao Nuked-OPN2 no modo YM2612 em toda a bateria sintética (`diferencial-sintese/tests/fm.rs`), incluindo algoritmo 7, um canal, seis canais, envelopes aleatórios e 25 sementes completas de escritas dinâmicas com DAC.
- **Estrutura interna:**
  - Cadência de 24 ciclos por amostra, 24 slots (4 operadores × 6 canais).
  - Barramento: escritas travadas com pipeline de IO e sinal de endereço/dado ativo no ciclo seguinte. Registradores de operador e canal gravados quando o slot/canal correspondente passa.
  - Envelope Generator (EG): pipeline de 2 ciclos. Cálculo de `eg_inc` no ciclo seguinte a partir de `eg_rate` e `eg_ksv` escalares; atraso de 2 ciclos para `eg_tl` e `eg_sl`; transição de Decai1 para Decai2 quando `(level >> 4) == (sl << 1)`.
  - Phase Generator (PG): F-Number e bloco travados por canal no ciclo anterior. Detune calculado com `DT` em 3 passos, multiplicador com meio passo quando zero.
  - DAC: amostrado no registrador `$2A`, sinal de 9 bits (`(d ^ 0x80) << 1` estendido com sinal), ligado por `$2B` substituindo a saída do canal 6.
  - Saída do pino: multiplexada por canal nos 24 ciclos internos, com repouso `3`.

## Sintetizador e agendamento em mclk (8.4)

- **Unidade:** relógio mestre NTSC (53 693 175 Hz). FM a 42 mclk por ciclo (1008 mclk por amostra de 24 ciclos); PSG a 15 mclk por ciclo (240 mclk por amostra de 16 ciclos).
- **Invariância à fatiação:** comprovada em `crates/audio/tests/sintetizador.rs` (passo único vs passos curtos fixos vs passos aleatórios produzem as mesmas amostras).
- **Contagem por quadro:** em 60 quadros de 896 040 mclk, contagem por quadro estritamente 888 ou 889 de FM e 3733 ou 3734 de PSG, acumulado idêntico à fórmula fracionária.
- **Determinismo e ordenação:** escrita com carimbo `mclk` é ordenada e aplicada exatamente no ciclo correspondente (`t / 42` ou `t / 15`).

## Harness diferencial e Bloco 2 (8.5)

- **Comparador em memória:** `diferencial-sintese/src/comparador.rs` e binário `diferencial-sintese/src/bin/sintese.rs`. Compara FM e PSG simultaneamente alimentados pelas mesmas escritas.
- **Relatório por bloco de 4096 amostras:** calcula hashes SHA-1 por bloco de 4096 amostras e hash global do relatório.
- **Bateria sintética:** executada via `cargo run --manifest-path diferencial-sintese/Cargo.toml --release --bin sintese -- --sintetico`, produzindo 100% `IGUAL`.
- **Script do Bloco 2 do Marcelo:** `scripts/arquivo/etapa8-bloco2.sh` preparado e testado para execução contra os traces de oráculo da ROM real.

## Prova com a ROM real (8.5 e fechamento, Marcelo em 10/11/2026)

Comando: `sh scripts/arquivo/etapa8-bloco2.sh "$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen" 600`.

- **Veredito:** `VEREDITO GERAL DA ETAPA: IGUAL` (exit code 0).
- **Hash do relatório:** `55a112765775af740cb43a49f1100e91c841630a`
- **SHA-1 de `relatorio-sintese.txt`:** `1013000deae3943a496a94031618c83a2cac4565`
- **Resultados detalhados por caso:**
  - Bateria sintética FM: 7 casos, 11 blocos (45 056 amostras) -> **100% IGUAL**.
  - Bateria sintética PSG: 2 casos, 3 blocos (12 288 amostras) -> **100% IGUAL**.
  - `boot-demos` (7200 quadros, 1 624 204 escritas FM, 8471 escritas PSG):
    - FM: 6 400 183 amostras, 1563 de 1563 blocos iguais -> **IGUAL**.
    - PSG: 26 880 772 amostras, 6563 de 6563 blocos iguais -> **IGUAL**.
  - `demos-quatro` (11700 quadros, 2 325 744 escritas FM, 12 709 escritas PSG):
    - FM: 10 400 362 amostras, 2540 de 2540 blocos iguais -> **IGUAL**.
    - PSG: 43 681 522 amostras, 10665 de 10665 blocos iguais -> **IGUAL**.
  - `lz-agua` (2000 quadros, 368 056 escritas FM, 2435 escritas PSG):
    - FM: 1 777 755 amostras, 435 de 435 blocos iguais -> **IGUAL**.
    - PSG: 7 466 572 amostras, 1823 de 1823 blocos iguais -> **IGUAL**.
  - `partida-ghz1` (8309 quadros, 2 449 778 escritas FM, 12 629 escritas PSG):
    - FM: 7 386 005 amostras, 1804 de 1804 blocos iguais -> **IGUAL**.
    - PSG: 31 021 224 amostras, 7574 de 7574 blocos iguais -> **IGUAL**.
- **Totais:** 25 964 305 amostras de FM e 109 050 090 amostras de PSG comparadas amostra a amostra sem nenhuma divergência (0 blocos diferentes em 32 967 blocos comparados).

## Saída web, reamostragem e filtro analógico (8.6 a 8.8)

- **Crate `sonic1-audio-saida`:**
  - `Reamostrador`: sinc com 256 fases e 32 taps, sem alocações dinâmicas repetidas por amostra.
  - `FiltroAnalogico`: simulação passa-baixa RC bipolar (~3386 Hz, Ganho unitário), marcado `APROXIMADO`.
  - `MisturadorSaida`: recebe quadros de FM e PSG na frequência nativa, executa reamostragem, controle de deriva proporcional (±0,5%), mistura, filtragem analógica e expõe buffer circular.
- **Testes unitários (`crates/audio-saida/tests/saida.rs`):**
  - Resposta em frequência do filtro passa-baixas confere atenuação < 0,2 dB a 100 Hz e ~ -3 dB a 3386 Hz.
  - Continuidade de seno na junção de quadros preservada sem descontinuidade.
  - Simulação de 600 quadros com jitter severo (3 ms) sem nenhuma perda de amostra ou corte.

## Medição de desempenho (8.9)

- `crates/audio-saida/tests/desempenho.rs` com flag `WEBSONIC_BENCH=1`.
- Mediana em 600 quadros sintéticos densos de estresse máximo: ~1840 µs (misturador e reamostragem gastam menos de 200 µs; síntese gasta ~1,6 ms).
- Dentro do orçamento estipulado de 3000 µs (3 ms).

## Estado das provas (atualizado ao fim do Bloco da IA)

| Item | Estado |
| --- | --- |
| 8.0 licenças, pins, ambiente | feito (bloco 1 do Marcelo) |
| 8.1 harness diferencial (`diferencial-sintese/`) | feito (FFI Nuked-OPN2 e Nuked-PSG, comparador em blocos de 4096) |
| 8.2 PSG | **igual ao Nuked-PSG** amostra a amostra |
| 8.3 FM YM2612 | **igual ao Nuked-OPN2** no modo YM2612 amostra a amostra |
| 8.4 Sintetizador e agendador em mclk | **feito** (invariância à fatiação e agendamento nominal) |
| 8.5 Harness diferencial de síntese | **feito** (bateria sintética e corpus com a ROM real **100% IGUAL**, hash `55a11276...`) |
| 8.6 Reamostrador polifásico sinc | **feito** (256 fases, 32 taps, controle dinâmico ±0,5%) |
| 8.7 Áudio web | **feito** (`site/audio-worklet.js`, integração em `crates/console` e `crates/web`) |
| 8.8 Filtro analógico passa-baixas | **feito** (APROXIMADO, fora da prova digital) |
| 8.9 Medição de desempenho | **feito** (mediana ~1840 µs < 3000 µs) |
| 8.10 Fechamento da etapa 8 | **concluída** (documentação, patches e relatório com veredito IGUAL) |

## Otimizações de síntese e medidas em WebAssembly (17/11/2026)

Durante a remediação de desempenho, o sintetizador e o misturador receberam otimizações mantendo exatidão total de bits:
- O avanço de FM e PSG salta ciclos sem escritas em laço interno (`crates/audio/src/sintetizador.rs`).
- `avancar_ate_compacto` entrega fatias `(i16, i16)` e `u16`, eliminando a cópia de 104 bytes por amostra FM.
- `processar_mono` no reamostrador sinc executa uma única convolução para o sinal mono do PSG.
- Desempenho nativo de síntese e saída (`WEBSONIC_BENCH=1 cargo test -p sonic1-audio-saida --release --test desempenho`): mediana de 2539 us (orçamento de 3000 us).
- Medição completa em Wasm (`partida-ghz1`, via `node --test site/tests/desempenho-wasm.test.mjs`): média de 6,70 ms por quadro com vídeo e áudio juntos.
- Fidelidade preservada: hash de 7.200 quadros `b8d4a450a5baf18634d191411896fd5caad98d6e` em `console_hashes` e 17.509 quadros aprovados em `guarda_rom`.
