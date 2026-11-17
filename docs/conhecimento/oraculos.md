# Segundo oraculo (Etapa 2)

## Genesis Plus GX por libretro (2.1 e 2.5)

- Nucleo: commit `f27655b592ef43f317272d79883bcaa10d3ac8ad` do `libretro/Genesis-Plus-GX`, compilado pelo Marcelo com `make -f Makefile.libretro`. `genesis_plus_gx_libretro.so` sha1 `ee8169cba49e9e7f2c6f35f343ffa6a00efbdf52`. Fica em `$SONIC1_EXT_DIR/trabalho/oracle2`, fora do repositorio.
- Hospedeiro: `oracle2/` (programa proprio, MIT, workspace separado), sem tela, sem som, sem entradas. Carrega o nucleo com `dlopen`. Imprime por quadro o SHA-1 da RAM do 68000 nas duas ordens de byte. Unica memoria que esse nucleo expoe: `retro_get_memory_data` so atende `SAVE_RAM` e `SYSTEM_RAM` (leitura de `libretro/libretro.c`, linha 3746).
- Prova com ROM sintetica (06/11/2026, 45 quadros): igual em 44 quadros com os pares de byte trocados e deslocamento +1 (o nucleo foi compilado com `LSB_FIRST`, que guarda a RAM com os pares trocados).
- Prova com a ROM real (Marcelo, 06/11/2026, `boot-demos`, 7200 quadros, `scripts/etapa2-oraculo2.sh`): **os dois oraculos concordam so nos primeiros 52 quadros, em qualquer deslocamento de -3 a +3 e nas duas ordens de byte**. A primeira diferenca fica no quadro 52 (51 para deslocamento +2, 50 para +3). Nos quadros iniciais a RAM provavelmente ainda esta zerada nos dois, por isso a ordem de byte nao aparece.

| id | caso | quadro | categoria | o que difere | causa | decisao |
| --- | --- | --- | --- | --- | --- | --- |
| D1a | boot-demos e demos-quatro | todos | ram68k | o segundo oraculo fecha o quadro um depois do principal | alinhamento do quadro (hipotese, apoiada em 14 quadros do boot-demos) | Comparar o principal no quadro N com o segundo no quadro N+1 |
| D1b | boot-demos e demos-quatro | todos | ram68k | pilha, `FFFCC0..FFFDFF` (de `v_systemstack_end` a `v_systemstack`) | bytes antigos que dependem do ciclo da interrupcao (hipotese) | Vale o oraculo principal. Tempo e do nivel 2 (Etapa 15) |
| D1c | boot-demos e demos-quatro | todos | ram68k | `v_vblank_routine`, `v_jpadhold1`, `v_jpadpress1` | escritas perto do VBlank (hipotese) | Vale o oraculo principal. Etapa 15 |

- Decisao da IA (07/11/2026, `docs/plano-fechamento-etapa-2.md`, D-R1), pela regra do requisitos ("sem evidencia, vale o oraculo principal, e a diferenca fica registrada"): o D1 virou D1a, D1b e D1c. "O segundo oraculo concorda nas demos" passa a significar: em todas as amostras das quatro demos, nada difere fora de D1b e D1c, comparando o principal no quadro N com o segundo no quadro N+1 (D1a). Amostras do boot-demos ate o quadro 5600 (14 quadros, sexta rodada). Amostras novas a medir (`scripts/etapa2-d1c.sh`): 6100, 6600, 7100 (fim da MZ e titulo), 7600, 8400 (SYZ), 10200 e 11000 (special stage). O resultado dessas amostras entra aqui na sub-etapa 2.11.
- Limite registrado: so a RAM do 68000 e comparada, porque o nucleo libretro do Genesis Plus GX so expoe essa memoria. VRAM, CRAM, VSRAM e RAM do Z80 ficam sem comparacao com o segundo oraculo (backlog, Etapa 14). O segundo oraculo nao aceita entradas, entao TAS e partidas gravadas nao passam por ele (backlog, Etapa 14).
- A exclusao da pilha na comparacao do nucleo contra o oraculo (Etapa 5) e outra decisao, do Marcelo (requisitos, criterio 2). Recomendacao da IA: aprovar.

- Historico: antes da decisao acima, o criterio da Etapa 2 ("o segundo oraculo concorda nas demos") nao estava cumprido. Nada aqui diz qual dos dois esta certo.
- Proximo passo de diagnostico (backlog): imprimir so os enderecos das palavras que diferem no quadro 52 (sem os valores) e cruzar com `refs/rotulos.txt` para ver o nome da variavel.

- Diagnóstico do D1 (preparado em 06/11/2026): `oraculo2 diferencas --a RAM_PRINCIPAL --b RAM_GPGX --rotulos refs/rotulos.txt` mostra, para os quadros 51 e 52, as faixas de RAM que diferem e a variável mais próxima. Resultado (host, 06/11/2026, `scripts/etapa2-final.sh`): no quadro 51 as RAMs são idênticas (0 bytes); no quadro 52 diferem **4 bytes, todos em `v_init`** (faixa FFFFFC a FFFFFF), nas duas ordens de byte. Os valores não foram lidos. Isto estreita a causa a esse marcador de inicialização, mas não diz qual oráculo está certo nem se a diferença se espalha depois. Passo seguinte: `scripts/etapa2-d1.sh` (quadros 53, 60, 120 e 600).

- D1, segunda rodada (host, 07/11/2026, `scripts/etapa2-d1.sh`): a diferença cresce devagar, com a ordem de byte de pares trocados: quadro 52 com 4 bytes (`v_init`), 53 com 1 (`v_megadrive`), 60 com 3, 120 com 14 (entre elas `v_vblank_routine`, `v_pcyc_num` e linhas de paleta), 600 com 295. `v_megadrive` é o byte de versão do console que o jogo lê, e o nucleo estava com a região em `auto` (o jogo é de várias regiões), enquanto o oráculo principal força Japão. **Hipótese (não provada):** a região diferente explica o D1. Mudança feita: `oracle2` agora responde `ntsc-j` à opção `genesis_plus_gx_region_detect` do nucleo. A prova é refazer `scripts/etapa2-oraculo2.sh` e `scripts/etapa2-d1.sh`.

- D1, terceira rodada (host, 07/11/2026, com a região `ntsc-j` no segundo oráculo): no quadro 53 a RAM passa a ser **idêntica** (0 bytes), o que confirma que a região explicava `v_megadrive`. O quadro 52 ainda difere (os 4 bytes de `v_init`, que no outro emulador mudam um quadro depois). No quadro 120 diferem 13 bytes e no 600 diferem 197. Os que aparecem são variáveis de tempo: `v_vblank_routine`, `v_pcyc_num`, linhas de paleta, `v_hscrolltablebuffer`, `v_lvllayout_end`, `v_spritequeue`. Leitura: os dois emuladores fecham o quadro em pontos diferentes em relação à interrupção de VBlank, então variáveis que o VBlank muda aparecem num quadro ou no seguinte. **Isto é hipótese, não prova.** Decisão (regra do requisitos): vale o oráculo principal, e a diferença fica registrada. A comparação com o segundo oráculo no nível 1 só vale para variáveis estáveis no quadro; o ponto exato da interrupção é do nível 2 (Etapa 15). Backlog.

- D1, quarta rodada (host, 07/11/2026, `scripts/etapa2-d1b.sh ROM 60 120 600 2000`, `boot-demos`). Compara a RAM do principal no quadro N com a do segundo oráculo no quadro N ou N+1. Bytes diferentes (pares trocados):

| N | principal N contra segundo N | principal N contra segundo N+1 | principal N+1 contra segundo N |
| --- | --- | --- | --- |
| 60 | 2 | 2 | 44 |
| 120 | 13 | 3 | 13 |
| 600 | 197 | 19 | 262 |
| 2000 | 503 | 49 | 995 |

  Leitura: a comparação com o quadro seguinte do segundo oráculo reduz a diferença em cerca de 10 vezes, e a outra direção piora. Isso **apoia** a hipótese de uma fase de um quadro entre os oráculos. **Não prova**: o resto não é zero. O resumo mostra só as 5 primeiras faixas de cada comparação. Os endereços que apareceram no resto são `v_vblank_routine` (em todos os N), `v_jpadhold1` (N=2000) e faixas `FFFDxx`, que ficam logo abaixo de `v_systemstack` (`FFFE00`) e provavelmente são a pilha, a confirmar com a lista completa. A pilha é área de exclusão prevista no requisitos, mas a exclusão ainda não foi aprovada. Falta a lista completa das faixas restantes (`COMPLETO=1`).

- D1, quinta rodada (host, 07/11/2026, `COMPLETO=1 sh scripts/etapa2-d1b.sh ROM 120 600 2000`, lista completa de faixas do principal no quadro N contra o segundo oráculo no quadro N+1). Todos os bytes que restam cabem em três grupos:

| N | total | pilha | v_vblank_routine | v_jpadhold1 e v_jpadpress1 | outros |
| --- | --- | --- | --- | --- | --- |
| 120 | 3 | 2 | 1 | 0 | 0 |
| 600 | 19 | 18 | 1 | 0 | 0 |
| 2000 | 49 | 46 | 1 | 2 | 0 |

  Pilha: o mapa de rótulos tem `v_systemstack_end` em `FFFCC0` e `v_systemstack` em `FFFE00` (`refs/rotulos.txt`, linhas `variavel`). Todos os endereços do grupo ficam entre `FFFD81` e `FFFDF7`, dentro dessa faixa. O inventário de memória mostra a faixa tocada nos três contextos (`docs/inventario-memoria.md`, linha de `v_objstate_end`, porque o mapa não tem tamanhos). Leitura: o conteúdo antigo da pilha depende do ciclo em que a interrupção chegou, e os dois oráculos não coincidem no ciclo. É a diferença esperada de tempo do nível 2.
  Ainda é **hipótese** que as 3 variáveis restantes (`v_vblank_routine`, `v_jpadhold1`, `v_jpadpress1`) sejam só fase de tempo. A previsão foi testada em 3 quadros e valeu. O teste em mais quadros é `scripts/etapa2-d1c.sh`.

- D1, sexta rodada (host, 07/11/2026, `scripts/etapa2-d1c.sh ROM`, principal no quadro N contra segundo oráculo no quadro N+1, `boot-demos`). Quadros 100, 600, 1100, 1600, 2100, 2600, 3100, 3600, 4100, 4600, 5100 e 5600 (este rodado à parte, o script demora porque cada amostra roda os dois oráculos desde o quadro 0). Em todos: `outros=nenhum`. Total entre 3 e 50 bytes: pilha entre 2 e 47, tempo entre 1 e 3. Com os quadros 120 e 2000 da rodada anterior, são 14 quadros distintos, todos sem variável de lógica fora da pilha e das três variáveis de tempo. Não cobre os quadros 5601 a 7200.
- Estado do D1 depois das seis rodadas: **fase de 1 quadro** entre os oráculos (apoiada em 14 quadros), **pilha** (diferença de ciclo, esperada) e **3 variáveis de tempo** (`v_vblank_routine`, `v_jpadhold1`, `v_jpadpress1`). A causa continua hipótese: nenhum teste olhou o ciclo da interrupção. Decisão registrada: vale o oráculo principal. Falta o aceite do Marcelo para a exclusão da pilha (`FFFCC0..FFFDFF`) na comparação entre oráculos.

## D1, sétima rodada (fechamento da Etapa 2, 08/11/2026)

Amostras novas, principal quadro N contra segundo oráculo quadro N+1: 6100 (total 2, pilha 2), 6600 (18, 17, tempo 1), 7100 (29, 28, 1), 7600 (41, 38, 3), 8400 (24, 23, 1), 10200 (45, 42, 3), 11000 (41, 38, 3). Todas com `outros=nenhum`. `quadros com variavel fora de pilha e tempo: 0 de 7`. Com as rodadas anteriores são 21 quadros. A causa continua hipótese.
