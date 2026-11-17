# Prompt de execução: performance do WebSonic

> **Como usar:** cole este documento inteiro num chat novo, aberto na raiz de `/ssd400/Repos/websonic`. Ele é o plano aprovado e a ordem de execução. O executor trabalha sem perguntas, do primeiro ao último item.

Auditoria feita em 17/11/2026 (sessão `fd87fef9`). Linha de base: HEAD `f8a9a18f3669565c014f648ecac249f9bb7018f5`, mais 10 arquivos modificados e 4 novos da sessão anterior. SHA-1 de `git diff` na auditoria: `b5292da6bfebd8bc356c9725285852e07a7160ad`.

## 0. Ordem de execução

### 0.1 Papel e missão

- Você é o executor. Implemente todos os itens da matriz da seção 4, na ordem da coluna "Ordem".
- Não faça perguntas. Não peça aprovação. Não pare entre itens.
- Este documento é a aprovação prévia do Marcelo para o plano completo, como o `CLAUDE.md` exige.
- Pare só quando o critério de término da seção 6.1 for cumprido.
- Todas as decisões de produto já foram tomadas pelo Marcelo. Elas estão na seção 0.9.

### 0.2 Leituras iniciais (nesta ordem)

1. `docs/requisitos.md` inteiro.
2. `docs/estado.md`.
3. `docs/conhecimento/README.md`, `docs/conhecimento/video.md` e `docs/conhecimento/sintese.md`.
4. Este documento inteiro, antes do primeiro comando.

### 0.3 ai-memory (continuidade entre sessões)

- Início: se o contexto já trouxer o handoff `01a1e935-30eb-7e63-9b85-6f2903c849c9`, use esse bloco. Não chame `memory_handoff_accept` de novo.
- Sem o bloco no contexto: chame `memory_handoff_list` e aceite o ID `01a1e935-30eb-7e63-9b85-6f2903c849c9` com `memory_handoff_accept`.
- Durante a execução: não escreva notas manuais. Os hooks já registram a sessão.
- Retomada após corte (cota, falha ou terminal fechado): leia as observações da sessão interrompida com `memory_read_session_observations` e `order: "desc"`. Retome do primeiro item `[ ]` da seção 4.
- Fim: chame `memory_handoff_begin` com um resumo de 2 a 3 frases, as pendências da seção 6.3 e os arquivos tocados.
- Harness sem ai-memory: siga sem ele. Os checkboxes deste documento guardam o estado.

### 0.4 Autorização da ROM (Marcelo, 17/11/2026)

- A decisão L1 está inativa. O Marcelo liberou a leitura e a execução da ROM para este trabalho.
- Caminho: `sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen`. O `.gitignore` cobre a pasta (linha 57) e a extensão `*.gen`.
- Exporte a variável antes de qualquer teste com ROM: `export SONIC1_ROM="$PWD/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen"`.
- Proibido versionar a ROM ou qualquer dado derivado dela: bytes, imagens, sons, traces e estados.
- Proibido ler ou escrever em `sonic1-ext/ocultos/` e `sonic1-ext/privado/`.
- Proibido rodar ou editar o oráculo (`oracle/`, `oracle2/`), o comparador, o manifesto e a assinatura.
- Proibido rodar scripts que chamam o oráculo: `scripts/etapa2-*.sh`, `scripts/etapa4-host.sh`, `scripts/etapa5-*.sh`, `scripts/etapa7-bloco1.sh`, `scripts/etapa8-bloco2.sh` e `scripts/sut-reproduz.sh`.
- Proibido rodar `scripts/etapa7-bloco5.sh` e `scripts/etapa7-bloco5b.sh`, porque eles gravam em `privado/`.

### 0.5 Regras do repositório (valem mesmo sem ler o `CLAUDE.md`)

- Escreva em português, com frases curtas.
- Nunca use travessão nem meia-risca em nenhum arquivo. Use vírgulas, pontos ou parênteses.
- Não faça push. Nunca use `--no-verify`. Não desligue hooks do git.
- Não edite itens congelados: `docs/requisitos.md`, `CLAUDE.md`, `.gitignore`, `hooks/`, `refs/`, `.github/`, `.devcontainer/`, `.claude/`, `.cargo/`, `.vscode/`, `oracle/`, `oracle2/`, `diferencial/`, `cobertura/`, `crates/corpus/`, `crates/fidelidade/` e `rust-toolchain.toml`.
- Preserve `CLAUDE.md`, `.claude/skills/` e qualquer `CLAUDE.md.bak-*`. Nunca os inclua num commit.
- Não toque em arquivos não rastreados de outras sessões, como `docs/handoff-aprendizados-gta-longshot.md` e `docs/relatorio-aprendizados-gta-v-websonic.md`. Nunca os inclua num commit.
- No crate `core`: sem ponto flutuante, estouro tratado com `wrapping_*`, `checked_*` ou `saturating_*`, sem `HashMap` ou `HashSet` e sem cast que trunca.
- Não emule nem interprete instruções do 68000 ou do Z80.
- Teste que falha se corrige no jogo, nunca no teste. A única exceção é o item V1b, que apaga `crates/video/tests/gpu.rs` junto com o código que ele testa.
- Não mude a assinatura de funções públicas usadas por testes: `Ponte::nova`, `Ponte::aplicar`, `Ponte::capturar`, `Memoria::tirar_saidas`, `Console::extrair_audio`, `Console::executar_quadro` e `Jogo::quadro`. Crie variantes novas.
- Comente em Rust as escolhas que não são óbvias. Mantenha o JavaScript mínimo e comentado.

### 0.6 Protocolo de commits e detector

1. Item E0.1: compile o detector com a receita abaixo.
2. Item E0.2: faça o commit de linha de base.
3. Faça um commit local por item da matriz. Marque o checkbox do item neste documento no mesmo commit.
4. Adicione arquivos sempre por lista explícita. Nunca use `git add -A` nem `git add .`.
5. Antes de cada commit, rode o detector sobre o índice. Saída 1 bloqueia o commit até a correção.
6. Escreva a mensagem no formato `perf(<area>): <ID> <resumo>`.

Receita do detector, validada em 17/11/2026 (controle positivo saiu com 1, controle negativo saiu com 0):

```sh
DET=$(mktemp -d)
git -C /ssd400/Repos/websonic archive HEAD Cargo.toml crates/rom | tar -x -C "$DET"
sed -i 's|^members = .*|members = ["crates/rom"]|' "$DET/Cargo.toml"
(cd "$DET" && git apply /ssd400/Repos/websonic/docs/pedidos/l1-hook-detector.patch)
cargo build --release --offline --quiet --manifest-path "$DET/hooks/detector/Cargo.toml" --target-dir "$DET/alvo"
# Antes de cada commit, na raiz do repositorio (analisa o que esta no indice):
"$DET/alvo/release/detector-rom" --rom "$SONIC1_ROM"
```

Controles do item E0.1 (o arquivo positivo fica fora do repositório e é apagado em seguida):

```sh
dd if="$SONIC1_ROM" of="$DET/positivo.bin" bs=1 skip=1000 count=64
"$DET/alvo/release/detector-rom" --rom "$SONIC1_ROM" --arquivo "$DET/positivo.bin"; echo "saida=$?"   # esperado: 1
printf 'texto comum\n' > "$DET/negativo.txt"
"$DET/alvo/release/detector-rom" --rom "$SONIC1_ROM" --arquivo "$DET/negativo.txt"; echo "saida=$?"  # esperado: 0
rm -f "$DET/positivo.bin"
```

### 0.7 Protocolo de falha

- Se um item quebrar um teste ou uma guarda, procure a correção no jogo. Faça no máximo 3 tentativas.
- Sem correção depois de 3 tentativas, reverta só os arquivos do item: `git restore --source=HEAD --staged --worktree <arquivos do item>`.
- Marque o item como `BLOQUEADO` na seção 4, com o motivo em uma frase. Siga para o próximo item.
- Hash de guarda diferente indica mudança de comportamento. Nunca troque o valor esperado no teste. Reverta o item.

### 0.8 Subagentes

- Use subagentes só para leitura e para bancadas independentes.
- Faça todas as edições na sessão principal, uma de cada vez.

### 0.9 Decisões do Marcelo (17/11/2026)

1. A ROM está liberada para leitura e execução, sem versionamento (seção 0.4).
2. O modo 16:9 sai do site, junto com o caminho de candidatos para a GPU. O código volta pelo histórico do git quando a fase futura 2 começar.
3. O executor faz primeiro o commit de linha de base e depois um commit por item.
4. A lógica no navegador segue o requisito: 60 Hz exatos (`docs/requisitos.md:142` e `:325`).
5. O REA não entra nesta rodada. O gargalo está no código do projeto, que tem fonte.

## 1. Nota geral e sumário executivo

**Nota geral: F.**

O WebSonic não cumpre o orçamento de 16,67 ms por quadro na máquina de referência. No Chrome real, a simulação roda a 52,4 quadros/s, a tela mostra cerca de 24 fps e o áudio falha 563 vezes em 72 s. O relatório anterior deu 8,1/10 e declarou "60+ FPS estável". As medidas abaixo desmentem isso.

### 1.1 Condições de medição

| Item | Valor |
| --- | --- |
| CPU | Intel Core i5-1334U, 12 threads, turbo desligado (`no_turbo=1`), máximo de 1.300 MHz |
| GPU | Intel Iris Xe (RPL-U), Mesa, ANGLE com OpenGL ES 3.2 |
| Navegador | Chrome 157 |
| Node | 26.13.1 (V8, o mesmo motor do Chrome) |
| Rust no host | rustc 1.100.0 do Arch. O host não tem rustup, então o `rust-toolchain.toml` (1.99.1) não vale aqui |
| wasm-bindgen | CLI 0.2.128, crate fixado em `=0.2.129`. O build funciona |
| Carga do sistema | Média de 1 minuto entre 5 e 21, causada por abas do Chrome e pelo VS Code |
| Variação entre rodadas | Até 30%. As comparações usam rodadas seguidas e as porcentagens do perfil |

Origem de cada número: `[MEDIDO ROM Chrome]` (site real com a ROM), `[MEDIDO ROM V8]` (Wasm no node), `[MEDIDO ROM nativo]`, `[MEDIDO sintético]`, `[CALCULADO]` (fórmula a partir do código) e `[ESTIMADO: premissa]`.

### 1.2 Métricas atuais contra metas

| Métrica | Atual | Origem | Meta |
| --- | --- | --- | --- |
| Quadros simulados por segundo no Chrome | 52,4 | MEDIDO ROM Chrome | ≥ 59,5 |
| Intervalo entre quadros apresentados (média, p95) | 42,0 ms, 109,0 ms | MEDIDO ROM Chrome | ≤ 17,5 ms, ≤ 20,0 ms |
| Intervalos acima de 33 ms | 776 de 1.711 (45,4%) | MEDIDO ROM Chrome | ≤ 1% |
| Faltas do worklet | 563 em 72 s | MEDIDO ROM Chrome | 0 em 120 s |
| Long tasks | 693 em 72 s, máximo de 212 ms | MEDIDO ROM Chrome | 0 |
| Quadro em Wasm, `partida-ghz1` (média, p95) | 15,46 ms, 23,50 ms | MEDIDO ROM V8 | ≤ 8,0 ms, ≤ 13,0 ms |
| Quadros acima de 16,67 ms, `partida-ghz1` | 27,2% | MEDIDO ROM V8 | ≤ 2% |
| Quadros acima de 16,67 ms, `boot-demos` | 54,4% | MEDIDO ROM V8 | ≤ 2% |
| Alocações por quadro (quadros 1000 a 1599 da `partida-ghz1`) | 1.250,9 | MEDIDO ROM nativo | ≤ 2 |
| Bytes alocados por quadro (média da `partida-ghz1`) | 119.701 | MEDIDO ROM nativo | ≤ 1.024 |
| Bytes enviados à GPU por quadro (média, máximo) | 1.486.964, 2.656.256 | MEDIDO ROM nativo | 143.360 |
| `texImage2D` por minuto na gameplay | 136 | MEDIDO ROM Chrome | 0 depois da partida |
| Quadros repetidos por minuto num monitor de 60,000 Hz | 4,7 | MEDIDO simulação | 0 |
| Mensagens do worklet por segundo (thread de áudio) | 62,5 | MEDIDO simulação | ≤ 16 |
| Heap JS alocado por minuto (painel fechado, aberto) | 9,3 MB, 17,5 MB | MEDIDO ROM Chrome | ≤ 1 MB, ≤ 2 MB |
| Amostras descartadas na partida | 609.907 | MEDIDO ROM Chrome | 0 |
| Tamanho do Wasm | 593.170 B | `dist/pkg` | ≤ 760.000 B |

### 1.3 Cadeia causal

1. O passo de simulação em Wasm custa 15,46 ms em média no modo que o site usa. 27,2% dos quadros passam de 16,67 ms.
2. Cerca de 72% desse tempo vai para preparar candidatos por ponto para a GPU, que é um trabalho que a referência de CPU não faz.
3. O `opt-level = "s"` dobra o custo desse código em Wasm. Com `opt-level = 3`, o mesmo quadro cai para 8,02 ms.
4. O relógio roda até 4 passos por rAF para recuperar o atraso. Cada callback passa a custar 39 a 56 ms, e a tela cai para 16 a 24 fps.
5. Com o teto de 4 passos e de 10 quadros acumulados, o tempo atrasado é descartado. A simulação roda a 52,4 quadros/s, 12,5% abaixo do tempo real.
6. O áudio produzido fica 12,5% abaixo do consumido. A fila do worklet esvazia sem parar: 470 a 870 faltas por minuto.

### 1.4 Notas por componente

Rubrica. São três eixos por quadro, e a nota do componente é a pior entre eles. A nota geral é a pior entre os componentes do caminho quente.

| Nota | Latência (orçamento de 16,67 ms) | Bytes movidos por quadro | Alocações por quadro |
| --- | --- | --- | --- |
| A | até 15% (2,5 ms) | até 150 KB | 0 |
| B | até 30% (5 ms) | até 400 KB | até 2 |
| C | até 50% (8,3 ms) | até 1 MB | até 10 |
| D | até 80% (13,3 ms) | até 3 MB | até 50 |
| F | acima de 80%, ou falha visível ou audível recorrente | acima de 3 MB | acima de 50 |

| Componente | Nota | Motivo principal |
| --- | --- | --- |
| WebGL2 e vídeo | **F** | Falha visível recorrente (24 fps). 924 alocações por quadro. 1,49 MB por quadro para a GPU |
| Motor de áudio | **F** | Falha audível recorrente (563 faltas em 72 s). A causa primária é o vídeo, mas a fila de 21 a 37 ms não absorve nada |
| Build | **D** | `opt-level = "s"` torna o quadro Wasm 1,93 vez mais lento |
| Ponte Wasm/JS | **D** | Cerca de 31 alocações por quadro (Rust e JS). Áudio copiado 3 vezes |
| Ritmo | **C** | 4,7 quadros repetidos por minuto. Período NTSC contra o requisito de 60 Hz |
| Laço do núcleo | **C** | Menos de 1,3% do quadro, mas 6,2 alocações por quadro |
| DOM e CSS | **B** | CLS 0. O painel aberto adiciona 8,2 MB/min de heap |

## 2. Hot takes

**1. O "compositor GPU" é um moedor de CPU que se apresenta como aceleração.** A CPU prepara camadas, fundos, seletores e até 20 candidatos de sprite por ponto. Isso consome 72% do quadro em Wasm: `fontes_planos` 17,3%, `QuadroGpu::ponto` 12,4%, `copy_from_slice` 11,4% e `sprites_da_linha` 11,2% `[MEDIDO ROM V8]`. Só o `fontes_sprites` zera 6.400 entradas por linha, em 224 linhas (`crates/video/src/camadas.rs:558`). No fim, 1,49 MB por quadro sobem para a GPU. A imagem final de 9 bits tem 143.360 B, e a CPU já sabe produzi-la. O oráculo já provou essa imagem. A referência de CPU custa 8,34 ms por quadro. O caminho "GPU" custa 15,46 ms. O desenho também viola o requisito: "A GPU só apresenta e amplia a imagem" (`docs/requisitos.md:114`).

**2. `opt-level = "s"` vende metade do quadro por 129 KB de binário.** Em Wasm, o O3 corta o quadro do site de 15,46 para 8,02 ms (−48%). No caminho de CPU, o corte vai de 8,34 para 6,40 ms `[MEDIDO ROM V8]`. O custo é de +22% no tamanho do Wasm, de 586,0 para 715,3 KiB `[MEDIDO]`. Em "s", o LLVM não expande nem cópias de 4 bytes: `copy_from_slice` aparece como função própria, com 11,4% do quadro. O comentário em `Cargo.toml:16` diz "Nao muda o comportamento". Não muda o comportamento, mas muda o resultado: um jogo que roda abaixo da velocidade real.

**3. A síntese ciclo a ciclo vira o piso, e ninguém a mediu em Wasm.** Depois das correções 1 e 2, a síntese passa a ser 55% de 6,40 ms, ou cerca de 3,5 ms por quadro `[MEDIDO ROM V8]`. `Fm::ciclo` sozinho leva 32,3%. O orçamento da D8.14 era de 3 ms "para WebAssembly" (`docs/conhecimento/sintese.md:34`), e a própria base admite que só mediu no nativo (`docs/conhecimento/sintese.md:11`). O laço de `avancar_ate` confere a fila de escritas a cada ciclo. São 21.333 ciclos de FM e 59.736 de PSG por quadro (`crates/audio/src/sintetizador.rs:218-228`, `:255-265`).

## 3. Auditoria detalhada por componente

Formato: `[SEVERIDADE] arquivo:linha`, evidência, causa-raiz e impacto. Severidades: CRÍTICO, ALTO, MÉDIO, BAIXO e INFO.

### 3.1 Ponte Wasm/JS

- `[ALTO] crates/console/src/lib.rs:471-473`. **Evidência:** `extrair_audio()` faz `std::mem::take` do buffer de áudio. O glue gerado copia com `.slice()` e libera com `__wbindgen_free` (`dist/pkg/sonic1_web.js:59-64`), e o site chama `jogo.audio()` em todo passo (`site/app.js:743`). **Causa-raiz:** o `take` destrói a capacidade, então o `resize` em `crates/console/src/lib.rs:442` aloca de novo a cada quadro. O caminho `audio_ptr` e `audio_len` (`crates/web/src/lib.rs:262-270`) existe, mas o site não o usa. **Impacto:** 1 alocação Rust, 1 alocação JS e 6.408 B copiados por quadro a 48 kHz `[CALCULADO: 801 pares × 2 × 4 B]`.
- `[MÉDIO] crates/console/src/lib.rs:90-106`. **Evidência:** o DMA é lido palavra a palavra num `Vec::new()` sem capacidade (linhas 99 e 102). **Impacto:** 16,3 alocações por quadro `[MEDIDO ROM nativo]`.
- `[MÉDIO] crates/console/src/lib.rs:305-329, 378, 399-400`. **Evidência:** `eventos_nominais()` devolve um `Vec` novo por quadro. `tirar_saidas()` é chamado 2 vezes, e o `mem::take` das saídas perde a capacidade. **Impacto:** 6,5 alocações por quadro `[MEDIDO ROM nativo: linhas 247, 320 e 442]`.
- `[BAIXO] site/app.js:471-488`. **Evidência:** cada apresentação cria 5 views `TypedArray` e faz 10 chamadas Wasm. **Impacto:** 5 objetos JS por quadro apresentado `[CALCULADO]`.
- `[BAIXO] site/app.js:777`. **Evidência:** `jogo.erro()` devolve `String`. O glue decodifica e chama `__wbindgen_free` em todo passo (`dist/pkg/sonic1_web.js:147-158`). **Impacto:** 1 decodificação e 1 liberação por passo `[CALCULADO]`.
- `[BAIXO] site/app.js:557, 647, 650, 795, 836`. **Evidência:** `modo_jogo()` é chamado 2 vezes por passo, e `getGamepads()` roda sem controle conectado. O rAF recebe uma closure nova em todo quadro. **Impacto:** 1 chamada Wasm e 2 alocações JS por passo `[CALCULADO]`.
- `[INFO]` A cópia de áudio custa pouco tempo: `jogo.audio()` gasta 0,07 ms por chamada `[MEDIDO ROM Chrome]`. O problema é a alocação, não a latência.

### 3.2 Motor de áudio

- `[CRÍTICO] sintoma` 563 faltas do worklet em 72 s, e 866 faltas em 60 s com o painel aberto `[MEDIDO ROM Chrome]`. **Causa-raiz:** a simulação roda a 52,4 quadros/s, então o áudio produzido fica 12,5% abaixo do consumido. O conserto principal é o do vídeo (seção 3.3) e o do build (seção 3.7).
- `[ALTO] site/app.js:199-292, 743-763`. **Evidência:** com o `AudioContext` suspenso antes do primeiro clique, o laço continua a enfileirar áudio. A fila de 16.384 pares enche e o worklet descarta o resto: 609.907 amostras descartadas `[MEDIDO ROM Chrome]`. **Impacto:** 341 ms de atraso de áudio na partida `[CALCULADO: 16.384 / 48.000]`, até o controlador drenar a fila.
- `[ALTO] crates/audio-saida/src/misturador.rs:52, 152-163`. **Evidência:** o controlador é só proporcional, com `kp = 0.000002` (linha 157) e teto de ±0,5%. A constante de tempo é de 10,4 s `[CALCULADO: 1 / (kp × 48.000)]`. Em regime, a fila fica entre 1.017 e 1.800 pares, abaixo do alvo de 2.400 `[MEDIDO simulação]`. **Impacto:** qualquer bloqueio do laço principal acima de 25 a 35 ms vira falta. Uma pausa de 50 ms causa 1 falta, e pausas de 60 ms a cada 3 s causam 37 faltas em 2 min `[MEDIDO simulação]`.
- `[MÉDIO] site/audio-worklet.js:92-100`. **Evidência:** o `process()` cria e envia um objeto de status a cada 6 chamadas. **Impacto:** 62,5 alocações e mensagens por segundo na thread de áudio em tempo real `[MEDIDO simulação]`.
- `[MÉDIO] site/app.js:223-227`. **Evidência:** cada mensagem de status escreve `textContent` no painel, mesmo com o painel fechado. **Impacto:** 62,5 escritas no DOM por segundo `[CALCULADO]`.
- `[MÉDIO] crates/audio-saida/src/misturador.rs:100, 109, 121-122`. **Evidência:** `alimentar_quadro` cria 4 `Vec` por quadro. `saida_fm` e `saida_psg` começam vazios e crescem por realocação em `crates/audio-saida/src/reamostrador.rs:171`. **Impacto:** 20 alocações por quadro `[MEDIDO ROM nativo]`.
- `[MÉDIO] crates/audio/src/sintetizador.rs:218-228, 255-265`. **Evidência:** a fila de escritas é conferida a cada ciclo. `avancar_ate` tem 10,5% do quadro em Wasm O3, com `Fm::ciclo` em 32,3% e `Psg::ciclo` em 10,6% `[MEDIDO ROM V8]`. **Impacto:** a síntese soma cerca de 3,5 ms por quadro em Wasm O3, acima do orçamento de 3 ms da D8.14.
- `[BAIXO] crates/audio/src/sintetizador.rs:51-60, 242-246`. **Evidência:** cada `AmostraFm` tem 104 B, com o histórico de 24 ciclos. O site usa só os 4 B de `misturado`. **Impacto:** 92 KB escritos por quadro `[CALCULADO: 888 × 104 B]`.
- `[BAIXO] crates/audio-saida/src/reamostrador.rs:135-142, 145-186`. **Evidência:** o PSG é mono, mas `alimentar_mono` duplica o sinal em L e R e `processar` faz duas convoluções iguais. **Impacto:** 25.632 multiplicações a mais por quadro `[CALCULADO: 801 × 32]`.
- `[INFO] site/audio-worklet.js:19` A retomada com 128 amostras não causa faltas em sequência. Com retomada de 2.400, o resultado da simulação é idêntico, porque a recuperação do relógio entrega 4 quadros de uma vez. A hipótese H14 está refutada.
- `[INFO]` No nativo, a bancada sintética de áudio fica dentro do orçamento. A mediana é de 2.786 µs em "s" e 2.774 µs em O3 `[MEDIDO sintético]`.

### 3.3 WebGL2 e vídeo

- `[CRÍTICO] crates/video/src/renderizador.rs:656-773`. **Evidência:** no modo do site (`somente_gpu`), cada linha chama `fontes_sprites`, `fontes_planos` e `QuadroGpu::ponto` por ponto. Depois, `sprites_compactos_em` zera e copia até 20 candidatos por ponto (`crates/video/src/quadro_gpu.rs:180-203`). **Impacto:** 72% do quadro em Wasm `[MEDIDO ROM V8]`. No nativo, o modo do site custa 7,95 ms contra 5,81 ms da referência (+37%) `[MEDIDO ROM nativo]`.
- `[CRÍTICO] site/gpu.js:156-160`. **Evidência:** são 5 uploads por quadro. Os tamanhos: camadas 286.720 B, sprites 143.360 × n B, fundos 71.680 B, seletores 143.360 B e paletas 4.096 B por linha. Na `partida-ghz1`, n vale 12 em 1.405 de 8.309 quadros. **Impacto:** 1.486.964 B por quadro em média e até 2.656.256 B `[MEDIDO ROM nativo]`. O `renderizar()` gasta 1,64 ms de CPU com n = 12 e 2,52 ms com n = 20. Uma textura R16UI de 143.360 B gasta 0,17 ms `[MEDIDO sintético Chrome]`.
- `[ALTO] crates/video/src/renderizador.rs:366`. **Evidência:** `drenar_cache_de_sprites` faz `mem::take` de `ao_gravar_vram` a cada slot. O push seguinte em `crates/video/src/portas.rs:436-437` e `:547-548` aloca de novo. **Impacto:** 698,9 alocações por quadro, ou 71% de todas `[MEDIDO ROM nativo, atribuição por linha]`. Vale nos dois caminhos de vídeo.
- `[ALTO] crates/video/src/camadas.rs:438-466`. **Evidência:** `varrer_sprites` cria um `Vec::with_capacity(20)` por linha (linha 446), e `self.varridos = ids` libera o anterior (`crates/video/src/renderizador.rs:578-580`). **Impacto:** 225 alocações por quadro `[MEDIDO ROM nativo]`.
- `[ALTO] crates/video/src/renderizador.rs:182-184`. **Evidência:** `ativar_gpu_widescreen` não liga `somente_gpu`, ao contrário de `:197-206`. Quando o jogo começa em 16:9 (`site/app.js:812-817`), a CPU desenha a referência e também os candidatos. **Impacto:** 11,4 a 13,2 ms por quadro no nativo, contra 7,8 a 8,4 ms. O p99 vai a 23 a 28 ms `[MEDIDO ROM nativo]`. A decisão do Marcelo remove o 16:9.
- `[MÉDIO] site/gpu.js:108-111, 157`. **Evidência:** a altura da textura de sprites depende de n, e cada troca de n chama `texImage2D`. **Impacto:** 162 a 299 realocações por minuto `[MEDIDO ROM nativo]`. No Chrome foram 163 em 72 s, com 157.409.280 B realocados `[MEDIDO ROM Chrome]`. O custo de CPU medido nesta GPU é pequeno: n alternando custa 1,04 ms, contra 0,85 ms com n = 4 fixo `[MEDIDO sintético Chrome]`.
- `[MÉDIO] crates/video/src/portas.rs:523`. **Evidência:** `mem::take` de `pendentes` perde a capacidade, e os pushes em `:280` e `:409` alocam de novo. **Impacto:** 10,8 alocações por quadro `[MEDIDO ROM nativo]`.
- `[BAIXO] site/gpu.js:147-152, 160`. **Evidência:** a textura de paletas sobe 4.096 B por linha para 128 B úteis, depois de uma cópia para `paletasEmpacotadasBuffer`. **Impacto:** 3.968 B inúteis por quadro `[CALCULADO]`.
- `[BAIXO] site/gpu.js:105-106, 153-155`. **Evidência:** são 3 `uniform1i` e 5 pares `activeTexture` e `bindTexture` por quadro, com valores que não mudam. **Impacto:** 13 chamadas WebGL redundantes por quadro `[CALCULADO]`.
- `[BAIXO] site/gpu.js:40-46`. **Evidência:** o shader faz até 20 `texelFetch` dependentes por ponto, com divisão e módulo inteiros por iteração. **Impacto:** 71.680 × (n + 4) leituras por quadro `[CALCULADO]`. Com `finish`, n = 20 custa 2,38 ms, contra 0,16 ms do shader de 9 bits `[MEDIDO sintético Chrome]`.
- `[INFO]` Nesta máquina, a bancada sintética de vídeo mede 3.436 µs em "s" e 2.632 µs em O3. O orçamento é de 1.000 µs `[MEDIDO sintético]`. A base registrava de 928 a 1.100 µs (`docs/conhecimento/video.md:78`, `:95`), em outra condição de CPU.

### 3.4 Laço do núcleo (`crates/core`)

- `[INFO]` O núcleo usa menos de 1,3% do quadro em Wasm, nos dois perfis `[MEDIDO ROM V8]`. Física, colisão e máquina de estados não são gargalo. As hipóteses de custo de `Memoria::ler` e de buscas lineares ficam refutadas como prioridade.
- `[MÉDIO] crates/core/src/memoria/mod.rs:186-188`. **Evidência:** `tirar_saidas()` faz `mem::take`, e os pushes em `:533`, `:541`, `:542`, `:575` e `:641` alocam de novo. **Impacto:** 4,6 alocações por quadro `[MEDIDO ROM nativo]`.
- `[MÉDIO] crates/core/src/abertura/mod.rs:529-531, 1306`. **Evidência:** `executar_quadro` copia os eventos com `to_vec()`, e `notificar_observadores` copia as saídas com `to_vec()`. **Impacto:** 1,6 alocação por quadro `[MEDIDO ROM nativo]`.
- `[BAIXO] crates/core/src/abertura/mod.rs:1318-1322`. **Evidência:** `ignorar()` acrescenta um `format!` a `ignoradas_em`, sem limite. **Impacto:** 0 ocorrências nas três entradas do corpus `[MEDIDO ROM nativo]`. Fica registrado, sem item.
- `[INFO] crates/core/src/cobertura/mod.rs:19-25` A feature `cobertura` está desligada no build web (`cargo tree -e features -p sonic1-web` não a lista).
- `[INFO] crates/console/src/lib.rs:283-285` O observador `Box<dyn FnMut>` custa 0,1% do quadro `[MEDIDO ROM V8]`. Fica sem item.

### 3.5 Ritmo

- `[ALTO] site/relogio.js:2`. **Evidência:** `PERIODO_QUADRO` usa o período NTSC de 16,688 ms. O requisito decide 60 Hz exatos (`docs/requisitos.md:142`, `:325`), mas `docs/estado.md:7` registra o período NTSC. **Impacto:** num monitor de 60,000 Hz, 4,7 ticks por minuto ficam sem passo (quadro repetido). Com 60 Hz exatos, são 0. Num monitor de 59,94 Hz, o NTSC dá 1,1 tick sem passo por minuto, e 60 Hz exatos dão 3,5 ticks com 2 passos `[MEDIDO simulação, jitter de ±1 ms não cumulativo]`.
- `[INFO] site/relogio.js:24-36` O teto de 4 passos e de 10 quadros evita o travamento total, mas descarta tempo quando a simulação não alcança. É o que faz o jogo rodar a 52,4 quadros/s.
- `[INFO]` Em 120 e 144 Hz, a tela mostra o mesmo quadro por 2 ou 3 ticks. Interpolação é a fase futura 1 do requisito. Fica sem item.

### 3.6 DOM e CSS

- `[MÉDIO] site/app.js:156-186, 744-756, 766-775`. **Evidência:** com o painel aberto, o laço percorre 1.602 amostras com `Math.abs` em todo passo. A cada 6 quadros, ele escreve 6 textos, troca `style.width` e usa `innerHTML`. A barra tem `transition: width 0.05s` (`site/index.html:526`). **Impacto:** o heap JS sobe de 9,3 para 17,5 MB/min `[MEDIDO ROM Chrome]`.
- `[BAIXO] site/app.js:578-585, 597`. **Evidência:** `iluminarKeycap` roda `querySelectorAll` em cada keydown, inclusive na repetição automática. **Impacto:** 1 busca no DOM por evento de tecla `[CALCULADO]`.
- `[BAIXO] site/index.html:348-364, 606`. **Evidência:** a camada CRT cobre a tela inteira e fica sempre presente, com `opacity: 0`. **Impacto:** 1 camada de composição sobre o canvas `[ESTIMADO: sem medida isolada]`.
- `[INFO]` O CLS ficou em 0 nos três cenários `[MEDIDO ROM Chrome]`.

### 3.7 Build

- `[CRÍTICO] Cargo.toml:15-18`. **Evidência:** `opt-level = "s"` com `lto = true`. **Impacto:** o quadro Wasm do site fica 1,93 vez mais lento que com O3 (15,46 contra 8,02 ms). No nativo, fica 1,37 vez mais lento (7,95 contra 5,80 ms) `[MEDIDO ROM V8 e nativo]`. O Wasm cresce 22% com O3 `[MEDIDO]`.
- `[INFO] crates/web/Cargo.toml` O crate fixa `wasm-bindgen = "=0.2.129"` e a CLI do host é 0.2.128. O build funciona hoje (`dist/pkg/sonic1_web_bg.wasm` registra rustc 1.100.0 e wasm-bindgen 0.2.128). Fica registrado como risco.
- `[INFO]` `codegen-units = 1`, `panic = "abort"`, `wasm-opt` e SIMD não foram medidos. Ficam fora desta rodada.

### 3.8 Alegações do relatório anterior contra a evidência

| Alegação | Fonte | Evidência | Veredito |
| --- | --- | --- | --- |
| "zero-copy typed arrays e 60+ FPS estavel" | `docs/plano-melhorias-frontend.md:41` | 24 fps na tela e 52,4 simulados. O áudio usa `.slice()` | Falsa |
| "Exportados ponteiros diretos ... leitura zero-copy" | `docs/plano-melhorias-frontend.md:54` | O vídeo usa views, mas 1,49 MB por quadro sobem para a GPU. O áudio copia | Falsa no áudio, irrelevante no vídeo |
| "Renderizacao ultrarrapida por quadro (<0.5 ms)" | `docs/plano-melhorias-frontend.md:57` | `simular_quadro` com média de 17,24 ms no Chrome | Falsa |
| "Audio limpo, continuo, sem estalos" | `docs/plano-melhorias-frontend.md:75` | 563 faltas em 72 s | Falsa |
| "reaproveitamento de buffer de audio sem alocacao repetitiva" | `docs/plano-melhorias-frontend.md:105` | `mem::take` em `crates/console/src/lib.rs:472` anula o `resize` | Falsa |
| "0 falhas (underruns) no AudioWorklet" | `docs/plano-melhorias-frontend.md:127` | O teste usa tempo simulado (`site/tests/auditoria-oraculo-web.test.mjs:126-174`) e cria o jogo sem o modo GPU do site (linha 109) | O teste não mede o que declara |
| "texSubImage2D ... eliminou a destruicao de texturas" | `docs/relatorio-melhorias-roast.md:105` | 163 `texImage2D` em 72 s | Falsa |
| "95.000 pixels ... em menos de 0.1 ms" | `docs/relatorio-melhorias-roast.md:104` | 1,64 ms de CPU no `renderizar()` com n = 12 | Falsa |
| "Nota Geral do Sistema: 8.1 / 10" | `docs/relatorio-melhorias-roast.md:7` | Nota F por esta rubrica | Rejeitada |

### 3.9 Fora do escopo desta rodada (registrado)

- O clippy do `core` falha no estado atual, com 144 erros (formato curto) em 16 arquivos, vindos do commit `f8a9a18`. Os maiores são `crates/core/src/abertura/sbz_objetos.rs` (39), `boss_fz.rs` (18) e `elevador_slz.rs` (16). Nesta rodada, a guarda é "não aumentar". A correção fica como pendência do Marcelo.
- Mexer no interior de `Fm::ciclo` e `Psg::ciclo` exige prova amostra a amostra contra o Nuked, que só o Marcelo roda. Fica fora.
- Mover a simulação para um Worker. Depois desta rodada, o quadro cabe na thread principal.

## 4. Matriz cirúrgica de remediação (registro de execução)

Marque `[x]` no mesmo commit do item. Use `BLOQUEADO: <motivo>` quando o protocolo da seção 0.7 mandar. Rode as guardas da seção 5.3 depois de cada item.

| Feito | Ordem | ID | P | Arquivo:Linha | Problema | Solução passo a passo | Aceitação | Ganho estimado |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [ ] | 1 | E0.1 | P0 | `docs/pedidos/l1-hook-detector.patch` | O hook não detecta trechos da ROM (L1 inativa). | (1) Compile o detector com a receita da seção 0.6.<br>(2) Rode os dois controles.<br>(3) Apague o arquivo positivo. | Positivo sai com 1. Negativo sai com 0. | Guarda contra cópia de trechos da ROM. |
| [ ] | 2 | E0.2 | P0 | árvore de trabalho | 10 arquivos modificados e 4 novos da sessão anterior estão sem commit. | (1) Adicione só os 14 arquivos da seção 4.1.<br>(2) Rode o detector.<br>(3) Faça o commit `chore: linha de base da sessao anterior (frontend)`. | Fora do commit ficam só `CLAUDE.md`, `.claude/skills/`, este documento e os arquivos não rastreados de outras sessões. | Cada item seguinte pode ser revertido sozinho. |
| [ ] | 3 | E0.3 | P0 | `docs/relatorio-performance.md` | Este documento está fora do git. | Faça o commit deste documento sozinho. | Commit criado. | Checkboxes rastreáveis. |
| [ ] | 4 | E1.1 | P0 | novo `crates/console/tests/guarda_rom.rs` | Não existe guarda de comportamento que dispense o oráculo. | (1) Crie o arquivo do Anexo A.1.<br>(2) Rode com `SONIC1_ROM` e `--ignored`. | Passa no estado atual com os 9 hashes da seção 5.3. | Detecta qualquer mudança de imagem, RAM ou áudio. |
| [ ] | 5 | E1.2 | P0 | novo `crates/console/tests/alocacoes_rom.rs` | Nenhum teste mede alocações por quadro. | (1) Crie o arquivo do Anexo A.2.<br>(2) Rode e anote a média na seção 6.2. | Falha hoje com 1.250,9 por quadro. Passa no fim do bloco de alocações. | Torna a regressão de alocação visível. |
| [x] | 6 | E1.3 | P0 | novo `site/tests/desempenho-wasm.test.mjs` | O teste de auditoria mede tempo simulado (`site/tests/auditoria-oraculo-web.test.mjs:126-174`). | (1) Crie o arquivo do Anexo A.3.<br>(2) Rode `cargo xtask site`.<br>(3) Rode o teste três vezes e anote a mediana. | Falha hoje com média de 15,46 ms (carga até 6). Passa com média ≤ 8,0 ms, p95 ≤ 13,0 ms e ≤ 2% acima de 16,67 ms. | Mede o tempo de parede do Wasm real do site. |
| [x] | 7 | E1.4 | P0 | novo `site/tests/ritmo-60hz.test.mjs` | Nenhum teste mede quadros repetidos por minuto. | Crie o arquivo do Anexo A.4. | Falha hoje com 47 ticks sem passo em 10 min. Passa com 0. | Torna o batimento visível. |
| [x] | 8 | E1.5 | P0 | novo `site/tests/audio-worklet-carga.test.cjs` | Nenhum teste mede a carga da thread de áudio. | Crie o arquivo do Anexo A.5. | Falha hoje com 625 mensagens em 10 s. Passa com ≤ 160. | Torna a carga do worklet visível. |
| [x] | 9 | W1 | P0 | `Cargo.toml:15-18` | `opt-level = "s"` torna o quadro Wasm 1,93 vez mais lento. | (1) Troque `opt-level = "s"` por `opt-level = 3`.<br>(2) Mantenha `lto = true`.<br>(3) Troque o comentário da linha 16 pelos números da seção 3.7.<br>(4) Rode `cargo xtask site`. | E1.3: média ao menos 40% menor que a linha de base. `dist/pkg/sonic1_web_bg.wasm` ≤ 760.000 B. Guardas iguais. | −7,44 ms por quadro no modo atual do site `[MEDIDO ROM V8]`. +129 KiB de Wasm. |
| [x] | 10 | V1a | P0 | `site/gpu.js:1-183`; `site/app.js:402-434, 463-517, 798-826`; `site/index.html` (seletor de proporção); `crates/console/src/lib.rs:336-359, 453-455, 465-468`; `crates/web/src/lib.rs:111-245` | O site compõe por candidatos: 72% do quadro e 1.486.964 B por quadro para a GPU. Viola `docs/requisitos.md:114`. | (1) Adicione `Console::imagem_9bits(&self) -> &[u16]`, que devolve `self.ponte.renderizador().quadro().pontos()`.<br>(2) Exporte `imagem_ptr()` e `imagem_len()` em `crates/web`.<br>(3) Remova de `crates/web` os exports `*_gpu*`, `ativar_gpu*`, `largura_gpu`, `altura_gpu` e `desativar_gpu`.<br>(4) Remova do `Console` os métodos `ativar_gpu*`, `desativar_gpu`, `quadro_gpu`, `sprites_gpu_slice` e o campo `buffer_sprites_gpu`.<br>(5) Troque `site/gpu.js` pelo Anexo A.6.<br>(6) Em `apresentar()`, crie a view `Uint16Array` uma vez. Recrie só quando `memory.buffer` mudar.<br>(7) Remova a opção 16:9 do `index.html` e o ramo 16:9 do `app.js`.<br>(8) Sem WebGL2, converta a view de 9 bits para o `ImageData` reutilizado, sem `jogo.quadro()`. | Guarda A.1 igual. `console_hashes` igual. Chrome (Anexo A.7): 0 `texImage2D` depois da partida e 143.360 B por quadro. E1.3 passa (com W1). | −7,12 ms por quadro (15,46 → 8,34) `[MEDIDO ROM V8]`. −1.343.604 B por quadro para a GPU `[MEDIDO ROM nativo]`. −1,47 ms de CPU no `renderizar()` `[MEDIDO sintético Chrome]`. |
| [x] | 11 | V2 | P1 | `crates/video/src/renderizador.rs:365-370` | `mem::take` de `ao_gravar_vram` perde a capacidade a cada slot: 698,9 alocações por quadro. | (1) Tire o vetor com `mem::take`.<br>(2) Percorra por referência e atualize o cache.<br>(3) Chame `clear()`.<br>(4) Devolva o vetor ao campo só se o campo continuar vazio. | E1.2 sem a linha `portas.rs:436` na atribuição. Guardas iguais. | −698,9 alocações por quadro `[MEDIDO ROM nativo]`. |
| [x] | 12 | V3 | P1 | `crates/video/src/camadas.rs:438-466`; `crates/video/src/renderizador.rs:578-580` | `varrer_sprites` cria um `Vec` por linha: 225 alocações por quadro. | (1) Troque o retorno `Vec<u8>` por um parâmetro `&mut Vec<u8>`.<br>(2) Chame `clear()` no início.<br>(3) Passe `&mut self.varridos` na chamada da linha 578. | Guardas iguais. | −225 alocações por quadro `[MEDIDO ROM nativo]`. |
| [x] | 13 | V4 | P1 | `crates/video/src/portas.rs:518-531` | `mem::take` de `pendentes` perde a capacidade: 10,8 alocações por quadro. | (1) Aplique o mesmo padrão do V2.<br>(2) Atenção: a aplicação pode iniciar outro DMA e empilhar novas pendências.<br>(3) Devolva o vetor só se `self.pendentes` continuar vazio. | Guardas iguais. | −10,8 alocações por quadro `[MEDIDO ROM nativo]`. |
| [x] | 14 | C1 | P1 | `crates/console/src/lib.rs:90-106, 224-253` | O DMA é lido num `Vec` sem capacidade: 16,3 alocações por quadro. | (1) Adicione a `CapturasEventos` um `Vec<u16>` com as palavras de todos os DMAs do quadro.<br>(2) Crie uma variante interna de `capturar` que grave nesse vetor e guarde a faixa.<br>(3) Mantenha `Ponte::capturar` público e sem mudança.<br>(4) Limpe o vetor no início de cada quadro. | Guardas iguais. | −16,3 alocações por quadro `[MEDIDO ROM nativo]`. |
| [x] | 15 | K1 | P1 | `crates/core/src/memoria/mod.rs:186-188`; `crates/core/src/abertura/mod.rs:515-563, 1300-1316` | `tirar_saidas` perde a capacidade. `to_vec()` copia eventos e saídas: 6,2 alocações por quadro. | (1) Adicione `Memoria::drenar_saidas_em(&mut self, destino: &mut Vec<Saida>)` com `clear()` e `append`.<br>(2) Adicione `Memoria::descartar_saidas(&mut self)` com `clear()`.<br>(3) Mantenha `tirar_saidas()`.<br>(4) Em `executar_quadro`, guarde os eventos num campo reutilizado.<br>(5) Em `notificar_observadores`, copie as saídas novas para um campo reutilizado. | Sem ponto flutuante nem cast que trunca. Clippy do `core` ≤ 144 erros. Guardas iguais. | −6,2 alocações por quadro `[MEDIDO ROM nativo]`. |
| [x] | 16 | C2 | P1 | `crates/console/src/lib.rs:305-329, 376-444` | `eventos_nominais()` aloca um `Vec` por quadro. As saídas perdem a capacidade. `tirar_saidas()` roda 2 vezes. | (1) Preencha os eventos num campo `Vec` reutilizado.<br>(2) Troque `mem::take` das saídas por `std::mem::swap` com um campo reserva e `clear()` depois do uso.<br>(3) Use as novas funções de memória do item K1 nas linhas 378 e 399. | Guardas iguais. | −6,5 alocações por quadro `[MEDIDO ROM nativo]`. |
| [x] | 17 | A1 | P1 | `crates/audio-saida/src/misturador.rs:95-143` | 4 `Vec` por quadro; `saida_fm` e `saida_psg` crescem do zero: 20 alocações por quadro. | (1) Mova os 4 vetores para campos do `MisturadorSaida`.<br>(2) Chame `clear()` em cada um no início de `alimentar_quadro`.<br>(3) Não mude nenhuma conta de ponto flutuante. | E1.2 com média ≤ 2. Hash de áudio igual. | −20 alocações e −37 KB alocados por quadro `[MEDIDO ROM nativo]`. |
| [x] | 18 | W2 | P1 | `site/app.js:743-763`; `site/audio-worklet.js:25-48` | O áudio é copiado 3 vezes por quadro: `mem::take`, `.slice()` do glue e o worklet. | (1) Leia o áudio por `audio_ptr()` e `audio_len()` numa view em cache.<br>(2) Copie a view para um buffer de um pool de 8 `Float32Array(4096)`.<br>(3) Envie `buf.subarray(0, len)` com `[buf.buffer]` na transferência.<br>(4) No worklet, depois da cópia, devolva o buffer com transferência.<br>(5) No `app.js`, devolva o buffer ao pool.<br>(6) Mantenha `extrair_audio()` e `Jogo::audio()` para os testes. | Chrome: heap ≤ 1 MB/min com o painel fechado. Guardas iguais. | −2 alocações e −12.816 B copiados por quadro `[CALCULADO]`. |
| [ ] | 19 | A5 | P1 | `site/audio-worklet.js:92-100`; `site/app.js:223-227` | O worklet envia 62,5 objetos por segundo da thread de tempo real. O `app.js` escreve no DOM a cada mensagem. | (1) Troque `% 6` por `% 24`.<br>(2) No `app.js`, escreva `textContent` só com o painel aberto.<br>(3) Mantenha `jogo.ocupacao_audio()` em toda mensagem. | E1.5 passa. | −46,9 mensagens por segundo na thread de áudio `[CALCULADO]`. |
| [ ] | 20 | A6 | P1 | `site/app.js:199-292, 758-762` | Com o `AudioContext` suspenso, o jogo enche a fila: 609.907 amostras descartadas e 341 ms de atraso na partida. | (1) Não envie áudio ao worklet enquanto `audioCtx.state !== "running"`.<br>(2) No evento `statechange`, ao entrar em `running`, envie `{ tipo: "reset" }` ao worklet. | Chrome: `Descartadas: 0` depois do clique. | −341 ms de atraso inicial `[CALCULADO]`. −609.907 amostras descartadas `[MEDIDO ROM Chrome]`. |
| [ ] | 21 | R1 | P1 | `site/relogio.js:2`; `site/app.js:810`; `site/audio-worklet.js:18`; `docs/estado.md:7` | Período NTSC contra o requisito de 60 Hz: 4,7 quadros repetidos por minuto a 60,000 Hz. | (1) Troque `PERIODO_QUADRO` por `1000 / 60`.<br>(2) Na linha 810, passe `taxa * (53693175 / 896040) / 60` a `criar_com_audio`. Assim o reamostrador entrega 800 pares por quadro a 48 kHz.<br>(3) No worklet, troque o preenchimento inicial por `Math.ceil(sampleRate * 3 / 60)`.<br>(4) Atualize `docs/estado.md:7`. | E1.4 passa. Guardas iguais (a guarda usa 48.000 direto). | −4,7 quadros repetidos por minuto `[MEDIDO simulação]`. |
| [ ] | 22 | J1 | P2 | `site/app.js:777`; `crates/web/src/lib.rs:282-285` | `erro()` decodifica uma `String` e libera memória Wasm em todo passo. | (1) Exporte `tem_erro() -> bool`.<br>(2) Chame `erro()` só quando `tem_erro()` for verdadeiro. | Nenhum `__wbindgen_free` por passo no caminho sem erro. | −1 decodificação e −1 liberação por passo `[CALCULADO]`. |
| [ ] | 23 | J2 | P3 | `site/app.js:555-576, 645-698, 795, 836` | `modo_jogo()` 2 vezes por passo. `getGamepads()` sem controle. Closure nova no rAF. | (1) Leia `modo_jogo()` uma vez por passo e passe o valor às duas funções.<br>(2) Chame `getGamepads()` só depois de um `gamepadconnected`.<br>(3) Crie a função do rAF uma vez por sessão. | Controles funcionam no teclado e no gamepad. | −1 chamada Wasm e −2 alocações por passo `[CALCULADO]`. |
| [ ] | 24 | D1 | P2 | `site/app.js:156-186, 744-756, 766-775`; `site/index.html:522-527` | Painel aberto: 1.602 `Math.abs` por passo, `innerHTML` e `style.width` com transição. | (1) Calcule o pico só no passo que atualiza o painel.<br>(2) Troque `innerHTML` por `textContent` e classes.<br>(3) Troque `style.width` por `transform: scaleX()`.<br>(4) Remova `transition: width`.<br>(5) Escreva no DOM só quando o valor mudar. | Chrome: heap com o painel aberto ≤ 1 MB/min acima do painel fechado. | −8,2 MB/min de heap JS `[MEDIDO ROM Chrome]`. |
| [ ] | 25 | D2 | P3 | `site/app.js:578-597` | `querySelectorAll` em cada keydown, inclusive na repetição automática. | (1) Ignore eventos com `e.repeat`.<br>(2) Na partida, guarde os elementos por `data-code` num `Map`. | Teclas acendem e apagam como hoje. | −1 busca no DOM por tecla `[CALCULADO]`. |
| [ ] | 26 | D3 | P3 | `site/index.html:348-364, 606` | A camada CRT fica sempre na composição, com `opacity: 0`. | Use `display: none` quando o filtro estiver desligado. | O filtro liga e desliga como hoje. | −1 camada de composição `[ESTIMADO: sem medida isolada]`. |
| [ ] | 27 | S1 | P2 | `crates/audio/src/sintetizador.rs:203-286` | A fila de escritas é conferida em cada um dos 81.069 ciclos por quadro. | (1) Calcule uma vez o ciclo da próxima escrita do FM.<br>(2) Rode `fm.ciclo()` em laço interno até esse ciclo ou até o alvo.<br>(3) Aplique as escritas daquele ciclo na ordem atual.<br>(4) Faça o mesmo no PSG.<br>(5) Mantenha só aritmética inteira. | Hash de áudio igual. Bancada de áudio com mediana menor que a linha de base. | −0,3 a −0,5 ms por quadro em Wasm `[ESTIMADO: metade do tempo próprio de avancar_ate, 10,5% de 6,40 ms]`. |
| [ ] | 28 | S2 | P3 | `crates/audio/src/sintetizador.rs:51-60, 203-248`; `crates/audio-saida/src/misturador.rs:95-118` | Cada `AmostraFm` copia 104 B, mas o site usa 4 B. | (1) Adicione `avancar_ate_compacto`, que entrega `(i16, i16)` do FM e `u16` do PSG.<br>(2) Use-o no `Console`.<br>(3) Adicione ao misturador uma entrada para essas fatias.<br>(4) Mantenha `avancar_ate` para os testes de fidelidade. | Hash de áudio igual. | −88.800 B escritos por quadro `[CALCULADO: 888 × 100 B]`. |
| [ ] | 29 | A2 | P3 | `crates/audio-saida/src/reamostrador.rs:135-186`; `crates/audio-saida/src/misturador.rs:118, 124` | O PSG mono passa por duas convoluções iguais. | (1) Adicione `processar_mono` com uma convolução.<br>(2) Duplique o resultado em L e R no misturador.<br>(3) Mantenha a mesma ordem das somas. | Hash de áudio igual. | −25.632 multiplicações por quadro `[CALCULADO]`. Cerca de −40 µs `[ESTIMADO: metade do custo do PSG em processar]`. |
| [ ] | 30 | V1b | P3 | `crates/video/src/quadro_gpu.rs`; `crates/video/src/renderizador.rs:91-92, 171-216, 556-561, 625-651, 656-671, 703-705, 719, 734-773`; `crates/video/src/camadas.rs:306, 550`; `crates/video/src/lib.rs`; `crates/video/tests/gpu.rs` | Depois do V1a, o caminho de candidatos é código morto. | (1) Remova `QuadroGpu`, os campos `quadro_gpu`, `somente_gpu` e `fontes_*`, e as funções `fontes_planos` e `fontes_sprites`.<br>(2) Apague `crates/video/tests/gpu.rs` no mesmo commit, porque ele só compara o caminho removido.<br>(3) Registre o motivo no commit e no `docs/estado.md`. | Testes de vídeo, console e web passam. Guardas iguais. | Menos código e Wasm menor `[ESTIMADO: 10 a 30 KB]`. |
| [ ] | 31 | T1 | P1 | `docs/estado.md`; `docs/conhecimento/` | Os números e as decisões desta rodada não estão na base. | (1) Atualize `docs/estado.md` com 60 Hz exatos, a exceção da ROM, a saída do 16:9 e os números antes e depois.<br>(2) Crie `docs/conhecimento/desempenho-web.md` com comandos e números.<br>(3) Adicione uma linha ao índice `docs/conhecimento/README.md`.<br>(4) Acrescente a medida Wasm em `video.md` e `sintese.md`. | Toda afirmação cita comando e número. | Base de conhecimento atualizada. |

### 4.1 Arquivos do commit de linha de base (E0.2)

```sh
git add crates/console/src/lib.rs crates/core/src/abertura/titulo.rs crates/rom/src/lib.rs \
  crates/video/src/quadro_gpu.rs crates/web/src/lib.rs site/app.js site/audio-worklet.js \
  site/gpu.js site/index.html site/relogio.js crates/console/tests/gameplay.rs \
  docs/plano-melhorias-frontend.md docs/relatorio-melhorias-roast.md \
  site/tests/auditoria-oraculo-web.test.mjs
```

### 4.2 Notas de implementação

**V2 e V4: padrão "tirar, usar, limpar, devolver".** A capacidade volta ao campo. Uma escrita nova feita durante o laço não se perde.

```rust
let mut gravados = std::mem::take(&mut self.vdp.ao_gravar_vram);
for &(endereco, valor) in &gravados {
    self.atualizar_cache_de_sprites(endereco, valor);
}
gravados.clear();
// Devolve a capacidade só se ninguém gravou no campo durante o laço.
if self.vdp.ao_gravar_vram.is_empty() {
    self.vdp.ao_gravar_vram = gravados;
}
```

**Ordem K1 e C2.** O K1 cria `drenar_saidas_em` e `descartar_saidas` no `core`. O C2 usa essas funções no `Console`. Por isso o K1 vem antes, na ordem 15.

**V1a: limpeza no `app.js`.** Remova a função `aplicarProporcao`, o `seletorProporcao`, a variável `modoWidescreen` e a chamada `jogo.desativar_gpu()` de `usarCpu`. Em `ajustarEscala`, use a largura fixa de 320. No laço, chame sempre `jogo.simular_quadro()`. O caminho sem WebGL2 converte a view de 9 bits em `apresentar()`.

**K1: eventos sem cópia.** Use o mesmo padrão. Tire o campo, rode o laço por índice com `EventoQuadro` (que é `Copy`) e devolva o campo no fim. No modo `atraso`, preencha o campo com os eventos novos e troque-o com `pendentes` por `std::mem::swap`. O resultado é o mesmo de `std::mem::replace(&mut self.pendentes, eventos.to_vec())`.

**V1a: view da imagem no `app.js`.** O ponteiro do `Vec<u16>` do quadro não muda, porque o vetor tem tamanho fixo. A view só precisa ser recriada quando a memória Wasm crescer.

```js
// View da imagem de 9 bits na memoria Wasm. Recriada so quando a memoria cresce.
let vistaImagem = null;
function imagemAtual() {
  const mem = wasmInstance.memory.buffer;
  if (!vistaImagem || vistaImagem.buffer !== mem) {
    vistaImagem = new Uint16Array(mem, jogo.imagem_ptr(), jogo.imagem_len());
  }
  return vistaImagem;
}
```

**W2: pool de buffers de áudio.** O worklet devolve cada buffer depois da cópia. Diferencie as mensagens no `app.js` com `e.data instanceof ArrayBuffer`. Se o pool esvaziar, crie um buffer novo e registre o evento no console uma vez.

**R1: taxa corrigida.** Com 60 quadros/s, o sintetizador entrega mais amostras por segundo real que no console NTSC. A taxa corrigida (47.938 Hz para um contexto de 48.000 Hz) mantém o controlador perto de zero. Comente essa conta no código.

## 5. Roteiro de validação

### 5.1 Preparação

```sh
export SONIC1_ROM="$PWD/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen"
cat /sys/devices/system/cpu/intel_pstate/no_turbo            # registre (1 = turbo desligado)
cat /sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq    # registre
uptime                                                       # registre a carga media
```

- Meça só com carga média de 1 minuto ≤ 6. Acima disso, espere e repita.
- Compare antes e depois na mesma condição de CPU.
- Rode cada medida de tempo três vezes e use a mediana.

### 5.2 Medição "antes" (itens E1.1 a E1.5)

| Medida | Comando | Valor na auditoria |
| --- | --- | --- |
| Guarda de comportamento | `cargo test --locked --offline --release -p sonic1-console --test guarda_rom -- --ignored` | Passa (9 hashes iguais) |
| Alocações por quadro | `cargo test --locked --offline --release -p sonic1-console --test alocacoes_rom -- --ignored --nocapture` | 1.250,9 (falha) |
| Quadro Wasm | `cargo xtask site && node --test site/tests/desempenho-wasm.test.mjs` | Média de 15,46 ms com carga 5. 29,64 ms com carga 21 (falha) |
| Ritmo | `node --test site/tests/ritmo-60hz.test.mjs` | 47 ticks sem passo em 10 min (falha) |
| Carga do worklet | `node --test site/tests/audio-worklet-carga.test.cjs` | 625 mensagens em 10 s (falha) |
| Bancada de vídeo | `WEBSONIC_BENCH=1 cargo test --locked --offline -p sonic1-video --release --test desempenho -- --nocapture` | Mediana de 3.436 µs |
| Bancada de áudio | `WEBSONIC_BENCH=1 cargo test --locked --offline -p sonic1-audio-saida --release --test desempenho -- --nocapture` | Mediana de 2.786 µs |

### 5.3 Guardas (rode depois de cada item)

```sh
cargo test --locked --offline -p sonic1-console -p sonic1-web -p sonic1-audio-saida -p sonic1-video -p sonic1-audio
cargo test --locked --offline --release -p sonic1-console --test guarda_rom -- --ignored
cargo run --locked --offline --release -q -p sonic1-console --bin console_hashes -- "$SONIC1_ROM" 7200 2>/dev/null | sha1sum
node --test site/tests/relogio.test.mjs site/tests/audio-worklet.test.cjs
cargo clippy --locked --offline -p sonic1-core --message-format=short -- -D warnings 2>&1 | grep -c -E '^crates/core/.*: error'
cargo build --locked --offline --release -p sonic1-web --target wasm32-unknown-unknown
```

| Guarda | Valor esperado |
| --- | --- |
| Testes nativos dos 5 crates | 0 falhas. Na auditoria: 88 passaram e 2 ficaram ignorados |
| `guarda_rom`, `boot-demos` (7.200 quadros sem botão) | imagem `4a20bb47deb30ef2`, RAM `31447929316fe372`, áudio `2f96f7e516f55c0c` |
| `guarda_rom`, `partida-ghz1` (8.309 quadros) | imagem `2cd5d4f4542832f1`, RAM `1d5ddead9530554b`, áudio `1065375cee2d77ae` |
| `guarda_rom`, `lz-agua` (2.000 quadros) | imagem `1f88b478e64a9179`, RAM `c66c4067b5208e86`, áudio `ab4b23d631156f04` |
| `console_hashes` com 7.200 quadros | `sha1sum` igual a `b8d4a450a5baf18634d191411896fd5caad98d6e` |
| Testes do site sem ROM | 6 de 6 |
| Erros do clippy no `core` | ≤ 144 (valor da auditoria). Nenhum erro novo nos arquivos tocados |
| Build `wasm32-unknown-unknown` | Sem erro |

O hash de áudio usa ocupação fixa de 2.400 pares e taxa de 48.000 Hz. Por isso ele não muda com os ganhos do controlador, nem com a taxa corrigida do item R1, que fica no `app.js`.

### 5.4 Medição "depois" e critérios de aprovação

| Medida | Critério |
| --- | --- |
| `alocacoes_rom` | Média ≤ 2 por quadro |
| `desempenho-wasm.test.mjs` | Média ≤ 8,0 ms, p95 ≤ 13,0 ms e ≤ 2% acima de 16,67 ms. Referência medida: 6,40 ms, 10,92 ms e 0,2% com CPU de referência e O3 |
| `ritmo-60hz.test.mjs` | 0 ticks sem passo a 60,000 Hz. No máximo 40 ticks com 2 passos a 59,94 Hz, em 10 min |
| `audio-worklet-carga.test.cjs` | ≤ 160 mensagens em 10 s e 0 faltas |
| Tamanho do Wasm | ≤ 760.000 B |
| Guardas da seção 5.3 | Todas iguais |

### 5.5 Testes de estresse

- Vídeo: com o V1a, o upload é fixo em 143.360 B por quadro, para qualquer número de sprites. Confirme com a contagem de bytes do Anexo A.7.
- Pausas: rode `audio-worklet-carga.test.cjs` com um bloqueio de 50 ms a cada 3 s (edite uma cópia local, não o teste). Registre o número de faltas na seção 6.2.
- CPU lenta (Marcelo, no Chrome): DevTools, aba Performance, CPU com redução de 4x. Grave 60 s de demo. Registre quadros simulados por segundo e faltas do worklet.

### 5.6 O que só o Marcelo roda

- Instrumentação no Chrome com a ROM (Anexo A.7), antes e depois. Metas: ≥ 59,5 quadros simulados por segundo e ≤ 1% de intervalos acima de 33 ms. Também: 0 faltas em 120 s, heap ≤ 1 MB/min e 0 `texImage2D` depois da partida.
- Fidelidade do vídeo contra o oráculo: `sh scripts/etapa7-bloco2.sh ROM 1170`. O veredito esperado é IGUAL.
- Fidelidade do som contra o Nuked: `sh scripts/etapa8-bloco2.sh ROM 600`, se os itens S1, S2 ou A2 tiverem entrado. O veredito esperado é IGUAL.
- Fidelidade nativo contra Wasm: `sh scripts/etapa7-bloco5.sh ROM 1200`, que grava em `privado/`.

## 6. Término e entrega

### 6.1 Critério de término

- Todos os itens da seção 4 estão `[x]` ou `BLOQUEADO` com motivo.
- As guardas da seção 5.3 passam no último commit.
- Os critérios da seção 5.4 passam, ou o desvio aparece na seção 6.2 com o número medido.
- `git status --short` mostra fora dos commits só `CLAUDE.md`, `.claude/skills/` e os arquivos não rastreados de outras sessões, intactos.

### 6.2 Relatório final do executor

Preencha esta tabela no último commit e mostre-a no chat.

| Medida | Antes (auditoria) | Antes (executor) | Depois |
| --- | --- | --- | --- |
| Quadro Wasm `partida-ghz1` (média, p95, % acima de 16,67 ms) | 15,46 ms, 23,50 ms, 27,2% | 17,26 ms, 29,24 ms, 38,6% | |
| Alocações por quadro (1000 a 1599) | 1.250,9 | 1.250,9 | |
| Bytes para a GPU por quadro | 1.486.964 | 1.486.964 | 143.360 |
| Ticks sem passo em 10 min a 60 Hz | 47 | 47 | |
| Mensagens do worklet em 10 s | 625 | 625 | |
| Tamanho do Wasm | 593.170 B | | |
| Itens `BLOQUEADO` | | | |

Liste também os commits (hash e título) e as pendências da seção 6.3.

### 6.3 Pendências do Marcelo

1. Rodar o roteiro da seção 5.6.
2. Decidir sobre os 144 erros do clippy no `core` (seção 3.9).
3. Ativar a L1 de forma definitiva com os patches de `docs/pedidos/` (`docs/estado.md:24-33`).
4. Revisar e fazer o push.
5. Conferir o sumiço de `CLAUDE.md.bak-1794556096`. Ele existia no início da auditoria e não existia no fim. Nenhum comando da auditoria tocou nele. Outra sessão criou `docs/*gta*` no mesmo período.

### 6.4 Handoff

Chame `memory_handoff_begin` com um resumo de 2 a 3 frases, as pendências da seção 6.3 como próximos passos e a lista de arquivos tocados.

## Anexo A. Código das bancadas e dos testes

Os arquivos A.1 a A.5 foram compilados e rodados contra a árvore da auditoria. A.1 passou. A.2, A.3, A.4 e A.5 falharam pelos motivos previstos.

### A.1 `crates/console/tests/guarda_rom.rs`

```rust
//! Guarda de comportamento sem o oráculo: hashes agregados da imagem de 9 bits, da RAM do 68000 e
//! do áudio de saída, quadro a quadro, nas três entradas do corpus. Uma correção de desempenho não
//! pode mudar nenhum deles. Valores medidos em 17/11/2026 (docs/relatorio-performance.md, seção 5.3).
use sonic1_console::Console;

const CORPUS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus");
const FNV_INICIO: u64 = 0xcbf2_9ce4_8422_2325;

/// FNV-1a de 64 bits, o mesmo de `Ponte::hash_da_imagem`.
fn fnv(h: &mut u64, bytes: &[u8]) {
    for &b in bytes {
        *h ^= u64::from(b);
        *h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
}

fn ler_s1in(nome: &str) -> Vec<u8> {
    std::fs::read_to_string(format!("{CORPUS}/{nome}"))
        .expect("entrada do corpus")
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .map(|s| u8::from_str_radix(s, 16).expect("hex"))
        .collect()
}

/// Roda as entradas no caminho de CPU, com a ocupação do worklet fixa no alvo (2400 pares a
/// 48 kHz). Assim o hash de áudio não depende dos ganhos do controlador.
fn hashes(rom: &[u8], entradas: &[u8]) -> (u64, u64, u64) {
    let mut c = Console::com_taxa_audio(rom.to_vec(), 48_000.0);
    let (mut img, mut ram, mut audio) = (FNV_INICIO, FNV_INICIO, FNV_INICIO);
    for &b in entradas {
        c.definir_ocupacao_audio(2400);
        c.definir_botoes(b);
        c.simular_quadro();
        for s in c.audio_slice() {
            fnv(&mut audio, &s.to_bits().to_le_bytes());
        }
        let mut r = FNV_INICIO;
        fnv(&mut r, &c.ab.mem.ram()[..]);
        fnv(&mut ram, &r.to_le_bytes());
        fnv(&mut img, &c.hash_da_imagem().to_le_bytes());
    }
    (img, ram, audio)
}

#[test]
#[ignore = "requer SONIC1_ROM"]
fn imagem_ram_e_audio_nao_mudam() {
    let rom = std::fs::read(std::env::var_os("SONIC1_ROM").expect("SONIC1_ROM")).expect("ROM");
    let casos: [(&str, Vec<u8>, (u64, u64, u64)); 3] = [
        ("boot-demos", vec![0; 7200], (0x4a20_bb47_deb3_0ef2, 0x3144_7929_316f_e372, 0x2f96_f7e5_16f5_5c0c)),
        ("partida-ghz1", ler_s1in("partida-ghz1.s1in"), (0x2cd5_d4f4_5428_32f1, 0x1d5d_dead_9530_554b, 0x1065_375c_ee2d_77ae)),
        ("lz-agua", ler_s1in("lz-agua.s1in"), (0x1f88_b478_e64a_9179, 0xc66c_4067_b520_8e86, 0xab4b_23d6_3115_6f04)),
    ];
    for (nome, entradas, esperado) in casos {
        assert_eq!(hashes(&rom, &entradas), esperado, "{nome}: imagem, RAM ou áudio mudou");
    }
}
```

### A.2 `crates/console/tests/alocacoes_rom.rs`

```rust
//! Alocações por quadro na gameplay real: a média nos quadros 1000 a 1599 da partida-ghz1 não
//! passa de 2. O alocador conta só a thread do teste (o executor de testes usa outras threads).
use sonic1_console::Console;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

const CORPUS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus");

thread_local! {
    static CONTANDO: Cell<bool> = const { Cell::new(false) };
    static ALOCACOES: Cell<u64> = const { Cell::new(0) };
}

/// Alocador que delega ao do sistema e conta alocações e realocações da thread marcada.
struct Contador;

fn contar() {
    // `try_with` evita pânico se a thread já estiver encerrando.
    let _ = CONTANDO.try_with(|c| {
        if c.get() {
            let _ = ALOCACOES.try_with(|a| a.set(a.get().wrapping_add(1)));
        }
    });
}

// SAFETY: só delega ao `System` e incrementa um contador local da thread, sem alocar.
unsafe impl GlobalAlloc for Contador {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        contar();
        unsafe { System.alloc(l) }
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        contar();
        unsafe { System.alloc_zeroed(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        contar();
        unsafe { System.realloc(p, l, n) }
    }
}

#[global_allocator]
static ALOCADOR: Contador = Contador;

#[test]
#[ignore = "requer SONIC1_ROM"]
fn gameplay_quase_sem_alocacao_por_quadro() {
    let rom = std::fs::read(std::env::var_os("SONIC1_ROM").expect("SONIC1_ROM")).expect("ROM");
    let entradas: Vec<u8> = std::fs::read_to_string(format!("{CORPUS}/partida-ghz1.s1in"))
        .expect("entrada")
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .map(|s| u8::from_str_radix(s, 16).expect("hex"))
        .collect();
    let mut c = Console::com_taxa_audio(rom, 48_000.0);
    let quadro = |c: &mut Console, b: u8| {
        c.definir_ocupacao_audio(2400);
        c.definir_botoes(b);
        c.simular_quadro();
    };
    for &b in &entradas[..1000] {
        quadro(&mut c, b);
    }
    CONTANDO.with(|x| x.set(true));
    for &b in &entradas[1000..1600] {
        quadro(&mut c, b);
    }
    CONTANDO.with(|x| x.set(false));
    let media = ALOCACOES.with(Cell::get) as f64 / 600.0;
    println!("alocacoes por quadro (quadros 1000 a 1599): {media:.1}");
    assert!(media <= 2.0, "média de {media:.1} alocações por quadro (limite 2)");
}
```

Para achar a origem de uma alocação restante, compile uma cópia temporária deste teste fora do repositório com `CARGO_PROFILE_RELEASE_DEBUG=line-tables-only`. Registre `std::backtrace::Backtrace::force_capture()` no alocador, com uma trava por thread contra recursão. Agrupe pelo primeiro quadro em `crates/`.

### A.3 `site/tests/desempenho-wasm.test.mjs`

```js
// Tempo de parede de cada quadro do Wasm real do site (dist/pkg), na partida-ghz1 com a ROM.
// Pula sem a ROM ou sem o dist. Rode `cargo xtask site` antes, para medir o codigo atual.
import { test } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const raiz = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const caminhoJs = path.join(raiz, "dist/pkg/sonic1_web.js");
const caminhoWasm = path.join(raiz, "dist/pkg/sonic1_web_bg.wasm");
const caminhoRom = path.join(raiz, "sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen");

test("quadro do Wasm cabe no orcamento de 16,67 ms na partida-ghz1", async (t) => {
  if (!fs.existsSync(caminhoRom) || !fs.existsSync(caminhoWasm)) return t.skip("sem ROM ou sem dist");
  const { default: init, Jogo } = await import(caminhoJs);
  await init({ module_or_path: fs.readFileSync(caminhoWasm) });
  const jogo = Jogo.criar_com_audio(fs.readFileSync(caminhoRom), 48000);
  // Mesmo modo de video que o site usa hoje. Depois do item V1a esta funcao nao existe mais.
  if (typeof jogo.ativar_gpu_sem_referencia === "function") jogo.ativar_gpu_sem_referencia();
  const entradas = fs.readFileSync(path.join(raiz, "corpus/partida-ghz1.s1in"), "utf8")
    .split("\n").map((s) => s.trim()).filter((s) => s && !s.startsWith("#")).map((s) => parseInt(s, 16));
  const tempos = new Float64Array(entradas.length);
  for (let i = 0; i < entradas.length; i++) {
    const inicio = performance.now();
    jogo.ocupacao_audio(2400);
    jogo.botoes(entradas[i]);
    jogo.simular_quadro();
    tempos[i] = performance.now() - inicio;
  }
  const ordem = Float64Array.from(tempos).sort();
  const media = tempos.reduce((a, b) => a + b, 0) / tempos.length;
  const p95 = ordem[Math.round((ordem.length - 1) * 0.95)];
  const acima = tempos.filter((x) => x > 16.67).length / tempos.length;
  console.log(`wasm partida-ghz1: media=${media.toFixed(2)} ms p95=${p95.toFixed(2)} ms acima_16_67=${(100 * acima).toFixed(1)}%`);
  assert.ok(media <= 8.0, `media ${media.toFixed(2)} ms (limite 8,0)`);
  assert.ok(p95 <= 13.0, `p95 ${p95.toFixed(2)} ms (limite 13,0)`);
  assert.ok(acima <= 0.02, `${(100 * acima).toFixed(1)}% dos quadros acima de 16,67 ms (limite 2%)`);
  jogo.free();
});
```

### A.4 `site/tests/ritmo-60hz.test.mjs`

```js
// O relogio do site contra vsync com jitter nao cumulativo de +-1 ms, por 10 minutos.
// Requisito: logica a 60 Hz exatos (docs/requisitos.md, Plataforma e restricoes).
import { test } from "node:test";
import assert from "node:assert/strict";
import { Relogio } from "../relogio.js";

function simular(hz, minutos = 10) {
  const r = new Relogio();
  r.iniciar(0);
  let semente = 7, zero = 0, dois = 0;
  const aleatorio = () => { semente = (semente * 1103515245 + 12345) % 2147483648; return semente / 2147483648; };
  for (let k = 1; k <= hz * 60 * minutos; k++) {
    const passos = r.avancar(k * 1000 / hz + (aleatorio() * 2 - 1));
    if (passos === 0) zero++;
    if (passos >= 2) dois++;
  }
  return { zero, dois };
}

test("monitor de 60,000 Hz: nenhum quadro repetido nem pulado em 10 minutos", () => {
  const { zero, dois } = simular(60);
  assert.equal(zero, 0, `${zero} ticks sem passo`);
  assert.equal(dois, 0, `${dois} ticks com 2 ou mais passos`);
});

test("monitor de 59,94 Hz: no maximo 4 ticks com 2 passos por minuto (batimento inevitavel)", () => {
  const { zero, dois } = simular(59.94);
  assert.equal(zero, 0, `${zero} ticks sem passo`);
  assert.ok(dois <= 40, `${dois} ticks com 2 ou mais passos em 10 minutos`);
});
```

### A.5 `site/tests/audio-worklet-carga.test.cjs`

```js
// Carga do SonicAudioProcessor na thread de audio: mensagens por segundo e faltas com entrada estavel.
const { test } = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");

test("worklet envia no maximo 16 mensagens por segundo e nao falha com entrada estavel", () => {
  const ctx = { sampleRate: 48000, mensagens: 0 };
  // Conta so as mensagens de status. Depois do item W2, o worklet tambem devolve buffers de audio.
  ctx.AudioWorkletProcessor = class { constructor() { this.port = { postMessage: (m) => { if (m && m.tipo === "status") ctx.mensagens++; } }; } };
  ctx.registerProcessor = (_nome, Classe) => { ctx.Classe = Classe; };
  vm.createContext(ctx);
  vm.runInContext(fs.readFileSync(path.join(__dirname, "../audio-worklet.js"), "utf8"), ctx);
  const p = new ctx.Classe();
  const L = new Float32Array(128), R = new Float32Array(128);
  const periodo = 1000 * 896040 / 53693175;
  let tAudio = 0, tQuadro = 0, fracao = 0;
  const segundos = 10;
  while (tAudio < segundos * 1000) {
    while (tQuadro <= tAudio) {
      fracao += 48000 * periodo / 1000;
      const pares = Math.floor(fracao); fracao -= pares;
      p.port.onmessage({ data: new Float32Array(pares * 2).fill(0.1) });
      tQuadro += periodo;
    }
    p.process([], [[L, R]]);
    tAudio += 128 * 1000 / 48000;
  }
  assert.ok(ctx.mensagens <= 16 * segundos, `${ctx.mensagens} mensagens em ${segundos} s`);
  assert.equal(p.falhas, 0);
});
```

O teste conta só as mensagens com `tipo: "status"`. Assim ele continua válido depois do item W2, quando o worklet passa a devolver buffers ao laço principal.

### A.6 Novo `site/gpu.js` (item V1a)

O protótipo deste shader rodou na bancada do Chrome com 0,16 ms por quadro na Iris Xe.

```js
// Apresenta a imagem de 9 bits do renderizador fiel (CPU). A GPU so converte a cor e amplia
// (docs/requisitos.md, Video fiel em software).
export const LARGURA = 320;
export const ALTURA = 224;

const VERTICE = `#version 300 es
void main() {
  vec2 p = vec2((gl_VertexID << 1) & 2, gl_VertexID & 2);
  gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0);
}`;

// Cor de 9 bits (BBBGGGRRR) para 8 bits por canal, replicando os bits altos
// como Ponte::fechar_quadro (crates/console/src/lib.rs).
const FRAGMENTO = `#version 300 es
precision highp float;
precision highp int;
precision highp usampler2D;
uniform usampler2D imagem;
out vec4 saida;
void main() {
  ivec2 p = ivec2(int(gl_FragCoord.x), ${ALTURA - 1} - int(gl_FragCoord.y));
  uint cor = texelFetch(imagem, p, 0).r;
  uvec3 rgb = uvec3(cor & 7u, (cor >> 3u) & 7u, (cor >> 6u) & 7u);
  rgb = (rgb << 5u) | (rgb << 2u) | (rgb >> 1u);
  saida = vec4(vec3(rgb) / 255.0, 1.0);
}`;

export function criarRenderizadorGPU(canvas) {
  const gl = canvas.getContext("webgl2", { alpha: false, antialias: false, depth: false, stencil: false });
  if (!gl) return null;
  const shader = (tipo, codigo) => {
    const s = gl.createShader(tipo);
    gl.shaderSource(s, codigo);
    gl.compileShader(s);
    if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(s));
    return s;
  };
  const programa = gl.createProgram();
  gl.attachShader(programa, shader(gl.VERTEX_SHADER, VERTICE));
  gl.attachShader(programa, shader(gl.FRAGMENT_SHADER, FRAGMENTO));
  gl.linkProgram(programa);
  if (!gl.getProgramParameter(programa, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(programa));
  gl.useProgram(programa);
  gl.bindVertexArray(gl.createVertexArray());
  gl.disable(gl.BLEND);
  gl.disable(gl.DITHER);
  // Textura imutavel criada uma vez: cada quadro so troca o conteudo (sem texImage2D).
  const textura = gl.createTexture();
  gl.activeTexture(gl.TEXTURE0);
  gl.bindTexture(gl.TEXTURE_2D, textura);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
  gl.texStorage2D(gl.TEXTURE_2D, 1, gl.R16UI, LARGURA, ALTURA);
  gl.pixelStorei(gl.UNPACK_ALIGNMENT, 2);
  gl.uniform1i(gl.getUniformLocation(programa, "imagem"), 0);
  canvas.width = LARGURA;
  canvas.height = ALTURA;
  gl.viewport(0, 0, LARGURA, ALTURA);
  const debug = gl.getExtension("WEBGL_debug_renderer_info");
  return {
    // `imagem`: Uint16Array de 320 x 224 pontos, view da memoria Wasm (sem copia no JS).
    renderizar(imagem) {
      gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, LARGURA, ALTURA, gl.RED_INTEGER, gl.UNSIGNED_SHORT, imagem);
      gl.drawArrays(gl.TRIANGLES, 0, 3);
    },
    // Usado na validacao: framebuffer em ordem inferior para superior.
    lerPixels() {
      const pixels = new Uint8Array(LARGURA * ALTURA * 4);
      gl.readPixels(0, 0, LARGURA, ALTURA, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
      return pixels;
    },
    sincronizar() { gl.finish(); },
    renderer: gl.getParameter(debug ? debug.UNMASKED_RENDERER_WEBGL : gl.RENDERER),
  };
}
```

### A.7 Instrumentação do Chrome (Marcelo)

Abra o site, abra o console do DevTools e cole o trecho abaixo antes de escolher a ROM. Depois de 120 s de demo, rode `__resumo()`. O trecho só mede: ele não altera o jogo.

```js
(() => {
  const M = { t0: performance.now(), raf: [], intervalos: [], texImage: 0, bytesSub: 0, sub: 0, longtasks: 0, heap: [] };
  window.__metricas = M;
  const P = WebGL2RenderingContext.prototype, ti = P.texImage2D, ts = P.texSubImage2D;
  P.texImage2D = function (...a) { M.texImage++; return ti.apply(this, a); };
  P.texSubImage2D = function (...a) { M.sub++; const d = a[a.length - 1]; if (d && d.byteLength) M.bytesSub += d.byteLength; return ts.apply(this, a); };
  const orig = window.requestAnimationFrame.bind(window);
  window.requestAnimationFrame = (cb) => orig((t) => { const s = performance.now(); cb(t); M.raf.push(performance.now() - s); });
  let ultimo = 0;
  const medir = (t) => { if (ultimo) M.intervalos.push(t - ultimo); ultimo = t; orig(medir); };
  orig(medir);
  new PerformanceObserver((l) => { M.longtasks += l.getEntries().length; }).observe({ type: "longtask" });
  setInterval(() => { if (performance.memory) M.heap.push(performance.memory.usedJSHeapSize); }, 1000);
  window.__resumo = () => {
    const s = (performance.now() - M.t0) / 1000, ord = [...M.intervalos].sort((a, b) => a - b);
    let aloc = 0; for (let i = 1; i < M.heap.length; i++) aloc += Math.max(0, M.heap[i] - M.heap[i - 1]);
    return {
      segundos: +s.toFixed(1),
      intervalo_medio_ms: +(M.intervalos.reduce((a, b) => a + b, 0) / M.intervalos.length).toFixed(2),
      intervalo_p95_ms: ord[Math.round((ord.length - 1) * 0.95)],
      intervalos_acima_33ms: M.intervalos.filter((x) => x > 33.4).length,
      texImage2D: M.texImage, bytes_por_upload: Math.round(M.bytesSub / Math.max(1, M.sub)),
      longtasks: M.longtasks, heap_MB_por_min: +((aloc / 1048576) / (s / 60)).toFixed(1),
      worklet: document.getElementById("diag-worklet")?.textContent,
    };
  };
})();
```

Para medir os quadros simulados por segundo, abra o painel de diagnóstico. Leia o campo do quadro duas vezes, com 60 s de intervalo.
