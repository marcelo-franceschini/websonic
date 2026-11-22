# Abertura (Etapa 7)

Plano: `docs/arquivo/planos/plano-etapa-7.md`. Cada afirmação cita o comando que a sustenta. Nada aqui tem dado da ROM, só rótulos, quadros e `mclk`.

## Extensão experimental da fase (13/11/2026)

- O modo nominal entra no laço de fase após `Level_StartGame`. O comparador histórico continua parando nesse ponto; testes novos podem chamar `continuar_apos_abertura`.
- `CARGO_TARGET_DIR=/tmp/websonic-diferencial-abertura cargo run --locked --offline --release --manifest-path /tmp/websonic-comparacao-isolada/diferencial-abertura/Cargo.toml --bin abertura -- comparar ROM 1170 refs/rotulos.txt`: veredito `IGUAL`, 1166 imagens, VRAM, CRAM e VSRAM iguais, 822 esperas iguais. O manifesto e o lock do comparador foram copiados para `/tmp`, com lock regenerado apenas nessa cópia; os originais ficaram intactos.
- `cargo test --locked --offline -p sonic1-core -p sonic1-teste-som`: testes passaram. A execução nominal com `console_hashes ROM 1300` avança na demo até `NaoImplementado("Sonic no ar ou rolando")`. Este caminho usa tempo nominal e não tem prova de fidelidade de fase.
- Uma cópia descartável do comparador, alterada apenas em `/tmp` para continuar depois da marca e não cortar as escritas, rodou 1220 quadros. Comparou 1220 imagens iguais e 876 esperas iguais; o veredito foi `DIVERGE` por contagem final de PSG e YM, com aviso de leituras gravadas ausentes no fechamento. A causa ainda não foi provada. Esta experiência não substitui o comparador congelado.
- `cargo test --locked --offline -p sonic1-core testes_mola` passou nos casos sintéticos de inicialização e lançamento vertical e horizontal. `testes_esquerda` passou para carregamento de objetos ao recuar a câmera e para recuperação do contador quando falta espaço. Eles não provam a fidelidade das rotinas contra o oráculo.

## Medidas do bloco 1 (Marcelo, 08/11/2026, `sh scripts/arquivo/etapa7-bloco1.sh`, caso `boot-demos`, sem botões)

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
- Ainda sem prova com a ROM real. Bloco 2 (`scripts/arquivo/etapa7-bloco2.sh`, 61 quadros, do ligar até a entrada de `GM_Sega`).

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

## Experimento do nível 2 (7.8): primeira medida (Marcelo, 08/11/2026, `scripts/arquivo/etapa7-bloco2.sh`, 330 quadros)

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
- Defeito achado pelo diagnóstico (`scripts/arquivo/etapa7-bloco5b.sh`): a imagem não mudava porque a CRAM ficava zerada (`cram_nao_zero=0`). Causa: a ponte lia os registradores de DMA no renderizador, que guarda as escritas atrás de um DMA ou preenchimento em andamento; o modo de preenchimento antigo escondia o DMA da paleta. Correção: a ponte guarda a própria cópia dos registradores 1 e 19 a 23. Depois da correção: `cram_nao_zero=64` a partir do quadro 100, VRAM com 3793 bytes não-zero e imagem ligada.
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

## Prova da fatia 1 com a ROM (Marcelo, 09/11/2026, 600 quadros, `sh scripts/arquivo/etapa7-bloco2.sh ROM 600`)

- A marca `PaletteFadeOut` da parte B chegou no quadro 455 (oráculo). O núcleo rodou a parte B nesse mesmo quadro.
- Esperas de VBlank: 195 do núcleo contra 195 do oráculo no ponto de comparação, todas iguais, `diverge=0`, e `z80_diverge=0`. São as 173 anteriores mais as 22 do segundo desvanecimento.
- `posicao_z80_divergencias=0`. Interrupções tratadas: 196 (174 anteriores mais 22).
- O núcleo entrou em `Titulo(Parado)` no quadro 477, depois do desvanecimento. Dali em diante ele ignora as VBlanks (97 ignoradas, de 505 a 600). Isso é o limite da fatia, não divergência.
- O veredito continua `DIVERGE` só na contagem final: vídeo 27146 contra 42999, PSG 12 contra 48, YM do 68000 128 contra 582, Z80 798 contra 1226. As diferenças são a parte C (`ClearScreen`, `DrawChunks`, emblema, música e objetos do título), que é a fatia 2. Não há divergência no prefixo.
- Limite da comparação: `ram68k` com `diverge_em_trecho_atomico=533` não é comparada nesses quadros. A RAM da parte B só é provada nas esperas de VBlank, que é o que D7.13 pede.

## Título, parte B, fatia 2 e parte C (7.5, 09/11/2026): fila de arte, fundo e objetos do título

Escrito, sem prova com a ROM ainda. Rodada pendente: `sh scripts/arquivo/etapa7-bloco2.sh ROM 530`, que deve mostrar a marca `Tit_MainLoop` no quadro 526.

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
- Por que a fila de pontos do Z80 acaba nesse VBlank (o núcleo faz mais acessos ao Z80 que o oráculo, ou o Z80 do núcleo está em outro ponto): **BLOQUEIA**. Causa achada na rodada seguinte: quadro de lag (D7.25).
- `BuildSprites` gira para sempre se a contagem de uma fila de prioridade for ímpar: **BACKLOG**. O original faz o mesmo, e isso só acontece com a RAM já errada.
- As escritas guardadas no comparador crescem durante a rodada: **DESCARTADO**. São dezenas de milhares, e o teto de 2 milhões para com erro.

## Fatia 2, rodada com o limite do DAC: quadro de lag (Marcelo, 09/11/2026, 530 quadros)

- A rodada terminou em 13 segundos, com pico de 489 MB (journal do systemd). O estouro de memória acabou.
- A marca `Tit_MainLoop` chegou no quadro 526, como previsto. Esperas: 218 do núcleo contra 221 do oráculo (as 3 a mais são do laço do título, que é a fatia 3). 196 iguais e 22 diferentes. A primeira diferente é a 196, logo depois do primeiro VBlank do aparecimento da parte C. Divergem só variáveis do driver de som e `v_vblank_byte`. A paleta e a fila de arte batem.
- `posicao_z80_divergencias=810` e 4 desistências do laço do DAC (D7.24). O primeiro acesso diferente ao Z80 é do segundo VBlank do aparecimento: o oráculo pede o barramento e lê o status do DAC direto, sem o par de BUSREQ das transferências padrão.
- Causa: esse VBlank foi um quadro de lag no oráculo. O primeiro VBlank do aparecimento começa a música do título (as vozes de todos os canais do YM, com as esperas do chip), e o laço principal não chega à espera seguinte antes do VBlank. Então `v_vblank_routine` está em zero e o tratador só roda o driver de som e o contador. O núcleo tratou esse VBlank como um passo do aparecimento, terminou os 22 passos um VBlank antes, e o VBlank que sobrou foi ignorado (quadro 526, `PreparoTit`). Daí a música e o contador defasados de um tique nas 22 esperas, e a fila do Z80 desalinhada até o laço do DAC.
- O vídeo (41225 contra 41125) só difere nos quadros 527 a 529, que são do VBlank do título (fatia 3).

Decisão:

- **D7.25. Quadro de lag pelo tempo gravado.** `WaitForVBlank` (`$29A0`) vira marca. O núcleo grava o pedido da rotina no fim do trecho, como antes, mas só registra a espera quando a marca chega. Uma interrupção antes da marca é quadro de lag: o tratador roda com a rotina zero, o pedido volta depois, e o estado do laço não muda. Nos estados de preparo (trechos atômicos), as interrupções, que antes eram ignoradas, agora rodam como lag, como no original. As exceções são o boot, o fim da fatia (`Parado`) e a interrupção que fecha o tratador adiado da voz SEGA. O console, sem tempo gravado, manda a marca junto com cada VBlank (sem lag). Motivo: o requisito põe os quadros de lag e o fim de cada espera no tempo gravado do nível 1, e a chegada à espera é o fato que separa um caso do outro. Testes: `vblank_antes_da_espera_e_quadro_de_lag` e `marca_de_espera_sem_pedido_nao_registra_nada` em `crates/teste-som/tests/abertura.rs`.

Riscos (**BACKLOG**):

- Uma interrupção entre o `move.b` do pedido e a chegada a `WaitForVBlank` (poucos ciclos) roda o tratador completo no original e o de lag no núcleo. Se a comparação acusar, a marca tem de ser a escrita do pedido.
- O núcleo roda o trecho inteiro antes do tratador de lag. Se o trecho e o tratador usarem a mesma memória (fila de som, contador de VBlank), a ordem muda. Se a comparação acusar, entram marcas no meio do trecho (D7.10).
- O tratador de lag no modo de fase (`VBla_00`, ramo da fase) ainda não existe: com a rotina zero o núcleo só roda o som. Entra com a fase da demo.

Achados:

- VBlank de lag tratado como passo do aparecimento: **BLOQUEIA**, corrigido (D7.25). Provado na rodada seguinte.

## Prova da fatia 2 com a ROM (Marcelo, 09/11/2026, 530 quadros, `sh scripts/arquivo/etapa7-bloco2.sh ROM 530`)

- Rodada de 12,7 segundos, com pico de 435 MB.
- Marcas: `PaletteFadeIn` nos quadros 371 e 504, `Tit_MainLoop` no 526 (e em cada quadro do laço do título), `WaitForVBlank` 221 vezes, do quadro 61 ao 529.
- Esperas de VBlank: 218 do núcleo, todas iguais às do oráculo (`diverge=0`, `z80_diverge=0`). O oráculo tem 221: as 3 a mais são do laço do título, que é a fatia 3.
- `posicao_z80_divergencias=0`. RAM do 68000 e do Z80 com `diverge=0` fora dos trechos atômicos. Nenhum aviso do núcleo.
- Interrupções: 223 do oráculo, 219 tratadas pelo núcleo (1 de lag, a do segundo VBlank do aparecimento) e 4 ignoradas (quadros 527 a 530, `Parado`).
- Escritas: PSG 23 e YM do 68000 482 nos dois lados. As escritas conferem por prefixo. As contagens finais diferem só nos VBlanks do título (527 a 529), que o núcleo ainda ignora: vídeo 41225 contra 41125, YM pelo Z80 61498 contra 58498 (o Z80 do núcleo só anda quando o 68000 o toca) e BUSREQ 912 contra 895. Por isso o veredito é `DIVERGE`, como na prova da fatia 1. Não há divergência no prefixo.
- A fatia 2 (parte C até `Tit_MainLoop`) está provada. A próxima é a fatia 3: o laço do título até `GotoDemo`.

## Título, fatia 3 (7.5, 09/11/2026): laço do título, `GotoDemo` e começo de `GM_Level`

Escrita, sem prova com a ROM ainda. Rodada pendente: `sh scripts/arquivo/etapa7-bloco2.sh ROM 970`. O temporizador de 376 quadros só anda no VBlank do título, então o `GotoDemo` deve chegar perto do quadro 903, o `GM_Level` perto do 933, e o desvanecimento do começo da fase acaba perto do 955.

- **O que roda.** `Tit_MainLoop` (estado `Laco`), um quadro por VBlank: objetos, `DeformLayers`, `BuildSprites`, `PalCycle_Title`, `RunPLC`, o objeto 0 que anda 2 pixels e puxa a câmera, a entrada do código e o temporizador. Depois o laço de 30 quadros do `GotoDemo` (estado `PreDemo`: `DeformLayers`, `PaletteCycle` da GHZ, `RunPLC` e o objeto 0), a preparação da demo e o começo de `GM_Level` até o fim do `PaletteFadeOut` (estado `Fase`).
- **VBlank do título (`VBlank_Title`, rotina `$04`).** Transferências padrão, `LoadTilesAsYouMove_BGOnly`, `ProcessPLC_9Tiles` e o temporizador genérico.
- **Desenho do fundo durante a rolagem (`desenho.rs`).** `DrawBG_Top` e `DrawBG_Bottom` (REV01), as faixas `DrawBlocks_LR`, `DrawBlocks_LR_3`, `DrawBlocks_TB` e `DrawBlocks_TB_2`, e as variantes `GetBlockData_2` e `Calc_VRAM_Pos_2`. Na tela do título só o `DrawBG_Bottom` desenha: o `Deform_GHZ` da REV01 move só os blocos 2 e 3 do fundo, e as flags do bloco 1 nunca ligam. O bloco 2 (colinas) anda meio pixel por pixel da câmera e pede uma coluna de três blocos a cada 16 pixels.
- **Ciclos de paleta (`paleta.rs`).** `PalCycle_Title` e `PaletteCycle` da GHZ, que são o mesmo código (`PCycGHZ_Go`) com tabelas diferentes.
- **Código e Start.** A entrada do código da seleção de fases (`Tit_ChkRegion` a `Tit_CountC`, FixBugs = 0) e o botão C estão escritos e testados com ROM sintética. Na rodada da demo não há botões.

Decisões desta fatia:

- **D7.26. `a0` no desenho do fundo do VBlank.** Num chunk zero, `DrawBlock` lê as flags de `(a0)` (como na D7.20). No VBlank do título, `a0` é o que o `ReadJoypads` deixa: `v_jpadhold1 + 4`, depois de gravar os dois controles. As macros de DMA de `VBlank_StandardTransfers` só usam `a5`. O valor sai do código, não de um registrador que sobrou de antes, então não é risco.
- **D7.27. Fim da fatia.** A fatia termina no fim do `PaletteFadeOut` do começo de `GM_Level` (estado `Fase(Parado)`). Assim a preparação da demo e o começo da fase são provados nas 22 esperas desse desvanecimento (D7.13). O resto da carga da fase entra na próxima fatia.
- **D7.28. Marca `GotoDemo` só para o relatório.** O núcleo decide o fim do laço do título pelo próprio temporizador (`v_generictimer`). A marca mostra em que quadro o oráculo chegou lá; se o núcleo errar o temporizador, a comparação acusa nas esperas.
- **D7.29. O que não existe para.** Start no título (`Tit_ChkLevSel`), a volta à Sega em `$1C00`, a demo do estágio especial, os ciclos de paleta de outras zonas e o fundo da SBZ devolvem `NaoImplementado`. Nenhum acontece na rodada da demo.

Provas sem a ROM: `cargo test -q -p sonic1-core -p sonic1-teste-som -p sonic1-console` (95 testes, 13 novos em `crates/teste-som/tests/titulo_laco.rs`), `cargo clippy --workspace --all-targets -- -D warnings` e o clippy do comparador sem erro, `cargo fmt` aplicado.

## Prova da fatia 3 com a ROM (Marcelo, 09/11/2026, 970 quadros, `sh scripts/arquivo/etapa7-bloco2.sh ROM 970`)

- Rodada de 13,9 segundos, com pico de 430 MB.
- Marcas: `Tit_MainLoop` 376 vezes, do quadro 526 ao 901 (o temporizador de 376 quadros, exato). `GotoDemo` no 902. `MainGameLoop`, `GM_Level` e `PaletteFadeOut` no 932.
- Esperas de VBlank: 645 do núcleo, todas iguais às do oráculo (`diverge=0`, `z80_diverge=0`). O oráculo tem 652: as 7 a mais são da carga da fase, depois do fim da fatia.
- `posicao_z80_divergencias=0`. RAM do 68000 e do Z80 com `diverge=0` fora dos trechos atômicos. Nenhum aviso do núcleo.
- Interrupções: 657 do oráculo, 647 tratadas pelo núcleo (1 de lag) e 10 ignoradas, todas em `Fase(Parado)`, a partir do quadro 962. Entre o fim do desvanecimento e o 962 não há interrupção: o `GM_Level` desliga as interrupções para descomprimir a arte dos cartões de título.
- Escritas: conferem por prefixo. As do DAC pelo Z80 batem até na contagem final (213416). Vídeo (55943 contra 52375), PSG (269 contra 251), YM do 68000 (2004 contra 1616) e BUSREQ (2806 contra 2776) diferem só depois do fim da fatia, na carga da fase. Por isso o veredito é `DIVERGE`, como nas provas das fatias 1 e 2.
- A abertura está provada do quadro 0 ao fim do `PaletteFadeOut` do começo de `GM_Level` (quadro ~954), com zero divergência nas esperas. Falta a fase da demo até `Level_StartGame` (~1166).

## Fase da demo, fatia 1 (09/11/2026): até o laço dos cartões de título

Escrita, sem prova com a ROM ainda. Rodada pendente: `sh scripts/arquivo/etapa7-bloco2.sh ROM 1170`. O laço dos cartões dura enquanto a fila de arte da GHZ descomprime, nove tiles por VBlank.

- **O que roda.** Do fim do `PaletteFadeOut` de `GM_Level` até a marca `Level_TtlCardLoop`, de uma vez (estado `PreparoCartoes`): a arte dos cartões (NemDec direto na VRAM), a primeira lista de PLC do cabeçalho da zona e a `plcid_Main2` (`AddPLC`, somadas ao fim da fila), o `Level_ClrRam`, o `ClearScreen`, os registradores da VDP da fase, 30 segundos de ar, a paleta do Sonic, a música da zona (`MusicList`, pela `QueueSound1`) e o objeto `TitleCard`. Depois o laço (estado `Cartoes`), um quadro por VBlank: objetos, sprites e `RunPLC`, até o "ACT" chegar ao alvo e a fila esvaziar. A fatia termina aí (`Fase(Parado)`).
- **VBlank dos cartões (`VBlank_TitleCards`, rotina `$0C`).** As transferências com a linha do HBlank e a rolagem antes dos sprites, a arte do Sonic quando muda, as cópias `_dup` das posições e das flags, o `LoadTilesAsYouMove` completo (frente, `DrawBG_Top`, `DrawBG_Bottom` e `DrawBG_Block3`), o `AnimateLevelGfx` (anel gigante e GHZ), o `HUD_Update` e o `ProcessPLC_9Tiles`.
- **Objeto `TitleCard` (`34`).** As quatro rotinas: a carga, que cria os quatro elementos pelas tabelas `Card_ConData` e `Card_ItemData`; a entrada, 16 pixels por quadro; a espera e a saída, 32 pixels por quadro, com o `AddPLC` das explosões e dos animais (essas duas só rodam depois de `Level_StartGame`).

Decisões desta fatia:

- **D7.30. Quadros de lag no preparo dos cartões.** As interrupções ficam ligadas em duas janelas do preparo: depois da arte dos cartões (as `AddPLC` e o `Level_ClrRam`) e depois do `Level_LoadPal` (a paleta, a música e o objeto). O núcleo roda o preparo inteiro na marca, então um quadro de lag nessas janelas roda antes dele (D7.25). Na primeira janela a ordem não importa: o `Level_ClrRam` não limpa a RAM do driver de som (`$F000`) nem o contador de VBlank (`$FE0C`), e o tratador de lag só mexe nisso. Na segunda, a fila da música é escrita só no fim, poucas instruções antes da espera. Um lag entre essa escrita e a espera tocaria a música um VBlank antes no original; é o segundo risco da D7.25, e a comparação acusaria nas esperas.
- **D7.31. O que não existe para.** A demo dos créditos (`f_demo` negativo), a água da LZ, os fundos especiais da MZ e da SBZ, os tiles animados das outras zonas, os números do HUD e o HUD de depuração devolvem `NaoImplementado`. Na demo da GHZ as flags do HUD estão em zero até depois dos cartões, então o `HUD_Update` não escreve nada nesta fatia.

Provas sem a ROM: `cargo test -q -p sonic1-core -p sonic1-teste-som -p sonic1-console` (102 testes, 7 novos em `crates/teste-som/tests/fase_cartoes.rs`), o clippy dos dois workspaces sem erro e `cargo fmt` aplicado.

## Prova da fase da demo, fatia 1 (Marcelo, 09/11/2026, 1170 quadros, `sh scripts/arquivo/etapa7-bloco2.sh ROM 1170`)

- Marcas: `Level_TtlCardLoop` 151 vezes, do quadro 962 ao 1114. O laço dos cartões acaba no 1114 e o núcleo para ali (`Fase(Parado)`), as 55 interrupções ignoradas vêm depois.
- Esperas de VBlank: 796 do núcleo, todas iguais às do oráculo (`diverge=0`, `z80_diverge=0`). O oráculo tem 826: as 30 a mais são do resto da carga da fase.
- `posicao_z80_divergencias=0`. RAM do 68000 e do Z80 com `diverge=0` fora dos trechos atômicos. Quadros de lag do núcleo: 5. Nenhum aviso.
- As contagens finais diferem (vídeo 96964 contra 84985, YM do Z80 281706 contra 267516) só depois do fim da fatia. Veredito `DIVERGE`, como nas fatias anteriores, sem divergência no prefixo.
- Provado do quadro 0 ao 1114. Falta o resto da carga da fase até `Level_StartGame` (~1166): `LevelDataLoad`, `LoadTilesFromStart`, objetos, `ObjPosLoad`, HUD e o desvanecimento final.

## Fase da demo, fatia 2 (09/11/2026): da saída dos cartões a `Level_StartGame`

Escrita, sem prova com a ROM ainda. Rodada pendente: `sh scripts/arquivo/etapa7-bloco2.sh ROM 1170`.

- **O que roda.** Na marca `Level_Delay` (de uma vez): a arte do HUD (`Hud_Base`), a paleta do Sonic, `LevelSizeLoad`, `DeformLayers`, `LevelDataLoad` (blocos, chunks, layout, paleta, segunda lista de arte), `LoadTilesFromStart` (frente e fundo da GHZ), `ColIndexLoad`, os objetos do Sonic e do HUD, `ObjPosLoad`, `ExecuteObjects`, `BuildSprites`, as variáveis de partida nova, `OscillateNumInit` e o temporizador da demo. Depois quatro VBlanks `$08` (`VBlank_Levels`), os 22 VBlanks do `PalFadeIn_Alt` (linhas 2 a 4) e, na marca `Level_StartGame`, os cartões começam a sair e o bit 7 do modo se apaga.
- **D7.32. O Sonic roda um quadro antes de `Level_StartGame`.** O plano dizia que ele ainda não rodou. Não é assim: o `ExecuteObjects` de `Level_LoadObj` executa `Sonic_Main` e cai em `Sonic_Control`, com os botões zerados. O núcleo porta só esse quadro (`fisica.rs`): `Sonic_MdNormal` no chão plano (pulo, rampa, movimento parado, rolar, limites, `SpeedToPos`, `Sonic_AnglePos` só no modo de piso, `Sonic_SlopeRepel`), `Sonic_Animate` com os roteiros comuns, `Sonic_Loops`, `Sonic_LoadGfx` e `ReactToItem` sem objetos. O resto devolve `NaoImplementado` e é da Etapa 9. A física completa (parede, teto, ar, andar) continua fora.
- **D7.33. `a0` em `LoadTilesFromStart`.** Num chunk vazio o `DrawBlock` lê as flags de `(a0)`. Na entrada de `LoadTilesFromStart`, `a0` é o fim da última linha copiada do layout de fundo (`LevelLayoutLoad2`), e o núcleo calcula esse valor. No fundo da GHZ, cada linha recarrega `a0` com o endereço da tabela de blocos de rolagem.
- **D7.34. Objetos da partida descobertos pela rodada.** Os objetos da lista da fase (`ObjPos`) são criados de verdade. Cada ID sem código devolve `ObjetoNaoImplementado(id)`, e a rodada diz qual falta. Só se implementa o que a primeira tela da GHZ1 pede.
- **FixBugs = 0.** `FindNearestTile` devolve `$FFFFFF00` num chunk vazio e o original lê a RAM ali; o núcleo reproduz.

## Prova da fase da demo, fatia 2 (Marcelo, 09/11/2026, 1170 quadros): `Level_StartGame` alcançado

- Marcas: `Level_Delay` no quadro 1140 e `Level_StartGame` no 1166, o fim da Etapa 7 pelo plano. O núcleo chegou lá sem erro (`Fase(Parado)` a partir do 1167).
- Esperas de VBlank: 822 do núcleo, todas iguais às do oráculo (`diverge=0`, `z80_diverge=0`). O oráculo tem 826: as 4 a mais são dos quadros 1167 a 1170, depois do fim. `posicao_z80_divergencias=0`. Nenhum aviso. RAM do 68000 e do Z80 com `diverge=0` fora dos trechos atômicos.
- Quadros de lag do núcleo: 30 (a carga da fase leva cerca de 26 quadros de VBlank sem a espera, e o núcleo os trata como lag, como o oráculo).
- Escritas conferem por prefixo. As contagens finais diferem só pelos 4 quadros depois do fim (vídeo 96554 contra 96964, PSG 531 contra 543, YM do 68000 2944 contra 3004). O veredito `DIVERGE` vem só dessas contagens.
- Objetos criados na partida da GHZ1: Sonic, HUD, anéis (`25`) e monitores (`26`). Nenhum outro ID faltou.
- **Zero divergência do quadro 0 a `Level_StartGame` (1166)** na RAM, nas esperas, no Z80 e na ordem das escritas. Falta a comparação da imagem, da VRAM, da CRAM e da VSRAM por quadro (7.7), o navegador e o fechamento.

## Critério da Etapa 7 cumprido (Marcelo, 09/11/2026, 1170 quadros, `sh scripts/arquivo/etapa7-bloco2.sh ROM 1170`): veredito IGUAL

- Do quadro 0 a `Level_StartGame` (quadro 1166): **zero divergência**.
- RAM do 68000 e do Z80 nas esperas de VBlank (822 iguais) e nos fechamentos fora de trechos atômicos; posição do Z80 (`posicao_z80_divergencias=0`); ordem e conteúdo das escritas de vídeo, PSG, YM do 68000 e BUSREQ (contagens finais iguais); YM do Z80 igual no prefixo.
- **Vídeo (7.7):** 1166 quadros comparados, `imagem=1166/0 vram=1166/0 cram=1166/0 vsram=1166/0` (iguais/diferentes). O renderizador de vídeo recebe só as escritas do núcleo, na ordem dele, com a posição (linha, coluna) do oráculo para a escrita de mesmo índice e o DMA lido da RAM do núcleo. O quadro 1166 e os 4 seguintes não entram: a marca cai no meio do quadro 1166 e o resto é a fase em andamento (Etapa 9).
- Defeitos achados e corrigidos pela comparação, na ordem: laço do DAC sem limite (D7.24), quadro de lag (D7.25), objetos `25` e `26` da partida, DMA da paleta lido depois do passo do laço principal (o observador recolhe as escritas logo depois do tratador de VBlank).
- Regras do comparador, para o fechamento: a contagem final das escritas esperadas corta no `mclk` da marca `Level_StartGame`; as escritas do Z80 no YM podem terminar atrás do oráculo (o Z80 do núcleo só anda quando o 68000 o toca, nível 1), nunca diferentes.
- Limites aceitos, no backlog: a física do Sonic fora do chão plano, objetos além de Sonic, HUD, anéis e monitores, e as zonas além da GHZ.

## Continuação de GHZ e Marble (14/11/2026)

- Eventos de GHZ2 e GHZ3, poste de checkpoint, chefe, cápsula e objetos adicionais foram implementados a partir do disassembly. As rotinas ainda precisam de comparação por quadro durante o jogo.
- Marble tem carga e animação iniciais de cenário. Um percurso com a ROM REV01 alcançou X=0x2A3 em MZ1, antes de encontrar um objeto ainda não implementado. Isso não valida a fidelidade visual da zona.
- A comparação de abertura com o oráculo solicitada para 2849 quadros confrontou somente 1166 imagens. O modo usado para no início da fase e deixou os quadros seguintes pendentes. O veredito geral não prova a demo.
- Comparação no Chrome entre composição GPU e renderizador CPU: 2849 quadros executados, 49 imagens amostradas e nenhuma diferença de pixel. Não é comparação com o oráculo.
- O relógio calcula a fração temporal, mas o renderizador ainda não a usa para interpolar posições. O seletor do site não anuncia interpolação até que ela esteja conectada.

## Verificação de continuidade (15/11/2026)

- Comparador de abertura, com 1170 quadros solicitados e ROM REV01: `IGUAL`, com 1166 imagens, VRAM, CRAM e VSRAM comparadas sem divergências. O comparador parou no início da fase, deixando quatro quadros pendentes.
- `SONIC1_ROM=<ROM REV01> cargo test -p sonic1-core --lib -- --ignored`: dez testes passaram, incluindo inicialização de Labyrinth, colisão GOAL da Special Stage e transição para resultados. A posição de Sonic no GOAL é preparada pelo teste; isso não valida o percurso completo.
- O console nominal ainda não agenda HBlank na linha da água. A troca de paleta no navegador e sua temporização permanecem sem validação contra o oráculo.
