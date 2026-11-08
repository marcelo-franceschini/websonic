# Plano da Etapa 6: Driver de som

Escrito em 08/11/2026. Aguarda a palavra "aprovado" do Marcelo, junto com os resultados do primeiro bloco de comandos.
Fontes lidas: `CLAUDE.md`, `docs/requisitos.md` (sem conflito achado), `docs/estado.md`, `docs/backlog.md`, `docs/conhecimento/` (oráculo, nucleo, corpus, descompressao), `docs/inventario-hardware.md`, `docs/inventario-barramento.md`, `refs/rotulos.txt`, e os `.asm` de texto do som só para ler nomes de rótulos e a estrutura (nada copiado).

Modo de trabalho pedido pelo Marcelo: a IA decide sozinha. Cada decisão abaixo tem o motivo e vai para `docs/conhecimento/som.md` na 6.7.

## 0. Pré-condições

- Etapa 3: concluída (185 de 185 blocos IGUAL, `docs/conhecimento/descompressao.md`). O Z80 do jogo é carregado por `DACDriverLoad` com Kosinski, e o descompressor já está provado.
- Etapa 5: concluída do lado da IA e do Marcelo (blocos A, B, B2 e assinatura feitos, commits `577675f` e `37c187c`). O cabeçalho da seção da Etapa 5 em `docs/estado.md` estava defasado ("faltam os blocos A e B"). A 6.0 corrige o texto. Isso não conflita com o requisito. Pendências da Etapa 5 que não bloqueiam esta: aprovar as duas exclusões propostas e a sessão de revisão.
- Congelados que a Etapa 6 não edita: `oracle/`, `oracle2/`, `crates/corpus/`, `crates/fidelidade/` (inclui `relatorio`), `refs/`, `hooks/`, `.github/`, `.devcontainer/`, `.cargo/`, os testes, traces, exclusões e bugs injetados já congelados. Mudança só por patch pronto, no mesmo bloco de comandos (seção 11).
- Livres: `crates/core`, `crates/teste-nucleo`, `diferencial/` (se não foi congelado no patch da Etapa 5; ver o resultado do bloco 1), `xtask`, `cobertura/` (arquivos novos), `docs/`, `scripts/`.

## 1. Como o som do Sonic 1 REV01 funciona (explicado com palavras próprias)

Tudo abaixo vem dos `.asm` de texto (só leitura de estrutura e de nomes) e do `refs/rotulos.txt`. A prova de cada ponto é o oráculo, nas sub-etapas indicadas.

- **O driver de música roda no 68000.** É uma variante do driver SMPS do 68000. Uma rotina (`UpdateMusic`) roda uma vez por VBlank, dentro da interrupção de nível 6. Ela lê uma fila de três bytes (`v_soundqueue0`, `v_soundqueue1`, `v_soundqueue2`), toca o pedido, e depois atualiza as trilhas: uma de DAC, seis de FM, três de PSG, mais as trilhas de efeitos (três de FM, três de PSG) e a trilha especial (uma de FM, uma de PSG). Cada trilha é uma estrutura na RAM com o mesmo layout.
- **O 68000 escreve direto nos chips.** Ele escreve no YM2612 pela janela do Z80 (`A04000` a `A04003`), depois de ler o bit de ocupado no mesmo endereço. Escreve no PSG em `C00011`. Os laços de espera do bit de ocupado (`WriteFMI.waitloop` e `WriteFMII.waitloop`) são laços de espera do tempo gravado.
- **O Z80 só toca a bateria e a voz SEGA.** O programa do Z80 (arquivo `sound/z80.asm`, pequeno) fica num laço: espera um número de amostra na posição `zDAC_Sample` da RAM do Z80, toca amostras DPCM (bateria) ou a voz SEGA em PCM, e escreve cada amostra no registrador `2A` do YM2612. A voz SEGA lê a ROM pela janela de banco do Z80 (registrador de banco com 9 bits, escrito um bit por vez em `6000`).
- **O 68000 negocia o barramento com o Z80** a cada quadro: pede o barramento (`stopZ80`), espera o BUSACK, lê o byte de estado `zDAC_Status` na RAM do Z80. Se a bateria está no meio de uma escrita no YM, solta o Z80 e tenta de novo. Isso é o "retry" dos requisitos.
- **A voz SEGA** (`PlaySegaSound`) escreve o número da amostra, solta o Z80 e fica num laço de cerca de 2 segundos, dentro da própria `UpdateMusic`. O Z80 lê a ROM pela janela de banco nesse tempo, e cada leitura de ROM do Z80 para o 68000 (eventos `espera_acumulada` com causa 2).
- **Existem dois programas de Z80:** (A) o código de partida (`SetupValues_Z80` em `sonic.asm`), curto, que limpa a RAM do Z80 com uma cópia em bloco (LDIR) e fica num salto para si mesmo; (B) o driver de DAC (`sound/z80.asm`), carregado por `DACDriverLoad` com Kosinski.
- **IDs de som** (nomes dos arquivos e rótulos `ptr_*`, sem dados): música `81` a `93` (19), efeitos `A0` a `CF` (48), especial `D0` (1), sinalizadores `E0` a `E4` (5: fade, voz SEGA, velocidade, lento, parar tudo), `80` (nada). Total: 73 IDs válidos, mais os inválidos de cada faixa para cobrir os desvios da fila.

## 2. Decisões da IA (nenhuma pergunta)

| # | Decisão | Motivo |
| --- | --- | --- |
| D6.1 | O tempo gravado do Z80 (nível 1) são **pontos do Z80**: a cada acesso do 68000 que toca o Z80 (BUSREQ, RESET, janela da RAM do Z80), o oráculo grava os ciclos do Z80 executados desde o ponto anterior, o PC, os ciclos que faltam da instrução em curso e o registrador de banco. O núcleo avança a máquina do Z80 por esses ciclos e confere a posição. | É a regra do requisito ("esperas pelo Z80" gravadas no nível 1). Confere por construção: se o custo de um trecho estiver errado, a posição não bate e o relatório acusa. No nível 2 os ciclos viram previstos pelo núcleo (Etapa 15). |
| D6.2 | A máquina do Z80 tem um **trecho por instrução**, como o oráculo: o efeito acontece no início do trecho e o custo é consumido depois. Descoberta no código do oráculo (`cpu/z80-emu/src/core.rs`, `tick`): a instrução executa inteira no primeiro ciclo e os `custo-1` ciclos seguintes só esperam. | Dá a mesma ordem e o mesmo ponto de escrita do oráculo sem interpretar opcode. O custo vem da tabela pública de tempos do Z80 (Zilog UM0080), por trecho, citada. |
| D6.3 | O estado da máquina é um **enum de trechos mais campos nomeados pelo propósito** (`posicao_da_amostra`, `restante`, `passo_do_pitch`, `acumulador_pcm`), nunca um banco de registradores nem nomes `a`, `b`, `de`. | Cumpre "sem registradores como estado" e continua legível. |
| D6.4 | Os dados do programa B (tabela de decodificação, tabela de amostras, bytes das amostras) **não entram no código**: a máquina os lê da RAM do Z80 espelhada, que o 68000 enche pela rotina de carga (Kosinski, já provada). A estrutura do programa (a ordem e o custo dos trechos) é lógica, como o código do 68000. | Regra "Origem dos dados". |
| D6.5 | A comparação do nível 1 é por **fonte e por ordem global**. Cada evento de som do núcleo tem fonte (68000 ou Z80), tipo (YM porta 0 a 3, PSG, DAC), valor e registrador. O relatório compara a sequência global na ordem de emissão. A ordem global do núcleo vem da estrutura: o Z80 só escreve nas janelas em que o 68000 não segura o barramento, e a máquina é avançada nos pontos gravados, antes do acesso. | O oráculo atualiza o Z80 depois de cada instrução do 68000; os pontos gravados valem no momento do acesso, com o Z80 posto no instante anterior. |
| D6.6 | O harness do som roda o **jogo real no oráculo** (a ROM do Marcelo), a partir de um estado salvo calmo (tela de título ou uma fase sem som novo), injeta o ID pela fila e deixa o jogo chamar `UpdateMusic` sozinho, quadro a quadro. O oráculo para na entrada de `UpdateMusic` e no retorno. O núcleo roda a mesma função uma vez por quadro, **contínuo** (o estado do núcleo evolui sozinho; não é reinjetado a cada quadro). Escritas na região do driver feitas pelo resto do jogo entre duas chamadas entram no núcleo como **entradas externas** registradas e contadas (esperado: só o ID injetado). | Prova o estado quadro a quadro de verdade, sem depender de ser a mesma função sozinha. O jogo todo ainda não existe no núcleo (Etapas 7 a 12), então o resto do quadro é do oráculo. |
| D6.7 | A resposta ao pedido "como cada música e efeito é disparado" é: o mesmo byte na mesma posição da fila nos dois lados, pela **rotina `QueueSound1`, `QueueSound2` ou `QueueSound3`** (chamada no oráculo por `chamar_rotina` e no núcleo pela função equivalente, comparadas pelo harness da Etapa 5). | Cobre também as três rotinas de fila, incluindo a terceira (bug conhecido). |
| D6.8 | A decisão de **quantos quadros** tocar por ID: até todas as trilhas de efeito pararem (efeitos) ou 2400 quadros (música, 40 s), o que vier primeiro. O limite é configurável e vai no relatório. | Música em laço nunca para; 40 s cobre o laço inicial e várias trocas de nota, e tem de passar por fade e pausa nos casos específicos. Casos de fade, pausa e velocidade usam roteiros próprios (seção 6). |
| D6.9 | A voz SEGA é testada com o ID `E1` como qualquer outro, mas o relatório separa a janela de cerca de 2 s: a comparação é feita na chamada que contém o laço (um registro só, com milhares de escritas do DAC). | Evita 125 registros quase iguais. |
| D6.10 | Os 68000 e Z80 só são comparados no que o requisito pede: **conteúdo e ordem das escritas, e o estado do driver**. O ciclo de cada escrita é do nível 2 (Etapa 15). Nada de DAC é sintetizado aqui. | Escopo da etapa. |

## 3. Modelo do barramento e das esperas (item 2 do pedido)

O que o núcleo passa a modelar, em `crates/core/src/som/barramento_z80.rs` (a `Memoria` da Etapa 5 já tem BUSREQ, RESET, RAM do Z80 e a janela; faltam o registrador de banco, o BUSACK vindo do estado e os pontos):

| Parte | Regra | Fonte e prova |
| --- | --- | --- |
| BUSREQ e RESET | Dois bits escritos pelo 68000 em `A11100` e `A11200`. BUSACK lido é verdadeiro quando BUSREQ está pedido e o Z80 não está em reset. No oráculo isso vale na hora, sem esperar o fim da instrução do Z80 (`bus.rs`, `busack()`). | Leitura do código do oráculo e teste da 6.1 (modelo independente, seção 8). |
| Z80 sem barramento | Com BUSACK falso, o 68000 não alcança a RAM do Z80: leitura devolve barramento aberto, escrita é ignorada (já na Etapa 5). | `nucleo.md` (5.1), `diferencial/tests/memoria.rs`. |
| Janela de banco | Registrador de 9 bits. Cada escrita do Z80 em `6000` empurra o bit 0 pela esquerda (desloca para a direita e põe o bit no topo). O endereço do 68000 lido pelo Z80 em `8000` a `FFFF` é `banco << 15` mais o deslocamento de 15 bits. | `bus.rs` (`Z80BankRegister`); teste: nove escritas formam o banco esperado (6.1). |
| Leitura de ROM pelo Z80 | Custa 3 ciclos do Z80 extras ao Z80 e uma parada do 68000 (causa 2 de `espera_acumulada`). Só a parada gravada vale no nível 1. | `timing.rs` do oráculo, inventário de barramento. |
| Parada do 68000 ao acessar a RAM do Z80 | 1 ciclo do 68000 por acesso (causa 1). Gravada. | idem |
| YM2612 | O 68000 e o Z80 escrevem pelas mesmas portas. O bit 7 do estado (ocupado) lido pelo 68000 vem do tempo gravado (`ClasseLeitura::YmStatus`, já existe). | Etapa 5. |
| Reset do Z80 | RESET ligado zera o PC do Z80 e a máquina volta ao trecho inicial; o oráculo também reinicia o YM2612 ao ligar o reset. | `bus.rs` (`set_z80_reset`); teste na 6.1. |
| DMA da VDP | O Z80 para durante DMA e leitura de ROM. Já incluído nos ciclos ativos gravados. | `timing.rs`. |

**Nível 1 (esta etapa):** conteúdo e ordem das escritas e o estado, com os pontos do Z80 gravados.
**Nível 2 (Etapa 15):** o núcleo prevê os ciclos ativos do Z80 entre pontos, o custo do 68000 e a parada por leitura de ROM. A tabela de custo por trecho do Z80 já nasce nesta etapa (D6.2), então a Etapa 15 só soma o lado do 68000.

## 4. Inventário do que o oráculo registra de som (item 3 do pedido)

O que o trace da Etapa 1 e o `.s1tempo` da Etapa 5 já têm:

| Dado | Onde | Observação |
| --- | --- | --- |
| Escritas no YM2612 (porta, valor, registrador, bit de DAC, origem 68000 ou Z80, mclk, seq) | trace, tipo 6 | já |
| Escritas no PSG (valor, origem, mclk) | trace, tipo 7 | já |
| BUSREQ e RESET (sinal, ativo) | trace, tipo 8 | já |
| BUSACK lido | trace, tipo 11, e tempo (classe 0) | já |
| Espera acumulada e consumida | trace, tipos 9 e 10, e tempo (tipo 3) | já |
| Leitura do estado do YM2612 | tempo (classe 2) | já |
| Desvios do Z80 (PC, destino, classe, tomado) | trace, tipo 4 | já; base da cobertura do Z80 |
| Hash da RAM do Z80 por quadro | trace, categoria `ram_z80` | já |

**O que falta, e o patch 0005 (seção 11, P6-1):**

1. **Pontos do Z80** no `.s1tempo` (versão 2): tipo 4, um por acesso do 68000 que toca o Z80. Campos: tipo do acesso (BUSREQ, RESET, leitura da janela, escrita da janela), PC do Z80, ciclos que faltam da instrução em curso, ciclos ativos executados desde o ponto anterior, registrador de banco (9 bits), e para a janela o endereço (13 bits) e o byte. Motivo: o Z80 avança em ciclos e o núcleo precisa saber quantos, e onde está, a cada acesso. Sem isso, nenhum dado da sequência do DAC em cada janela é reproduzível no nível 1.
2. **Acessor do ciclo restante** no núcleo do Z80 do oráculo (`t_cycles_wait`), só leitura. O `pc()` já existe.
3. **Registro do banco** nos pontos (o `Z80BankRegister` já existe no barramento).

Nada na lógica de emulação muda. O formato do trace (`crates/fidelidade`, congelado) **não** muda. Só o arquivo de tempo, que é do núcleo, ganha a versão 2 (o leitor aceita 1 e 2).

Para o bloco 1, a IA entrega também `cargo xtask inventario-som`, que lê traces e tempos e escreve `docs/inventario-som.md`: escritas do YM e do PSG por origem e por registrador, quantidade de escritas de DAC por quadro, pontos do Z80 por tipo, PCs distintos do Z80 em cada ponto, desvios distintos do Z80, e os valores distintos do registrador de banco. Isso fecha o desenho do programa do Z80 com dados, sem nenhum byte da ROM.

## 5. Estado do driver e como é comparado (item 4 do pedido)

**Estado do driver, por memória:**
- RAM do 68000, região `v_snddriver_ram` até o fim (rótulos `v_snddriver_ram.*`, `SMPS_RAM`, `SMPS_Track`, no `refs/rotulos.txt`): fila, prioridade, tempo, fade, trilhas de música (10), de efeitos (6) e especial (2), cópia da música do 1UP (`v_1up_ram_copy`).
- RAM do Z80 inteira (8 KiB), dividida em: programa, tabelas, `zDAC_Status`, `zDAC_Sample`, pilha.
- Registrador de banco do Z80 e posição da máquina do Z80.

**Comparação quadro a quadro, em cada chamada de `UpdateMusic`:**
1. **Região do driver na RAM do 68000:** byte a byte, depois da chamada. Qualquer diferença é divergência, com o endereço e o nome do campo (pelo mapa de rótulos).
2. **RAM do Z80:** hash por quadro (como o trace) e, na divergência, a lista dos deslocamentos diferentes.
3. **Banco e posição do Z80:** igual ao último ponto gravado.
4. **Sequência de escritas de som:** a lista do núcleo (fonte, tipo, porta, registrador, valor) contra a do oráculo, na ordem. Primeiro índice divergente, números de escritas esperados e obtidos.
5. **Entradas externas:** contagem de escritas feitas por fora do driver entre duas chamadas (deve ser igual ao número de IDs injetados).

A pilha da chamada é a exclusão proposta da Etapa 5 (`cobertura/exclusoes-propostas.txt`); a mesma regra vale aqui e só vale com a aprovação do Marcelo. Se não aprovar, o harness do som compara a pilha também (a região do driver não contém a pilha, então o efeito é nulo na prática).

## 6. Enumeração dos IDs e roteiros (item 5 do pedido)

`crates/teste-som/ids.txt` (metadado, só ID e nome do arquivo de texto do disassembly, sem dados): `80` nada; `81` a `93` músicas (GHZ, LZ, MZ, SLZ, SYZ, SBZ, invencibilidade, vida extra, special stage, título, final, chefe, FZ, fim de fase, game over, continue, créditos, afogando, esmeralda); `A0` a `CF` efeitos; `D0` especial; `E0` a `E4` sinalizadores. A lista sai do `refs/rotulos.txt` (`ptr_mus*`, `ptr_snd*`, `ptr_flg*`) por `cargo xtask ids-som`, e o teste confere que o número bate com os `ptr_*end`.

**Roteiros por ID (todos com a mesma entrada nos dois lados):**
- Cada ID válido, sozinho, a partir do mesmo estado calmo. 73 casos.
- Cada ID inválido por faixa (`00`, `7F`, `94`, `9F`, `D1`, `DF`, `E5`, `FF`): cobre os desvios da validação.
- Prioridade: efeito de prioridade baixa durante efeito de prioridade alta e o contrário (pares escolhidos pela tabela `SoundPriorities`, lida em tempo de execução).
- Sobreposição de música e efeito no mesmo canal de FM e de PSG (o efeito toma a trilha e a música a recupera).
- Fade (`E0`), velocidade (`E2`, `E3`), parar tudo (`E4`), pausa pela variável `f_pausemusic`, vida extra com retomada da música (`88` durante `81` a `86`), afogamento (`92` seguido de troca).
- Fila: `QueueSound3` (o bug conhecido), dois IDs no mesmo quadro, ID repetido.
- Voz SEGA (`E1`) e bateria (comandos de DAC dentro das músicas, os três samples).
- Reset do Z80 no meio de uma amostra (roteiro sintético com o oráculo, `chamar_rotina` no código do reset).

## 7. Relatório de comparação (item 6 do pedido)

`diferencial som` imprime, por ID e roteiro: o número de escritas esperadas e obtidas (separadas em YM por porta, DAC, PSG), o hash SHA-1 da sequência esperada e da obtida (cada evento serializado como `fonte,tipo,porta,registrador,valor`, igual dos dois lados), o primeiro índice divergente (ou `-`), os quadros comparados, as entradas externas, e o veredito `IGUAL` ou `DIVERGE`. Nunca bytes de áudio, nunca valores da ROM além do hash. A saída vai para `$SONIC1_EXT_DIR/privado/etapa6/relatorio-som.txt`. O relatório tem uma linha de resumo com contagens e um hash do relatório inteiro. O Marcelo cola de volta só as linhas `DIVERGE` e o resumo. Para o manifesto: o pacote `relatorio` ganha o comando `som` por patch (P6-2), que lê esse arquivo e falha se houver `DIVERGE` ou ID sem linha.

## 8. Estratégia de teste sem ROM (item 8 do pedido)

A IA roda tudo isto no contêiner (aqui, no host), sem ROM, com dados sintéticos:

- **Vetores sintéticos de comandos de som:** uma ROM de brinquedo montada por nós (`crates/teste-som`) com uma "música" mínima: cabeçalho de trilhas, uma nota de FM, um comando de loop, uma nota de PSG, uma troca de voz. As tabelas de ponteiros do driver (`MusicIndex`, `SoundIndex`, voz) são preenchidas por nós. O formato do cabeçalho vem da documentação pública do SMPS (Sonic Retro, "SMPS" e o código do driver sem copiar) e fica registrado em `som.md`. Os vetores comprovam a ordem e o conteúdo das escritas contra valores calculados à mão pelo formato público.
- **Propriedades:** determinismo (duas execuções, mesmos eventos), nunca entra em pânico (qualquer ID, qualquer RAM do driver aleatória com semente fixa, qualquer sequência de pontos do Z80), ordem preservada (a fila de eventos é FIFO e a numeração é estritamente crescente), cada trecho do Z80 consome exatamente o custo, a soma dos ciclos avançados é igual à soma dos pontos, o banco depois de nove escritas é o esperado.
- **Modelo independente simples do barramento:** `crates/teste-som/src/modelo_barramento.rs`, escrito a partir da tabela da seção 3 sem reutilizar o código do núcleo (um autômato de poucas linhas: BUSREQ, RESET, ACK, banco). Cruza com o núcleo em sequências aleatórias de acessos: o ACK, a visibilidade da RAM do Z80 e o banco devem concordar.
- **Diferencial sintético com o oráculo** (`diferencial/tests/som.rs`): a ROM sintética roda no oráculo (que executa código real do 68000 e do Z80 que escrevemos como bytes de teste) e no núcleo, e as sequências de escritas e de pontos do Z80 devem concordar. Isso exercita o patch 0005 e a máquina do Z80 com um programa de brinquedo de poucas instruções (laço, escrita no YM, `djnz`, `ldir`) que usa os mesmos trechos que o programa real.
- **WebAssembly:** `cargo test --target wasm32-wasip1 -p sonic1-core -p sonic1-teste-som`. O núcleo não lê arquivo (o tempo e a ROM entram como bytes), então roda igual.

## 9. Cobertura e bugs (item 7 do pedido)

- **Desvios do driver do 68000:** cada decisão do código Rust usa `decisao!("Rotina:nome", condicao)` (macro da Etapa 5), com o nome da rotina do disassembly. O mapa de decisões para os desvios do original é **inferido por dados** pelo mesmo método da Etapa 5: o harness do som registra a sequência de decisões do núcleo e a sequência de desvios condicionais do 68000 do oráculo por chamada de `UpdateMusic`, e `diferencial som --proposta-de-mapa` propõe as linhas únicas. Os candidatos ambíguos são resolvidos por casos novos, não por leitura do assembly. O mapa final vai em `cobertura/mapa-decisoes.txt` (acréscimo) e o Marcelo congela.
- **Desvios do Z80:** o oráculo já registra `desvio80`. A cobertura do Z80 reaproveita `cargo fidelidade cobertura extrair`, que precisa de uma extensão para o Z80 (patch P6-2): soma dos desvios do Z80 por (endereço, tomado, não tomado). O programa B tem dezenas de desvios; os trechos da máquina marcam suas decisões do mesmo jeito (`decisao!("Z80:nome", ...)`). Mapa inferido por dados.
- **Universo:** vale o que o requisito diz (o que o oráculo executou); desvio que nenhum caso executou continua no backlog da Etapa 14.
- **Bugs injetados:** cada `decisao!` do som vira ponto de injeção (feature `mutacao` já existe). O orçamento de tempo é o da Etapa 5. A rodada completa fica para a auditoria.
- **Bugs conhecidos do som (lista de verificação):** 20 itens em `docs/bugs-conhecidos.md`, `BK-0276` a `BK-0295`, rótulos `UpdateMusic`, `PlaySoundID`, `Sound_PlayBGM`, `Sound_PlaySFX`, `Sound_PlaySpecial`, `StopSFX`, `StopSpecialSFX`, `StopAllSound`, `InitMusicPlayback`, `DoFadeIn`, `PSGDoVolFX`, `SendPSGNoteOff`, `cfFadeInToPrevious`, `cfStopSpecialFM4`, `SendVoiceTL`. Cada um ganha um teste cujo nome tem `BK_nnnn`: um sintético que reproduz o comportamento da ROM de referência (sem a correção) e a linha no relatório do harness que o exercita. Item que nenhum ID alcança fica marcado no relatório, com o roteiro que falta. `cargo fidelidade bugs-conhecidos` passa a contar 20 de 315 (no mínimo).

## 10. Estrutura de módulos e o que roda em wasm (item 9 do pedido)

```
crates/core/src/som/
  mod.rs               API: Som (estado do driver), eventos de som, tempo do Z80
  eventos.rs           EventoSom { fonte, tipo, porta, registrador, valor }
  barramento_z80.rs    BUSREQ, RESET, ACK, banco, mapeamento da janela
  z80/
    maquina.rs         máquina de estados do Z80: avanço por ciclos, pontos, conferência
    programa_b.rs      driver de DAC (bateria e voz SEGA), trechos e custos
    programa_a.rs      código de partida (limpeza da RAM com LDIR)
    custo.rs           tabela de ciclos por trecho, com a fonte pública
  driver/
    fila.rs            QueueSound1/2/3, CycleSoundQueue, PlaySoundID
    iniciar.rs         Sound_PlayBGM, Sound_PlaySFX, Sound_PlaySpecial, InitMusicPlayback
    trilhas.rs         DACUpdateTrack, FMUpdateTrack, PSGUpdateTrack e auxiliares
    coordenadas.rs     cf* (flags de coordenação)
    fm.rs              WriteFM*, SetVoice, SendVoiceTL, notas e frequências
    psg.rs             PSG notas, envelopes, ruído
    fade.rs            fade out e fade in
    pausa.rs           PauseMusic, TempoWait, velocidade
    sega.rs            PlaySegaSound
crates/teste-som/      vetores, propriedades, ROM sintética, modelo independente (MIT, lints comuns)
diferencial/           ganha o subcomando `som`, o relatório e os testes sintéticos (GPLv3)
xtask/                 inventario-som, ids-som
```

Os lints do `core` valem em tudo de `som/` (sem float, estouro explícito, sem HashMap/HashSet, sem cast que trunca, sem `#[allow]`). Cada função cita o rótulo (`/// Rotina do disassembly: UpdateMusic`). Nada copiado.

**Em wasm:** tudo de `crates/core/src/som/` (puro, sem E/S). O site só vai ouvir os eventos na Etapa 8. **Só nativo:** `diferencial/`, o harness do som, o leitor de `.s1tempo` do arquivo (o núcleo recebe bytes), os scripts. Vale a regra de sempre: as escritas de som saem como fila de eventos, e o crate `audio` (Etapa 8) só as consome.

## 11. Sub-etapas (item 1 do pedido)

Legenda: **IA** roda com dados sintéticos; **Marcelo** roda no host com a ROM. Haverá **três blocos de comandos** para o Marcelo (1 agora; 2 depois da 6.5; 3 no fecho), cada um num script só.

| # | Entrega | Prova (comando e resultado esperado) | Roda |
| --- | --- | --- | --- |
| 6.0 | Registro das Etapas 3 e 5 em `estado.md`. Patch 0005 do oráculo (`docs/pedidos/etapa6-oraculo.patch`): pontos do Z80 no `.s1tempo` v2 e acessor do ciclo restante. `cargo xtask inventario-som` e `ids-som`. Leitor de `.s1tempo` v2 no núcleo. `scripts/etapa6-bloco1.sh` | IA: patch aplicado numa cópia limpa e `sh oracle/scripts/testa.sh` passa com o teste novo (programa de Z80 de brinquedo: um ponto por acesso, ciclos somam o total, PC bate). `cargo test -p xtask` e `-p sonic1-core` (leitor v2 e v1). **Bloco 1 (Marcelo):** aplica o patch, regenera os tempos do corpus, gera `docs/inventario-som.md` e imprime o resumo | IA; Marcelo (bloco 1) |
| 6.1 | Modelo do barramento e das esperas (seção 3): banco, ACK, janela, reset, pontos. Modelo independente. `crates/core/src/som/barramento_z80.rs` | IA: testes de unidade, propriedades, modelo independente cruzado, e diferencial com o oráculo em ROM sintética (sequências de acessos ao Z80: BUSREQ, RESET, leitura e escrita da janela, nove escritas de banco). `cargo test -p sonic1-core -p sonic1-teste-som`, `cargo test --target wasm32-wasip1 ...` | IA |
| 6.2 | Máquina do Z80: programa A (LDIR) e programa B (DAC e voz SEGA), custo por trecho, avanço por ciclos, conferência de ponto. Roda no Z80 do oráculo com o programa de brinquedo e com os dois programas reais montados por nós como **bytes de teste escritos a partir do manual do Z80** (sem usar os bytes da ROM) | IA: `diferencial/tests/som.rs` compara escritas e posição da máquina contra o oráculo executando os bytes de teste, por janelas de ciclos variados (soma, uma janela de 1 ciclo, uma que cai no meio do `djnz`). Esperado: 0 divergências | IA |
| 6.3 | Driver do 68000, em partes: 6.3a fila e `PlaySoundID`; 6.3b iniciar música, efeito e especial; 6.3c trilhas (DAC, FM, PSG), notas, duração, modulação; 6.3d coordenadas `cf*`, voz, envelopes de PSG; 6.3e fade, pausa, velocidade, parar. Cada parte com `decisao!` | IA: cada parte com vetores da ROM sintética (comparando com valores calculados à mão pelo formato público do SMPS) e propriedades. O oráculo roda as rotinas em ROM sintética (`chamar_rotina` sobre código de teste que escrevemos) quando o caso é pequeno. Sem ROM real | IA |
| 6.4 | Bateria e voz SEGA pelo DAC: integração do driver com a máquina do Z80 (comando de DAC das trilhas escreve em `zDAC_Sample`), `PlaySegaSound`, laços de espera (YM ocupado, BUSACK, retry, laço dos 2 s) consumindo o tempo gravado | IA: teste sintético com `.s1tempo` v2 de brinquedo: a voz SEGA produz o número esperado de escritas de DAC, e a espera consome exatamente os ciclos gravados | IA |
| 6.5 | Harness `diferencial som` (D6.6), enumeração de IDs (seção 6), relatório (seção 7), `scripts/etapa6-bloco2.sh` | IA: o subcomando roda inteiro numa ROM sintética (com `ids.txt` reduzido) e imprime `IGUAL` nos casos e `DIVERGE` num caso com bug plantado no núcleo. **Bloco 2 (Marcelo):** relatório do som com todos os IDs | IA; Marcelo (bloco 2) |
| 6.6 | Cobertura do 68000 e do Z80 (seção 9), mapa de decisões proposto, bugs injetados, testes `BK_0276` a `BK_0295` | IA: sintético; Marcelo no bloco 3: mapa conferido, `cargo fidelidade cobertura` e `mutantes` com orçamento padrão | IA; Marcelo (bloco 3) |
| 6.7 | Fechamento: base de conhecimento (`som.md`), `estado.md`, `backlog.md`, patches P6-2 (relatório) e P6-3 (congelar), manifesto ampliado, resumo da etapa. `scripts/etapa6-bloco3.sh` (assina, roda `cargo fidelidade`) | `cargo fidelidade fidelidade`: `veredito: PASSOU` com a seção do som; `cargo xtask bugs-conhecidos` mostra 20 itens do som cobertos | Marcelo (bloco 3) |

Ordem: 6.0, **parar para o bloco 1**, 6.1, 6.2, 6.3, 6.4, 6.5, **bloco 2**, correções do que o bloco 2 mostrar, 6.6, 6.7, **bloco 3**.

## 12. Itens que mexem em arquivo congelado (item 11 do pedido)

| Pedido | Arquivo congelado | Conteúdo | Patch |
| --- | --- | --- | --- |
| P6-1 | `oracle/` | `oracle/patches/0005-*.patch` novo, `oracle/patches/SHA1` atualizado, `oracle/runner` com a escrita do tempo v2 e o teste do Z80 de brinquedo | `docs/pedidos/etapa6-oraculo.patch`, entregue no bloco 1 com o comando de aplicar |
| P6-2 | `crates/fidelidade/relatorio` | comando `som` (lê `relatorio-som.txt`), cobertura de desvios do Z80, seção do som no veredito | `docs/pedidos/etapa6-relatorio.patch`, no bloco 3 |
| P6-3 | `refs/manifesto`, `.github/` | acréscimo ao manifesto (arquivos novos do som), `esperado-*` se mudar | `docs/pedidos/etapa6-congelar.patch`, no bloco 3 |

## 13. Riscos e o que NÃO será feito (item 10 do pedido)

**Não será feito nesta etapa:** síntese de FM, de PSG e do DAC (Etapa 8); o ciclo exato de cada escrita de som e a previsão do tempo do Z80 e do 68000 (Etapa 15); a rotina de VBlank que chama `UpdateMusic` e o resto do laço do jogo (Etapas 7 a 12, o harness usa o oráculo para isso); o som da tela de seleção de fases e dos objetos além de enfileirar o ID; qualquer coisa de áudio no site.

| Risco | Mitigação |
| --- | --- |
| A máquina do Z80 acumula erro de ciclo e a posição não bate com os pontos gravados | A conferência acusa na primeira janela; o trecho errado é corrigido no jogo, nunca no teste. Os custos citam a tabela pública do Z80, e os pontos do oráculo dão um teste por trecho. |
| O oráculo atualiza o Z80 depois de cada instrução do 68000 (granularidade), então um acesso no meio de uma instrução longa vê o Z80 antes | O ponto grava o Z80 no instante em que o oráculo o viu. O núcleo avança até esse ciclo, antes do acesso. A mesma regra vale no nível 2. |
| Os estados calmos do corpus podem não ter o Z80 em espera de amostra | O bloco 1 mostra os PCs do Z80 nos pontos. Se o estado calmo escolhido não serve, o harness procura o primeiro quadro em que serve (Z80 no laço de espera, `zDAC_Sample` zerado). |
| O driver é grande (cerca de 2800 linhas de assembly) | Fatiado em 6.3a a 6.3e, cada fatia com vetores sintéticos e o harness no fim. Nenhum atalho: sem interpretador, sem tradutor, sem tabelas copiadas. |
| Cobertura do driver baixa porque o corpus não toca todos os IDs | O harness do som é por ID; o extrator de cobertura soma os desvios dessa rodada (a Etapa 5 já aceita traces extras). |
| Bugs conhecidos do som sem caminho alcançável pelo harness | O relatório lista o item e o roteiro que falta; vira roteiro novo na mesma etapa. |
| O jgenesis em depuração entra em pânico em casos de borda | Mesma regra da Etapa 5: o bloco de comandos usa release. |
| A IA não pode rodar nada com a ROM | O bloco 2 devolve só vereditos e hashes. A correção segue "teste que falha se corrige no jogo". |

## 14. Revisão final e anti-ciclo

Só o que a Etapa 6 mudou. Não reauditar sandbox, hooks nem CI, a menos que `.devcontainer/managed-settings.json`, `.devcontainer/Dockerfile` ou `.github/` mudem nesta etapa (o P6-3 pode tocar `.github/esperado-*`; nesse caso e só nesse, a revisão olha o diff). Classes: BLOQUEIA, BACKLOG, DESCARTADO. Uma rodada de correção. BACKLOG em `docs/backlog.md`, uma linha cada.
