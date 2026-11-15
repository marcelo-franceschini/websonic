//! Inicialização e deformação do fundo de Labyrinth Zone, REV01.
use super::boot::{rom_b, rom_l, rom_w};
use super::ram::*;
use super::{rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

const BG1_XBLOCK: u32 = 0xFF_F74C;
const BG1_YBLOCK: u32 = 0xFF_F74D;
const WATER1: u32 = 0xFF_F646;
const LZ_DEFORM: u32 = 0xFF_F7D8;
const LZ_SCROLL_DATA: u32 = 0x6448;
const DROWN_WOBBLE_DATA: u32 = 0x1_44F0;
const PAL_CONVEYOR_SEQ: u32 = 0x1A34;
const PAL_WATERFALL: u32 = 0x1B9E;
const PAL_CONVEYOR: u32 = 0x1BBE;
const PAL_CONVEYOR_UW: u32 = 0x1BD0;
const PAL_SBZ3_WATERFALL: u32 = 0x1BE2;
const PAL_BUFFER: u32 = 0xFF_F650;
const CONVEYOR_REVERSE: u32 = 0xFF_F7C0;

/// Inicializa a altura do fundo com metade da posição vertical da câmera.
pub fn iniciar_fundo(m: &mut Memoria, camera_y: u16) {
    ww(m, V_BGSCREENPOSY, camera_y >> 1);
}

/// Atualiza cores da cachoeira e das esteiras, nas paletas seca e submersa.
pub fn ciclo_paleta(m: &mut Memoria) {
    let tempo = rw(m, V_PCYC_TIME).wrapping_sub(1);
    ww(m, V_PCYC_TIME, tempo);
    if tempo & 0x8000 != 0 {
        ww(m, V_PCYC_TIME, 2);
        let passo = rw(m, V_PCYC_NUM);
        ww(m, V_PCYC_NUM, passo.wrapping_add(1));
        let tabela = if rb(m, V_ACT) == 3 {
            PAL_SBZ3_WATERFALL
        } else {
            PAL_WATERFALL
        };
        let fonte = tabela.wrapping_add(u32::from(passo & 3).wrapping_mul(8));
        let primeira = rom_l(m, fonte);
        let segunda = rom_l(m, fonte.wrapping_add(4));
        for destino in [0xFF_FB56, 0xFF_FAD6] {
            wl(m, destino, primeira);
            wl(m, destino.wrapping_add(4), segunda);
        }
    }
    let quadro = rw(m, V_FRAMECOUNT) & 7;
    if rom_b(m, PAL_CONVEYOR_SEQ.wrapping_add(u32::from(quadro))) == 0 {
        return;
    }
    let anterior = rw(m, PAL_BUFFER) & 3;
    let reversa = rb(m, CONVEYOR_REVERSE) != 0;
    let proximo = if reversa {
        if anterior == 0 {
            2
        } else {
            anterior.wrapping_sub(1)
        }
    } else if anterior >= 2 {
        0
    } else {
        anterior.wrapping_add(1)
    };
    ww(m, PAL_BUFFER, proximo);
    let desloc = u32::from(proximo).wrapping_mul(6);
    for (fonte, destino) in [(PAL_CONVEYOR, 0xFF_FB76), (PAL_CONVEYOR_UW, 0xFF_FAF6)] {
        let primeira = rom_l(m, fonte.wrapping_add(desloc));
        let segunda = rom_w(m, fonte.wrapping_add(desloc).wrapping_add(4));
        wl(m, destino, primeira);
        ww(m, destino.wrapping_add(4), segunda);
    }
}

fn scroll(m: &mut Memoria, posicao: u32, bloco: u32, delta: i32, bit: u8) {
    let anterior = rl(m, posicao);
    let novo = anterior.wrapping_add(u32::from_be_bytes(delta.to_be_bytes()));
    wl(m, posicao, novo);
    let bit_atual = u8::try_from((novo >> 16) & 0x10).unwrap_or(0);
    let bloco_atual = rb(m, bloco);
    if bit_atual == bloco_atual {
        wb(m, bloco, bloco_atual ^ 0x10);
        let diferenca = i32::from_be_bytes(novo.wrapping_sub(anterior).to_be_bytes());
        let direcao = if diferenca < 0 {
            bit
        } else {
            bit.wrapping_add(1)
        };
        let flags = rb(m, V_BG1_SCROLL_FLAGS) | 1u8.wrapping_shl(u32::from(direcao));
        wb(m, V_BG1_SCROLL_FLAGS, flags);
    }
}

/// Atualiza posição e flags do fundo e produz as 224 linhas de rolagem da LZ.
pub fn deformar(m: &mut Memoria) {
    let dx = i32::from(i16::from_be_bytes(rw(m, V_SCRSHIFTX).to_be_bytes())).wrapping_shl(7);
    let dy = i32::from(i16::from_be_bytes(rw(m, V_SCRSHIFTY).to_be_bytes())).wrapping_shl(7);
    scroll(m, V_BGSCREENPOSX, BG1_XBLOCK, dx, 2);
    scroll(m, V_BGSCREENPOSY, BG1_YBLOCK, dy, 0);
    let bg_y = rw(m, V_BGSCREENPOSY);
    ww(m, V_BGSCRPOSY_VDP, bg_y);
    let fase = rb(m, LZ_DEFORM);
    let acumulador = rw(m, LZ_DEFORM).wrapping_add(0x80);
    ww(m, LZ_DEFORM, acumulador);
    let camera_y = rw(m, V_SCREENPOSY);
    let mut indice_bg = fase.wrapping_add(bg_y.to_be_bytes()[1]);
    let mut indice_fg = fase.wrapping_add(camera_y.to_be_bytes()[1]);
    let fg = 0u16.wrapping_sub(rw(m, V_SCREENPOSX));
    let bg = 0u16.wrapping_sub(rw(m, V_BGSCREENPOSX));
    let nivel = i16::from_be_bytes(rw(m, WATER1).to_be_bytes());
    let mut linha_y = camera_y;
    let mut destino = V_HSCROLLTABLEBUFFER;
    for _ in 0..224 {
        let (frente, fundo) = if i16::from_be_bytes(linha_y.to_be_bytes()) >= nivel {
            let onda_fg = i16::from(i8::from_be_bytes([rom_b(
                m,
                LZ_SCROLL_DATA.wrapping_add(u32::from(indice_fg)),
            )]));
            let onda_bg = i16::from(i8::from_be_bytes([rom_b(
                m,
                DROWN_WOBBLE_DATA.wrapping_add(u32::from(indice_bg)),
            )]));
            (
                fg.wrapping_add(u16::from_be_bytes(onda_fg.to_be_bytes())),
                bg.wrapping_add(u16::from_be_bytes(onda_bg.to_be_bytes())),
            )
        } else {
            (fg, bg)
        };
        wl(
            m,
            destino,
            u32::from(frente).wrapping_shl(16) | u32::from(fundo),
        );
        destino = destino.wrapping_add(4);
        linha_y = linha_y.wrapping_add(1);
        indice_bg = indice_bg.wrapping_add(1);
        indice_fg = indice_fg.wrapping_add(1);
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn ciclo_de_cores_atualiza_paletas_e_respeita_reversao() {
        let mut rom = vec![0; 0x20000];
        rom[usize::try_from(PAL_CONVEYOR_SEQ).unwrap()] = 1;
        let cachoeira = usize::try_from(PAL_WATERFALL).unwrap();
        rom[cachoeira..cachoeira.wrapping_add(8)].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        let esteira = usize::try_from(PAL_CONVEYOR.wrapping_add(12)).unwrap();
        rom[esteira..esteira.wrapping_add(6)].copy_from_slice(&[9, 8, 7, 6, 5, 4]);
        let mut m = Memoria::nova(rom);
        wb(&mut m, CONVEYOR_REVERSE, 1);
        ciclo_paleta(&mut m);
        assert_eq!(rl(&mut m, 0xFF_FB56), 0x0102_0304);
        assert_eq!(rl(&mut m, 0xFF_FAD6), 0x0102_0304);
        assert_eq!(rw(&mut m, PAL_BUFFER), 2);
        assert_eq!(rl(&mut m, 0xFF_FB76), 0x0908_0706);
        assert_eq!(rw(&mut m, 0xFF_FB7A), 0x0504);
        assert_eq!(rw(&mut m, V_PCYC_TIME), 2);
    }
    #[test]
    fn fundo_divide_altura_inicial() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        iniciar_fundo(&mut m, 0x351);
        assert_eq!(rw(&mut m, V_BGSCREENPOSY), 0x1A8);
    }
    #[test]
    fn linhas_acima_da_agua_nao_ondulam_e_indices_avancam() {
        let mut rom = vec![0; 0x20000];
        rom[usize::try_from(LZ_SCROLL_DATA).unwrap().wrapping_add(2)] = 3;
        rom[usize::try_from(DROWN_WOBBLE_DATA).unwrap().wrapping_add(2)] = 0xFE;
        let mut m = Memoria::nova(rom);
        ww(&mut m, WATER1, 2);
        ww(&mut m, V_SCREENPOSX, 10);
        ww(&mut m, V_BGSCREENPOSX, 4);
        deformar(&mut m);
        assert_eq!(rl(&mut m, V_HSCROLLTABLEBUFFER), 0xFFF6_FFFC);
        assert_eq!(
            rl(&mut m, V_HSCROLLTABLEBUFFER.wrapping_add(8)),
            0xFFF9_FFFA
        );
        assert_eq!(rw(&mut m, LZ_DEFORM), 0x80);
    }
    #[test]
    fn fundo_aplica_deslocamento_e_marcas_de_direcao() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        ww(&mut m, V_SCRSHIFTX, 0x200);
        ww(&mut m, V_SCRSHIFTY, 0xFE00);
        wb(&mut m, BG1_XBLOCK, 0);
        wb(&mut m, BG1_YBLOCK, 0x10);
        deformar(&mut m);
        assert_eq!(rl(&mut m, V_BGSCREENPOSX), 0x10000);
        assert_eq!(rl(&mut m, V_BGSCREENPOSY), 0xFFFF0000);
        assert_eq!(rb(&mut m, V_BG1_SCROLL_FLAGS) & 0x09, 0x09);
    }
}
