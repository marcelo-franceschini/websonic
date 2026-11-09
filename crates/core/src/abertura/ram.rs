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
pub const V_VDP_BUFFER2: u32 = 0xFF_F640;
pub const F_HBLANK_PAL: u32 = 0xFF_F644;
pub const F_WTR_STATE: u32 = 0xFF_F64E;
pub const V_GENERICTIMER: u32 = 0xFF_F614;
pub const V_PCYC_NUM: u32 = 0xFF_F632;
pub const V_PCYC_TIME: u32 = 0xFF_F634;
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
pub const MAP_TSON: u32 = 0xAF12;
pub const MAP_PSB: u32 = 0xAE4E;
/// Rótulo `Tit_MainLoop`: começo do laço do título, depois da parte C.
pub const TIT_MAIN_LOOP: u32 = 0x315E;
