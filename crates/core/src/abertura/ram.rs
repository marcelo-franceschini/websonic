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
