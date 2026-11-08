# Bugs conhecidos do original (lista de verificação)

Gerado por `cargo xtask bugs-conhecidos` a partir dos `.asm` de texto do disassembly fixado. Só arquivo, linha e rótulo global mais próximo acima. Nada de código nem de comentário.

Cada item é uma linha de condição que cita `FixBugs`. O núcleo reproduz o comportamento SEM a correção (a ROM de referência). Cada item precisa de pelo menos um teste cujo nome contenha o id com sublinhado (por exemplo `BK_0001`). A coluna etapa é uma sugestão pela pasta.

315 itens.

| id | arquivo | linha | rótulo | etapa | estado |
| --- | --- | --- | --- | --- | --- |
| BK-0001 | _Variables.asm | 425 | v_timingvariables_end | 9 a 12 | pendente |
| BK-0002 | _inc/DynamicLevelEvents.asm | 85 | DLE_GHZ1 | 9 a 12 | pendente |
| BK-0003 | _inc/DynamicLevelEvents.asm | 238 | DLE_LZ3 | 9 a 12 | pendente |
| BK-0004 | _inc/LZWaterFeatures.asm | 302 | DynWater_LZ3_Routine4 | 9 a 12 | pendente |
| BK-0005 | _inc/LZWaterFeatures.asm | 363 | LZWindTunnels | 9 a 12 | pendente |
| BK-0006 | _inc/LZWaterFeatures.asm | 389 | LZWindTunnels | 9 a 12 | pendente |
| BK-0007 | _inc/LZWaterFeatures.asm | 411 | LZWindTunnels | 9 a 12 | pendente |
| BK-0008 | _inc/LZWaterFeatures.asm | 419 | LZWindTunnels | 9 a 12 | pendente |
| BK-0009 | _inc/LevelLayoutLoad.asm | 68 | LevelLayoutLoad | 9 a 12 | pendente |
| BK-0010 | _inc/LevelSizeLoad & BgScrollSpeed.asm | 13 | LevelSizeLoad | 9 a 12 | pendente |
| BK-0011 | _inc/LevelSizeLoad & BgScrollSpeed.asm | 89 | LevSz_StartLoc | 9 a 12 | pendente |
| BK-0012 | _inc/LevelSizeLoad & BgScrollSpeed.asm | 287 | BgScrollSpeed | 9 a 12 | pendente |
| BK-0013 | _inc/MoveSonicInDemo.asm | 69 | MDemo_On | 9 a 12 | pendente |
| BK-0014 | _inc/ObjPosLoad.asm | 35 | OPL_Main | 9 a 12 | pendente |
| BK-0015 | _inc/ObjPosLoad.asm | 48 | OPL_Main | 9 a 12 | pendente |
| BK-0016 | _inc/ObjPosLoad.asm | 203 | OPL_MovedRight | 9 a 12 | pendente |
| BK-0017 | _inc/ObjPosLoad.asm | 260 | OPL_SpawnObj | 9 a 12 | pendente |
| BK-0018 | _inc/ObjPosLoad.asm | 287 | OPL_MakeItem | 9 a 12 | pendente |
| BK-0019 | _inc/Object Pointers.asm | 185 | NullObject | 9 a 12 | pendente |
| BK-0020 | _inc/PaletteCycle.asm | 7 | PaletteCycle | 9 a 12 | pendente |
| BK-0021 | _inc/Pattern Load Cues.asm | 232 | PLC_SYZ2 | 9 a 12 | pendente |
| BK-0022 | _inc/ScrollHoriz & ScrollVertical.asm | 39 | MoveScreenHoriz | 9 a 12 | pendente |
| BK-0023 | _inc/ScrollHoriz & ScrollVertical.asm | 47 | MoveScreenHoriz | 9 a 12 | pendente |
| BK-0024 | _inc/ScrollHoriz & ScrollVertical.asm | 84 | SH_MoveCameraLeft | 9 a 12 | pendente |
| BK-0025 | _incObj/01 Sonic.asm | 193 | Sonic_Display | 9 a 11 | pendente |
| BK-0026 | _incObj/01 Sonic.asm | 261 | Sonic_Water | 9 a 11 | pendente |
| BK-0027 | _incObj/01 Sonic.asm | 288 | Sonic_Water | 9 a 11 | pendente |
| BK-0028 | _incObj/01 Sonic.asm | 541 | Sonic_WallSpeedAdjust | 9 a 11 | pendente |
| BK-0029 | _incObj/01 Sonic.asm | 580 | Sonic_WallSpeedAdjust | 9 a 11 | pendente |
| BK-0030 | _incObj/01 Sonic.asm | 607 | Sonic_WallSpeedAdjust | 9 a 11 | pendente |
| BK-0031 | _incObj/01 Sonic.asm | 673 | Sonic_MoveLeft | 9 a 11 | pendente |
| BK-0032 | _incObj/01 Sonic.asm | 737 | Sonic_MoveRight | 9 a 11 | pendente |
| BK-0033 | _incObj/01 Sonic.asm | 830 | Sonic_AngledRollSpeed | 9 a 11 | pendente |
| BK-0034 | _incObj/01 Sonic.asm | 848 | Sonic_AngledRollSpeed | 9 a 11 | pendente |
| BK-0035 | _incObj/01 Sonic.asm | 1085 | Sonic_LevelBound | 9 a 11 | pendente |
| BK-0036 | _incObj/01 Sonic.asm | 1103 | Sonic_LevelBound | 9 a 11 | pendente |
| BK-0037 | _incObj/01 Sonic.asm | 1132 | Sonic_LevelBound | 9 a 11 | pendente |
| BK-0038 | _incObj/01 Sonic.asm | 1181 | Sonic_ChkRoll | 9 a 11 | pendente |
| BK-0039 | _incObj/01 Sonic.asm | 1241 | Sonic_Jump | 9 a 11 | pendente |
| BK-0040 | _incObj/01 Sonic.asm | 1292 | Sonic_JumpHeight | 9 a 11 | pendente |
| BK-0041 | _incObj/01 Sonic.asm | 1306 | Sonic_JumpHeight | 9 a 11 | pendente |
| BK-0042 | _incObj/01 Sonic.asm | 1533 | Sonic_FloorDown | 9 a 11 | pendente |
| BK-0043 | _incObj/01 Sonic.asm | 1564 | Sonic_FloorDown | 9 a 11 | pendente |
| BK-0044 | _incObj/01 Sonic.asm | 1657 | Sonic_FloorLeft | 9 a 11 | pendente |
| BK-0045 | _incObj/01 Sonic.asm | 1778 | Sonic_FloorRight | 9 a 11 | pendente |
| BK-0046 | _incObj/01 Sonic.asm | 1825 | Sonic_ResetOnFloor | 9 a 11 | pendente |
| BK-0047 | _incObj/01 Sonic.asm | 1835 | Sonic_ResetOnFloor | 9 a 11 | pendente |
| BK-0048 | _incObj/01 Sonic.asm | 1855 | Sonic_Hurt | 9 a 11 | pendente |
| BK-0049 | _incObj/01 Sonic.asm | 1876 | Sonic_Hurt | 9 a 11 | pendente |
| BK-0050 | _incObj/01 Sonic.asm | 1894 | Sonic_HurtStop | 9 a 11 | pendente |
| BK-0051 | _incObj/01 Sonic.asm | 1907 | Sonic_HurtStop | 9 a 11 | pendente |
| BK-0052 | _incObj/01 Sonic.asm | 1940 | Sonic_Death | 9 a 11 | pendente |
| BK-0053 | _incObj/01 Sonic.asm | 1969 | Sonic_HandleDeath | 9 a 11 | pendente |
| BK-0054 | _incObj/01 Sonic.asm | 1997 | Sonic_HandleDeath | 9 a 11 | pendente |
| BK-0055 | _incObj/01 Sonic.asm | 2157 | Sonic_Animate | 9 a 11 | pendente |
| BK-0056 | _incObj/01 Sonic.asm | 2234 | Sonic_Animate | 9 a 11 | pendente |
| BK-0057 | _incObj/09 Sonic in Special Stage.asm | 63 | SonicSS_Control | 9 a 11 | pendente |
| BK-0058 | _incObj/09 Sonic in Special Stage.asm | 847 | SonicSS_ChkUP | 9 a 11 | pendente |
| BK-0059 | _incObj/09 Sonic in Special Stage.asm | 882 | SonicSS_ChkDOWN | 9 a 11 | pendente |
| BK-0060 | _incObj/0A LZ Drowning Countdown.asm | 44 | Drown_Main | 9 a 11 | pendente |
| BK-0061 | _incObj/0A LZ Drowning Countdown.asm | 76 | Drown_ChkWater | 9 a 11 | pendente |
| BK-0062 | _incObj/0A LZ Drowning Countdown.asm | 270 | Drown_Countdown | 9 a 11 | pendente |
| BK-0063 | _incObj/0A LZ Drowning Countdown.asm | 376 | Drown_Countdown | 9 a 11 | pendente |
| BK-0064 | _incObj/0D Signpost.asm | 13 | Signpost | 9 a 11 | pendente |
| BK-0065 | _incObj/0D Signpost.asm | 121 | Sign_SonicRun | 9 a 11 | pendente |
| BK-0066 | _incObj/0D Signpost.asm | 172 | GotThroughAct | 9 a 11 | pendente |
| BK-0067 | _incObj/0E, 0F Title Screen - Sonic, Press Start, TM.asm | 20 | TSon_Main | 9 a 11 | pendente |
| BK-0068 | _incObj/0E, 0F Title Screen - Sonic, Press Start, TM.asm | 85 | PSB_Main | 9 a 11 | pendente |
| BK-0069 | _incObj/0E, 0F Title Screen - Sonic, Press Start, TM.asm | 104 | PSB_Main | 9 a 11 | pendente |
| BK-0070 | _incObj/11 GHZ Bridge.asm | 32 | Bri_Main | 9 a 11 | pendente |
| BK-0071 | _incObj/11 GHZ Bridge.asm | 55 | Bri_Main | 9 a 11 | pendente |
| BK-0072 | _incObj/11 GHZ Bridge.asm | 109 | Bri_Action | 9 a 11 | pendente |
| BK-0073 | _incObj/11 GHZ Bridge.asm | 154 | Bri_StoodOn | 9 a 11 | pendente |
| BK-0074 | _incObj/11 GHZ Bridge.asm | 351 | Bri_ChkDel | 9 a 11 | pendente |
| BK-0075 | _incObj/13, 14 MZ, SLZ Fire Balls and Maker.asm | 12 | LavaMaker | 9 a 11 | pendente |
| BK-0076 | _incObj/13, 14 MZ, SLZ Fire Balls and Maker.asm | 68 | LavaBall | 9 a 11 | pendente |
| BK-0077 | _incObj/13, 14 MZ, SLZ Fire Balls and Maker.asm | 147 | LBall_ChkDel | 9 a 11 | pendente |
| BK-0078 | _incObj/15 Swinging Platforms.asm | 76 | Swing_CreateLinks | 9 a 11 | pendente |
| BK-0079 | _incObj/15 Swinging Platforms.asm | 149 | Swing_Swinging | 9 a 11 | pendente |
| BK-0080 | _incObj/15 Swinging Platforms.asm | 171 | Swing_StoodOn | 9 a 11 | pendente |
| BK-0081 | _incObj/15 Swinging Platforms.asm | 277 | Swing_ChkDel | 9 a 11 | pendente |
| BK-0082 | _incObj/17 GHZ Spiked Pole Helix.asm | 26 | Hel_Main | 9 a 11 | pendente |
| BK-0083 | _incObj/17 GHZ Spiked Pole Helix.asm | 55 | Hel_Main | 9 a 11 | pendente |
| BK-0084 | _incObj/17 GHZ Spiked Pole Helix.asm | 101 | Hel_ParentSpike | 9 a 11 | pendente |
| BK-0085 | _incObj/17 GHZ Spiked Pole Helix.asm | 129 | Hel_ChkDel | 9 a 11 | pendente |
| BK-0086 | _incObj/18 Platforms.asm | 78 | Plat_Action | 9 a 11 | pendente |
| BK-0087 | _incObj/18 Platforms.asm | 102 | Plat_StoodOn | 9 a 11 | pendente |
| BK-0088 | _incObj/18 Platforms.asm | 337 | Plat_ChkDel | 9 a 11 | pendente |
| BK-0089 | _incObj/1A, 53 Collapsing Ledges and Floors.asm | 37 | Ledge_Main | 9 a 11 | pendente |
| BK-0090 | _incObj/1A, 53 Collapsing Ledges and Floors.asm | 118 | Ledge_FragmentPiece | 9 a 11 | pendente |
| BK-0091 | _incObj/1A, 53 Collapsing Ledges and Floors.asm | 252 | CFlo_FragmentPiece | 9 a 11 | pendente |
| BK-0092 | _incObj/1A, 53 Collapsing Ledges and Floors.asm | 321 | FragmentatePlatform | 9 a 11 | pendente |
| BK-0093 | _incObj/1A, 53 Collapsing Ledges and Floors.asm | 341 | FragmentatePlatform | 9 a 11 | pendente |
| BK-0094 | _incObj/1A, 53 Collapsing Ledges and Floors.asm | 349 | FragmentatePlatform | 9 a 11 | pendente |
| BK-0095 | _incObj/1D Unused - Switch.asm | 41 | Swi_Action | 9 a 11 | pendente |
| BK-0096 | _incObj/1E, 20 Badnik - Ball Hog and Cannonball.asm | 38 | Hog_Main | 9 a 11 | pendente |
| BK-0097 | _incObj/1E, 20 Badnik - Ball Hog and Cannonball.asm | 176 | CBal_Animate | 9 a 11 | pendente |
| BK-0098 | _incObj/1F Badnik - Crabmeat.asm | 43 | Crab_Main | 9 a 11 | pendente |
| BK-0099 | _incObj/1F Badnik - Crabmeat.asm | 226 | Crab_BallMove | 9 a 11 | pendente |
| BK-0100 | _incObj/21 HUD.asm | 27 | HUD_Flash | 9 a 11 | pendente |
| BK-0101 | _incObj/22, 23 Badnik - Buzz Bomber and Missile.asm | 73 | Buzz_Action_Fire | 9 a 11 | pendente |
| BK-0102 | _incObj/22, 23 Badnik - Buzz Bomber and Missile.asm | 184 | Msl_Animate | 9 a 11 | pendente |
| BK-0103 | _incObj/22, 23 Badnik - Buzz Bomber and Missile.asm | 205 | Msl_ChkCancel | 9 a 11 | pendente |
| BK-0104 | _incObj/22, 23 Badnik - Buzz Bomber and Missile.asm | 234 | Msl_FromBuzz | 9 a 11 | pendente |
| BK-0105 | _incObj/22, 23 Badnik - Buzz Bomber and Missile.asm | 245 | Msl_FromBuzz | 9 a 11 | pendente |
| BK-0106 | _incObj/25, 37 Rings.asm | 136 | Ring_Animate | 9 a 11 | pendente |
| BK-0107 | _incObj/25, 37 Rings.asm | 177 | CollectRing | 9 a 11 | pendente |
| BK-0108 | _incObj/25, 37 Rings.asm | 266 | RLoss_Count | 9 a 11 | pendente |
| BK-0109 | _incObj/25, 37 Rings.asm | 302 | RLoss_Count | 9 a 11 | pendente |
| BK-0110 | _incObj/25, 37 Rings.asm | 336 | RLoss_Bounce | 9 a 11 | pendente |
| BK-0111 | _incObj/25, 37 Rings.asm | 349 | RLoss_Bounce | 9 a 11 | pendente |
| BK-0112 | _incObj/26, 2E Monitors and Power-Ups.asm | 20 | Mon_Main | 9 a 11 | pendente |
| BK-0113 | _incObj/26, 2E Monitors and Power-Ups.asm | 48 | Mon_Main | 9 a 11 | pendente |
| BK-0114 | _incObj/26, 2E Monitors and Power-Ups.asm | 156 | Mon_Solid | 9 a 11 | pendente |
| BK-0115 | _incObj/26, 2E Monitors and Power-Ups.asm | 171 | Mon_Display | 9 a 11 | pendente |
| BK-0116 | _incObj/26, 2E Monitors and Power-Ups.asm | 264 | Pow_ChkEggman | 9 a 11 | pendente |
| BK-0117 | _incObj/26, 2E Monitors and Power-Ups.asm | 301 | Pow_ChkShoes | 9 a 11 | pendente |
| BK-0118 | _incObj/26, 2E Monitors and Power-Ups.asm | 359 | Pow_ChkRings | 9 a 11 | pendente |
| BK-0119 | _incObj/26, 2E Monitors and Power-Ups.asm | 404 | Pow_Delete | 9 a 11 | pendente |
| BK-0120 | _incObj/28, 29 Animals and Points.asm | 545 | Points | 9 a 11 | pendente |
| BK-0121 | _incObj/28, 29 Animals and Points.asm | 573 | Poi_Slower | 9 a 11 | pendente |
| BK-0122 | _incObj/2C Badnik - Jaws.asm | 26 | Jaws_Main | 9 a 11 | pendente |
| BK-0123 | _incObj/2F, 35 MZ Large Grassy Platforms and Burning Grass.asm | 103 | LGrass_Display | 9 a 11 | pendente |
| BK-0124 | _incObj/2F, 35 MZ Large Grassy Platforms and Burning Grass.asm | 277 | LGrass_ChkGone | 9 a 11 | pendente |
| BK-0125 | _incObj/2F, 35 MZ Large Grassy Platforms and Burning Grass.asm | 307 | LGrass_DelFlames | 9 a 11 | pendente |
| BK-0126 | _incObj/31 MZ Chained Stompers.asm | 153 | CStom_MainBlock | 9 a 11 | pendente |
| BK-0127 | _incObj/31 MZ Chained Stompers.asm | 179 | CStom_Ceiling | 9 a 11 | pendente |
| BK-0128 | _incObj/31 MZ Chained Stompers.asm | 185 | CStom_ChkDel | 9 a 11 | pendente |
| BK-0129 | _incObj/32 Button.asm | 35 | But_Pressed | 9 a 11 | pendente |
| BK-0130 | _incObj/32 Button.asm | 69 | But_Pressed | 9 a 11 | pendente |
| BK-0131 | _incObj/32 Button.asm | 111 | But_Pressed | 9 a 11 | pendente |
| BK-0132 | _incObj/34 Title Cards.asm | 97 | Card_MoveIn | 9 a 11 | pendente |
| BK-0133 | _incObj/34 Title Cards.asm | 140 | Card_MoveOut | 9 a 11 | pendente |
| BK-0134 | _incObj/36 Spikes.asm | 69 | Spikes_SideWays | 9 a 11 | pendente |
| BK-0135 | _incObj/36 Spikes.asm | 99 | Spikes_Upright | 9 a 11 | pendente |
| BK-0136 | _incObj/36 Spikes.asm | 123 | Spikes_Hurt | 9 a 11 | pendente |
| BK-0137 | _incObj/36 Spikes.asm | 135 | Spikes_Hurt | 9 a 11 | pendente |
| BK-0138 | _incObj/36 Spikes.asm | 161 | Spikes_Display | 9 a 11 | pendente |
| BK-0139 | _incObj/39 Game Over.asm | 51 | Over_MoveIn | 9 a 11 | pendente |
| BK-0140 | _incObj/3A Got Through Card.asm | 81 | Got_MoveIn | 9 a 11 | pendente |
| BK-0141 | _incObj/3A Got Through Card.asm | 242 | Got_SBZ2_MoveOut | 9 a 11 | pendente |
| BK-0142 | _incObj/3B GHZ Purple Rock.asm | 21 | Rock_Main | 9 a 11 | pendente |
| BK-0143 | _incObj/3B GHZ Purple Rock.asm | 45 | Rock_Solid | 9 a 11 | pendente |
| BK-0144 | _incObj/3C GHZ, SLZ Smashable Wall.asm | 81 | Smash_Fragment | 9 a 11 | pendente |
| BK-0145 | _incObj/3D, 48 Boss - GHZ Main and Wrecking Ball.asm | 312 | BGHZ_ShipDel | 9 a 11 | pendente |
| BK-0146 | _incObj/3E Prison Capsule.asm | 192 | Pri_Animals | 9 a 11 | pendente |
| BK-0147 | _incObj/3E Prison Capsule.asm | 204 | Pri_EndAct | 9 a 11 | pendente |
| BK-0148 | _incObj/3E Prison Capsule.asm | 225 | Pri_EndAct | 9 a 11 | pendente |
| BK-0149 | _incObj/40 Badnik - Moto Bug.asm | 45 | Moto_Main | 9 a 11 | pendente |
| BK-0150 | _incObj/41 Springs.asm | 11 | Springs | 9 a 11 | pendente |
| BK-0151 | _incObj/43 Badnik - Roller.asm | 38 | Roll_Main | 9 a 11 | pendente |
| BK-0152 | _incObj/43 Badnik - Roller.asm | 55 | Roll_Action | 9 a 11 | pendente |
| BK-0153 | _incObj/44 GHZ Edge Walls.asm | 39 | Edge_Display | 9 a 11 | pendente |
| BK-0154 | _incObj/45 Unused - MZ Sideways Stomper.asm | 100 | SStom_MainBlock | 9 a 11 | pendente |
| BK-0155 | _incObj/45 Unused - MZ Sideways Stomper.asm | 128 | SStom_WallBracket | 9 a 11 | pendente |
| BK-0156 | _incObj/45 Unused - MZ Sideways Stomper.asm | 135 | SStom_ChkDel | 9 a 11 | pendente |
| BK-0157 | _incObj/45 Unused - MZ Sideways Stomper.asm | 162 | SStom_Move_Index | 9 a 11 | pendente |
| BK-0158 | _incObj/4C, 4D MZ Lava Geyser and Maker.asm | 10 | GeyserMaker | 9 a 11 | pendente |
| BK-0159 | _incObj/4C, 4D MZ Lava Geyser and Maker.asm | 56 | GMake_Wait | 9 a 11 | pendente |
| BK-0160 | _incObj/4C, 4D MZ Lava Geyser and Maker.asm | 94 | GMake_ChkType | 9 a 11 | pendente |
| BK-0161 | _incObj/4C, 4D MZ Lava Geyser and Maker.asm | 103 | GMake_Display | 9 a 11 | pendente |
| BK-0162 | _incObj/4C, 4D MZ Lava Geyser and Maker.asm | 119 | GMake_Delete | 9 a 11 | pendente |
| BK-0163 | _incObj/4C, 4D MZ Lava Geyser and Maker.asm | 136 | LavaGeyser | 9 a 11 | pendente |
| BK-0164 | _incObj/4C, 4D MZ Lava Geyser and Maker.asm | 184 | Geyser_Main | 9 a 11 | pendente |
| BK-0165 | _incObj/4C, 4D MZ Lava Geyser and Maker.asm | 249 | Geyser_ChkDel | 9 a 11 | pendente |
| BK-0166 | _incObj/4E MZ Wall of Lava.asm | 106 | LWall_Solid | 9 a 11 | pendente |
| BK-0167 | _incObj/4E MZ Wall of Lava.asm | 126 | LWall_Solid | 9 a 11 | pendente |
| BK-0168 | _incObj/50 Badnik - Yadrin.asm | 79 | Yad_Main | 9 a 11 | pendente |
| BK-0169 | _incObj/51 MZ Smashable Green Block.asm | 109 | Smab_Fragment | 9 a 11 | pendente |
| BK-0170 | _incObj/52 Moving Blocks.asm | 21 | MBlock_Index | 9 a 11 | pendente |
| BK-0171 | _incObj/52 Moving Blocks.asm | 85 | MBlock_StandOn | 9 a 11 | pendente |
| BK-0172 | _incObj/55 Badnik - Basaran.asm | 104 | Bas_Action_DropDown_Delete | 9 a 11 | pendente |
| BK-0173 | _incObj/57 SYZ, LZ Spiked Ball and Chain.asm | 69 | SBall_Main | 9 a 11 | pendente |
| BK-0174 | _incObj/59 SLZ Elevators.asm | 116 | Elev_StoodOn | 9 a 11 | pendente |
| BK-0175 | _incObj/5C SLZ Foreground Pylon.asm | 21 | Pyl_Main | 9 a 11 | pendente |
| BK-0176 | _incObj/5D SLZ Fan.asm | 41 | Fan_Action | 9 a 11 | pendente |
| BK-0177 | _incObj/5D SLZ Fan.asm | 104 | Fan_Action | 9 a 11 | pendente |
| BK-0178 | _incObj/5F Badnik - Walking Bomb.asm | 172 | Bom_BurnFuseAndExplode | 9 a 11 | pendente |
| BK-0179 | _incObj/62 LZ Gargoyle.asm | 95 | Gar_AniFire | 9 a 11 | pendente |
| BK-0180 | _incObj/62 LZ Gargoyle.asm | 107 | Gar_AniFire | 9 a 11 | pendente |
| BK-0181 | _incObj/62 LZ Gargoyle.asm | 115 | Gar_AniFire | 9 a 11 | pendente |
| BK-0182 | _incObj/63 LZ Conveyor.asm | 125 | LCon_Main_Spawner | 9 a 11 | pendente |
| BK-0183 | _incObj/63 LZ Conveyor.asm | 157 | LCon_Main_Spawner | 9 a 11 | pendente |
| BK-0184 | _incObj/64 LZ Air Bubbles.asm | 89 | Bub_ChkWater | 9 a 11 | pendente |
| BK-0185 | _incObj/69 SBZ Spinning Platforms and Trapdoors.asm | 27 | Spin_Main | 9 a 11 | pendente |
| BK-0186 | _incObj/6A SBZ Saws and Pizza Cutters.asm | 153 | Saw_Type3_SpeedingFromLeft | 9 a 11 | pendente |
| BK-0187 | _incObj/6A SBZ Saws and Pizza Cutters.asm | 201 | Saw_Type4_SpeedingFromRight | 9 a 11 | pendente |
| BK-0188 | _incObj/6F SBZ Spin Platform Conveyor.asm | 110 | SpinC_Main | 9 a 11 | pendente |
| BK-0189 | _incObj/6F SBZ Spin Platform Conveyor.asm | 137 | SpinC_Main_Spawner | 9 a 11 | pendente |
| BK-0190 | _incObj/6F SBZ Spin Platform Conveyor.asm | 158 | SpinC_Main_Spawner | 9 a 11 | pendente |
| BK-0191 | _incObj/71 Invisible Solid Barriers.asm | 35 | Invis_Solid | 9 a 11 | pendente |
| BK-0192 | _incObj/72 SBZ Teleporter.asm | 63 | Tele_Action | 9 a 11 | pendente |
| BK-0193 | _incObj/73, 74 Boss - MZ Main and Fire.asm | 392 | BossMarble_ShipDel | 9 a 11 | pendente |
| BK-0194 | _incObj/73, 74 Boss - MZ Main and Fire.asm | 529 | BossFire | 9 a 11 | pendente |
| BK-0195 | _incObj/73, 74 Boss - MZ Main and Fire.asm | 576 | BossFire_Action | 9 a 11 | pendente |
| BK-0196 | _incObj/73, 74 Boss - MZ Main and Fire.asm | 724 | BossFire_Delete2 | 9 a 11 | pendente |
| BK-0197 | _incObj/73, 74 Boss - MZ Main and Fire.asm | 745 | BossFire_Animate | 9 a 11 | pendente |
| BK-0198 | _incObj/75, 76 Boss - SYZ Main and Blocks.asm | 404 | BossSpringYard_FindBlocks | 9 a 11 | pendente |
| BK-0199 | _incObj/75, 76 Boss - SYZ Main and Blocks.asm | 528 | BossSpringYard_ShipDelete | 9 a 11 | pendente |
| BK-0200 | _incObj/77 Boss - LZ Main.asm | 367 | BossLabyrinth_ShipDel | 9 a 11 | pendente |
| BK-0201 | _incObj/77 Boss - LZ Main.asm | 386 | BossLabyrinth_FaceMain | 9 a 11 | pendente |
| BK-0202 | _incObj/77 Boss - LZ Main.asm | 437 | BossLabyrinth_FlameMain | 9 a 11 | pendente |
| BK-0203 | _incObj/78 Badnik - Caterkiller.asm | 33 | Cat_HideForInit | 9 a 11 | pendente |
| BK-0204 | _incObj/78 Badnik - Caterkiller.asm | 382 | Cat_BodySeg1 | 9 a 11 | pendente |
| BK-0205 | _incObj/78 Badnik - Caterkiller.asm | 391 | Cat_BodySeg1 | 9 a 11 | pendente |
| BK-0206 | _incObj/7A, 7B Boss - SLZ Main and Spike Balls.asm | 78 | BossStarLight_Done | 9 a 11 | pendente |
| BK-0207 | _incObj/7A, 7B Boss - SLZ Main and Spike Balls.asm | 85 | BossStarLight_Done | 9 a 11 | pendente |
| BK-0208 | _incObj/7A, 7B Boss - SLZ Main and Spike Balls.asm | 276 | BSLZ_MakeBall | 9 a 11 | pendente |
| BK-0209 | _incObj/7A, 7B Boss - SLZ Main and Spike Balls.asm | 396 | BSLZ_Escape | 9 a 11 | pendente |
| BK-0210 | _incObj/7A, 7B Boss - SLZ Main and Spike Balls.asm | 407 | BSLZ_Escape | 9 a 11 | pendente |
| BK-0211 | _incObj/7A, 7B Boss - SLZ Main and Spike Balls.asm | 687 | BossSpikeball_HitBoss | 9 a 11 | pendente |
| BK-0212 | _incObj/7A, 7B Boss - SLZ Main and Spike Balls.asm | 694 | BossSpikeball_HitBoss | 9 a 11 | pendente |
| BK-0213 | _incObj/7A, 7B Boss - SLZ Main and Spike Balls.asm | 934 | BossSpikeball_MoveFrag | 9 a 11 | pendente |
| BK-0214 | _incObj/7D Hidden Bonuses.asm | 74 | Bonus_Points | 9 a 11 | pendente |
| BK-0215 | _incObj/7E, 7F Special Stage Results and Chaos Emeralds.asm | 87 | SSR_Move | 9 a 11 | pendente |
| BK-0216 | _incObj/82, 83 SBZ Eggman Cutscene and Crumbling Floor.asm | 145 | SEgg_Leap | 9 a 11 | pendente |
| BK-0217 | _incObj/85,84,86 Boss - FZ Main, Cylinders, and Plasma Balls.asm | 62 | BossFinal_ObjData2 | 9 a 11 | pendente |
| BK-0218 | _incObj/85,84,86 Boss - FZ Main, Cylinders, and Plasma Balls.asm | 250 | BossFinal_Eggman_Crush | 9 a 11 | pendente |
| BK-0219 | _incObj/85,84,86 Boss - FZ Main, Cylinders, and Plasma Balls.asm | 269 | BossFinal_Eggman_Crush | 9 a 11 | pendente |
| BK-0220 | _incObj/85,84,86 Boss - FZ Main, Cylinders, and Plasma Balls.asm | 526 | BossFinal_Eggman_Escape | 9 a 11 | pendente |
| BK-0221 | _incObj/85,84,86 Boss - FZ Main, Cylinders, and Plasma Balls.asm | 565 | BossFinal_Eggman_Escape | 9 a 11 | pendente |
| BK-0222 | _incObj/85,84,86 Boss - FZ Main, Cylinders, and Plasma Balls.asm | 644 | BossFinal_Cockpit | 9 a 11 | pendente |
| BK-0223 | _incObj/85,84,86 Boss - FZ Main, Cylinders, and Plasma Balls.asm | 775 | EggmanCylinder_Main | 9 a 11 | pendente |
| BK-0224 | _incObj/85,84,86 Boss - FZ Main, Cylinders, and Plasma Balls.asm | 1079 | BossPlasma_MakeBalls | 9 a 11 | pendente |
| BK-0225 | _incObj/85,84,86 Boss - FZ Main, Cylinders, and Plasma Balls.asm | 1155 | BossPlasma_Drop | 9 a 11 | pendente |
| BK-0226 | _incObj/85,84,86 Boss - FZ Main, Cylinders, and Plasma Balls.asm | 1199 | BossPlasma_Move | 9 a 11 | pendente |
| BK-0227 | _incObj/DebugMode.asm | 25 | Debug_Init | 9 a 11 | pendente |
| BK-0228 | _incObj/DebugMode.asm | 42 | Debug_Init | 9 a 11 | pendente |
| BK-0229 | _incObj/DebugMode.asm | 107 | Debug_Init | 9 a 11 | pendente |
| BK-0230 | _incObj/DebugMode.asm | 187 | Debug_Move | 9 a 11 | pendente |
| BK-0231 | _incObj/DebugMode.asm | 206 | Debug_Move | 9 a 11 | pendente |
| BK-0232 | _incObj/DebugMode.asm | 225 | Debug_Move | 9 a 11 | pendente |
| BK-0233 | _incObj/DebugMode.asm | 242 | Debug_Move | 9 a 11 | pendente |
| BK-0234 | _incObj/DebugMode.asm | 294 | Debug_ChgItem | 9 a 11 | pendente |
| BK-0235 | _incObj/DebugMode.asm | 434 | __LABEL__ | 9 a 11 | pendente |
| BK-0236 | _incObj/DebugMode.asm | 468 | __LABEL__ | 9 a 11 | pendente |
| BK-0237 | _incObj/DebugMode.asm | 566 | __LABEL__ | 9 a 11 | pendente |
| BK-0238 | _incObj/Sonic Collision.asm | 60 | Sonic_CalcRoomAhead | 9 a 11 | pendente |
| BK-0239 | _incObj/Sonic Collision.asm | 128 | Sonic_FindFloor | 9 a 11 | pendente |
| BK-0240 | _incObj/Sonic Collision.asm | 148 | Sonic_FindFloor | 9 a 11 | pendente |
| BK-0241 | _incObj/Sonic Collision.asm | 160 | Sonic_FindSmaller | 9 a 11 | pendente |
| BK-0242 | _incObj/Sonic Collision.asm | 248 | Sonic_FindWallRight | 9 a 11 | pendente |
| BK-0243 | _incObj/Sonic Collision.asm | 268 | Sonic_FindWallRight | 9 a 11 | pendente |
| BK-0244 | _incObj/Sonic Collision.asm | 375 | Sonic_FindCeiling | 9 a 11 | pendente |
| BK-0245 | _incObj/Sonic Collision.asm | 396 | Sonic_FindCeiling | 9 a 11 | pendente |
| BK-0246 | _incObj/Sonic Collision.asm | 502 | Sonic_FindWallLeft | 9 a 11 | pendente |
| BK-0247 | _incObj/Sonic Collision.asm | 523 | Sonic_FindWallLeft | 9 a 11 | pendente |
| BK-0248 | _incObj/Sonic Collision.asm | 585 | ObjHitWallLeft | 9 a 11 | pendente |
| BK-0249 | _incObj/Sonic ReactToItem.asm | 24 | ReactToItem | 9 a 11 | pendente |
| BK-0250 | _incObj/Sonic ReactToItem.asm | 214 | React_Monitor | 9 a 11 | pendente |
| BK-0251 | _incObj/Sonic ReactToItem.asm | 341 | React_ChkHurt | 9 a 11 | pendente |
| BK-0252 | _incObj/Sonic ReactToItem.asm | 412 | HurtSonic | 9 a 11 | pendente |
| BK-0253 | _incObj/Sonic ReactToItem.asm | 467 | KillSonic | 9 a 11 | pendente |
| BK-0254 | _incObj/Sonic ReactToItem.asm | 477 | KillSonic | 9 a 11 | pendente |
| BK-0255 | _incObj/sub ChkObjectVisible.asm | 53 | ChkPartiallyVisible | 9 a 11 | pendente |
| BK-0256 | _incObj/sub ExitPlatform.asm | 16 | ExitPlatform2 | 9 a 11 | pendente |
| BK-0257 | _incObj/sub FindNearestTile & FindFloor & FindWall.asm | 40 | FindNearestTile | 9 a 11 | pendente |
| BK-0258 | _incObj/sub FindNearestTile & FindFloor & FindWall.asm | 194 | FindFloor2 | 9 a 11 | pendente |
| BK-0259 | _incObj/sub FindNearestTile & FindFloor & FindWall.asm | 210 | FindFloor2 | 9 a 11 | pendente |
| BK-0260 | _incObj/sub FindNearestTile & FindFloor & FindWall.asm | 257 | FindFloor2 | 9 a 11 | pendente |
| BK-0261 | _incObj/sub FindNearestTile & FindFloor & FindWall.asm | 269 | FindFloor2 | 9 a 11 | pendente |
| BK-0262 | _incObj/sub FindNearestTile & FindFloor & FindWall.asm | 378 | FindWall2 | 9 a 11 | pendente |
| BK-0263 | _incObj/sub FindNearestTile & FindFloor & FindWall.asm | 394 | FindWall2 | 9 a 11 | pendente |
| BK-0264 | _incObj/sub FindNearestTile & FindFloor & FindWall.asm | 441 | FindWall2 | 9 a 11 | pendente |
| BK-0265 | _incObj/sub FindNearestTile & FindFloor & FindWall.asm | 453 | FindWall2 | 9 a 11 | pendente |
| BK-0266 | _incObj/sub PlatformObject & SlopeObject.asm | 53 | PlatformObject | 9 a 11 | pendente |
| BK-0267 | _incObj/sub SmashObject.asm | 26 | SmashObject | 9 a 11 | pendente |
| BK-0268 | _incObj/sub SmashObject.asm | 51 | SmashObject | 9 a 11 | pendente |
| BK-0269 | _incObj/sub SolidObject.asm | 31 | SolidObject | 9 a 11 | pendente |
| BK-0270 | _incObj/sub SolidObject.asm | 48 | SolidObject | 9 a 11 | pendente |
| BK-0271 | _incObj/sub SolidObject.asm | 86 | SolidObject_NoRenderChk | 9 a 11 | pendente |
| BK-0272 | _incObj/sub SolidObject.asm | 256 | Solid_NoCollision | 9 a 11 | pendente |
| BK-0273 | _incObj/sub SolidObject.asm | 284 | Solid_Below | 9 a 11 | pendente |
| BK-0274 | _incObj/sub SolidObject.asm | 297 | Solid_TopBtmAir | 9 a 11 | pendente |
| BK-0275 | _incObj/sub SolidWall.asm | 46 | EdgeWall_SolidWall | 9 a 11 | pendente |
| BK-0276 | s1.sounddriver.asm | 189 | UpdateMusic | 6 | pendente |
| BK-0277 | s1.sounddriver.asm | 682 | PlaySoundID | 6 | pendente |
| BK-0278 | s1.sounddriver.asm | 697 | PlaySoundID | 6 | pendente |
| BK-0279 | s1.sounddriver.asm | 817 | Sound_PlayBGM | 6 | pendente |
| BK-0280 | s1.sounddriver.asm | 1010 | Sound_PlaySFX | 6 | pendente |
| BK-0281 | s1.sounddriver.asm | 1134 | Sound_PlaySpecial | 6 | pendente |
| BK-0282 | s1.sounddriver.asm | 1243 | StopSFX | 6 | pendente |
| BK-0283 | s1.sounddriver.asm | 1265 | StopSFX | 6 | pendente |
| BK-0284 | s1.sounddriver.asm | 1280 | StopSFX | 6 | pendente |
| BK-0285 | s1.sounddriver.asm | 1325 | StopSpecialSFX | 6 | pendente |
| BK-0286 | s1.sounddriver.asm | 1469 | StopAllSound | 6 | pendente |
| BK-0287 | s1.sounddriver.asm | 1494 | InitMusicPlayback | 6 | pendente |
| BK-0288 | s1.sounddriver.asm | 1510 | InitMusicPlayback | 6 | pendente |
| BK-0289 | s1.sounddriver.asm | 1516 | InitMusicPlayback | 6 | pendente |
| BK-0290 | s1.sounddriver.asm | 1652 | DoFadeIn | 6 | pendente |
| BK-0291 | s1.sounddriver.asm | 1938 | PSGDoVolFX | 6 | pendente |
| BK-0292 | s1.sounddriver.asm | 2005 | SendPSGNoteOff | 6 | pendente |
| BK-0293 | s1.sounddriver.asm | 2175 | cfFadeInToPrevious | 6 | pendente |
| BK-0294 | s1.sounddriver.asm | 2298 | cfStopSpecialFM4 | 6 | pendente |
| BK-0295 | s1.sounddriver.asm | 2391 | SendVoiceTL | 6 | pendente |
| BK-0296 | sonic.asm | 139 | Vectors | 7 a 12 | pendente |
| BK-0297 | sonic.asm | 140 | Vectors | 7 a 12 | pendente |
| BK-0298 | sonic.asm | 1211 | ClearScreen | 7 a 12 | pendente |
| BK-0299 | sonic.asm | 1400 | RunPLC | 7 a 12 | pendente |
| BK-0300 | sonic.asm | 1417 | RunPLC | 7 a 12 | pendente |
| BK-0301 | sonic.asm | 1508 | ProcessPLC_ShiftCue | 7 a 12 | pendente |
| BK-0302 | sonic.asm | 1932 | GM_Title | 7 a 12 | pendente |
| BK-0303 | sonic.asm | 2013 | Tit_LoadText | 7 a 12 | pendente |
| BK-0304 | sonic.asm | 2032 | Tit_LoadText | 7 a 12 | pendente |
| BK-0305 | sonic.asm | 2115 | Tit_EnterCheat | 7 a 12 | pendente |
| BK-0306 | sonic.asm | 2148 | Tit_ResetCheat | 7 a 12 | pendente |
| BK-0307 | sonic.asm | 2181 | Tit_EnterLevelSelect | 7 a 12 | pendente |
| BK-0308 | sonic.asm | 2231 | LevSel_NoCheat | 7 a 12 | pendente |
| BK-0309 | sonic.asm | 2828 | Level_TtlCardLoop | 7 a 12 | pendente |
| BK-0310 | sonic.asm | 2855 | Level_CheckTtlCard | 7 a 12 | pendente |
| BK-0311 | sonic.asm | 3020 | Level_MainLoop | 7 a 12 | pendente |
| BK-0312 | sonic.asm | 3052 | Level_CheckRestart | 7 a 12 | pendente |
| BK-0313 | sonic.asm | 3269 | GM_Special | 7 a 12 | pendente |
| BK-0314 | sonic.asm | 3626 | End_LoadData | 7 a 12 | pendente |
| BK-0315 | sonic.asm | 3741 | End_SlowFade | 7 a 12 | pendente |
