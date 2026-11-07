# Segundo oraculo (Etapa 2)

## Genesis Plus GX por libretro (2.1 e 2.5)

- Nucleo: commit `f27655b592ef43f317272d79883bcaa10d3ac8ad` do `libretro/Genesis-Plus-GX`, compilado pelo Marcelo com `make -f Makefile.libretro`. `genesis_plus_gx_libretro.so` sha1 `ee8169cba49e9e7f2c6f35f343ffa6a00efbdf52`. Fica em `$SONIC1_EXT_DIR/trabalho/oracle2`, fora do repositorio.
- Hospedeiro: `oracle2/` (programa proprio, MIT, workspace separado), sem tela, sem som, sem entradas. Carrega o nucleo com `dlopen`. Imprime por quadro o SHA-1 da RAM do 68000 nas duas ordens de byte. Unica memoria que esse nucleo expoe: `retro_get_memory_data` so atende `SAVE_RAM` e `SYSTEM_RAM` (leitura de `libretro/libretro.c`, linha 3746).
- Prova com ROM sintetica (06/11/2026, 45 quadros): igual em 44 quadros com os pares de byte trocados e deslocamento +1 (o nucleo foi compilado com `LSB_FIRST`, que guarda a RAM com os pares trocados).
- Prova com a ROM real (Marcelo, 06/11/2026, `boot-demos`, 7200 quadros, `scripts/etapa2-oraculo2.sh`): **os dois oraculos concordam so nos primeiros 52 quadros, em qualquer deslocamento de -3 a +3 e nas duas ordens de byte**. A primeira diferenca fica no quadro 52 (51 para deslocamento +2, 50 para +3). Nos quadros iniciais a RAM provavelmente ainda esta zerada nos dois, por isso a ordem de byte nao aparece.

| id | caso | quadro | categoria | o que difere | causa | decisao |
| --- | --- | --- | --- | --- | --- | --- |
| D1 | boot-demos | 52 | ram68k | hash da RAM do 68000 | **desconhecida**. Hipoteses, nenhuma provada: estado inicial da RAM, tempo da interrupcao de VBlank em relacao ao laco principal, leitura de versao ou do barramento do Z80 no boot | Vale o oraculo principal (regra do requisitos: sem evidencia, vale o principal, e a diferenca fica registrada). O diagnostico fica no backlog |

- O criterio da Etapa 2, "o segundo oraculo concorda nas demos", **nao foi cumprido**. Nada aqui diz qual dos dois esta certo.
- Proximo passo de diagnostico (backlog): imprimir so os enderecos das palavras que diferem no quadro 52 (sem os valores) e cruzar com `refs/rotulos.txt` para ver o nome da variavel.

- Diagnóstico do D1 (preparado em 06/11/2026): `oraculo2 diferencas --a RAM_PRINCIPAL --b RAM_GPGX --rotulos refs/rotulos.txt` mostra, para os quadros 51 e 52, as faixas de RAM que diferem e a variável mais próxima. Resultado (host, 06/11/2026, `scripts/etapa2-final.sh`): no quadro 51 as RAMs são idênticas (0 bytes); no quadro 52 diferem **4 bytes, todos em `v_init`** (faixa FFFFFC a FFFFFF), nas duas ordens de byte. Os valores não foram lidos. Isto estreita a causa a esse marcador de inicialização, mas não diz qual oráculo está certo nem se a diferença se espalha depois. Passo seguinte: `scripts/etapa2-d1.sh` (quadros 53, 60, 120 e 600).
