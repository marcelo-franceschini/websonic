//! Objeto 3A: Cartoes de fim de ato ("SONIC HAS PASSED ACT X" e contagem de bonus).
//!
//! Exibe os textos de vitoria, soma o bonus de tempo e de aneis a pontuacao
//! com efeitos sonoros, e avanca para o proximo ato/zona pela tabela `LevelOrder`.

use super::boot::{rom_b, rom_w};
use super::objetos::display_sprite;
use super::ram::*;
use super::{ErroAbertura, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;

fn s16(w: u16) -> i16 {
    i16::from_be_bytes(w.to_be_bytes())
}

fn u16_de(i: i16) -> u16 {
    u16::from_be_bytes(i.to_be_bytes())
}

const GOT_ITEM_DATA: u32 = 0xCE04;
const LEVEL_ORDER: u32 = 0xCD82;

/// Executa o objeto `GotThroughCard` (ID 0x3A).
pub fn executar(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    match rotina {
        0 => got_chk_plc(m, a0),
        2 => got_move_in(m, a0),
        4 => got_wait(m, a0),
        6 => got_bonus(m, a0),
        8 => got_wait(m, a0),
        0x0A => got_next_level(m, a0),
        _ => Ok(()),
    }
}

fn got_chk_plc(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    if super::rl(m, V_PLC_BUFFER) != 0 {
        return Ok(());
    }

    // Inicializa os 7 elementos consecutivos na RAM de objetos a partir de a0.
    let ato = rb(m, V_ACT);
    for i in 0..7u32 {
        let entrada = GOT_ITEM_DATA.wrapping_add(i.wrapping_mul(8));
        let start_x = rom_w(m, entrada);
        let main_x = rom_w(m, entrada.wrapping_add(2));
        let screen_y = rom_w(m, entrada.wrapping_add(4));
        let routine = rom_b(m, entrada.wrapping_add(6));
        let frame = rom_b(m, entrada.wrapping_add(7));
        let desloc = i.wrapping_mul(OBJECT_SIZE);
        let a1 = a0.wrapping_add(desloc);
        wb(m, a1.wrapping_add(OB_ID), ID_GOT_THROUGH_CARD);
        ww(m, a1.wrapping_add(OB_X), start_x);
        ww(m, a1.wrapping_add(CARD_FINALX), start_x);
        ww(m, a1.wrapping_add(CARD_MAINX), main_x);
        ww(m, a1.wrapping_add(OB_SCREENY), screen_y);
        wb(m, a1.wrapping_add(OB_ROUTINE), routine);

        let final_frame = if frame == 6 {
            frame.wrapping_add(ato)
        } else {
            frame
        };
        wb(m, a1.wrapping_add(OB_FRAME), final_frame);

        wl(m, a1.wrapping_add(OB_MAP), MAP_GOT);
        ww(m, a1.wrapping_add(OB_GFX), ARTTILE_TITLE_CARD | 0x8000); // Tile_Prio
        wb(m, a1.wrapping_add(OB_RENDER), RENDER_SCREEN);
    }
    Ok(())
}

fn got_move_in(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let target_x = s16(rw(m, a0.wrapping_add(CARD_MAINX)));
    let mut cur_x = s16(rw(m, a0.wrapping_add(OB_X)));

    if cur_x == target_x {
        // Ao chegar, o elemento de Ring Bonus entra imediatamente em Got_Wait.
        if rb(m, a0.wrapping_add(OB_FRAME)) == 4 {
            wb(m, a0.wrapping_add(OB_ROUTINE), 4);
            ww(m, a0.wrapping_add(OB_TIME_FRAME), 180);
            return got_wait(m, a0);
        }
    } else {
        cur_x = if cur_x < target_x {
            cur_x.wrapping_add(16)
        } else {
            cur_x.wrapping_sub(16)
        };
        ww(m, a0.wrapping_add(OB_X), u16_de(cur_x));
    }

    if (0..0x80 + 320 + 64).contains(&cur_x) {
        display_sprite(m, a0);
    }
    Ok(())
}

fn got_wait(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let timer = rw(m, a0.wrapping_add(OB_TIME_FRAME)).wrapping_sub(1);
    ww(m, a0.wrapping_add(OB_TIME_FRAME), timer);
    if timer == 0 {
        let r = rb(m, a0.wrapping_add(OB_ROUTINE)).wrapping_add(2);
        wb(m, a0.wrapping_add(OB_ROUTINE), r);
    }
    display_sprite(m, a0);
    Ok(())
}

fn got_bonus(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    display_sprite(m, a0);
    wb(m, F_ENDACTBONUS, 1);

    let mut d0 = 0u32;
    let time_bonus = rw(m, V_TIMEBONUS);
    if time_bonus > 0 {
        let sub = time_bonus.min(10);
        d0 = d0.wrapping_add(u32::from(sub));
        ww(m, V_TIMEBONUS, time_bonus.wrapping_sub(sub));
    }

    let ring_bonus = rw(m, V_RINGBONUS);
    if ring_bonus > 0 {
        let sub = ring_bonus.min(10);
        d0 = d0.wrapping_add(u32::from(sub));
        ww(m, V_RINGBONUS, ring_bonus.wrapping_sub(sub));
    }

    if d0 != 0 {
        super::fisica::add_points(m, d0);
        let quadro = rb(m, V_VBLANK_BYTE);
        if quadro & 3 == 0 {
            wb(m, V_SOUNDQUEUE1, SFX_SWITCH);
        }
    } else {
        wb(m, V_SOUNDQUEUE1, SFX_CASH);
        wb(m, a0.wrapping_add(OB_ROUTINE), 8);
        ww(m, a0.wrapping_add(OB_TIME_FRAME), 180);
    }
    Ok(())
}

fn got_next_level(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let zona = rb(m, V_ZONE) & 7;
    let ato = rb(m, V_ACT) & 3;
    let idx = u32::from(zona).wrapping_mul(4).wrapping_add(u32::from(ato));
    let prox = rom_w(m, LEVEL_ORDER.wrapping_add(idx.wrapping_mul(2)));
    ww(m, V_ZONE_ACT, prox);

    if prox == 0 {
        wb(m, V_GAMEMODE, GM_TITLE_ID);
    } else {
        wb(m, V_LASTLAMP, 0);
        if rb(m, 0xFF_F7CD) != 0 {
            wb(m, V_GAMEMODE, ID_SPECIAL);
        } else {
            ww(m, F_RESTART, 1);
        }
    }
    display_sprite(m, a0);
    Ok(())
}

#[cfg(test)]
mod testes_got_through {
    use super::super::rl;
    use super::*;

    fn memoria_teste() -> Memoria {
        let mut rom = vec![0; 0x10000];
        rom[0xCD83] = 1;
        rom[0xCD86] = 2;
        rom[0xCE06..0xCE08].copy_from_slice(&0x124u16.to_be_bytes());
        rom[0xCE0A] = 2;
        rom[0xCE1B] = 6;
        rom[0xCE33] = 4;
        Memoria::nova(rom)
    }

    #[test]
    fn got_through_expande_os_sete_elementos_com_dados_corretos() {
        let mut m = memoria_teste();
        let a0 = V_ENDCARD;
        wb(&mut m, V_ACT, 1); // Ato 2
        got_chk_plc(&mut m, a0).unwrap();

        // Elemento 0: SONIC HAS
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ID)), ID_GOT_THROUGH_CARD);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_FRAME)), 0);
        assert_eq!(rw(&mut m, a0.wrapping_add(CARD_MAINX)), 0x124);

        // Elemento 2: ACT 2 (frame 6 + act 1 = 7)
        let a2 = a0.wrapping_add(2 * OBJECT_SIZE);
        assert_eq!(rb(&mut m, a2.wrapping_add(OB_FRAME)), 7);

        // Elemento 5: Ring bonus (frame 4)
        let a5 = a0.wrapping_add(5 * OBJECT_SIZE);
        assert_eq!(rb(&mut m, a5.wrapping_add(OB_FRAME)), 4);
    }

    #[test]
    fn cartao_espera_um_quadro_apos_chegar_ao_alvo() {
        let mut m = memoria_teste();
        let a0 = V_ENDCARD;
        wb(&mut m, a0.wrapping_add(OB_ROUTINE), 2);
        wb(&mut m, a0.wrapping_add(OB_FRAME), 4);
        ww(&mut m, a0.wrapping_add(OB_X), 0x100);
        ww(&mut m, a0.wrapping_add(CARD_MAINX), 0x110);
        got_move_in(&mut m, a0).unwrap();
        assert_eq!(rw(&mut m, a0.wrapping_add(OB_X)), 0x110);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 2);
        got_move_in(&mut m, a0).unwrap();
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 4);
        assert_eq!(rw(&mut m, a0.wrapping_add(OB_TIME_FRAME)), 179);
    }

    #[test]
    fn som_do_bonus_usa_contador_de_vblank() {
        let mut m = memoria_teste();
        ww(&mut m, V_TIMEBONUS, 20);
        wb(&mut m, V_FRAMEBYTE, 0);
        wb(&mut m, V_VBLANK_BYTE, 1);
        got_bonus(&mut m, V_ENDCARD).unwrap();
        assert_eq!(rb(&mut m, V_SOUNDQUEUE1), 0);
        wb(&mut m, V_VBLANK_BYTE, 4);
        got_bonus(&mut m, V_ENDCARD).unwrap();
        assert_eq!(rb(&mut m, V_SOUNDQUEUE1), SFX_SWITCH);
    }

    #[test]
    fn got_bonus_transfere_pontos_e_toca_sons() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        let a0 = V_ENDCARD;
        ww(&mut m, V_TIMEBONUS, 20); // 200 pontos
        ww(&mut m, V_RINGBONUS, 10); // 100 pontos
        wl(&mut m, V_SCORE, 0);

        got_bonus(&mut m, a0).unwrap();
        assert_eq!(rw(&mut m, V_TIMEBONUS), 10);
        assert_eq!(rw(&mut m, V_RINGBONUS), 0);
        assert_eq!(rl(&mut m, V_SCORE), 20);
        assert_eq!(rb(&mut m, F_ENDACTBONUS), 1);

        // Proximo passo esgota bonus e aciona sfx_cash
        got_bonus(&mut m, a0).unwrap();
        assert_eq!(rw(&mut m, V_TIMEBONUS), 0);
        assert_eq!(rw(&mut m, V_RINGBONUS), 0);
        assert_eq!(rl(&mut m, V_SCORE), 30);

        got_bonus(&mut m, a0).unwrap();
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 8);
        assert_eq!(rw(&mut m, a0.wrapping_add(OB_TIME_FRAME)), 180);
        assert_eq!(rb(&mut m, V_SOUNDQUEUE1), SFX_CASH);
    }

    #[test]
    fn got_next_level_avanca_para_proximo_ato_e_reinicia() {
        let mut m = memoria_teste();
        let a0 = V_ENDCARD;
        // Green Hill Zone 1 (zona 0, ato 0)
        wb(&mut m, V_ZONE, 0);
        wb(&mut m, V_ACT, 0);
        wb(&mut m, V_LASTLAMP, 5);

        got_next_level(&mut m, a0).unwrap();
        // Avanca para GHZ 2 (0x0001)
        assert_eq!(rw(&mut m, V_ZONE_ACT), 0x0001);
        assert_eq!(rb(&mut m, V_LASTLAMP), 0);
        assert_eq!(rw(&mut m, F_RESTART), 1);

        // GHZ 3 (zona 0, ato 2)
        wb(&mut m, V_ZONE, 0);
        wb(&mut m, V_ACT, 2);
        got_next_level(&mut m, a0).unwrap();
        // Avanca para Marble Zone 1 (0x0200)
        assert_eq!(rw(&mut m, V_ZONE_ACT), 0x0200);
    }

    #[test]
    fn got_next_level_encaminha_anel_gigante_para_special_stage() {
        let mut m = memoria_teste();
        wb(&mut m, 0xFF_F7CD, 1);
        got_next_level(&mut m, V_ENDCARD).unwrap();
        assert_eq!(rw(&mut m, V_ZONE_ACT), 1);
        assert_eq!(rb(&mut m, V_GAMEMODE), ID_SPECIAL);
        assert_eq!(rw(&mut m, F_RESTART), 0);
    }
}
