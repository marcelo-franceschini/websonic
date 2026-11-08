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
