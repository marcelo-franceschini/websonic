# Plano da Etapa 7: Telas de abertura

Escrito em 08/11/2026. Fonte única: `docs/requisitos.md`. Modo de trabalho desta etapa: a IA decide sozinha, registra a decisão e segue. Paradas só para blocos de comandos do Marcelo. A aprovação deste plano vem junto com o resultado do primeiro bloco.

Pré-condições conferidas em `docs/estado.md`: Etapa 3 feita (185 de 185 blocos IGUAL), Etapa 4 feita (`video-comparar` IGUAL em 7200 e 2000 quadros), Etapa 6 feita (relatório do som 78 IGUAL, 0 DIVERGE). Nenhum conflito com `docs/requisitos.md` foi achado. Há três pontos que o texto do requisito deixa abertos e que este plano decide (D7.1, D7.3 e D7.4). Estão marcados abaixo para o Marcelo ver.

## 1. O trecho

**Começa** no quadro 0 do caso `boot-demos` (ligar do console, vetor de reset, `EntryPoint`, `$206`), sem nenhum botão apertado.

**Termina** no fim do primeiro quadro em que o 68000 chega a `Level_StartGame` (`$3AD8`) com o bit 7 de `v_gamemode` ainda ligado (modo `$88`, demo); o próprio `Level_StartGame` o desliga. Esse é o início da primeira demo: a fase já foi carregada, o cartão de título já passou e o Sonic já foi criado, mas ele ainda não rodou.

Como o ponto final é achado, sem ler binário:
1. Por rótulo: `oraculo rodar --parar-em Level_StartGame` no caso `boot-demos` (o mapa `refs/rotulos.txt` dá o endereço). A saída traz o quadro (`N_fim`) e o `mclk`. Só números.
2. Por estado: o perfil por quadro mostra o modo `$88` de 933 a 1166 e modo 8 a partir de 1167. O bloco 1 confirmou: `N_fim` = 1166 (`Level_StartGame` no quadro 1166, modo 8 no 1167). O patch 0006 dá o instante exato.

Por que parar aí e não no primeiro `Level_MainLoop`: o laço principal da fase roda o objeto do Sonic, a física e os anéis, que são a Etapa 9. Parar em `Level_StartGame` cumpre "início da primeira demo" sem puxar a Etapa 9. O quadro seguinte, que já roda o Sonic, não entra.

## 2. Reuso (o que já existe e como se combina)

| Peça | Origem | Uso na Etapa 7 |
| --- | --- | --- |
| `Memoria`, portas, saídas de hardware | Etapa 5 | RAM espelhada, E/S, VDP/PSG/YM como `Saida` |
| `Nucleo::executar_quadro`, pontos de checagem, `TempoGravado` | Etapa 5 | A abertura vira passos do laço de quadro. Interrupções chegam nos pontos gravados |
| Descompressores Nemesis, Enigma e Kosinski | Etapa 3 | Arte, mapas e blocos da Sega, do título e da Green Hill, lidos da ROM em tempo de execução |
| `sonic1-video` (renderizador) | Etapa 4 | Recebe as escritas do núcleo e devolve a imagem de 320x224 em 9 bits |
| `Driver` de som, máquina do Z80, voz SEGA, DAC | Etapa 6 | Chamado pelo tratador de VBlank como no original. A voz SEGA roda no Z80 do núcleo |
| `sonic1-fidelidade`, `relatorio`, manifesto | Etapas 1 e 2 | Formato do trace, comparador, manifesto assinado |
| `diferencial-som/` (padrão de harness) | Etapa 6 | Molde do novo `diferencial-abertura/` |
| `sonic1-rom`, site, `sonic1-web` | Etapa 0 | Identificação da ROM e casca do navegador |

**Como se combinam.** Cada modo do jogo (`GM_Sega`, `GM_Title`, `GM_Level`) é uma máquina de estados. Cada passo da máquina é o trecho entre duas chamadas de `WaitForVBlank` do original. O laço de quadro faz, por quadro: aplica a entrada, roda o passo do modo até a espera do VBlank, entrega a interrupção no ponto gravado, roda o tratador de VBlank (`VBlank` e a variante do modo, indexada por `v_vbla_routine`) e fecha o quadro. Isto é infraestrutura e não tem par no original (permitido pelo requisito). Cada decisão de lógica dentro dos passos usa `decisao!` e cita o rótulo.

**Onde mora.** `crates/core/src/abertura/` (módulos `boot`, `vblank`, `sega`, `titulo`, `fase_demo`, `objetos`, `paleta`, `plc`, `joypad`). Dentro do crate `core` para os lints e o CI já valerem sem mexer em arquivo congelado (D7.2).

## 3. O que o oráculo precisa fornecer a mais

Hoje o `.s1tempo` tem interrupções, leituras de hardware, esperas do Z80 e pontos do Z80. Falta, para a Etapa 7:

1. **Marcas de rótulo** (patch 0006, `.s1tempo` versão 3): lista de rótulos dada por arquivo; o oráculo grava `mclk`, linha e coluna na primeira vez, e em toda vez, que o PC chega a cada um. É o "fim de cada laço de espera" do requisito (verificação da ROM, voz SEGA, esperas de DMA) e a base do experimento do nível 2.
2. **Ponto de comparação por quadro** (D7.4). O trace congelado hasheia a RAM no fechamento do quadro (linha 232). Esse instante cai no meio do tratador de VBlank, onde o 68000 pode estar dentro do driver de som. O núcleo não tem ciclo por instrução no nível 1, então não consegue parar ali. O patch 0006 grava também, por quadro, o hash da RAM do 68000 e da RAM do Z80 no instante em que o tratador de VBlank termina (o RTE do nível 6), mais o PC no fechamento do quadro. A comparação usa os dois: o ponto "pós-VBlank" em todos os quadros, e o ponto de fechamento só nos quadros em que o PC de fechamento está fora de qualquer tratador (o relatório diz quantos são).
3. **Posição das escritas de vídeo** (D7.3). O trace já tem linha e coluna de cada escrita da VDP. Sem custo em ciclos, o núcleo não sabe a coluna das escritas que faz. No nível 1 a posição da n-ésima escrita do quadro vem do trace do oráculo (tempo gravado), e o conteúdo e a ordem têm de ser iguais. O arquivo `.s1tempo` v3 leva essas posições (tipo 5), sem o valor escrito. No nível 2 isso é substituído pela previsão.
4. **Despejo sob demanda.** `--despejar-ram` existe, mas só no fim da execução. O patch 0006 aceita `--despejar-em QUADRO:PONTO` (RAM do 68000, RAM do Z80, VRAM, CRAM, VSRAM) para localizar o campo divergente. Os despejos ficam em `privado/`, fora do git e fora da vista da IA.

Pedidos de patch: `docs/pedidos/etapa7-oraculo.patch` (itens 1 a 4) e `docs/pedidos/etapa7-relatorio.patch` (manifesto com `diferencial-som/` e `diferencial-abertura/`, comando `abertura`, pendência do backlog da Etapa 6). Ambos com o comando de aplicar no mesmo bloco.

## 4. Comparação

Comparador novo: `diferencial-abertura/` (workspace GPLv3 próprio, como `diferencial-som/`, nunca no build do jogo). Roda só pelo Marcelo, com a ROM, fora da IA (D1).

Por quadro, do 0 a `N_fim`, o relatório confere:

| Campo | Como |
| --- | --- |
| RAM do 68000 | hash no ponto pós-VBlank (todos os quadros) e no fechamento (quadros com PC fora do tratador). Máscara: só a pilha (`FFFC00` a `FFFFFF`), exclusão **proposta** em `cobertura/exclusoes-propostas.txt`. Precisa do "aprovado" do Marcelo (ver 9) |
| RAM do Z80 | hash no ponto pós-VBlank, menos as posições `1FFD` e `1FFF` (decisão da Etapa 6) |
| Escritas de vídeo | lista do quadro: porta, valor, tamanho, na ordem. Linha e coluna: as do trace (D7.3). Hash por quadro |
| Escritas de som | YM2612 (inclui DAC) e PSG: fonte, porta, valor, na ordem. O ciclo de cada uma fica para o nível 2 |
| Imagem | hash da imagem de 320x224 em 9 bits, `sonic1-video` alimentado só pelas escritas do núcleo |
| VRAM, CRAM, VSRAM | hash por quadro (a imagem pode esconder diferença) |

**Relatório.** Uma linha por quadro com hashes e veredito `IGUAL` ou `DIVERGE`. No primeiro quadro `DIVERGE` imprime: o campo, e para a RAM o nome e o endereço da variável (pelo mapa de rótulos), o tamanho e a contagem de bytes diferentes. **Nunca imprime valores, pixels, bytes nem áudio.** Para ir fundo, o harness grava o despejo em `privado/` e o Marcelo roda o diagnóstico local. Critério: zero `DIVERGE` do quadro 0 a `N_fim`.

## 5. Navegador

- Página: seletor de ROM (já existe). Valida o SHA-1 (já existe) e, só se for a REV01, cria o console. Recusa o resto. A ROM nunca sai da máquina e nunca é guardada.
- `crates/web`: `Console` em wasm com `novo(rom)`, `quadro(botoes)` (roda um quadro, devolve o ponteiro da imagem em 9 bits já convertida para RGBA, mais as escritas de som do quadro) e `hash_do_quadro()`. Sem lógica própria: tudo no Rust.
- Canvas 320x224 com escala inteira (`image-rendering: pixelated`, maior fator inteiro que cabe na janela). `requestAnimationFrame` a 60 Hz por contador de tempo, no máximo um quadro por chamada, sem acumular atraso.
- **Tempo no navegador (D7.5).** Não há tempo gravado. O núcleo usa o tempo nominal: o VBlank chega quando o laço principal espera, sem quadros de lag. A duração dos laços de espera vem de constantes de quadros medidas no oráculo (pelas marcas) e registradas como provisórias, até o modelo do nível 2 (experimento 7.8 e Etapa 15). Consequência dita na página: "tempo aproximado, sem slowdown".
- **Som (D7.6).** Silêncio, claramente marcado na página como "som: Etapa 8". O núcleo já gera as escritas nos chips; só não sintetiza. É a opção mais simples e honesta. Nada de síntese improvisada.
- Entrada: teclado (setas, A/S/D, Enter). Mapa de teclas em quatro linhas de JS, comentado.
- **Prova sem o Marcelo olhar.** (a) Os testes de `crates/web` rodam em `wasm32-wasip1` no wasmtime, com ROM sintética, pelas mesmas funções que o JavaScript chama: o quadro tem 320x224, é determinístico, nunca entra em pânico e o hash por quadro é igual ao do nativo. (b) A prova com a ROM real: o mesmo caminho compilado para wasm e rodado no wasmtime lê a ROM por um diretório pré-aberto (`--dir`) e imprime o relatório de hashes por quadro, que o `diferencial-abertura` compara com o do nativo e com o do oráculo. (c) O JavaScript é só cola e fica fora da prova automática. Se o Marcelo quiser ver, abre o site e escolhe a ROM. Não é exigido.

## 6. Experimento do nível 2 (não trava a etapa)

**Pergunta.** Um modelo que só lê o código do 68000 na ROM (sem executar) e soma custos prevê (a) o ciclo em que termina a espera da voz SEGA e (b) a duração do boot?

**Marcas do oráculo (ciclos em mclk, divididos por 7 para ciclos do 68000):** `GameProgram`, `CheckSumOk`, `GameInit`, `GM_Sega`, `PlaySegaSound` (entrada) e `Sega_WaitEnd` (primeira chegada, depois da voz). O bloco 1 já mede cada uma (primeira chegada). Duração do boot = marca `GM_Sega` menos o quadro 0. Espera da voz = primeira chegada em `Sega_WaitEnd` menos a de `PlaySegaSound` (o bloco 1 mediu 134 quadros). Correção: a marca `Sega_WaitPal` vem antes da voz e não serve para isso.

**Modelo.** Módulo `crates/core/src/nivel2/`: decodifica o laço lendo os bytes da ROM carregada (só tamanhos de instrução e modos de endereçamento, nunca executa), soma a tabela de ciclos do manual do 68000 (Motorola M68000 Family Programmer's Reference Manual, tabelas de tempo) e soma o que o hardware acrescenta: espera por acesso do Z80 à ROM durante a voz, atualização da memória (refresh) e DMA. As voltas do laço vêm do valor inicial do contador, lido da ROM. O custo de cada trecho é calculado uma vez no carregamento.

**Métrica.** Erro em ciclos do 68000: `previsto - medido`, e o erro relativo em ppm, por marca. Formato do relatório (uma linha por marca, só números): `marca<TAB>medido_ciclos<TAB>previsto_ciclos<TAB>erro_ciclos<TAB>erro_ppm`. Comparador: `xtask nivel2-experimento` (somente o Marcelo roda com a ROM). Teste sintético: um laço conhecido com contagem exata, e um modelo independente de uma linha.

**Resultado.** Entra em `docs/conhecimento/abertura.md` mesmo que o erro seja grande. Se o erro passar de 0,1% ou o modelo não fechar, vira linha de BACKLOG para a Etapa 15 (dono: IA). Nunca é BLOQUEIA.

## 7. Estratégia de teste sem ROM

- **Vetores sintéticos por rotina.** Cada rotina do núcleo tem um teste com memória sintética e saída esperada calculada à mão a partir de comportamento documentado (manual do 68000 para as operações, Sega Retro para a lógica pública, com a fonte citada no teste). Exemplo: a soma de verificação sobre uma ROM de 4 palavras, o fade de paleta sobre 8 cores inventadas, o preenchimento de VRAM sobre uma tela inventada.
- **ROM sintética de abertura.** Escrita por nós no padrão de `rom_sintetica.rs`, mínima, para o laço de quadro, o boot e o canvas do navegador não dependerem da ROM real.
- **Propriedades.** Determinismo (duas execuções, mesmos hashes), nunca entra em pânico com ROM de bytes aleatórios e com entrada aleatória, pilha de chamadas das máquinas de estado sempre termina em um modo conhecido, escritas de vídeo sempre dentro do inventário.
- **Modelos independentes.** Soma de verificação e fade de paleta têm um gabarito em fórmula direta, escrito em outro arquivo, comparado ao núcleo em milhares de entradas aleatórias com semente fixa.
- **Oráculo sobre ROM sintética.** Onde o oráculo roda uma ROM sintética que chama a mesma rotina (como na Etapa 6), `diferencial-abertura/tests/` compara núcleo e oráculo. A Marcelo roda só o que precisa da ROM real.

## 8. Sub-etapas

Quem roda: **IA** (sem a ROM, no repositório) ou **Marcelo** (com a ROM, fora da IA).

| # | Entrega | Prova (comando, resultado esperado) | Quem |
| --- | --- | --- | --- |
| 7.0 | Plano, patch 0006 do oráculo e do relatório, `diferencial-abertura/` esqueleto, laço de quadro com modos e tratadores de VBlank por índice | `sh oracle/scripts/testa.sh`: testes antigos + os novos de marcas, ponto pós-VBlank e posição de escrita passam. `cargo test -p sonic1-core abertura::laco`: ok | IA (testes), Marcelo (aplica o patch, bloco 2) |
| 7.1 | **Boot e inicialização do console**: `EntryPoint`, `SkipSetup`, `GameProgram`, `GameInit`, `VDPSetupGame`, `JoypadInit`, `ClearScreen`, `DACDriverLoad`, `MainGameLoop` e `GameModeArray`. O Z80 recebe o programa de partida | testes sintéticos por rotina (RAM zerada, registradores da VDP escritos na ordem, Z80 em reset/liberado). Com ROM: comparação do quadro 0 até `GM_Sega` | IA, Marcelo |
| 7.2 | **Verificação da ROM no boot**: `CheckSumCheck`, `CheckSumOk`, `CheckSumError`. A soma é nativa sobre a ROM do usuário em tempo de execução. A duração vem da marca gravada | modelo independente da soma em 5000 ROMs aleatórias; o fim do laço cai na marca `CheckSumOk` | IA, Marcelo |
| 7.3 | **Tela e voz SEGA**: `GM_Sega`, `PalCycle_Sega`, `PlaySegaSound`, `Sega_WaitPal`, `Sega_WaitEnd`, arte e mapa pelos descompressores, `VBlank_Sega`, `VBlank_SegaPCM`, `PalLoad`, `TilemapToVRAM`. Voz pelo Z80 do núcleo | testes sintéticos de ciclo de paleta; relatório com a ROM do quadro 0 até o fim da Sega: sem `DIVERGE` | IA, Marcelo |
| 7.4 | **Transição para o título**: `Sega_GotoTitle`, fade, `ClearPLC`, `RunPLC`, fila de arte (`AddPLC`, `QuickPLC`, `ProcessPLC`), `PalFadeIn/Out` | testes por rotina da fila de arte; relatório até o início de `GM_Title` | IA, Marcelo |
| 7.5 | **Título e menus**: `GM_Title`, objetos do título, `ExecuteObjects`, `BuildSprites` (genéricos), `Tit_MainLoop`, códigos secretos e seletor de fases (lógica completa; sem entrada no trecho, mas o código está no jogo), `GotoDemo`, `LevSelControls` | vetores sintéticos do motor de objetos e do `BuildSprites`; relatório até o quadro em que `GotoDemo` começa | IA, Marcelo |
| 7.6 | **Fase de demonstração**: `GM_Level` no caminho da demo (cartão de título, `LevelDataLoad`, `LevelSizeLoad`, `DeformLayers` inicial, `ObjPosLoad`, `Level_LoadObj`, paleta, música da fase, atrasos) até `Level_StartGame`. Sem a física do Sonic (Etapa 9) | vetores sintéticos de carga do layout; relatório completo do quadro 0 a `N_fim`: zero `DIVERGE`. **É o critério da etapa** | IA, Marcelo |
| 7.7 | **Navegador**: `crates/web` com `Console`, canvas, teclado, página marcando som e tempo como provisórios | testes wasm no wasmtime (ROM sintética); relatório de hashes com a ROM real pelo caminho wasm igual ao nativo | IA, Marcelo |
| 7.8 | **Experimento do nível 2** | `cargo xtask nivel2-experimento` com a ROM: tabela de erro por marca | IA (código), Marcelo (roda) |
| 7.9 | Fechamento: `estado.md`, `abertura.md`, índice, backlog, manifesto assinado, resumo | revisão só do que a Etapa 7 mudou | IA, Marcelo (assina) |

Entre as sub-etapas a IA não para: implementa, prova com dados sintéticos, atualiza docs, faz commit local e segue. Os blocos de comandos juntam as comparações reais:
- **Bloco 1** (já): perfil do trecho, marcas do oráculo, estado do ambiente. Decide `N_fim` e dá os dados do experimento.
- **Bloco 2** (depois de 7.1 a 7.3 prontas): aplicar os patches, rodar `testa.sh`, gerar os traces com o patch 0006 e rodar o relatório até o fim da Sega.
- **Bloco 3** (depois de 7.4 a 7.6): relatório completo até `N_fim`, mais a prova wasm.
- **Bloco 4** (depois de 7.7 e 7.8): relatório do navegador, experimento, manifesto e assinatura.

## 9. Decisões, riscos e o que não será feito

**Decisões (D7.x), com o motivo.**
- **D7.1.** `N_fim` é `Level_StartGame` e não o primeiro `Level_MainLoop`. Motivo: o quadro seguinte exige a física do Sonic (Etapa 9). Cumpre "início da primeira demo".
- **D7.2.** O código da abertura fica em `sonic1-core`. Motivo: lints e CI já cobrem; evita editar `.github/` e `checa-core.sh` (congelados).
- **D7.3.** No nível 1, linha e coluna das escritas de vídeo vêm do trace do oráculo, com conteúdo e ordem conferidos. Motivo: sem custo em ciclos o núcleo não sabe a coluna. O requisito lista, como tempo gravado, quadros de lag, interrupções, fins de espera e esperas do Z80. Isto **amplia** essa lista com a posição das escritas de vídeo. Não muda nenhum critério de pronto e é substituído pela previsão na Etapa 15. Marcado para o Marcelo.
- **D7.4.** Dois pontos de comparação por quadro (pós-VBlank e fechamento). Motivo: o fechamento do trace congelado cai no meio do tratador de VBlank. O trace congelado não muda. Marcado para o Marcelo.
- **D7.5.** O navegador usa tempo nominal, sem lag, com durações de espera provisórias. Motivo: não há oráculo no navegador. O requisito já aceita "sem slowdown como limitação temporária".
- **D7.6.** Som do navegador: silêncio marcado. Síntese é a Etapa 8.
- **D7.7.** A pilha é mascarada na comparação da RAM do 68000. O núcleo não guarda registradores nem a pilha de chamadas das rotinas nativas. Depende do "aprovado" da exclusão (backlog da Etapa 5).
- **D7.8.** Cada modo é uma máquina de estados com um passo por `WaitForVBlank`. Rust estável não tem corrotinas; a máquina explícita é legível e testável.

**Riscos.**
- Tamanho: a Etapa 7 reimplementa boot, Sega, título, motor de objetos genérico e a carga da Green Hill. É a maior até aqui. Mitigação: sub-etapas pequenas, relatório de comparação já a partir da 7.3.
- Primeiro quadro com vídeo no meio do tratador de VBlank pode exigir mais pontos de checagem. Mitigação: D7.4.
- O oráculo adia as escritas do 68000 (`pending_writes`) e fecha o quadro na instrução que cruza a linha 232 (Etapas 4 e 5). O patch 0006 observa em `write_byte` e `write_word`, como nas etapas anteriores.
- Desempenho do renderizador no wasm (~1 ms nativo, sem folga). Mitigação: medir na 7.7; otimização em BACKLOG se passar de 16 ms por quadro no total.
- Estado inicial do console real é indefinido; o oráculo começa em zero (R5 da Etapa 1). O núcleo faz igual.

**Não será feito.** Jogabilidade das zonas, física do Sonic, objetos de fase além dos necessários até `Level_StartGame`, síntese de áudio (Etapa 8), gamepad (Etapa 13), special stage, créditos, finais, tela larga, filtros, pausa e debug em jogo, guardar a ROM no navegador.

**Itens que exigem arquivo congelado.**
1. `oracle/` (patch 0006): `docs/pedidos/etapa7-oraculo.patch` + `git apply` + `sh oracle/scripts/testa.sh`.
2. `crates/fidelidade/relatorio/` e manifesto: `docs/pedidos/etapa7-relatorio.patch`.
3. CI e site: o job `verificar` já roda `fmt`, `clippy` e `test` nativo e wasm no workspace principal, e o `publicar` monta `site/` com `cargo xtask`. Se o crate `web` ganhar dependência nova (apenas `sonic1-core` e `sonic1-video`), o fluxo não muda. Se a 7.7 mostrar que o workflow precisa de passo novo, o patch pronto vai em `docs/pedidos/etapa7-ci.patch` no bloco 4. Estimativa: não precisa.

**Aprovação pedida junto com o bloco 1.** Responder "aprovado" cobre: este plano, D7.3 e D7.4, e a exclusão da pilha na comparação (D7.7, `cobertura/exclusoes-propostas.txt`). Se o Marcelo não aprovar a exclusão da pilha, o critério de zero divergência na RAM do 68000 não pode ser cumprido pelo núcleo atual (ele não guarda a pilha de chamadas) e a Etapa 7 pararia na 7.1.

## 10. Ajustes depois do bloco 1 (08/11/2026)

- **Voz SEGA sem RTE.** `PlaySegaSound` roda dentro do tratador de VBlank por cerca de 134 quadros. Nesses quadros não há ponto pós-VBlank. A comparação usa só o fechamento, onde o PC está no laço da voz e o estado é estável. O relatório separa os quadros por tipo de ponto.
- **Marcas.** A parada em rótulo do oráculo dá o quadro, não o instante. O patch 0006 grava o `mclk` exato, linha e coluna. Os números do bloco 1 estão em `docs/conhecimento/abertura.md`.
- **Marca da voz do experimento:** `PlaySegaSound` e `Sega_WaitEnd`. A previsão ingênua (18 por 65536 voltas de 14 ciclos, mais o excesso do Z80 e do refresh) é o que o modelo precisa acertar.
