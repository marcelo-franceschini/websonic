//! Fundo e ciclo de paleta da Spring Yard Zone, REV01.
use super::boot::{rom_l, rom_w};
use super::ram::*;
use super::{rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

const BG1_YBLOCK: u32 = 0xFF_F74D;
const PAL_PRETO_AMARELO: u32 = 0x1C6E;
const PAL_VERMELHO_BRANCO: u32 = 0x1C8E;

/// `BgScroll_SYZ`: inicializa o fundo a 3/16 da altura da câmera.
pub fn iniciar_fundo(m: &mut Memoria, camera_y: u16) {
    let y = i32::from(i16::from_be_bytes(camera_y.to_be_bytes())).wrapping_mul(48) >> 8;
    ww(
        m,
        V_BGSCREENPOSY,
        u16::try_from(u32::from_be_bytes(y.to_be_bytes()) & 0xFFFF)
            .unwrap_or(0)
            .wrapping_add(1),
    );
    wl(m, V_BGSCREENPOSX, 0);
}

/// `PalCycle_SYZ`: anima as luzes da quarta linha da paleta.
pub fn ciclo_paleta(m: &mut Memoria) {
    let tempo = rw(m, V_PCYC_TIME).wrapping_sub(1);
    ww(m, V_PCYC_TIME, tempo);
    if tempo & 0x8000 == 0 {
        return;
    }
    ww(m, V_PCYC_TIME, 5);
    let ciclo = rw(m, V_PCYC_NUM);
    ww(m, V_PCYC_NUM, ciclo.wrapping_add(1));
    let indice = u32::from(ciclo & 3);
    let base = PAL_PRETO_AMARELO.wrapping_add(indice.wrapping_mul(8));
    let a = rom_l(m, base);
    let b = rom_l(m, base.wrapping_add(4));
    wl(m, 0xFF_FB6E, a);
    wl(m, 0xFF_FB72, b);
    let base = PAL_VERMELHO_BRANCO.wrapping_add(indice.wrapping_mul(4));
    let a = rom_w(m, base);
    let b = rom_w(m, base.wrapping_add(2));
    ww(m, 0xFF_FB76, a);
    ww(m, 0xFF_FB7A, b);
}

/// `DLE_SYZ`: limites do ato 2 e preparação do chefe no ato 3.
/// A interpolação do limite inferior fica a cargo de `DynamicLevelEvents`.
pub fn eventos(m: &mut Memoria) -> Result<(), super::ErroAbertura> {
    let x = rw(m, V_SCREENPOSX);
    match rb(m, V_ACT) {
        0 => {}
        1 => {
            let y = rw(m, V_PLAYER.wrapping_add(OB_Y));
            ww(
                m,
                V_LIMITBTM1,
                if x >= 0x25A0 && y < 0x4D0 {
                    0x420
                } else {
                    0x520
                },
            );
        }
        2 => match rb(m, V_DLE_ROUTINE) {
            0 if x >= 0x2AC0 => {
                if let Some(a) = super::objetos::find_free_obj(m) {
                    wb(m, a.wrapping_add(OB_ID), 0x76);
                    wb(m, V_DLE_ROUTINE, 2);
                }
            }
            2 if x >= 0x2C00 => {
                ww(m, V_LIMITBTM1, 0x4CC);
                if let Some(a) = super::objetos::find_free_obj(m) {
                    wb(m, a.wrapping_add(OB_ID), 0x75);
                    wb(m, V_DLE_ROUTINE, 4);
                }
                wb(m, V_SOUNDQUEUE0, 0x8C);
                wb(m, F_LOCKSCREEN, 1);
                super::plc::add_plc(m, 0x11)?;
            }
            4 => ww(m, V_LIMITLEFT2, x),
            0 | 2 => {}
            _ => {
                return Err(super::ErroAbertura::NaoImplementado(
                    "rotina de eventos da SYZ3",
                ));
            }
        },
        _ => return Err(super::ErroAbertura::NaoImplementado("ato da SYZ")),
    }
    Ok(())
}

fn escrever_faixa(m: &mut Memoria, cursor: &mut u32, valor: u16, quantidade: u32) {
    for _ in 0..quantidade {
        ww(m, *cursor, valor);
        *cursor = cursor.wrapping_add(2);
    }
}

fn gradiente(m: &mut Memoria, cursor: &mut u32, inicial: u16, passo: i32, quantidade: u32) {
    let mut acumulador = u32::from(inicial);
    for _ in 0..quantidade {
        ww(m, *cursor, u16::try_from(acumulador & 0xFFFF).unwrap_or(0));
        *cursor = cursor.wrapping_add(2);
        acumulador = acumulador
            .rotate_left(16)
            .wrapping_add(u32::from_be_bytes(passo.to_be_bytes()))
            .rotate_left(16);
    }
}

/// `Deform_SYZ`: atualiza o deslocamento vertical e monta a tabela de rolagem.
pub fn deformar(m: &mut Memoria) {
    let delta = i32::from(i16::from_be_bytes(rw(m, V_SCRSHIFTY).to_be_bytes())).wrapping_mul(48);
    let anterior = rl(m, V_BGSCREENPOSY);
    let novo = anterior.wrapping_add(u32::from_be_bytes(delta.to_be_bytes()));
    wl(m, V_BGSCREENPOSY, novo);
    let bit = u8::try_from((novo >> 16) & 0x10).unwrap_or(0);
    let bloco = rb(m, BG1_YBLOCK);
    if bit == bloco {
        wb(m, BG1_YBLOCK, bloco ^ 0x10);
        let flags = rb(m, V_BG1_SCROLL_FLAGS) | if delta < 0 { 0x10 } else { 0x20 };
        wb(m, V_BG1_SCROLL_FLAGS, flags);
    }
    let y = rw(m, V_BGSCREENPOSY);
    ww(m, V_BGSCRPOSY_VDP, y);
    let frente = 0u16.wrapping_sub(rw(m, V_SCREENPOSX));
    let frente_s = i16::from_be_bytes(frente.to_be_bytes());
    let metade = frente_s >> 1;
    let oitavo = frente_s >> 3;
    let quarto = frente_s >> 2;
    let passo_nuvem = i32::from(oitavo.wrapping_sub(frente_s)).wrapping_shl(12);
    let passo_arbusto =
        (i32::from(frente_s.wrapping_sub(metade)).wrapping_mul(16) / 14).wrapping_shl(12);
    let mut cursor = V_BG_SCROLL_BUFFER;
    gradiente(
        m,
        &mut cursor,
        u16::from_be_bytes(metade.to_be_bytes()),
        passo_nuvem,
        8,
    );
    escrever_faixa(m, &mut cursor, u16::from_be_bytes(oitavo.to_be_bytes()), 5);
    escrever_faixa(m, &mut cursor, u16::from_be_bytes(quarto.to_be_bytes()), 6);
    gradiente(
        m,
        &mut cursor,
        u16::from_be_bytes(metade.to_be_bytes()),
        passo_arbusto,
        14,
    );

    let mut fonte = V_BG_SCROLL_BUFFER.wrapping_add(u32::from((y & 0x1F0) >> 3));
    let mut destino = V_HSCROLLTABLEBUFFER;
    let mut linhas = 16u32.wrapping_sub(u32::from(y & 15));
    for _ in 0..15 {
        let fundo = rw(m, fonte);
        fonte = fonte.wrapping_add(2);
        for _ in 0..linhas {
            wl(m, destino, (u32::from(frente) << 16) | u32::from(fundo));
            destino = destino.wrapping_add(4);
        }
        linhas = 16;
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn ato_dois_muda_limite_conforme_cena_e_altura_de_sonic() {
        let mut m = Memoria::nova(vec![]);
        wb(&mut m, V_ACT, 1);
        ww(&mut m, V_SCREENPOSX, 0x25A0);
        ww(&mut m, V_PLAYER.wrapping_add(OB_Y), 0x4CF);
        eventos(&mut m).unwrap();
        assert_eq!(rw(&mut m, V_LIMITBTM1), 0x420);
        ww(&mut m, V_PLAYER.wrapping_add(OB_Y), 0x4D0);
        eventos(&mut m).unwrap();
        assert_eq!(rw(&mut m, V_LIMITBTM1), 0x520);
    }

    #[test]
    fn ato_tres_carrega_blocos_chefe_e_trava_limite() {
        let mut m = Memoria::nova(vec![0; 0x20000]);
        wb(&mut m, V_ACT, 2);
        ww(&mut m, V_SCREENPOSX, 0x2AC0);
        eventos(&mut m).unwrap();
        assert_eq!(rb(&mut m, V_DLE_ROUTINE), 2);
        assert_eq!(rb(&mut m, V_LVLOBJSPACE), 0x76);
        ww(&mut m, V_SCREENPOSX, 0x2C00);
        eventos(&mut m).unwrap();
        assert_eq!(rb(&mut m, V_DLE_ROUTINE), 4);
        assert_eq!(rb(&mut m, V_LVLOBJSPACE.wrapping_add(OBJECT_SIZE)), 0x75);
        assert_eq!(rw(&mut m, V_LIMITBTM1), 0x4CC);
        assert_eq!(rb(&mut m, F_LOCKSCREEN), 1);
        eventos(&mut m).unwrap();
        assert_eq!(rw(&mut m, V_LIMITLEFT2), 0x2C00);
    }

    #[test]
    fn fundo_e_rolagem_vertical() {
        let mut m = Memoria::nova(vec![]);
        iniciar_fundo(&mut m, 0x100);
        assert_eq!(rw(&mut m, V_BGSCREENPOSY), 49);
        ww(&mut m, V_SCRSHIFTY, 0x1000);
        deformar(&mut m);
        assert_eq!(rw(&mut m, V_BGSCREENPOSY), 52);
        assert_eq!(rw(&mut m, V_BGSCRPOSY_VDP), 52);
    }
    #[test]
    fn ciclo_usa_dados_da_rom_e_espera_seis_quadros() {
        let mut rom = vec![0; 0x20000];
        rom[0x1C6E..0x1C76].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        rom[0x1C8E..0x1C92].copy_from_slice(&[9, 10, 11, 12]);
        let mut m = Memoria::nova(rom);
        ciclo_paleta(&mut m);
        assert_eq!(rl(&mut m, 0xFF_FB6E), 0x0102_0304);
        assert_eq!(rl(&mut m, 0xFF_FB72), 0x0506_0708);
        assert_eq!(rw(&mut m, 0xFF_FB76), 0x090A);
        assert_eq!(rw(&mut m, 0xFF_FB7A), 0x0B0C);
        assert_eq!(rw(&mut m, V_PCYC_TIME), 5);
    }
    #[test]
    fn tabela_repete_faixas_e_contem_deslocamento_da_frente() {
        let mut m = Memoria::nova(vec![]);
        ww(&mut m, V_SCREENPOSX, 80);
        ww(&mut m, V_BGSCREENPOSY, 0);
        deformar(&mut m);
        assert_eq!(
            rw(&mut m, V_BG_SCROLL_BUFFER),
            u16::from_be_bytes((-40i16).to_be_bytes())
        );
        assert_eq!(
            rw(&mut m, V_BG_SCROLL_BUFFER + 16),
            u16::from_be_bytes((-10i16).to_be_bytes())
        );
        assert_eq!(
            rw(&mut m, V_BG_SCROLL_BUFFER + 26),
            u16::from_be_bytes((-20i16).to_be_bytes())
        );
        assert_eq!(rl(&mut m, V_HSCROLLTABLEBUFFER), 0xFFB0_FFD8);
        assert_eq!(rl(&mut m, V_HSCROLLTABLEBUFFER + 15 * 4), 0xFFB0_FFD8);
    }
}
