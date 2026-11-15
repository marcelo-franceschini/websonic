//! Ciclos de paleta (`_inc/PaletteCycle.asm`): `PalCycle_Title` no laço do título e
//! `PaletteCycle` da GHZ no `GotoDemo`. Os dois usam o mesmo código (`PCycGHZ_Go`) com tabelas
//! diferentes na ROM.

use super::boot::{rom_l, rom_w};
use super::ram::*;
use super::{ErroAbertura, rb, rw, wl, ww};
use crate::memoria::Memoria;

/// Quadros entre dois passos do ciclo da água (`move.w #6-1,(v_pcyc_time).w`).
const QUADROS_POR_PASSO: u16 = 6;

/// `PalCycle_Title`: ciclo da água do título.
pub fn pal_cycle_title(m: &mut Memoria) {
    pcyc_ghz_go(m, PAL_TITLECYC_WATER);
}

/// `PaletteCycle`: GHZ e final usam a água; o ciclo da MZ não realiza operações na REV01.
pub fn palette_cycle(m: &mut Memoria) -> Result<(), ErroAbertura> {
    match rb(m, V_ZONE) {
        0 | 6 => {
            pcyc_ghz_go(m, PAL_GHZCYC_WATER);
            Ok(())
        }
        ID_MZ => Ok(()),
        ID_SLZ => {
            ciclo_slz(m);
            Ok(())
        }
        ID_SYZ => {
            super::spring_yard_cenario::ciclo_paleta(m);
            Ok(())
        }
        ID_LZ => {
            super::labirinto_cenario::ciclo_paleta(m);
            Ok(())
        }
        _ => Err(ErroAbertura::NaoImplementado(
            "ciclo de paleta de outra zona",
        )),
    }
}

/// `PalCycle_SLZ`: três cores, em seis passos, carregadas da ROM.
fn ciclo_slz(m: &mut Memoria) {
    let tempo = rw(m, V_PCYC_TIME).wrapping_sub(1);
    ww(m, V_PCYC_TIME, tempo);
    if tempo & 0x8000 == 0 {
        return;
    }
    ww(m, V_PCYC_TIME, 7);
    let proximo = rw(m, V_PCYC_NUM).wrapping_add(1);
    let passo = if proximo < 6 { proximo } else { 0 };
    ww(m, V_PCYC_NUM, passo);
    let fonte = 0x1C4A_u32.wrapping_add(u32::from(passo).wrapping_mul(6));
    let cor = rom_w(m, fonte);
    let outras = rom_l(m, fonte.wrapping_add(2));
    ww(m, V_PALETTE_LINE_3.wrapping_add(22), cor);
    wl(m, V_PALETTE_LINE_3.wrapping_add(26), outras);
}

/// `PCycGHZ_Go`: a cada seis quadros, quatro cores da tabela vão para as cores 8 a 11 da linha 3
/// da paleta. A tabela tem quatro passos de oito bytes.
fn pcyc_ghz_go(m: &mut Memoria, tabela: u32) {
    let tempo = rw(m, V_PCYC_TIME).wrapping_sub(1);
    ww(m, V_PCYC_TIME, tempo);
    // `bpl .return`: ainda há tempo (o contador não ficou negativo).
    if tempo & 0x8000 == 0 {
        return;
    }
    ww(m, V_PCYC_TIME, QUADROS_POR_PASSO.wrapping_sub(1));
    let passo = rw(m, V_PCYC_NUM);
    ww(m, V_PCYC_NUM, passo.wrapping_add(1));
    let origem = tabela.wrapping_add(u32::from(passo & 3).wrapping_mul(8));
    let destino = V_PALETTE_LINE_3.wrapping_add(8 * 2);
    wl(m, destino, rom_l(m, origem));
    wl(m, destino.wrapping_add(4), rom_l(m, origem.wrapping_add(4)));
}
