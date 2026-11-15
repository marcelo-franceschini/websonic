//! Endereços da RAM e do hardware usados pela abertura (rótulos do disassembly).
//! Conferidos contra `refs/rotulos.txt` por `crates/teste-som/tests/abertura_layout.rs`.

pub const V_GAMEMODE: u32 = 0xFF_F600;
pub const V_VDP_BUFFER1: u32 = 0xFF_F60C;
pub const V_HBLANK_HREG: u32 = 0xFF_F624;
pub const V_SCRPOSY_VDP: u32 = 0xFF_F616;
pub const V_SCRPOSX_VDP: u32 = 0xFF_F61A;
pub const V_CROSSRESETRAM: u32 = 0xFF_FE00;
pub const V_MEGADRIVE: u32 = 0xFF_FFF8;
pub const V_INIT: u32 = 0xFF_FFFC;
pub const V_RAM_START: u32 = 0xFF_0000;

/// A pilha do sistema (`v_systemstack_end` a `v_systemstack`). O núcleo não guarda a pilha de
/// chamadas, então a comparação da RAM do 68000 a ignora (exclusão aprovada, D7.7).
pub const PILHA_INICIO: u32 = 0xFF_FCC0;
pub const PILHA_FIM: u32 = 0xFF_FE00;

// Hardware (mapa de memória do Mega Drive).
pub const VDP_DADOS: u32 = 0xC0_0000;
pub const VDP_CONTROLE: u32 = 0xC0_0004;
pub const PSG: u32 = 0xC0_0011;
pub const Z80_RAM: u32 = 0xA0_0000;
pub const Z80_BUSREQ: u32 = 0xA1_1100;
pub const Z80_RESET: u32 = 0xA1_1200;
pub const VERSAO: u32 = 0xA1_0001;
pub const PORTA1_CONTROLE: u32 = 0xA1_0009;
pub const PORTA2_CONTROLE: u32 = 0xA1_000B;
pub const PORTA3_CONTROLE: u32 = 0xA1_000D;
pub const PORTA1_CONTROLE_ALTO: u32 = 0xA1_0008;
pub const EXPANSAO_CONTROLE_ALTO: u32 = 0xA1_000C;

// Rótulos da ROM (tabelas lidas em tempo de execução).
pub const SETUP_VALUES: u32 = 0x294;
pub const SETUP_VALUES_VDP: u32 = 0x2AE;
pub const SETUP_VALUES_VDP_END: u32 = 0x2C6;
pub const SETUP_VALUES_Z80: u32 = 0x2CA;
pub const SETUP_VALUES_Z80_END: u32 = 0x2F0;
pub const SETUP_VALUES_PSG: u32 = 0x2FC;
pub const SETUP_VALUES_PSG_END: u32 = 0x300;
pub const VDP_SETUP_ARRAY: u32 = 0x129E;
pub const VDP_SETUP_ARRAY_END: u32 = 0x12C4;
pub const DAC_DRIVER: u32 = 0x7_2E7C;
pub const CHECKSUM: u32 = 0x18E;
pub const ROM_END_LOC: u32 = 0x1A4;
pub const END_OF_HEADER: u32 = 0x200;

// Variáveis e rótulos da tela da Sega e do tratador de VBlank.
pub const V_VBLANK_ROUTINE: u32 = 0xFF_F62A;
pub const V_VBLANK_COUNT: u32 = 0xFF_FE0C;
pub const V_VBLANK_BYTE: u32 = 0xFF_FE0F;
pub const V_VDP_BUFFER2: u32 = 0xFF_F640;
pub const F_HBLANK_PAL: u32 = 0xFF_F644;
pub const F_WTR_STATE: u32 = 0xFF_F64E;
pub const V_GENERICTIMER: u32 = 0xFF_F614;
pub const V_PCYC_NUM: u32 = 0xFF_F632;
pub const V_PCYC_TIME: u32 = 0xFF_F634;
pub const V_RANDOM: u32 = 0xFF_F636;
pub const V_PFADE_START: u32 = 0xFF_F626;
pub const V_JPADHOLD1: u32 = 0xFF_F604;
pub const V_PLC_BUFFER: u32 = 0xFF_F680;
pub const V_PLC_BUFFER_END: u32 = 0xFF_F700;
pub const V_PALETTE: u32 = 0xFF_FB00;
pub const V_PALETTE_LINE_1: u32 = 0xFF_FB00;
pub const V_PALETTE_LINE_2: u32 = 0xFF_FB20;
pub const V_PALETTE_WATER: u32 = 0xFF_FA80;
pub const V_PALETTE_FADING: u32 = 0xFF_FB80;
pub const V_SPRITETABLEBUFFER: u32 = 0xFF_F800;
pub const V_SPRITETABLEBUFFER_END: u32 = 0xFF_FA80;
pub const V_HSCROLLTABLEBUFFER: u32 = 0xFF_CC00;
pub const V_HSCROLLTABLEBUFFER_END_PADDED: u32 = 0xFF_D000;
pub const V_SOUNDQUEUE1: u32 = 0xFF_F00B;
pub const BGM_STOP: u8 = 0xE4;
pub const SFX_SEGA: u8 = 0xE1;
pub const NEM_SEGA_LOGO: u32 = 0x1_E700;
pub const ENI_SEGA_LOGO: u32 = 0x1_EB70;
pub const PAL_INDEX: u32 = 0x2160;
pub const PAL_SEGA1: u32 = 0x20B8;
pub const PAL_SEGA1_END: u32 = 0x20C4;
pub const PAL_SEGA2: u32 = 0x20C4;

// Objetos, sprites e a tela do título.
pub const V_OBJSPACE: u32 = 0xFF_D000;
pub const V_OBJSPACE_END: u32 = 0xFF_F000;
pub const OBJECT_SIZE: u32 = 0x40;
pub const V_PLAYER: u32 = 0xFF_D000;
pub const V_TITLESONIC: u32 = 0xFF_D040;
pub const V_SONICTEAM: u32 = 0xFF_D080;
pub const V_PRESSSTART: u32 = 0xFF_D080;
pub const V_TITLETM: u32 = 0xFF_D0C0;
pub const V_TTLSONICHIDE: u32 = 0xFF_D100;
pub const V_SPRITEQUEUE: u32 = 0xFF_AC00;
pub const V_SPRITECOUNT: u32 = 0xFF_F62C;
pub const V_CREDITSNUM: u32 = 0xFF_FFF4;
pub const V_SCREENPOSX: u32 = 0xFF_F700;
pub const V_BGSCREENPOSX: u32 = 0xFF_F708;
pub const V_BG3SCREENPOSX: u32 = 0xFF_F718;
pub const V_ZONE: u32 = 0xFF_FE10;
pub const V_PALETTE_FADING_END: u32 = 0xFF_FC00;
pub const V_PALETTE_FADING_LINE_3: u32 = 0xFF_FBC0;
pub const GM_TITLE_ID: u8 = 4;

// Campos de um objeto.
pub const OB_ID: u32 = 0x00;
pub const OB_RENDER: u32 = 0x01;
pub const OB_GFX: u32 = 0x02;
pub const OB_MAP: u32 = 0x04;
pub const OB_X: u32 = 0x08;
pub const OB_SCREENY: u32 = 0x0A;
pub const OB_Y: u32 = 0x0C;
pub const OB_HEIGHT: u32 = 0x16;
pub const OB_PRIORITY: u32 = 0x18;
pub const OB_ACTWID: u32 = 0x19;
pub const OB_FRAME: u32 = 0x1A;
pub const OB_ROUTINE: u32 = 0x24;

// Rótulos da ROM da tela do título.
pub const MAP_CRED: u32 = 0x1_7A8E;
pub const NEM_JAP_NAMES: u32 = 0x2_184C;
pub const NEM_CREDIT_TEXT: u32 = 0x6_203A;
pub const ENI_JAP_NAMES: u32 = 0x2_178E;
pub const PAL_FADE_IN: u32 = 0x1DA4;
pub const PAL_FADE_OUT: u32 = 0x1E4A;
pub const ARTTILE_CREDITS_FONT: u16 = 0x5A0;
pub const ARTTILE_SONIC_TEAM_FONT: u16 = 0xA6;
pub const ID_CREDITS_TEXT: u8 = 0x8A;
pub const F_CREDITSCHEAT: u32 = 0xFF_FFE3;
pub const V_PALETTE_WATER_FADING: u32 = 0xFF_FA00;

// Cenário da fase e do título (parte B da Etapa 7): limites, câmera, fundo e buffers de rolagem.
pub const V_LASTLAMP: u32 = 0xFF_FE30;
pub const V_DEBUGUSE: u32 = 0xFF_FE08;
pub const V_ZONE_ACT: u32 = 0xFF_FE10;
pub const V_ACT: u32 = 0xFF_FE11;
pub const F_DEMO: u32 = 0xFF_FFF0;
pub const V_UNUSED2: u32 = 0xFF_FFEA;
pub const V_UNUSED7: u32 = 0xFF_F740;
pub const V_UNUSED8: u32 = 0xFF_F741;
pub const V_DLE_ROUTINE: u32 = 0xFF_F742;
pub const F_NOBGSCROLL: u32 = 0xFF_F744;
pub const V_UNUSED9: u32 = 0xFF_F746;
pub const V_UNUSED10: u32 = 0xFF_F748;
pub const V_UNUSED11: u32 = 0xFF_F730;
pub const V_FG_XBLOCK: u32 = 0xFF_F74A;
pub const V_FG_YBLOCK: u32 = 0xFF_F74B;
pub const V_BG2_XBLOCK: u32 = 0xFF_F74E;
pub const V_BG3_XBLOCK: u32 = 0xFF_F750;
pub const V_FG_SCROLL_FLAGS: u32 = 0xFF_F754;
pub const V_BG1_SCROLL_FLAGS: u32 = 0xFF_F756;
pub const V_BG2_SCROLL_FLAGS: u32 = 0xFF_F758;
pub const V_BG3_SCROLL_FLAGS: u32 = 0xFF_F75A;
pub const F_BGSCROLLVERT: u32 = 0xFF_F75C;
pub const V_SCRSHIFTX: u32 = 0xFF_F73A;
pub const V_SCRSHIFTY: u32 = 0xFF_F73C;
pub const V_LOOKSHIFT: u32 = 0xFF_F73E;
pub const V_LIMITLEFT1: u32 = 0xFF_F720;
pub const V_LIMITRIGHT1: u32 = 0xFF_F722;
pub const V_LIMITTOP1: u32 = 0xFF_F724;
pub const V_LIMITBTM1: u32 = 0xFF_F726;
pub const V_LIMITLEFT2: u32 = 0xFF_F728;
pub const V_LIMITRIGHT2: u32 = 0xFF_F72A;
pub const V_LIMITTOP2: u32 = 0xFF_F72C;
pub const V_LIMITBTM2: u32 = 0xFF_F72E;
pub const V_LIMITLEFT3: u32 = 0xFF_F732;
pub const V_SCREENPOSY: u32 = 0xFF_F704;
pub const V_BGSCREENPOSY: u32 = 0xFF_F70C;
pub const V_BG2SCREENPOSX: u32 = 0xFF_F710;
pub const V_BG2SCREENPOSY: u32 = 0xFF_F714;
pub const V_BG3SCREENPOSY: u32 = 0xFF_F71C;
pub const V_BGSCRPOSY_VDP: u32 = 0xFF_F618;
pub const V_256LOOP1: u32 = 0xFF_F7AC;
pub const V_BG_SCROLL_BUFFER: u32 = 0xFF_A800;
pub const V_16X16: u32 = 0xFF_B000;
pub const V_256X256: u32 = 0xFF_0000;
pub const V_LVLLAYOUT_FG: u32 = 0xFF_A400;
pub const V_LVLLAYOUT_BG: u32 = 0xFF_A440;
/// Início do layout de fase; o `LevelLayoutLoad` limpa $800 bytes a partir daqui (ver `cenario`).
pub const V_LVLLAYOUT: u32 = 0xFF_A400;
/// Cada linha do layout ocupa $80 bytes na RAM (`layout_row`).
pub const LAYOUT_ROW: u32 = 0x80;
pub const OB_INERTIA: u32 = 0x14;
pub const OB_STATUS: u32 = 0x22;

// Tabelas da ROM do cenário e da tela do título (rótulos do disassembly).
pub const LEVEL_SIZE_ARRAY: u32 = 0x5F2A;
pub const START_LOC_ARRAY: u32 = 0x611E;
pub const LOOP_CHUNK_NUMS: u32 = 0x618E;
pub const LEVEL_INDEX: u32 = 0x68B96;
pub const BLK16_GHZ: u32 = 0x3_C19C;
pub const BLK256_GHZ: u32 = 0x3_F544;
pub const NEM_TITLE_FG: u32 = 0x1_ED80;
pub const NEM_TITLE_SONIC: u32 = 0x1_FD8C;
pub const NEM_TITLE_TM: u32 = 0x2_175A;
pub const ART_TEXT: u32 = 0x5F0;
pub const ART_TEXT_END: u32 = 0xB10;
pub const ARTTILE_TITLE_FOREGROUND: u16 = 0x200;
pub const ARTTILE_TITLE_SONIC: u16 = 0x300;
pub const ARTTILE_TITLE_TRADEMARK: u16 = 0x510;
pub const ARTTILE_LEVEL_SELECT_FONT: u16 = 0x680;

// Fila de arte (`v_plc_*`): o estado da descompressão Nemesis por quadro fica nestas variáveis.
pub const V_PLC_BUFFER_DEST: u32 = 0xFF_F684;
pub const V_PLC_BUFFER_ONLY_END: u32 = 0xFF_F6E0;
pub const V_PLC_PTRNEMCODE: u32 = 0xFF_F6E0;
pub const V_PLC_REPEATCOUNT: u32 = 0xFF_F6E4;
pub const V_PLC_PALETTEINDEX: u32 = 0xFF_F6E8;
pub const V_PLC_PREVIOUSROW: u32 = 0xFF_F6EC;
pub const V_PLC_DATAWORD: u32 = 0xFF_F6F0;
pub const V_PLC_SHIFTVALUE: u32 = 0xFF_F6F4;
pub const V_PLC_PATTERNSLEFT: u32 = 0xFF_F6F8;
pub const V_PLC_FRAMEPATTERNSLEFT: u32 = 0xFF_F6FA;
/// Tamanho de uma entrada da fila (`plc_slot_size`): longa do endereço da arte e palavra da VRAM.
pub const PLC_SLOT_SIZE: u32 = 6;
/// Rótulo `ArtLoadCues`: lista de PLCs. O `plcid_Main` é o índice zero.
pub const ART_LOAD_CUES: u32 = 0x1_DD86;
pub const PLCID_MAIN: u16 = 0;
/// Rotinas de escrita do Nemesis por quadro: o endereço delas é o que `v_plc_ptrnemcode` guarda.
pub const NEM_PCD_WRITE_ROW_TO_VDP: u32 = 0x14FA;
pub const NEM_PCD_WRITE_ROW_TO_VDP_XOR: u32 = 0x1504;
pub const NEM_GHZ_1ST: u32 = 0x3_CB3C;
pub const ENI_TITLE: u32 = 0x1_EC6C;
pub const PALID_TITLE: u32 = 1;
pub const BGM_TITLE: u8 = 0x8A;
pub const F_DEBUGMODE: u32 = 0xFF_FFFA;
pub const V_TITLE_DCOUNT: u32 = 0xFF_FFE4;
pub const V_TITLE_CCOUNT: u32 = 0xFF_FFE6;

// Objetos da tela do título (0E, 0F) e campos de animação.
pub const ID_TITLE_SONIC: u8 = 0x0E;
pub const ID_PSBTM: u8 = 0x0F;
pub const OB_ANI_FRAME: u32 = 0x1B;
pub const OB_ANIM: u32 = 0x1C;
pub const OB_PREV_ANI: u32 = 0x1D;
pub const OB_TIME_FRAME: u32 = 0x1E;
pub const OB_DELAY_ANI: u32 = 0x1F;
pub const ANI_TSON: u32 = 0xAD8E;
pub const ANI_PSBTM: u32 = 0xAD9C;
pub const ANI_RING: u32 = 0xA608;
pub const MAP_TSON: u32 = 0xAF12;
pub const MAP_PSB: u32 = 0xAE4E;
/// Rótulo `Tit_MainLoop`: começo do laço do título, depois da parte C.
pub const TIT_MAIN_LOOP: u32 = 0x315E;

// Laço do título, `GotoDemo` e começo de `GM_Level` (fatia 3).
pub const V_PALETTE_LINE_3: u32 = 0xFF_FB40;
pub const V_JPADPRESS1: u32 = 0xFF_F605;
pub const F_LEVSELCHEAT: u32 = 0xFF_FFE0;
pub const V_DEMONUM: u32 = 0xFF_FFF2;
pub const V_LIVES: u32 = 0xFF_FE12;
pub const V_LASTSPECIAL: u32 = 0xFF_FE16;
pub const V_RINGS: u32 = 0xFF_FE20;
pub const V_TIME: u32 = 0xFF_FE22;
pub const V_SCORE: u32 = 0xFF_FE26;
pub const V_SCORELIFE: u32 = 0xFF_FFC0;
/// Tabelas de quatro cores dos ciclos de paleta da água (título e GHZ).
pub const PAL_TITLECYC_WATER: u32 = 0x1B5E;
pub const PAL_GHZCYC_WATER: u32 = 0x1B7E;
/// Sequências do código da seleção de fases (`LevSelCode_J` e `LevSelCode_US`).
pub const LEVSELCODE_J: u32 = 0x3396;
pub const LEVSELCODE_US: u32 = 0x339E;
/// `Demo_Levels`: a fase de cada demo, em palavras.
pub const DEMO_LEVELS: u32 = 0x3454;
/// Comando de som que apaga a música aos poucos (`bgm_Fade`).
pub const BGM_FADE: u8 = 0xE0;
/// Som do anel (`sfx_Ring`), tocado quando o código entra.
pub const SFX_RING: u8 = 0xB5;
/// Modos de jogo (`id_Demo`, `id_Special`) e a zona `id_SBZ`.
pub const ID_DEMO: u8 = 0x08;
pub const ID_SPECIAL: u8 = 0x10;
pub const ID_SBZ: u8 = 5;
/// Botões (`btnDir`, `btnC`, `btnStart`).
pub const BTN_DIR: u8 = 0x0F;
pub const BTN_C: u8 = 0x20;
pub const BTN_START: u8 = 0x80;

// Fase da demo, fatia 1: do fim do primeiro desvanecimento ao laço dos cartões de título.
pub const V_SOUNDQUEUE0: u32 = 0xFF_F00A;
pub const V_AIR: u32 = 0xFF_FE14;
pub const ID_LZ: u8 = 1;
pub const ID_MZ: u8 = 2;
pub const ID_LZ_ACT4: u16 = 0x103;
pub const ID_FZ: u16 = 0x502;
/// Intervalos que `Level_ClrRam` limpa.
pub const V_MISC_VARIABLES: u32 = 0xFF_F628;
pub const V_MISC_VARIABLES_END: u32 = 0xFF_F680;
pub const V_LEVELVARIABLES: u32 = 0xFF_F700;
pub const V_LEVELVARIABLES_END: u32 = 0xFF_F800;
pub const V_TIMINGANDSCREENVARIABLES: u32 = 0xFF_FE60;
pub const V_TIMINGANDSCREENVARIABLES_END: u32 = 0xFF_FF80;
/// Arte e cabeçalhos da fase.
pub const NEM_TITLECARD: u32 = 0x3_9204;
pub const ARTTILE_TITLE_CARD: u16 = 0x580;
pub const LEVEL_HEADERS: u32 = 0x1_DD16;
pub const MUSIC_LIST: u32 = 0x3796;
pub const PALID_SONIC: u32 = 3;
pub const PLCID_MAIN2: u16 = 1;
pub const PLCID_EXPLODE: u16 = 2;
pub const PLCID_GHZANIMALS: u16 = 0x15;
/// Cartões de título (objeto `34`): quatro elementos seguidos a partir de `v_titlecard`.
pub const ID_TITLE_CARD: u8 = 0x34;
pub const V_TITLECARD: u32 = 0xFF_D080;
pub const V_TTLCARDACT: u32 = 0xFF_D100;
pub const MAP_CARD: u32 = 0xD074;
pub const CARD_CONDATA: u32 = 0xCAD2;
pub const CARD_ITEMDATA: u32 = 0xCAC2;
/// `card_mainX` e `card_finalX` (`objoff_30` e `objoff_32`).
pub const CARD_MAINX: u32 = 0x30;
pub const CARD_FINALX: u32 = 0x32;
/// Bit de prioridade do tile (`Tile_Prio`).
pub const TILE_PRIO: u16 = 0x8000;
/// Cópias que o VBlank das fases faz para o `LoadTilesAsYouMove`.
pub const V_SCREENPOSX_DUP: u32 = 0xFF_FF10;
pub const V_BGSCREENPOSX_DUP: u32 = 0xFF_FF18;
pub const V_BG2SCREENPOSX_DUP: u32 = 0xFF_FF20;
pub const V_BG3SCREENPOSX_DUP: u32 = 0xFF_FF28;
pub const V_FG_SCROLL_FLAGS_DUP: u32 = 0xFF_FF30;
pub const V_BG1_SCROLL_FLAGS_DUP: u32 = 0xFF_FF32;
pub const V_BG2_SCROLL_FLAGS_DUP: u32 = 0xFF_FF34;
pub const V_BG3_SCROLL_FLAGS_DUP: u32 = 0xFF_FF36;
/// Arte do Sonic que muda por quadro (`v_sgfx_buffer`) e a flag de troca.
pub const F_SONFRAMECHG: u32 = 0xFF_F767;
pub const V_SGFX_BUFFER: u32 = 0xFF_C800;
pub const V_SGFX_BUFFER_END: u32 = 0xFF_CAE0;
pub const ARTTILE_SONIC: u16 = 0x780;
/// Tiles animados (`AnimateLevelGfx`).
pub const F_PAUSE: u32 = 0xFF_F63A;
pub const V_GFXBIGRING: u32 = 0xFF_F7BE;
pub const V_LANI0_FRAME: u32 = 0xFF_F7B0;
pub const V_LANI0_TIME: u32 = 0xFF_F7B1;
pub const V_LANI1_FRAME: u32 = 0xFF_F7B2;
pub const V_LANI1_TIME: u32 = 0xFF_F7B3;
pub const V_LANI2_FRAME: u32 = 0xFF_F7B4;
pub const V_LANI2_TIME: u32 = 0xFF_F7B5;
pub const ART_GHZWATER: u32 = 0x6_6A96;
pub const ART_GHZFLOWER1: u32 = 0x6_6C96;
pub const ART_GHZFLOWER2: u32 = 0x6_7096;
pub const ART_BIGRING: u32 = 0x6_A2E4;
pub const ARTTILE_GHZ_WATERFALL: u16 = 0x378;
pub const ARTTILE_GHZ_BIG_FLOWER_1: u16 = 0x35C;
pub const ARTTILE_GHZ_SMALL_FLOWER: u16 = 0x36C;
pub const ARTTILE_GIANT_RING: u16 = 0x400;
/// Flags de atualização do HUD.
pub const F_LIFECOUNT: u32 = 0xFF_FE1C;
pub const F_RINGCOUNT: u32 = 0xFF_FE1D;
pub const F_TIMECOUNT: u32 = 0xFF_FE1E;
pub const F_SCORECOUNT: u32 = 0xFF_FE1F;
pub const F_ENDACTBONUS: u32 = 0xFF_F7D6;

// Fase da demo, fatia 2: o resto da carga da fase, o Sonic e o HUD.
pub const V_SONSPEEDMAX: u32 = 0xFF_F760;
pub const V_SONSPEEDACC: u32 = 0xFF_F762;
pub const V_SONSPEEDDEC: u32 = 0xFF_F764;
pub const V_SONFRAMENUM: u32 = 0xFF_F766;
pub const V_ANGLEBUFFER: u32 = 0xFF_F768;
pub const V_ANGLEBUFFER2: u32 = 0xFF_F76A;
pub const V_TRACKPOS: u32 = 0xFF_F7A8;
pub const V_TRACKBYTE: u32 = 0xFF_F7A9;
pub const V_TRACKSONIC: u32 = 0xFF_CB00;
pub const V_256LOOP2: u32 = 0xFF_F7AD;
pub const V_256ROLL1: u32 = 0xFF_F7AE;
pub const V_256ROLL2: u32 = 0xFF_F7AF;
pub const F_LOCKSCREEN: u32 = 0xFF_F7AA;
pub const F_PLAYERCTRL: u32 = 0xFF_F7C8;
pub const F_LOCKCTRL: u32 = 0xFF_F7CC;
pub const F_WTUNNELMODE: u32 = 0xFF_F7C7;
pub const F_SLIDEMODE: u32 = 0xFF_F7CA;
pub const V_COLLINDEX: u32 = 0xFF_F796;
pub const V_JPADHOLD2: u32 = 0xFF_F602;
pub const V_JPADPRESS2: u32 = 0xFF_F603;
pub const V_OPL_ROUTINE: u32 = 0xFF_F76C;
pub const V_OPL_SCREEN: u32 = 0xFF_F76E;
pub const V_OPL_DATA: u32 = 0xFF_F770;
pub const V_OBJSTATE: u32 = 0xFF_FC00;
pub const V_OBJSTATE_END: u32 = 0xFF_FCC0;
pub const V_LVLOBJSPACE: u32 = 0xFF_D800;
pub const V_LVLOBJEND: u32 = 0xFF_F000;
pub const V_OSCILLATE: u32 = 0xFF_FE5E;
pub const V_FRAMECOUNT: u32 = 0xFF_FE04;
pub const F_RESTART: u32 = 0xFF_FE02;
pub const V_LIFECOUNT: u32 = 0xFF_FE1B;
pub const F_TIMEOVER: u32 = 0xFF_FE1A;
pub const V_SHIELD: u32 = 0xFF_FE2C;
pub const V_INVINC: u32 = 0xFF_FE2D;
pub const V_SHOES: u32 = 0xFF_FE2E;
pub const V_UNUSED1: u32 = 0xFF_FE2F;
pub const V_BTNPUSHTIME1: u32 = 0xFF_F790;
pub const V_BTNPUSHTIME2: u32 = 0xFF_F792;
pub const V_HUD: u32 = 0xFF_D040;
pub const V_FRAMEBYTE: u32 = 0xFF_FE05;
pub const V_TIMEMIN: u32 = 0xFF_FE23;
pub const V_LASTLAMP_FASE: u32 = 0xFF_FE30;
pub const F_DEBUGCHEAT: u32 = 0xFF_FFE2;
pub const F_HBLANK_PAL_ADDR: u32 = 0xFF_F644;
pub const ID_SONIC: u8 = 0x01;
pub const ID_HUD: u8 = 0x21;
// Campos do objeto do Sonic.
pub const OB_VELX: u32 = 0x10;
pub const OB_VELY: u32 = 0x12;
pub const OB_WIDTH: u32 = 0x17;
pub const OB_COLTYPE: u32 = 0x20;
pub const OB_RESPAWNNO: u32 = 0x23;
pub const OB_ANGLE: u32 = 0x26;
pub const OB_SUBTYPE: u32 = 0x28;
pub const SON_FLASHTIME: u32 = 0x30;
pub const SON_INVTIME: u32 = 0x32;
pub const SON_SHOETIME: u32 = 0x34;
pub const SON_ANGLERIGHT: u32 = 0x36;
pub const SON_ANGLELEFT: u32 = 0x37;
pub const SON_STICKTOCONVEX: u32 = 0x38;
pub const SON_JUMPING: u32 = 0x3C;
pub const SON_STANDONOBJECT: u32 = 0x3D;
pub const SON_LOCKTIME: u32 = 0x3E;
// Tabelas e arte na ROM.
pub const SINE_DATA: u32 = 0x29EA;
pub const ANGLEMAP: u32 = 0x6_2900;
pub const COLLARRAY1: u32 = 0x6_2A00;
pub const COLLARRAY2: u32 = 0x6_3A00;
pub const REACT_SIZES: u32 = 0x1_B5E4;
pub const ANGLE_DATA: u32 = 0x2CD0;
pub const V_ITEMBONUS: u32 = 0xFF_F7D0;
pub const V_UNUSED3: u32 = 0xFF_FFEC;
pub const COLPOINTERS: u32 = 0x4102;
pub const SONIC_DYN_PLC: u32 = 0x2_2310;
pub const ART_SONIC: u32 = 0x2_2610;
pub const ANI_SONIC: u32 = 0x1_421C;
pub const MAP_SONIC: u32 = 0x2_1CF4;
pub const MAP_HUD: u32 = 0x1_CD6A;
pub const ART_HUD: u32 = 0x1_D2A6;
pub const ART_LIVESNUMS: u32 = 0x1_D5A6;
pub const OBJPOS_INDEX: u32 = 0x6_B000;
pub const DEMO_DATA_PTR: u32 = 0x4082;
pub const SONANI_WALK: u32 = 0x1_425A;
pub const SONANI_RUN: u32 = 0x1_4262;
pub const SONANI_ROLL: u32 = 0x1_426A;
pub const SONANI_ROLL2: u32 = 0x1_4272;
pub const SONANI_PUSH: u32 = 0x1_427A;
pub const BG_SCROLLBLOCKMAP_GHZ: u32 = 0x724A;
pub const LEVEL_INDEX_FASE: u32 = 0x6_8B96;
// Pedaços de VRAM do HUD e da vida.
pub const ARTTILE_HUD: u16 = 0x6CA;
pub const ARTTILE_HUDSCORE_E: u32 = 0x6E2;
pub const ARTTILE_HUDSCORE: u32 = 0x6E4;
pub const ARTTILE_HUDTIMEMINS: u32 = 0x6F2;
pub const ARTTILE_HUDTIMESECS: u32 = 0x6F6;
pub const ARTTILE_HUDRINGS: u32 = 0x6FA;
pub const ARTTILE_LIVES_NUM: u32 = 0x7DD;
// Animações do Sonic.
pub const ANIM_WALK: u8 = 0;
pub const ANIM_RUN: u8 = 1;
pub const ANIM_ROLL: u8 = 2;
pub const ANIM_ROLL2: u8 = 3;
pub const ANIM_PUSH: u8 = 4;
pub const ANIM_WAIT: u8 = 5;
pub const ANIM_BALANCE: u8 = 6;
pub const ANIM_LOOKUP: u8 = 7;
pub const ANIM_DUCK: u8 = 8;
pub const ANIM_DEATH: u8 = 0x18;
pub const ANIM_HURT: u8 = 0x1A;
pub const SON_RESTARTIME: u32 = 0x3A;

pub const ID_SBZ_ACT2: u16 = 0x501;
pub const ID_RINGS: u8 = 0x25;
pub const MAP_RING: u32 = 0xA610;
pub const ARTTILE_RING: u16 = 0x7B2;
pub const V_ANI1_FRAME: u32 = 0xFF_FEC3;
pub const V_ANI0_TIME: u32 = 0xFF_FEC0;
pub const V_ANI0_FRAME: u32 = 0xFF_FEC1;
pub const V_ANI1_TIME: u32 = 0xFF_FEC2;
pub const V_ANI2_TIME: u32 = 0xFF_FEC4;
pub const V_ANI2_FRAME: u32 = 0xFF_FEC5;
pub const V_ANI3_TIME: u32 = 0xFF_FEC6;
pub const V_ANI3_FRAME: u32 = 0xFF_FEC7;
pub const V_ANI3_BUF: u32 = 0xFF_FEC8;
pub const OSCILLATE_SETTINGS: u32 = 0x41C8;
pub const ID_COLLAPSE_LEDGE: u8 = 0x1A;
pub const ID_CRABMEAT: u8 = 0x1F;
pub const ID_SCENERY: u8 = 0x1C;
pub const ID_BRIDGE: u8 = 0x11;
pub const ID_MONITOR: u8 = 0x26;
pub const ID_EXPLOSION_ITEM: u8 = 0x27;
pub const ID_ANIMALS: u8 = 0x28;
pub const ID_POINTS: u8 = 0x29;
pub const ID_CHOPPER: u8 = 0x2B;
pub const ID_PURPLE_ROCK: u8 = 0x3B;
pub const ID_SPRING: u8 = 0x41;
pub const ID_SPIKES: u8 = 0x36;
pub const ID_EDGE_WALLS: u8 = 0x44;
pub const ID_WATER_SOUND: u8 = 0x49;
pub const MAP_SPRING: u32 = 0xE3A8;
pub const ANI_SPRING: u32 = 0xE38C;
pub const SPRING_POWERS: u32 = 0xE192;
pub const MAP_MONITOR: u32 = 0xAC14;
pub const ARTTILE_MONITOR: u16 = 0x680;
pub const ANI_MONITOR: u32 = 0xAB92;
pub const OB_ROUTINE_2: u32 = 0x25;
pub const STATUS_XFLIP: u8 = 1;
pub const STATUS_IN_AIR: u8 = 2;
pub const STATUS_ROLLING: u8 = 4;
pub const STATUS_ON_OBJECT: u8 = 8;
pub const STATUS_PUSHING: u8 = 0x20;
pub const STATUS_UNDERWATER: u8 = 0x40;
pub const RENDER_CAM_FIELD: u8 = 4;
pub const RENDER_SCREEN: u8 = 0;

// Placa de fim de ato (Signpost, 0x0D) e cartoes de vitoria (GotThroughCard, 0x3A).
pub const ID_SIGNPOST: u8 = 0x0D;
pub const ID_GOT_THROUGH_CARD: u8 = 0x3A;
pub const ID_LEVEL: u8 = 0x0C;
pub const MAP_SIGN: u32 = 0x00F3C4;
pub const ANI_SIGN: u32 = 0x00F3A8;
pub const ARTTILE_SIGNPOST: u16 = 0x680;
pub const SFX_SIGNPOST: u8 = 0xCF;
pub const PLCID_TITLECARD: u16 = 0x10;
pub const PLCID_SIGNPOST: u16 = 0x12;
pub const BGM_GOT_THROUGH: u8 = 0x8E;
pub const SFX_CASH: u8 = 0xC5;
pub const SFX_SWITCH: u8 = 0xCD;
pub const MAP_GOT: u32 = 0x00D266;
pub const GOT_ITEMDATA: u32 = 0x00CE04;
pub const LEVEL_ORDER_ADDR: u32 = 0x00CD82;
pub const V_ENDCARD: u32 = 0xFF_D5C0;
pub const V_TIMEBONUS: u32 = 0xFF_F7D2;
pub const V_RINGBONUS: u32 = 0xFF_F7D4;
pub const V_TIMESEC: u32 = 0xFF_FE24;
pub const V_TIMECENT: u32 = 0xFF_FE25;
pub const V_CONTINUES: u32 = 0xFF_FE18;
pub const V_EMERALDS: u32 = 0xFF_FE57;
pub const V_EMLDLIST: u32 = 0xFF_FE58;
pub const OB_OFF30: u32 = 0x30;
pub const OB_OFF32: u32 = 0x32;
pub const OB_OFF34: u32 = 0x34;
