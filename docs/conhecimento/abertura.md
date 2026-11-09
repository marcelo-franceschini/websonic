# Abertura (Etapa 7)

Plano: `docs/plano-etapa-7.md`. Cada afirmação cita o comando que a sustenta. Nada aqui tem dado da ROM, só rótulos, quadros e `mclk`.

## Medidas do bloco 1 (Marcelo, 08/11/2026, `sh scripts/etapa7-bloco1.sh`, caso `boot-demos`, sem botões)

Perfil (mudanças de `v_gamemode`): quadro 1 modo 0 (boot e Sega); 329 modo 4 (título); 933 modo `136` (`$88`, demo); 1167 modo 8.

Marcas (a parada devolve o quadro fechado e o `mclk` do **fim desse quadro**, não o instante do PC; ver "Achados"):

| Marca | Quadros | mclk |
| --- | --- | --- |
| SkipSetup, GameProgram | 3 | 2585576 |
| CheckSumOk, GameInit | 52 | 46491494 |
| GM_Sega | 61 | 54555879 |
| Sega_WaitPal | 89 | 79645048 |
| PlaySegaSound | 164 | 146847967 |
| Sega_WaitEnd | 298 | 266917357 |
| Sega_GotoTitle, GM_Title | 328 | 293798554 |
| Tit_MainLoop | 526 | 471214485 |
| GotoDemo | 902 | 808125507 |
| GM_Level | 932 | 835006711 |
| Level_TtlCardLoop | 962 | 861887915 |
| Level_LoadObj, Level_Delay | 1140 | 1021383006 |
| Level_StartGame, Level_MainLoop | 1166 | 1044680077 |

## Achados

- **`N_fim`:** `Level_StartGame` é alcançado no quadro 1166. O modo é `$88` (bit 7 = demo) de 933 a 1166 e vira 8 em 1167. Então `Level_StartGame` limpa o bit 7 (leitura do texto do `.asm`, só para achar a ordem). A definição por estado do plano deve usar "bit 7 de `v_gamemode` cai", não "modo 8 já aparece".
- **Granularidade:** `EntryPoint` não é "alcançado" (é o PC inicial). Marcas no mesmo quadro têm o mesmo `mclk` (por exemplo `CheckSumOk` e `GameInit`). A parada em rótulo só dá o quadro. O ciclo exato pede o patch 0006 (marcas com `mclk`, linha e coluna).
- **Voz SEGA (correção do plano):** `Sega_WaitPal` vem **antes** da voz (quadro 89), não depois. A voz começa em `PlaySegaSound` (164) e o fim é a primeira chegada em `Sega_WaitEnd` (298). Duração medida: 134 quadros, 119.570.000 mclk aproximados, cerca de 2,2 s, coerente com o requisito ("cerca de 2 segundos"). `PlaySegaSound` roda **dentro do tratador de VBlank** (chamada pela fila de som) e é um laço de 18 por 65536 voltas de `nop` e `dbf` (leitura do texto do driver), com retorno adulterado.
- **Consequência para a comparação (D7.4):** durante a voz o 68000 não sai do tratador, então não há RTE do nível 6 em cerca de 134 quadros. Nesses quadros só existe o ponto de fechamento, e o PC fica dentro do laço da voz, onde o estado é estável.
- Boot: verificação da ROM termina no quadro 52, `GM_Sega` começa no 61, a Sega termina no 328, o título dura até o 932 (demo por tempo ocioso), a carga da fase e o cartão vão até o 1166.

## Núcleo e comparador (7.0 a 7.2, 08/11/2026)

- `crates/core/src/abertura/`: `Abertura` recebe `EventoQuadro` por quadro (marca, leitura, ponto do Z80) e roda os trechos do laço principal na marca final de cada um. `boot.rs` reproduz `EntryPoint`, `GameProgram` e `CheckSumCheck`, `CheckSumOk`, `GameInit`, `VDPSetupGame`, `DACDriverLoad` e `JoypadInit`, lendo as tabelas da ROM nos endereços dos rótulos. A soma de verificação soma as palavras de `$200` até a última palavra completa antes do endereço final do cabeçalho, em 16 bits (leitura do texto do `.asm`, só da lógica).
- Prova sintética (ROM escrita por nós, `crates/teste-som/tests/abertura.rs`, 6 testes; `abertura_layout.rs` confere os endereços contra `refs/rotulos.txt`): 24 registradores da VDP na ordem; programa de partida do Z80 byte a byte; 4 volumes do PSG; soma de verificação contra um modelo de uma linha em 60 ROMs; soma errada recusada; modo Sega depois das duas marcas; determinismo; ROM minúscula dá erro e não pânico. Resultado: todos passam.
- Comparador: `diferencial-abertura/` (`abertura comparar ROM QUADROS rotulos`). Prova de fumaça com ROM sintética (`tests/fumaca.rs`): programa parado não diverge; marca vista no oráculo sem o mesmo efeito no jogo é acusada.
- Ainda sem prova com a ROM real. Bloco 2 (`scripts/etapa7-bloco2.sh`, 61 quadros, do ligar até a entrada de `GM_Sega`).

## Bloco 2, rodada 2 (Marcelo, 08/11/2026, 62 quadros)

- RAM do 68000: 60 quadros iguais, 0 `diverge` fora de trecho atômico (2 em trecho atômico). RAM do Z80: 54 iguais, 0 `diverge` (8 em trecho atômico).
- Escritas até a entrada de `GM_Sega`: as 230 de vídeo do núcleo e as 4 do PSG são iguais às do oráculo, na ordem. As mudanças do BUSREQ e do RESET do Z80: as 13 do núcleo são o começo exato das 18 do oráculo (o oráculo registra só mudanças; hipótese confirmada). As escritas a mais do oráculo (vídeo 281, PSG 8, YM 64, Z80 18) vêm do começo de `GM_Sega` e da primeira interrupção, no quadro 61. Isso fecha 7.1 e 7.2 no que o comparador mede.
- Marcas por quadro: `GameProgram` 3, `CheckSumOk` 52, `VDPSetupGame` 55, `DACDriverLoad` 56, `JoypadInit` 61, `MainGameLoop` 61, `GM_Sega` 61. Então a carga do programa do Z80 (descompressão) ocupa os quadros 56 a 61.
- **Aberto:** `posicao_z80_divergencias=6106`. A máquina do Z80 do núcleo não está onde o oráculo viu o Z80 em cada ponto durante a carga do programa do Z80 (uma escrita na janela por byte). Diagnóstico adicionado ao comparador (primeiras divergências e contagem por tipo de acesso).

## Posição do Z80 na carga do programa (rodada 3 do bloco 2)

- Diagnóstico (comparador com `z80_divergencia`): 1691 acessos `AcessoDiferente` (o oráculo gravou `LeituraJanela`, o núcleo fez `EscritaJanela`) e 4415 `Posicao` em `EscritaJanela`. Causa: a rotina `KosDec` lê do destino cada byte que copia da própria saída. Com o destino na RAM do Z80, cada leitura é um acesso do 68000 à janela do Z80 e vira um ponto do Z80 gravado. O núcleo só escrevia.
- Correção no jogo (não no teste): `descompressao::kosinski::descomprimir_com_origens` diz a origem de cada byte; `boot::dac_driver_load` lê o destino antes de escrever nas cópias. Testes do núcleo e da descompressão seguem passando. Prova com a ROM: bloco 2 de novo (`posicao_z80_divergencias` deve cair para 0).

## Posição do Z80 zerada e tela da Sega no núcleo (08/11/2026)

- Rodada 3 do bloco 2 (Marcelo): `posicao_z80_divergencias=0`. RAM do 68000 e do Z80 sem divergência fora de trecho atômico, vídeo e PSG iguais até a entrada de `GM_Sega`. Boot (7.1 e 7.2) fechado.
- 7.3 escrita (sem prova com a ROM ainda): `abertura/vblank.rs` (tratador de VBlank com as rotinas `Sega`, `SegaPCM` e `PaletteFade`, `ReadJoypads`, DMA de paleta, sprites e rolagem), `abertura/sega.rs` (`PaletteFadeOut`, `ClearScreen`, `PalLoad`, `TilemapToVRAM`, `PalCycle_Sega`) e `rotinas::nem_dec` (Nemesis pela porta de dados da VDP). O laço principal da Sega é uma máquina de estados: o VBlank gravado roda o tratador e depois o passo seguinte do laço. O tratador da voz SEGA fica adiado até a marca `Sega_WaitEnd`, porque o driver só volta ao fim da voz (cerca de 134 quadros).
- Provas sintéticas (`crates/teste-som/tests/sega.rs`): o desvanecimento zera uma cor em tantos passos quanto a soma dos canais (modelo direto), o ciclo de paleta termina e é determinístico, o VBlank da Sega faz os três DMAs e decrementa o temporizador, rotina de VBlank desconhecida dá erro e não pânico.

## Interrupção no fim do quadro (D7.12, rodada 3e do bloco 3)

- Rodada 3d: nas esperas de VBlank, **129 iguais, 0 divergem** (RAM do 68000 e do Z80). Durante a Sega o estado do jogo bate com o oráculo. O fechamento do quadro cai dentro do tratador de VBlank em 262 de 330 quadros (por isso a D7.4).
- Rodada 3e: o primeiro desalinhamento entre os acessos do núcleo ao Z80 e os pontos gravados está no acesso 10794. Os quatro acessos do VBlank que fica pendente depois da voz encontram a fila de pontos vazia. Causa: o oráculo grava a interrupção no fim de um quadro (instante em que ela é aceita) e os acessos do tratador no quadro seguinte.
- Correção (D7.12): `Abertura::executar_quadro` deixa na fila as leituras e os pontos do Z80 na hora e roda os eventos (marcas e interrupções) um quadro depois. `Abertura::terminar` roda os do último quadro. O comparador compara o fechamento de cada quadro um quadro depois, com o que o oráculo tinha naquele fechamento.

## 7.3 provada com a ROM real (Marcelo, 08/11/2026, rodada 3f, quadros 0 a 329)

- Nas 129 esperas de VBlank da Sega: RAM do 68000 e RAM do Z80 iguais ao oráculo, 0 divergências.
- Posição do Z80 em todo acesso do 68000: `posicao_z80_divergencias=0`.
- Escritas do Z80 no YM2612 (a voz SEGA pelo DAC): 54000 contra 54000, iguais em conteúdo e ordem. Escritas de vídeo, PSG, YM do 68000 e mudanças do BUSREQ e do RESET: iguais até onde o núcleo vai (as 6875 de vídeo, 8 do PSG, 64 do YM do 68000, 529 mudanças do Z80). As que faltam (51 de vídeo, 4 do PSG, 64 do YM, 5 do Z80) são do começo do título, quadros 328 e 329, que é a sub-etapa 7.5.
- O VBlank que fica pendente depois da voz roda como quadro de lag, só com o driver de som (`v_vblank_routine` já é zero). O núcleo roda os eventos um quadro depois dos pontos do Z80 (D7.12).
- Veredito do relatório continua `DIVERGE` só porque as contagens finais incluem o título.

## Experimento do nível 2 (7.8): primeira medida (Marcelo, 08/11/2026, `scripts/etapa7-bloco2.sh`, 330 quadros)

Modelo: `crates/core/src/nivel2/`, lê as constantes dos laços nos bytes da ROM e soma os ciclos da tabela do manual do 68000. Medida: ciclos do 68000 entre a primeira chegada do PC ao início e ao fim do laço no oráculo (mclk dividido por 7).

| Laço | Medido (ciclos) | Previsto | Erro |
| --- | --- | --- | --- |
| Soma de verificação da ROM | 6385118 | 6285326 | -99792 (-15628 ppm, -1,56%) |
| Limpeza da RAM de `GameInit` | 363338 | 357660 | -5678 (-15627 ppm, -1,56%) |
| Espera da voz SEGA | não saiu na tabela (o modelo não reconheceu o trecho, ou o oráculo não chegou ao fim; diagnóstico adicionado) | | |

Leitura: os dois erros têm o mesmo valor relativo (15628 ppm), o que aponta para um fator de hardware constante e não para erro na contagem de instruções. A razão medido por previsto é 1,01588, igual a 128/126: perda de 2 ciclos a cada 128, a atualização (refresh) da memória. Hipótese, sem fonte de hardware conferida: com esse fator o erro previsto cai para -25 ciclos na soma e +1 ciclo na limpeza (conta feita à mão, a conferir na próxima rodada). O fator foi ajustado nesses dois pontos e não prova nada fora deles.

## Experimento do nível 2 (7.8): resultado final (Marcelo, 09/11/2026, bloco 4d)

Medida em ciclos do 68000 (mclk do oráculo dividido por 7), do início ao fim de cada laço, contra o modelo (`crates/core/src/nivel2/`):

| Laço | Medido | Previsto (tabela do manual) | Erro | Previsto com refresh (128/126) | Erro com refresh |
| --- | --- | --- | --- | --- | --- |
| Soma de verificação da ROM | 6385118 | 6285326 | -99792 (-15628 ppm) | 6385093 | -25 (-3 ppm) |
| Limpeza da RAM de `GameInit` | 363338 | 357660 | -5678 (-15627 ppm) | 363337 | -1 (-2 ppm) |
| Espera da voz SEGA | 17039347 | 16515520 | -523827 (-30742 ppm) | 16777671 | -261676 (-15357 ppm) |

- **Conclusão 1 (laços curtos):** a contagem de instruções pela tabela do manual está certa. O desvio de 1,5877% é um fator constante, igual a 128/126, isto é, 2 ciclos perdidos a cada 128 pela atualização (refresh) da memória. Com ele o erro cai para -3 e -2 ppm. Confirmado em dois laços independentes, que tiveram o mesmo desvio relativo antes do ajuste. Falta conferir o fator em fonte de hardware (hipótese).
- **Conclusão 2 (voz SEGA, caso longo):** com o refresh o modelo ainda erra 1,54% (-261676 ciclos). O laço é feito só de `nop` e `dbf`, então o excesso vem do hardware: o Z80 lê a ROM durante a voz e segura o 68000. A voz tem 27000 amostras (54000 escritas no YM2612 pelo DAC, endereço e dado). O excesso dá cerca de 9,5 a 9,7 ciclos do 68000 por amostra, se cada amostra custar uma leitura do Z80 na ROM. Hipótese a provar na Etapa 15: modelar a parada do 68000 por leitura do Z80 (cada leitura soma 3 ciclos ao relógio do Z80 e uma parada ao 68000, ver `som.md`).
- **Erro do modelo para a duração do boot:** os dois trechos dominantes do boot (soma da ROM, 6,39 M ciclos, e limpeza da RAM) ficam a poucos ppm com o refresh. O boot inteiro até `GM_Sega` ainda tem partes não modeladas (preenchimento da VRAM por DMA, espera do Z80, outros laços curtos), por isso a duração total do boot não foi prevista nesta rodada.
- **Classificação:** a previsão da espera da voz erra mais de 0,1%: **BACKLOG para a Etapa 15** (dono: IA). O experimento não trava a etapa.

## Navegador (7.7), primeira prova (Marcelo, 09/11/2026, bloco 5)

- Caminho do navegador (console nominal, `crates/console`): determinismo PASSOU; hashes por quadro idênticos no nativo e em WebAssembly (wasmtime na imagem), 330 de 330 quadros; site monta; testes em wasm do núcleo, vídeo, console e som passam.
- Defeito achado pelo diagnóstico (`scripts/etapa7-bloco5b.sh`): a imagem não mudava porque a CRAM ficava zerada (`cram_nao_zero=0`). Causa: a ponte lia os registradores de DMA no renderizador, que guarda as escritas atrás de um DMA ou preenchimento em andamento; o modo de preenchimento antigo escondia o DMA da paleta. Correção: a ponte guarda a própria cópia dos registradores 1 e 19 a 23. Depois da correção: `cram_nao_zero=64` a partir do quadro 100, VRAM com 3793 bytes não-zero e imagem ligada.
- Falta ver a imagem do quadro 150 (logo da Sega) e a prova de que a imagem distinta muda ao longo dos quadros (`imagens_distintas` maior que 1).

## Título, parte A (7.5, 09/11/2026): "SONIC TEAM PRESENTS"

- Escrito: `abertura/objetos.rs` (`ExecuteObjects`, `DisplaySprite`, `BuildSprites` com as quatro orientações, objeto `CreditsText`) e `abertura/titulo.rs` (`GM_Title` até `PaletteFadeIn`, `FadeIn_FromBlack`). Máquina de estados do título: entrada, primeiro desvanecimento (22 VBlanks), carga (`PreparoA`, até a marca `PaletteFadeIn`), aparecimento (22 VBlanks). O resto do título (`Parado`) ainda não existe.
- Provas sintéticas: `crates/teste-som/tests/objetos.rs` compara `BuildSprites` com um modelo direto nas quatro orientações (peças, ligações, tiles e flips), a ordem por prioridade e a ausência de pânico com mapas e filas aleatórios.
- Armadilha achada no teste: `adda.w` estende o sinal da palavra; um deslocamento de mapa positivo não pode virar endereço de RAM alta.

## Título, parte A, prova com a ROM real (Marcelo, 09/11/2026, 420 quadros)

- Esperas de VBlank: 173 do oráculo, 173 do núcleo, **173 iguais, 0 divergem**, RAM do Z80 igual em todas. Interrupções de nível 6: 174 contra 174. Posição do Z80: 0 divergências. Escritas do Z80 no YM2612 (54000), do PSG (12) e do YM pelo 68000 (128) iguais em contagem. Marcas: `GM_Title` 328, segundo `DACDriverLoad` 350, `PaletteFadeIn` 371.
- A única divergência: sequência de BUSREQ e RESET do Z80, índice 618. O segundo `DACDriverLoad` solta o RESET do Z80 quando ele já está solto. O oráculo registra essa escrita; o comparador descartava as que não mudam o estado. Era erro do comparador (a regra "só mudanças" foi deduzida do boot, onde nenhuma escrita repete o estado). Corrigido: agora compara toda escrita.
- Vídeo: 13414 do núcleo contra 21358 do oráculo, sem divergência de conteúdo no prefixo. A diferença é o resto do título (arte do emblema e do Sonic grande, fundo da Green Hill), parte B, que ainda não existe.

## Título, parte B, fatia 1 (7.5, 09/11/2026): cenário da GHZ e artes do título

Escrito, sem prova com a ROM ainda. Rodada pendente: bloco 2 com 418 quadros (ver `docs/estado.md`).

- **O que roda.** Do fim do aparecimento do "SONIC TEAM PRESENTS" (`PaletteFadeIn`, 22 VBlanks) até o fim do segundo `PaletteFadeOut` (22 VBlanks). Em `titulo.rs`, `parte_b1`: `NemDec` do emblema (`Nem_TitleFg`, tile `$200`), do Sonic grande (`Nem_TitleSonic`, `$300`) e do "TM" (`Nem_TitleTM`, `$510`); `Art_Text` na VRAM (tile `$680`, uma palavra por vez); zeragem de `v_lastlamp`, `v_debuguse`, `f_demo`, `v_unused2`, `v_zone_act` e `v_pcyc_time`; depois, em `cenario.rs`, `LevelSizeLoad`, `DeformLayers`, `EniDec` e `KosDec` da GHZ e `LevelLayoutLoad`.
- **Rótulos reproduzidos (REV01, `FixBugs = 0`).** `GM_Title` (linhas 1952 a 1995 do `.asm` de texto), `LevelSizeLoad` e `BgScrollSpeed` (`_inc/LevelSizeLoad & BgScrollSpeed.asm`), `DeformLayers` e `Deform_GHZ` (`_inc/DeformLayers (REV01).asm`), `ScrollHoriz`, `MoveScreenHoriz`, `ScrollVertical` e `DynamicLevelEvents`/`DLE_GHZ1` (`_inc/ScrollHoriz & ScrollVertical.asm`, `_inc/DynamicLevelEvents.asm`), `LevelLayoutLoad` (`_inc/LevelLayoutLoad.asm`), `PaletteFadeOut` (`_inc/Palette Fading.asm`). Só nomes de rótulos e lógica foram lidos; nenhum dado da ROM entrou no código: as tabelas (`LevelSizeArray`, `StartLocArray`, `LoopChunkNums`, `Level_Index`, blocos e artes) são lidas da ROM do usuário pelos endereços dos rótulos (`refs/rotulos.txt`).
- **Marca nova.** `Marca::PaletteFadeOut` (`0x1E4A`, o mesmo rótulo do primeiro desvanecimento, que é distinguido pelo estado do título). O console espera 0 quadros nela.
- **Estados novos.** `Titulo::PreparoB` (espera a marca), `Titulo::FadeOutB(n)` (22 VBlanks) e `Titulo::Parado` depois dele. `vblank_fade_out` é o passo comum dos dois desvanecimentos do título.

Decisões desta fatia (a IA decidiu, sem parar):

- **D7.13. Parte B em fatias que terminam numa espera de VBlank.** O comparador só compara a RAM nas esperas de VBlank e nos fechamentos fora de trechos atômicos. Uma fatia só pode ser provada se terminar numa espera. A fatia 1 termina na espera do segundo desvanecimento. Motivo: a RAM mudada num trecho sem espera só é vista na espera seguinte, então a fatia mínima provável é a que vai até ela. As fatias seguintes seguem o mesmo critério.
- **D7.14. Recorte da fase.** O cenário cobre a GHZ1 (zona 0, ato 0) sem lampião e sem créditos. Os outros casos devolvem `NaoImplementado`, nunca um valor padrão. Motivo: o título e a primeira demo só passam por esse caminho. Os outros casos ficam em `docs/backlog.md`.
- **D7.15. Contas de 16 e 32 bits.** As variáveis `d0`, `d1`, `d3` e `d4` das rotinas de rolagem aparecem como variáveis locais de cada função, com o tamanho e o sinal que a instrução usa (`wrapping_*`, `rotate_left(16)` para `swap`, `checked_div` para `divs.w` com o estouro que deixa o dividendo). Nada disso vira estado do núcleo: a RAM guarda só o que o original guarda.
- **D7.16. Limpeza de `$800` bytes no `LevelLayoutLoad`.** A referência (`FixBugs = 0`) limpa $800 bytes a partir de `v_lvllayout`, e não $400. Assim o `v_bgscroll_buffer` (`$FFA800`) também zera. O núcleo copia o efeito, e o teste sintético confere a sujeira em `$FFA7F0`.
- **D7.17. `RunPLC` ausente.** Dentro do desvanecimento a fila de arte está vazia (o `ClearPLC` do começo de `GM_Title` zerou). O `RunPLC` sai na primeira instrução com `v_plc_buffer` zero, então não entra na fatia. A fila (`NewPLC`, `ProcessPLC`) entra na fatia 2.

Provas sem a ROM (`cargo test -q -p sonic1-core -p sonic1-teste-som`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`: todos passam):

- `crates/teste-som/tests/cenario.rs`: `LevelSizeLoad` com a entrada da GHZ1 de uma ROM sintética (limites, posição de Sonic, câmera em X 0 e Y no limite de baixo, `v_256loop1` = `B57F1F20`); `DeformLayers` sem movimento (Y `$300`, 8 linhas de deslocamento vertical, 24 linhas de nuvem com `$FFFF` e 200 linhas zero: 224 no total); `LevelLayoutLoad` com uma linha de frente, uma de fundo e a limpeza de `$FFA7F0`. Os valores esperados saem das contas das rotinas, não de dados da ROM.
- `crates/teste-som/tests/cenario_layout.rs`: endereços de RAM e de tabelas da ROM conferidos contra `refs/rotulos.txt`.

Limites conhecidos desta fatia (ver `docs/backlog.md`): a fase é a GHZ1 sem lampião e sem créditos; o `DeformLayers` de outras zonas e o `DynamicLevelEvents` de GHZ2 e GHZ3 não existem; o fundo do título ainda não tem a parte de `DrawChunks`, que vem na fatia 2.

## Prova da fatia 1 com a ROM (Marcelo, 09/11/2026, 600 quadros, `sh scripts/etapa7-bloco2.sh ROM 600`)

- A marca `PaletteFadeOut` da parte B chegou no quadro 455 (oráculo). O núcleo rodou a parte B nesse mesmo quadro.
- Esperas de VBlank: 195 do núcleo contra 195 do oráculo no ponto de comparação, todas iguais, `diverge=0`, e `z80_diverge=0`. São as 173 anteriores mais as 22 do segundo desvanecimento.
- `posicao_z80_divergencias=0`. Interrupções tratadas: 196 (174 anteriores mais 22).
- O núcleo entrou em `Titulo(Parado)` no quadro 477, depois do desvanecimento. Dali em diante ele ignora as VBlanks (97 ignoradas, de 505 a 600). Isso é o limite da fatia, não divergência.
- O veredito continua `DIVERGE` só na contagem final: vídeo 27146 contra 42999, PSG 12 contra 48, YM do 68000 128 contra 582, Z80 798 contra 1226. As diferenças são a parte C (`ClearScreen`, `DrawChunks`, emblema, música e objetos do título), que é a fatia 2. Não há divergência no prefixo.
- Limite da comparação: `ram68k` com `diverge_em_trecho_atomico=533` não é comparada nesses quadros. A RAM da parte B só é provada nas esperas de VBlank, que é o que D7.13 pede.

## Título, parte B, fatia 2 e parte C (7.5, 09/11/2026): fila de arte, fundo e objetos do título

Escrito, sem prova com a ROM ainda. Rodada pendente: `sh scripts/etapa7-bloco2.sh ROM 530`, que deve mostrar a marca `Tit_MainLoop` no quadro 526.

- **O que roda.** Da chamada de `PaletteFadeOut` (segunda vez, quadro 455 na rodada anterior) até a marca `Tit_MainLoop`: a parte C (`parte_c` em `titulo.rs`), o desvanecimento de aparecimento com 22 VBlanks e a fila de arte, e o primeiro VBlank do título, que fica pendente. Os passos ficam em `mod.rs` (`PreparoC`, `FadeInC`, `PreparoTit`).
- **Fila de arte (`plc.rs`).** `NewPLC`, `ClearPLC`, `RunPLC`, `ProcessPLC_9Tiles` e `ProcessPLC`. A descompressão Nemesis anda uma linha de 8 pixels por vez. O estado entre os quadros fica nas variáveis `v_plc_*` da RAM, do mesmo jeito que no original, e a comparação enxerga cada uma delas.
- **Desenho do fundo (`desenho.rs`).** `DrawChunks`, `DrawBlocks_LR_2`, `GetBlockData`, `DrawBlock` e `Calc_VRAM_Pos`, com as quatro combinações de flip.
- **Objetos (`objetos.rs`).** `TitleSonic` (0E, rotinas 0 a 6) e `PSBTM` (0F, rotinas 0 a 4), com `AnimateSprite` para os códigos de fim `$FF` e `$FE`. Os outros códigos devolvem `NaoImplementado`.
- **Artes e dados da ROM.** Tudo vem da ROM do usuário pelos rótulos: `ArtLoadCues`, `Eni_Title`, `Nem_GHZ_1st`, `Ani_TSon`, `Ani_PSBTM`, `Map_TSon`, `Map_PSB`. A música é `bgm_Title = $8A`, que vai para `v_soundqueue1`. A paleta é `palid_Title = 1`.

Decisões desta fatia:

- **D7.18. A fatia 2 termina na marca `Tit_MainLoop`.** A marca chega logo antes da primeira espera do laço do título. Assim a fatia termina numa espera, como D7.13 pede. O VBlank do título (rotina `$04`) ainda não existe, e entra na fatia 3.
- **D7.19. Fila de arte por quadro com o estado em RAM.** O decodificador segue `NemPCD_NewRow`, `NemPCD_ProcessCompressedData` e `NemDec_BuildCodeTable` do disassembly, lendo a tabela de códigos de `v_ngfx_buffer`. Um teste sintético confere que, em dois quadros (9 e 3 tiles) e no modo XOR, as escritas na VDP são iguais às do `NemDec` completo, e que a tabela na RAM é igual.
- **D7.20. Chunk zero e `a0` (risco).** Em `GetBlockData`, um chunk zero não atualiza `a0`, e `DrawBlock` lê as flags de `(a0)`. O original usa o que sobrou no registrador. Aqui `a0` começa em `v_palette_water + $80` (`$FFFB00`): é o que `FadeOut_DecColor` deixa nele depois de escurecer as 64 cores da paleta de água, e `ClearScreen` não o muda. Antes começava em zero e a primeira rodada deu pânico ao ler fora da RAM (correção `d0545f7`). Se a comparação acusar diferença nas linhas do fundo, esta é a causa provável.
- **D7.21. Parte alta de `d5` (risco).** O `RunPLC` guarda `d5` em `v_plc_dataword`. A parte alta desse registrador no original é o que sobrou de instruções anteriores, e aqui é zero. Se a comparação acusar diferença nessa variável, esta é a causa.
- **D7.22. Fim da arte sem limpar a última posição.** `ProcessPLC_ShiftCue` (FixBugs = 0) não limpa a última posição da fila, como no original. Isso é o comportamento da referência, não um bug do núcleo.
- **D7.23. Animação parcial.** `AnimateSprite` cobre os códigos `$FF` e `$FE`. Se o roteiro da tela do título usar outro código, o núcleo para com `NaoImplementado("flag de animação")` e não segue em silêncio.

Provas sem a ROM: `cargo test -q -p sonic1-core -p sonic1-teste-som` (todos passam, inclusive `crates/teste-som/tests/plc.rs` com 3 testes), `cargo clippy --workspace --all-targets -- -D warnings` (sem erro), `cargo fmt` aplicado.

## Fatia 2 com a ROM: travamento na música do título (Marcelo, 09/11/2026, rodadas de 530 quadros)

- Rodada 1: pânico em `desenho.rs` (leitura fora da RAM, ver D7.20). Corrigido em `d0545f7`.
- Rodadas 2 a 4: o comparador cresceu até ser morto (OOM com 3,6 GB, depois SIGTERM sob o teto de 6 GB do `systemd-run`). O `NemDec_BuildCodeTable` da fila lia zeros para sempre se faltasse o `$FF` final: virou erro (`1edb5f7`), mas não era a causa.
- Rodadas 5 e 6, com diagnóstico (uma linha por quadro no comparador, `ea9c260`, e rastreio temporário no núcleo): até o quadro 504 as filas do comparador ficam estáveis (esperas de 0 a 2, 54000 escritas guardadas, nenhuma divergência do Z80). A parte C roda inteira no quadro 505 e termina. O travamento é no 19º VBlank do aparecimento (faltam 4): o tratador de VBlank não volta.
- Causa provável: o laço de `UpdateMusic` que espera o DAC aceitar comandos (bit 7 de `zDAC_Status`, `$1FFD`) não tinha limite. A música do título (`$8A`) é a primeira que o núcleo toca, e a trilha de DAC põe o Z80 a tocar amostras. Esse bit fica ligado só enquanto o Z80 escreve uma amostra no YM. No console o Z80 anda enquanto o 68000 solta o barramento e tenta de novo. No núcleo o Z80 só anda ao consumir pontos gravados. Com a fila de pontos vazia, ele para dentro da janela, o bit nunca limpa, e cada volta empilha saídas de BUSREQ e registros de acesso.

Decisão:

- **D7.24. Laço do DAC ocupado com limite.** `UpdateMusic` desiste depois de 1000 voltas (`LIMITE_VOLTAS_DAC_OCUPADO`) e segue com o Z80 parado, que é o estado do caminho normal depois do laço. A janela de ocupado tem 45 ciclos do Z80, então no console poucas voltas bastam. Cada desistência conta em `esperas_estouradas` e deixa o aviso `EsperaDoSomEstourada` na `Memoria`, porque o tratador cria um `Driver` novo a cada VBlank e o contador se perderia. As outras duas esperas do driver (BUSACK e YM ocupado) deixam o mesmo aviso. O comparador imprime os avisos do núcleo (`aviso_nucleo`) e não dá OK com esse aviso. Teste: `dac_ocupado_sem_tempo_gravado_desiste_conta_e_avisa` em `crates/teste-som/tests/driver.rs`.

Achados:

- Laço do DAC ocupado sem limite: **BLOQUEIA**, corrigido (D7.24).
- Por que a fila de pontos do Z80 acaba nesse VBlank (o núcleo faz mais acessos ao Z80 que o oráculo, ou o Z80 do núcleo está em outro ponto): **BLOQUEIA**. A próxima rodada mostra o aviso, as divergências do Z80 e o primeiro acesso diferente.
- `BuildSprites` gira para sempre se a contagem de uma fila de prioridade for ímpar: **BACKLOG**. O original faz o mesmo, e isso só acontece com a RAM já errada.
- As escritas guardadas no comparador crescem durante a rodada: **DESCARTADO**. São dezenas de milhares, e o teto de 2 milhões para com erro.
