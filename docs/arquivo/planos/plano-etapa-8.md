# Plano da Etapa 8: Sintetizador

Data: 09/11/2026. Modo de trabalho: a IA decide sozinha, registra decisões em `docs/conhecimento/sintese.md` e só para para pedir comandos ao Marcelo. A aprovação deste plano vem junto com os resultados do bloco 1.

## Pré-condições conferidas

- `CLAUDE.md`, `docs/requisitos.md` (inteiro), `docs/estado.md`, `docs/backlog.md`, índice da base e `oraculo.md`, `oraculos.md`, `som.md` lidos.
- Nenhum conflito com o requisito. A Etapa 6 está concluída no `estado.md` (78 IGUAL, 0 DIVERGE). A Etapa 7 também.
- Critério ("Pronto quando"), sem reescrever: FM, incluindo o DAC, idêntico ao Nuked-OPN2 no modo YM2612, e PSG idêntico ao Nuked-PSG, amostra a amostra, na taxa de cada chip, com as escritas do oráculo nos mesmos instantes. A parte analógica é medida e marcada como aproximada. A música toca no navegador, reamostrada sem cortes.
- Critério 6 do "Critério de pronto" (requisito) é o mesmo: som idêntico, amostra a amostra, aos núcleos de referência, alimentados com as mesmas escritas nos mesmos instantes.

## Decisões já tomadas (D8.x)

- **D8.1, licenças (verificadas em 09/11/2026 nos arquivos públicos `LICENSE` e nos cabeçalhos):** Nuked-OPN2 é LGPL 2.1 ou superior (cabeçalho de `ym3438.h`, versão 1.0.9). Nuked-PSG é GPL 2 ou superior (cabeçalho de `ympsg.h`, versão 1.0.1, "Yamaha YM7101 PSG"). Ambas permitem uso local como oráculo. As duas ficam num workspace GPLv3 novo (`diferencial-sintese/`, como `diferencial-som/`), compatível com GPLv2+ e com a LGPL. Nunca entram no `core`, no `crates/audio` nem no site. O fonte dos chips de referência não vai para o git: fica clonado em `$SONIC1_EXT_DIR/trabalho/nuked` em commits fixados (OPN2 `335747d78cb0abbc3b55b004e62dad9763140115`, PSG `d15a168c676f4669e23660be9225b34ad7c1764e`, a conferir no bloco 1).
- **D8.2, origem do código dos chips:** o `crates/audio` é escrito a partir de documentação pública do chip (lista em "Fontes" abaixo), cita a fonte em cada tabela e função, e nunca copia código nem comentários do Nuked. O Nuked é usado como referência de comparação e como instrumento: a IA pode ler a interface pública (`ym3438.h`, `ympsg.h`) e o estado interno que o comparador expõe, para desenhar o harness e diagnosticar divergências por experimento (vetor sintético, camada que diverge). Quando a diferença só se explica pela estrutura interna do Nuked, a IA descreve o comportamento com palavras próprias em `sintese.md` e reescreve a lógica, sem transcrever.
- **D8.3, oráculo principal e chips de referência:** o jgenesis é o oráculo principal das **escritas** (conteúdo, ordem, instante em mclk). Os Nuked são a referência do **chip**. O critério 6 é definido contra os Nuked. Regra de desempate na seção 3 abaixo.
- **D8.4, onde fica cada coisa:** `crates/audio` (sintetizador fiel, lints do core: sem float, estouro explícito, sem HashMap, sem cast que trunca, sem `#[allow]`), `crates/audio-saida` (reamostrador, parte analógica aproximada, buffer; pode usar `f32` porque não entra na prova amostra a amostra), `diferencial-sintese/` (workspace de oráculos), `site/audio-worklet.js` (mínimo e comentado).
- **D8.5, unidade de tempo:** mclk do oráculo (relógio mestre NTSC, 53 693 175 Hz). Um ciclo interno do OPN2 do Nuked são 6 ciclos do FM, ou seja 42 mclk (FM = mestre/7, conforme o README do Nuked-OPN2: "um clock interno são seis clocks do mestre" a conferir contra a documentação do YM2612 no 8.0). Uma amostra do FM são 24 ciclos internos (1008 mclk, cerca de 53 267 Hz). O PSG é mestre/15 e sai uma amostra a cada 16 ciclos do PSG (240 mclk, cerca de 223 722 Hz); a unidade exata do `YMPSG_Clock` é confirmada no 8.1. Tudo isto é **hipótese a provar** nas sub-etapas 8.0 e 8.1, com comando e resultado.
- **D8.6, regra de instante de uma escrita:** uma escrita com mclk `t` é aplicada depois de o chip ter avançado até o ciclo `floor(t / 42)` (FM) ou o equivalente do PSG, contado desde o mclk 0 do trace. O mesmo agendador alimenta o Nuked e o nosso chip. Como o oráculo dá o mclk do **início da instrução** (limite V5 em `oraculo.md`), o instante é o do oráculo, não o do barramento real; isso é registrado, não corrigido (nível 2, Etapa 15).

## 1. Sub-etapas

Quem roda: **IA** = no contêiner/host da sessão, só com dados sintéticos. **Marcelo** = bloco de comandos, fora da IA.

| # | Entrega | Prova (comando, resultado esperado) | Quem |
| --- | --- | --- | --- |
| 8.0 | Licenças e fontes: `docs/conhecimento/sintese.md` com licença, commit e hash do `LICENSE` de cada referência, lista de fontes públicas, ambiente (compilador C, alvos wasm). `scripts/etapa8-bloco1.sh` | `sh scripts/etapa8-bloco1.sh` imprime commits, SHA-1 dos arquivos, versão do `cc`, alvos de Rust. Esperado: commits iguais aos fixados, `cc` presente | Marcelo (bloco 1) |
| 8.1 | Cronograma de escritas e harness: `diferencial-sintese/` com os dois Nuked compilados por `build.rs` (crate `cc`), leitor das escritas de YM e PSG do trace (eventos 6 e 7, com mclk) e dos tempos do som, agendador (D8.6), geração de amostras, hash por bloco, relatório. Formato das amostras na seção 2 | `cargo test` do workspace (IA, sintético): Nuked contra Nuked dá IGUAL; planta de divergência (um bit trocado na amostra 5000) dá DIVERGE no índice 5000 e no bloco certo; o agendador respeita a regra D8.6; determinismo (duas execuções, mesmo hash). Confirma taxa e unidade do relógio de cada chip lendo as amostras geradas por vetores simples | IA |
| 8.2 | Núcleo do PSG (`crates/audio/src/psg`): 3 tons, ruído (registrador de deslocamento, modos), atenuação de 2 dB por passo, latch/dado, reset, divisor de relógio, saída digital por canal e mistura em inteiro | Testes sintéticos: determinismo, nunca panica em escritas aleatórias, saída limitada, vetores publicados (tom com período N, atenuação 0 a 15, 15 = mudo, ruído com semente e retroalimentação documentados, ver Fontes), modelo independente simples (contadores) cruzado em 1000 sequências aleatórias | IA |
| 8.3a | FM, parte 1: registradores e canais (6 canais, 4 operadores, modo do canal 3, DAC e seleção do canal 6), gerador de fase (F-number, bloco, código de tecla, detune, multiplicador), LFO (frequência, PM, AM) | Testes: tabela de detune e de incremento de fase contra os valores do manual; fase de uma nota conhecida após N amostras (cálculo à mão no teste); LFO com períodos documentados | IA |
| 8.3b | FM, parte 2: gerador de envelope (ataque, decaimento 1 e 2, sustain, release, taxas, escala de taxa por tecla, TL, nível de sustain, SSG-EG, contador do envelope a cada 3 amostras) | Testes: tempos de ataque e decaimento para taxas dadas (tabela do manual), SSG-EG nos 8 modos (formas de onda do manual), saturação do nível em 10 bits | IA |
| 8.3c | FM, parte 3: operador (tabela log-seno e exponencial gerada por fórmula inteira, citada), modulação de fase, retroalimentação, 8 algoritmos, saída do canal | Testes: seno puro com um operador, algoritmos com operadores de ganho conhecido, retroalimentação zero igual a sem retroalimentação, modelo independente de um operador | IA |
| 8.3d | FM, parte 4: DAC (canal 6, registrador `$2A`, `$2B`), saída de 9 bits do modo YM2612 (truncamento, "efeito de escada" do modelo 1 se estiver documentado), panning L/R, soma dos 6 canais com saturação | Testes: DAC com valores extremos, canal 6 com DAC ligado ignora o FM, panning, saturação | IA |
| 8.3e | FM, parte 5: interface do barramento (portas 0 a 3, endereço/dado, busy, status, timers A e B, CSM, reset) e agendador por ciclo interno | Testes: busy por 32 ciclos internos após escrita (a confirmar no 8.0), timers com período do manual, CSM, ordem das escritas | IA |
| 8.4 | Mistura e agendador no núcleo do som: `Sintetizador` recebe `(mclk, chip, porta, valor)` e devolve amostras até um mclk, nas duas taxas, sem float | Testes: o mesmo cronograma em fatias diferentes de tempo dá as mesmas amostras (invariância à fatiação); contagem de amostras por quadro de 896 040 mclk (888 ou 889 de FM, 3733 ou 3734 de PSG) | IA |
| 8.5 | Comparação com os Nuked, em três camadas: (a) **bateria sintética** (vetores por registrador, canal, operador, algoritmo, SSG-EG, LFO, DAC, PSG, escritas rápidas e em instantes variados); (b) **escritas do corpus** (`boot-demos`, `demos-quatro`, `lz-agua`, `partida-ghz1`); (c) **música e efeitos** (os 78 IDs da Etapa 6) se o harness do som entregar o mclk de cada escrita (a conferir no 8.1; se não, só (a) e (b) e o resto vira BACKLOG) | `sh scripts/etapa8-bloco2.sh ROM`. Esperado: veredito IGUAL em todos, FM e PSG separados. Em caso de DIVERGE, o relatório indica a camada que difere (fase, envelope, operador, canal, mistura) | Marcelo (bloco 2 e as rodadas de correção) |
| 8.6 | Reamostragem (seção 4) | Testes sintéticos de contagem e de continuidade; simulação de 600 quadros com jitter de 0 a 3 ms | IA |
| 8.7 | Saída no navegador: AudioWorklet, fila de blocos, integração com `crates/web` e `crates/console` (escritas de som com mclk dentro do quadro, vindas do núcleo no tempo nominal) | `cargo test` nativo e wasm (na imagem), site montado; escuta e contadores de falta de dados no bloco do navegador | IA e Marcelo (bloco 3) |
| 8.8 | Parte analógica aproximada (seção 5) | Teste de resposta em frequência com senoides sintéticas; relatório com a marca APROXIMADO | IA |
| 8.9 | Desempenho (seção 6) | `WEBSONIC_BENCH=1` no nativo e no wasm | Marcelo (bloco 3) |
| 8.10 | Fechamento: `estado.md`, `sintese.md`, índice, backlog, patch do relatório e do manifesto, resumo para o chat de planejamento, revisão só do que a Etapa 8 mudou | `cargo fmt`, `clippy -D warnings`, testes nativos e wasm; checagem anti-ROM | IA, Marcelo (patch) |

Cada sub-etapa termina com commit local, sem parar. Os blocos de comandos do Marcelo ficam em `scripts/etapa8-blocoN.sh`. O bloco 1 vem agora. O bloco 2 vem depois de 8.1 a 8.5 prontas no lado da IA. O bloco 3 reúne navegador e desempenho. Se uma rodada de correção for necessária, vira o bloco 4, e cada rodada entrega todas as correções possíveis juntas.

## 2. Como o relatório mede "idêntico"

- **Amostra do FM:** par (esquerda, direita) de inteiros com sinal de 9 bits, como saem do Nuked (`mol`, `mor`) em cada amostra de 24 ciclos internos. Serializado como 2 × `i16` little-endian. O DAC é o mesmo caminho (entra na soma do canal 6).
- **Amostra do PSG:** a saída digital do chip: o nível de saída (atenuação 0 a 15, ou mudo) e o sinal de cada um dos 4 canais. A conversão para mistura é determinística e vive do nosso lado. O formato exato é fixado no 8.1 depois de ver o que o `YMPSG_GetOutput` e o estado interno expõem (ele devolve `float`, que não serve para comparar bit a bit). Serializado como bytes.
- **Bloco:** 4096 amostras de um chip. Hash SHA-1 por bloco do conjunto de bytes serializado (os dois lados, referência e núcleo).
- **Como o oráculo entrega as amostras ao comparador:** as amostras **não passam por arquivo**. O harness lê as escritas (mclk, porta, valor) do trace, roda as duas implementações em memória no mesmo processo, calcula o SHA-1 de cada bloco em memória e compara. Só hashes, contagens e vereditos saem.
- **Relatório por caso e por chip:** nº de amostras, nº de blocos, hash do bloco de referência e do núcleo (quando diferem, os dois), **primeiro índice divergente** (amostra, bloco, mclk aproximado, escrita mais recente), camada que difere, veredito `IGUAL` ou `DIVERGE`. Hash final do relatório. Nenhuma amostra, nenhum valor de registrador de dado da ROM no relatório (só contagens e índices).
- **Veredito da etapa:** `IGUAL` em todos os casos de FM e de PSG. A parte analógica aparece em seção à parte, marcada APROXIMADO, e não entra no veredito.

## 3. Regra de desempate

- Fonte: requisito, "Oráculos": quando os oráculos discordam, decidem a documentação do hardware e os testes publicados. Sem evidência, vale o oráculo principal, e a diferença fica registrada.
- **Escritas (conteúdo, ordem, mclk):** vale o jgenesis (principal). O segundo oráculo (Genesis Plus GX) só compara conteúdo e ordem em um nível já registrado (D1a a D1c).
- **Amostras do chip:** o critério 6 é contra os Nuked. Se o núcleo difere do Nuked: o núcleo está errado e se corrige no jogo, nunca no teste. A exceção só existe com evidência: se documentação pública ou teste publicado mostrar que o Nuked difere do hardware, a diferença é registrada em `sintese.md` com a decisão, e o critério 6 continua contra o Nuked (é o que o requisito diz). Nesse caso o relatório mostra a divergência como `DIFERENÇA REGISTRADA`, nunca como IGUAL.
- **Sem comparação de amostra com o jgenesis:** ele reamostra o próprio caminho. Se o Marcelo quiser comparar, vira informação, não veredito.
- A divergência é registrada numa linha de tabela em `sintese.md` com id (`S1`, `S2`...), caso, chip, camada, causa, decisão e evidência.

## 4. Reamostragem para o navegador

- **Método:** reamostrador em fase com sinc janelada (polifásico), implementado em `crates/audio-saida` com `f32`. FM a ~53 267 Hz e PSG a ~223 722 Hz são reamostrados separadamente (cada um do seu jeito para a mesma taxa) e somados. O polifásico guarda 256 fases e 32 taps (custo medido no 8.9). Razão de reamostragem com **ajuste fino**: a entrada é o relógio de 60 Hz exato do jogo, a saída é a taxa do `AudioContext`; um controlador proporcional de baixa banda ajusta a razão em até ±0,5% conforme a ocupação do buffer, para acompanhar os 60 Hz sem cortes nem deriva (decisão do requisito, "Taxa de quadros").
- **Sem cortes, prova:** o teste registra por quadro: amostras de FM de entrada (888 ou 889), de PSG (3733 ou 3734), amostras de saída e ocupação do buffer. Verifica: (1) a soma das entradas em N quadros é igual a `floor(N × 896 040 / 1008)` (±0 pelo acumulador fracionário); (2) a saída por quadro fica em ±1 amostra da taxa nominal (800 a 48 kHz, 735 a 44,1 kHz); (3) a ocupação do buffer nunca chega a zero em 36 000 quadros (10 minutos) com jitter de entrega sintético de até 3 ms; (4) a descontinuidade (diferença entre amostras vizinhas) na junção entre quadros não passa da de uma senoide pura.
- **Entrega ao navegador:** `AudioWorkletProcessor` (arquivo `site/audio-worklet.js`, cerca de 40 linhas comentadas) com uma fila circular. O thread principal recebe do WebAssembly os blocos já reamostrados (`Float32Array`) e envia por `port.postMessage` com transferência. Sem `SharedArrayBuffer` (o GitHub Pages não permite cabeçalhos COOP e COEP). O worklet preenche as 128 amostras de cada chamada da fila, e conta quantas vezes faltou dado (`falhas`). Se faltar, repete a última amostra com decaimento curto e incrementa o contador, que a página mostra. O áudio só inicia depois de um gesto do usuário (política do navegador). Meta antes de ligar a saída: 3 quadros de buffer (50 ms).
- **JavaScript:** só carregar o worklet, repassar blocos e ligar o `AudioContext`. A lógica toda está em Rust.

## 5. Parte analógica

- **O que é medido:** a cadeia depois do DAC do modelo 1 (filtro passa-baixa e amplificador de saída) como **aproximação**: um filtro passa-baixa de primeira ordem mais um ganho, em `f32`, com corte e ganho configuráveis. Valores padrão só entram se houver fonte pública citável (8.0 procura; se não houver, o padrão é "sem filtro" e o relatório diz "SEM FONTE"). A medição é do **nosso** filtro: resposta a senoides sintéticas de 100 Hz a 20 kHz, ganho em dB em 8 frequências.
- **Como fica marcada:** seção `parte analógica (APROXIMADO, não verificada)` no relatório, com o ganho medido e a fonte (ou "sem fonte"). A palavra APROXIMADO aparece na seção e na linha do veredito geral.
- **Fora da prova:** a prova amostra a amostra compara a saída digital do chip (9 bits do FM, níveis do PSG), antes da reamostragem e antes de qualquer filtro. O filtro vive em `audio-saida`, que o harness não importa.

## 6. Desempenho

- **Orçamento:** 16,67 ms por quadro de 60 Hz no total. O jogo já gasta até cerca de 1 ms no renderizador no nativo (backlog da Etapa 4) e o resto do quadro. Orçamento de áudio: **3 ms por quadro em wasm** (FM por ciclo interno, 21 312 ciclos por quadro, mais PSG, mais reamostragem e mistura).
- **Como medir:** `WEBSONIC_BENCH=1` (mesmo mecanismo do renderizador) gera 600 quadros de escritas sintéticas densas (6 canais, 4 operadores, SSG-EG, LFO, DAC, PSG com 3 tons e ruído) e imprime mediana, p95 e máximo por quadro em µs, no nativo e em wasm (`wasmtime`, executado pelo Marcelo no bloco 3). Se passar de 3 ms, o plano B do requisito é um modo mais leve (fora desta etapa, registrado em backlog).

## 7. Estratégia de teste sem ROM

- **Vetores sintéticos por chip:** listas de escritas montadas à mão ou geradas por semente fixa, cobrindo registradores, envelopes, canais, algoritmos, SSG-EG, LFO, DAC, PSG. Nenhum byte vem da ROM; valores são inventados e documentados.
- **Propriedades:** determinismo (duas execuções, mesmo hash), nunca panica (escritas aleatórias em todos os endereços e portas, tempos aleatórios), saída limitada (FM em 9 bits, PSG em 0..15 por canal), invariância à fatiação do tempo, reset restaura o estado inicial.
- **Modelo independente simples:** para o PSG, um modelo de contadores de poucas linhas. Para o FM, um modelo de um operador (fase, envelope linear por taxa, seno por fórmula) no nível de precisão do manual, para cruzar nos casos fáceis. O modelo vive em `crates/teste-audio` e não importa `crates/audio`.
- **Casos de referência publicados (cite a fonte no teste):** tabelas do manual do YM2608 (famílias OPN) para detune, atenuação por passo e taxas; documentação do SN76489 para atenuação e ruído (ver Fontes).
- **Cobertura da bateria sintética contra o Nuked** (8.5a) é a prova forte. Os testes da IA são a rede de segurança.

## 8. Itens que mexem em arquivo congelado

- **Dockerfile:** nenhum. O Nuked é compilado no host (workspace de oráculos), não na imagem. O CI não roda o `diferencial-sintese/`. Se um dia precisar de `cc` na imagem, vira pedido.
- **`.devcontainer/dominios.txt`:** nenhum. `github.com` já é permitido (leitura).
- **`.github/` (CI):** nenhum item novo se os novos crates entrarem no `workspace` e o CI já roda `cargo test` do workspace. Se o CI listar crates explicitamente, o patch sai pronto na 8.10.
- **Manifesto e relatório (`relatorio` é congelado):** patch pronto na 8.10 para incluir `diferencial-sintese/` e o relatório da síntese no manifesto, no padrão de `docs/pedidos/etapa7-relatorio.patch`. Comando de aplicar no último bloco: `git apply docs/pedidos/etapa8-relatorio.patch`.
- `oracle/`, `oracle2/`, `diferencial/`, `crates/fidelidade/`: não editados.

## 9. Riscos e o que NÃO será feito

- **Risco principal:** a identidade amostra a amostra com o Nuked exige reproduzir detalhes de ciclo (latências de escrita, ordem dos 24 slots, arredondamentos, contador do envelope). Escrever sem copiar o código pode exigir várias rodadas. Mitigação: comparação em camadas (fase, envelope, operador, canal, mistura) para localizar a divergência, bateria sintética grande por rodada, e correções agrupadas por bloco. Se esse custo explodir, o plano B é registrado em backlog e a decisão é do requisito, não minha.
- **Risco 2:** o instante da escrita tem a imprecisão do mclk do oráculo (início da instrução). O harness usa o mesmo instante nos dois lados, então não afeta a identidade; afeta só a fidelidade ao hardware real (nível 2).
- **Risco 3:** o desempenho em wasm do FM por ciclo pode passar do orçamento. Medido no 8.9.
- **Risco 4:** o `YMPSG` expõe o float; a forma digital escolhida pode não bastar. Resolve no 8.1.
- **NÃO será feito nesta etapa:** música de cada zona (Etapas 9 a 12); sincronia fina de áudio com o vídeo (Etapas 9 e 13); a parte analógica como verificada (nunca, por definição do requisito); modo realçado com YM3438 ou outro chip; comparação de amostras com o jgenesis como veredito; tempo previsto das escritas (Etapa 15); salvar a ROM no navegador.

## Fontes públicas a usar (cada afirmação do núcleo cita a sua)

- Yamaha YM2608 Application Manual (operadores, envelope, detune, LFO, SSG-EG, registradores; a família OPN tem o mesmo modelo).
- Pesquisa de Nemesis sobre o YM2612 (fórum SpritesMind/Sega Retro "YM2612"), para os desvios do YM2612 em relação à família (DAC, saída de 9 bits, canal 6).
- Documentação do Mega Drive: Sega Genesis Software Manual (mapa do YM2612 e do PSG), Sega Retro (páginas do YM2612 e do SN76489/PSG), SMS Power (SN76489, Maxim).
- README e cabeçalho público do Nuked-OPN2 e do Nuked-PSG (interface, modo YM2612, licença).
- Clownacy, "The sound drivers of Sonic the Hedgehog" (já nas Fontes do requisito), para o que o Sonic 1 de fato usa.

Cada uso é conferido no 8.0 e registrado em `sintese.md` com data. Afirmação sobre taxa de relógio, DAC, YM2612 e PSG só entra como "provada" com comando e resultado.
