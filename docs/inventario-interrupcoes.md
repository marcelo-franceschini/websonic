# Inventário de interrupções por rotina

Gerado por `cargo xtask inventario-interrupcoes`. Só nomes de rotina, níveis, contagens e deslocamentos dentro da rotina.

Fonte: arquivos `.s1tempo` dos casos: boot-demos, demos-quatro, lz-agua, partida-ghz1.

Quadros: 29209. Com VBlank gravado: 26347. Quadros de lag (VBlank com o PC fora de `WaitForVBlank`): 337.

Cada linha é uma rotina do original que o corpus interrompeu. É a lista de onde os pontos de checagem podem ser necessários: uma rotina que lê ou escreve memória compartilhada (`docs/inventario-memoria.md`) e aparece aqui precisa de ponto.

| rotina | nível | vezes | menor deslocamento | maior deslocamento |
| --- | --- | --- | --- | --- |
| WaitForVBlank | 6 | 26078 | 0x4 | 0x8 |
| WaitForVBlank | 4 | 1267 | 0x4 | 0x8 |
| Kos_RLELoop | 6 | 107 | 0x0 | 0xa |
| Deform_LZ | 4 | 82 | 0x62 | 0x8c |
| BuildSprites | 4 | 48 | 0x14 | 0xd4 |
| BuildSpr_Normal | 4 | 45 | 0x0 | 0x32 |
| GetBlockData_2 | 6 | 22 | 0xc | 0x46 |
| OscillateNumDo | 4 | 20 | 0x0 | 0x4e |
| boss_mz_x | 6 | 20 | 0x2 | 0x48 |
| NemDec_ProcessCompressedData | 6 | 18 | 0x0 | 0x26 |
| Kos_Loop | 6 | 12 | 0x0 | 0x16 |
| Kos_RLE | 6 | 12 | 0x0 | 0x28 |
| EniDec_FetchInlineValue | 6 | 11 | 0x0 | 0x28 |
| Kos_RLELoop | 4 | 10 | 0x4 | 0x6 |
| NemPCD_WritePixel | 6 | 10 | 0x0 | 0x8 |
| BuildSprites | 6 | 9 | 0x2e | 0xdc |
| Level_LoadPal | 6 | 9 | 0xa | 0xa |
| Level_NoMusicFade | 6 | 9 | 0x2a | 0x2a |
| BuildSpr_Normal | 6 | 7 | 0x6 | 0x18 |
| Kos_SeparateRLE2 | 6 | 7 | 0x2 | 0xc |
| NemPCD_WritePixel_Loop | 6 | 7 | 0x0 | 0x4 |
| SS_Scroll_CloudsBubbles | 6 | 7 | 0x26 | 0x38 |
| BuildSpr_Draw | 4 | 6 | 0x0 | 0xe |
| DrawBlock | 6 | 6 | 0x2 | 0x20 |
| Kos_SeparateRLE | 6 | 6 | 0x6 | 0x10 |
| EniDec_FetchByte | 6 | 5 | 0x6 | 0xe |
| ExecuteObjects | 4 | 4 | 0x12 | 0x20 |
| ExecuteObjects | 6 | 4 | 0x10 | 0x20 |
| NemBCT_ShortCode_Loop | 6 | 4 | 0x0 | 0x4 |
| Sball_Twirl | 4 | 4 | 0x3e | 0x44 |
| DrawBlocks_LR_3 | 6 | 3 | 0xc | 0xc |
| EniDec_101 | 6 | 3 | 0x8 | 0x8 |
| Kos_Loop | 4 | 3 | 0x2 | 0x4 |
| Kos_RLE | 4 | 3 | 0x0 | 0x6 |
| Level_SkipScroll | 4 | 3 | 0x0 | 0x14 |
| NemPCD_NoLoadField01 | 6 | 3 | 0x4 | 0x4 |
| SignpostArtLoad | 6 | 3 | 0x4 | 0x14 |
| Tilemap_Cell | 6 | 3 | 0x0 | 0x2 |
| Deform_GHZ | 6 | 2 | 0xb2 | 0xe8 |
| DisplaySprite | 4 | 2 | 0x16 | 0x16 |
| DrawBlocks_LR_2 | 4 | 2 | 0x12 | 0x20 |
| DrawBlocks_LR_2 | 6 | 2 | 0x12 | 0x20 |
| DrownCount | 4 | 2 | 0x0 | 0x0 |
| EniDec_111 | 6 | 2 | 0x0 | 0xa |
| Eni_Loop | 6 | 2 | 0x22 | 0x22 |
| FindFloor2 | 4 | 2 | 0x24 | 0x24 |
| GetBlockData | 6 | 2 | 0x0 | 0x0 |
| GetBlockData_2 | 4 | 2 | 0x32 | 0x3c |
| Kos_SeparateRLE | 4 | 2 | 0xa | 0x10 |
| NemPCD_ProcessCompressedData | 6 | 2 | 0x0 | 0x0 |
| NemPCD_WriteRowToVDP_XOR | 6 | 2 | 0x4 | 0x4 |
| OPL_Main | 6 | 2 | 0x3a | 0x3a |
| OscillateNumDo | 6 | 2 | 0x30 | 0x30 |
| PalCycle_LZ | 4 | 2 | 0x4e | 0x78 |
| PauseGame | 4 | 2 | 0x14 | 0x68 |
| React_LoopObjects | 6 | 2 | 0x4 | 0x4 |
| RunPLC | 6 | 2 | 0x54 | 0x54 |
| SS_LoadData | 6 | 2 | 0x2e | 0x40 |
| SpeedToPos | 4 | 2 | 0x20 | 0x20 |
| DrawBlock | 4 | 1 | 0x12 | 0x12 |
| Drown_Animate | 4 | 1 | 0x0 | 0x0 |
| EniDec | 4 | 1 | 0x16 | 0x16 |
| EniDec | 6 | 1 | 0x16 | 0x16 |
| EniDec_111 | 4 | 1 | 0x0 | 0x0 |
| FadeIn_AddColor | 4 | 1 | 0x4 | 0x4 |
| FindNearestTile | 4 | 1 | 0x10 | 0x10 |
| Level_LoadPal | 4 | 1 | 0xa | 0xa |
| Mon_Display | 4 | 1 | 0x10 | 0x10 |
| NemBCT_Loop | 4 | 1 | 0x8 | 0x8 |
| OPL_MakeItem | 4 | 1 | 0xa | 0xa |
| OPL_Next | 4 | 1 | 0xe | 0xe |
| OPL_NoMove | 4 | 1 | 0x0 | 0x0 |
| PaletteCycle | 4 | 1 | 0x4 | 0x4 |
| PauseGame | 6 | 1 | 0x0 | 0x0 |
| SBall_ChkDel | 4 | 1 | 0xc | 0xc |
| SS_BGLoad | 6 | 1 | 0x5c | 0x5c |
| SS_ShowLayout | 6 | 1 | 0xc6 | 0xc6 |
| SignpostArtLoad | 4 | 1 | 0x4 | 0x4 |
| Size_of_SegaPCM | 4 | 1 | 0x0 | 0x0 |
| SonicSS_MakeGhostSolid | 6 | 1 | 0x12 | 0x12 |
| Sync4 | 4 | 1 | 0x0 | 0x0 |
| SynchroAnimate | 6 | 1 | 0x0 | 0x0 |
