//! Cena do Eggman e chao desmoronando em Scrap Brain Zone Ato 2 (objetos 0x82 e 0x83).
use super::ram::*;
use super::{ErroAbertura, fisica, objetos, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

const BOSS_SBZ2_X: u16 = 0x2050;
const BOSS_SBZ2_Y: u16 = 0x0510;

const MAP_SEGG: u32 = 0x1_A1E4;
const ANI_SEGG: u32 = 0x1_A1B2;
const MAP_BUT: u32 = 0x00C52E;
const MAP_FFLOOR: u32 = 0x1_A4C2;

const ARTTILE_EGGMAN: u16 = 0x0400;
const ARTTILE_EGGMAN_BUTTON: u16 = 0x04A4;
const ARTTILE_EGGMAN_TRAP_FLOOR: u16 = 0x0518;

const PARENT_OBJ: u32 = 0x34;
const GENERIC_TIMER: u32 = 0x3C;
const CHILD_CMD: u32 = OB_SUBTYPE;
const BLOCK_LIST_START: u32 = 0x30;

const CMD_SW: u16 = 0x5357; // "SW"
const CMD_GO: u16 = 0x474F; // "GO"

const SFX_WALL_SMASH: u8 = 0xCB;

/// Objeto 0x82: Eggman na cutscene da SBZ2 e o botao no chao.
pub fn scrap_eggman(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    match rotina {
        0 => segg_main(m, a0),
        2 => segg_eggman(m, a0),
        4 => segg_switch(m, a0),
        _ => Ok(()),
    }
}

fn segg_main(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    ww(m, a0.wrapping_add(OB_X), BOSS_SBZ2_X.wrapping_add(0x110));
    ww(m, a0.wrapping_add(OB_Y), BOSS_SBZ2_Y.wrapping_add(0x94));
    wb(m, a0.wrapping_add(OB_COLTYPE), 0x48 | 0x80);
    wb(m, a0.wrapping_add(OB_BOSSHITS), 16);
    let st = rb(m, a0.wrapping_add(OB_STATUS));
    wb(m, a0.wrapping_add(OB_STATUS), st & !STATUS_XFLIP);
    wb(m, a0.wrapping_add(OB_2NDROUT), 0);
    wb(m, a0.wrapping_add(OB_ROUTINE), 2);
    wb(m, a0.wrapping_add(OB_ANIM), 0);
    wb(m, a0.wrapping_add(OB_PRIORITY), 3);
    wl(m, a0.wrapping_add(OB_MAP), MAP_SEGG);
    ww(m, a0.wrapping_add(OB_GFX), ARTTILE_EGGMAN);
    wb(m, a0.wrapping_add(OB_RENDER), 0x84);
    wb(m, a0.wrapping_add(OB_ACTWID), 32);

    if let Some(a1) = objetos::find_free_obj(m) {
        wl(m, a1.wrapping_add(PARENT_OBJ), a0);
        wb(m, a1.wrapping_add(OB_ID), 0x82);
        ww(m, a1.wrapping_add(OB_X), BOSS_SBZ2_X.wrapping_add(0xE0));
        ww(m, a1.wrapping_add(OB_Y), BOSS_SBZ2_Y.wrapping_add(0xAC));
        wb(m, a1.wrapping_add(OB_2NDROUT), 0);
        wb(m, a1.wrapping_add(OB_ROUTINE), 4);
        wb(m, a1.wrapping_add(OB_ANIM), 0);
        wb(m, a1.wrapping_add(OB_PRIORITY), 3);
        wl(m, a1.wrapping_add(OB_MAP), MAP_BUT);
        ww(m, a1.wrapping_add(OB_GFX), ARTTILE_EGGMAN_BUTTON);
        wb(m, a1.wrapping_add(OB_RENDER), 0x84);
        wb(m, a1.wrapping_add(OB_ACTWID), 16);
        wb(m, a1.wrapping_add(OB_FRAME), 0);
    }

    segg_eggman(m, a0)
}

fn segg_eggman(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let r2 = rb(m, a0.wrapping_add(OB_2NDROUT));
    match r2 {
        0 => segg_chk_sonic(m, a0),
        2 => segg_pre_leap(m, a0),
        4 => segg_leap(m, a0),
        _ => segg_move(m, a0),
    }
    objetos::animate_sprite(m, a0, ANI_SEGG)?;
    objetos::display_sprite(m, a0);
    Ok(())
}

fn segg_chk_sonic(m: &mut Memoria, a0: u32) {
    let egg_x = rw(m, a0.wrapping_add(OB_X));
    let player_x = rw(m, V_PLAYER.wrapping_add(OB_X));
    let diff = egg_x.wrapping_sub(player_x);
    if diff < 128 {
        wb(m, a0.wrapping_add(OB_2NDROUT), 2);
        ww(m, a0.wrapping_add(GENERIC_TIMER), 180);
        wb(m, a0.wrapping_add(OB_ANIM), 1);
    }
    segg_move(m, a0);
}

fn segg_pre_leap(m: &mut Memoria, a0: u32) {
    let t = rw(m, a0.wrapping_add(GENERIC_TIMER));
    let novo_t = t.wrapping_sub(1);
    ww(m, a0.wrapping_add(GENERIC_TIMER), novo_t);
    if novo_t == 0 {
        wb(m, a0.wrapping_add(OB_2NDROUT), 4);
        wb(m, a0.wrapping_add(OB_ANIM), 2);
        let y = rw(m, a0.wrapping_add(OB_Y)).wrapping_add(4);
        ww(m, a0.wrapping_add(OB_Y), y);
        ww(m, a0.wrapping_add(GENERIC_TIMER), 15);
    }
    segg_move(m, a0);
}

fn segg_leap(m: &mut Memoria, a0: u32) {
    let t = s16(rw(m, a0.wrapping_add(GENERIC_TIMER))).wrapping_sub(1);
    ww(m, a0.wrapping_add(GENERIC_TIMER), as_u16(t));
    if t == 0 {
        ww(m, a0.wrapping_add(OB_VELX), u16_de(-0xFCi16));
        ww(m, a0.wrapping_add(OB_VELY), u16_de(-0x3C0i16));
    } else if t < 0 {
        let cur_x = rw(m, a0.wrapping_add(OB_X));
        if cur_x <= BOSS_SBZ2_X.wrapping_add(0xE2) {
            ww(m, a0.wrapping_add(OB_VELX), 0);
        }
        let vy = rw(m, a0.wrapping_add(OB_VELY)).wrapping_add(0x24);
        ww(m, a0.wrapping_add(OB_VELY), vy);
        if s16(vy) >= 0 {
            let cur_y = rw(m, a0.wrapping_add(OB_Y));
            if cur_y >= BOSS_SBZ2_Y.wrapping_add(0x85) {
                ww(m, a0.wrapping_add(CHILD_CMD), CMD_SW);
                if cur_y >= BOSS_SBZ2_Y.wrapping_add(0x8B) {
                    ww(m, a0.wrapping_add(OB_Y), BOSS_SBZ2_Y.wrapping_add(0x8B));
                    ww(m, a0.wrapping_add(OB_VELY), 0);
                }
            }
        }
        let vx = rw(m, a0.wrapping_add(OB_VELX));
        let cur_vy = rw(m, a0.wrapping_add(OB_VELY));
        if vx == 0 && cur_vy == 0 {
            // Procura o chao falso (objeto 0x83) para iniciar a quebra
            let mut slot = V_LVLOBJSPACE;
            while slot < V_LVLOBJEND {
                if rb(m, slot.wrapping_add(OB_ID)) == 0x83 {
                    ww(m, slot.wrapping_add(CHILD_CMD), CMD_GO);
                    wb(m, a0.wrapping_add(OB_2NDROUT), 6);
                    wb(m, a0.wrapping_add(OB_ANIM), 1);
                    break;
                }
                slot = slot.wrapping_add(OBJECT_SIZE);
            }
        }
    }
    segg_move(m, a0);
}

fn segg_move(m: &mut Memoria, a0: u32) {
    fisica::speed_to_pos(m, a0);
}

fn segg_switch(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    if rb(m, a0.wrapping_add(OB_2NDROUT)) == 0 {
        let parent = rl(m, a0.wrapping_add(PARENT_OBJ));
        if rw(m, parent.wrapping_add(CHILD_CMD)) == CMD_SW {
            wb(m, a0.wrapping_add(OB_FRAME), 1);
            wb(m, a0.wrapping_add(OB_2NDROUT), 2);
        }
    }
    objetos::display_sprite(m, a0);
    Ok(())
}

/// Objeto 0x83: Blocos do chao que desmoronam quando Eggman aperta o interruptor.
pub fn false_floor(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    match rotina {
        0 => ffloor_main(m, a0),
        2 => ffloor_chk_break(m, a0),
        4 => ffloor_break(m, a0),
        6 => ffloor_all_gone(m, a0),
        8 => ffloor_block(m, a0),
        10 => ffloor_frag(m, a0),
        _ => Ok(()),
    }
}

fn ffloor_main(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    ww(m, a0.wrapping_add(OB_X), BOSS_SBZ2_X.wrapping_add(0x30));
    ww(m, a0.wrapping_add(OB_Y), BOSS_SBZ2_Y.wrapping_add(0xC0));
    wb(m, a0.wrapping_add(OB_ACTWID), 128);
    wb(m, a0.wrapping_add(OB_HEIGHT), 16);
    wb(m, a0.wrapping_add(OB_RENDER), 0x84);

    let mut pos_x = BOSS_SBZ2_X.wrapping_sub(0x40);
    for i in 0..8u32 {
        if let Some(a1) = objetos::find_free_obj(m) {
            ww(
                m,
                a0.wrapping_add(BLOCK_LIST_START)
                    .wrapping_add(i.wrapping_mul(2)),
                a1 as u16,
            );
            wb(m, a1.wrapping_add(OB_ID), 0x83);
            wl(m, a1.wrapping_add(OB_MAP), MAP_FFLOOR);
            ww(
                m,
                a1.wrapping_add(OB_GFX),
                ARTTILE_EGGMAN_TRAP_FLOOR | 0x6000,
            );
            wb(m, a1.wrapping_add(OB_RENDER), 0x84);
            wb(m, a1.wrapping_add(OB_ACTWID), 16);
            wb(m, a1.wrapping_add(OB_HEIGHT), 16);
            wb(m, a1.wrapping_add(OB_PRIORITY), 3);
            ww(m, a1.wrapping_add(OB_X), pos_x);
            ww(m, a1.wrapping_add(OB_Y), BOSS_SBZ2_Y.wrapping_add(0xC0));
            wb(m, a1.wrapping_add(OB_ROUTINE), 8);
        }
        pos_x = pos_x.wrapping_add(0x20);
    }

    wb(m, a0.wrapping_add(OB_ROUTINE), 2);
    Ok(())
}

fn ffloor_chk_break(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    if rw(m, a0.wrapping_add(CHILD_CMD)) == CMD_GO {
        wb(m, a0.wrapping_add(OB_FRAME), 0);
        wb(m, a0.wrapping_add(OB_ROUTINE), 4);
    }
    ffloor_solid(m, a0);
    Ok(())
}

fn ffloor_solid(m: &mut Memoria, a0: u32) {
    let quebrados = u16::from(rb(m, a0.wrapping_add(OB_FRAME)));
    let restantes = 8u16.saturating_sub(quebrados);
    let largura = restantes.wrapping_shl(4); // restantes * 16
    let d4 = BOSS_SBZ2_X.wrapping_add(0xB0).wrapping_sub(largura);
    wb(m, a0.wrapping_add(OB_ACTWID), largura as u8);
    ww(m, a0.wrapping_add(OB_X), d4);
    let d1 = largura.wrapping_add(SONIC_SOLID_WIDTH);
    let _ = objetos::solid_object(m, a0, d1, 16, 17);
}

fn ffloor_break(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let tf = rb(m, a0.wrapping_add(OB_TIMEFRAME));
    let (novo_tf, estouro) = tf.overflowing_sub(0x0E);
    wb(m, a0.wrapping_add(OB_TIMEFRAME), novo_tf);
    if estouro {
        let count = rb(m, a0.wrapping_add(OB_FRAME));
        let offset = u32::from(count).wrapping_mul(2);
        let block_ptr = ext(rw(
            m,
            a0.wrapping_add(BLOCK_LIST_START).wrapping_add(offset),
        ));
        ww(m, block_ptr.wrapping_add(CHILD_CMD), CMD_GO);
        let novo_count = count.wrapping_add(1);
        wb(m, a0.wrapping_add(OB_FRAME), novo_count);
        if novo_count >= 8 {
            return ffloor_all_gone(m, a0);
        }
    }
    ffloor_solid(m, a0);
    Ok(())
}

fn ffloor_all_gone(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let st = rb(m, a0.wrapping_add(OB_STATUS));
    wb(m, a0.wrapping_add(OB_STATUS), st & !STATUS_ON_OBJECT);
    let pst = rb(m, V_PLAYER.wrapping_add(OB_STATUS));
    wb(m, V_PLAYER.wrapping_add(OB_STATUS), pst & !STATUS_ON_OBJECT);
    objetos::delete_object(m, a0);
    Ok(())
}

fn ffloor_block(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    if rw(m, a0.wrapping_add(CHILD_CMD)) == CMD_GO {
        wb(m, a0.wrapping_add(OB_ROUTINE), 10);
        wb(m, a0.wrapping_add(OB_ACTWID), 8);
        wb(m, a0.wrapping_add(OB_HEIGHT), 8);

        const FRAG_SPEED: [u16; 4] = [0x0080, 0x0000, 0x0120, 0x00C0];
        const FRAG_POS_X: [i16; 4] = [-8, 0x10, 0, 0x10];
        const FRAG_POS_Y: [i16; 4] = [-8, 0, 0x10, 0x10];

        let base_x = rw(m, a0.wrapping_add(OB_X));
        let base_y = rw(m, a0.wrapping_add(OB_Y));

        // Fragmento 0 fica no proprio a0
        ww(m, a0.wrapping_add(OB_VELY), FRAG_SPEED[0]);
        ww(
            m,
            a0.wrapping_add(OB_X),
            base_x.wrapping_add(FRAG_POS_X[0] as u16),
        );
        ww(
            m,
            a0.wrapping_add(OB_Y),
            base_y.wrapping_add(FRAG_POS_Y[0] as u16),
        );
        wb(m, a0.wrapping_add(OB_FRAME), 1);

        // Fragmentos 1, 2, 3 em novos slots
        for i in 1..4 {
            if let Some(a1) = objetos::find_free_obj(m) {
                for off in 0..OBJECT_SIZE {
                    let val = rb(m, a0.wrapping_add(off));
                    wb(m, a1.wrapping_add(off), val);
                }
                ww(m, a1.wrapping_add(OB_VELY), FRAG_SPEED[i]);
                ww(
                    m,
                    a1.wrapping_add(OB_X),
                    base_x.wrapping_add(FRAG_POS_X[i] as u16),
                );
                ww(
                    m,
                    a1.wrapping_add(OB_Y),
                    base_y.wrapping_add(FRAG_POS_Y[i] as u16),
                );
                wb(m, a1.wrapping_add(OB_FRAME), (i + 1) as u8);
                wb(m, a1.wrapping_add(OB_ROUTINE), 10);
            }
        }
        wb(m, V_SOUNDQUEUE1, SFX_WALL_SMASH);
    }
    objetos::display_sprite(m, a0);
    Ok(())
}

fn ffloor_frag(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    if rb(m, a0.wrapping_add(OB_RENDER)) & 0x80 == 0 {
        objetos::delete_object(m, a0);
        return Ok(());
    }
    fisica::object_fall(m, a0);
    objetos::display_sprite(m, a0);
    Ok(())
}

fn s16(w: u16) -> i16 {
    i16::from_be_bytes(w.to_be_bytes())
}

fn as_u16(i: i16) -> u16 {
    u16::from_be_bytes(i.to_be_bytes())
}

fn u16_de(i: i16) -> u16 {
    u16::from_be_bytes(i.to_be_bytes())
}

fn ext(w: u16) -> u32 {
    0xFF_0000 | u32::from(w)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn inicializacao_sbz2_cutscene() {
        let mut m = Memoria::nova(vec![0; 0x1000]);
        let a0: u32 = 0xFF_D800;
        wb(&mut m, a0.wrapping_add(OB_ID), 0x82);
        wb(&mut m, a0.wrapping_add(OB_ROUTINE), 0);

        let res = scrap_eggman(&mut m, a0);
        assert!(res.is_ok());
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_BOSSHITS)), 16);
        assert_eq!(
            rw(&mut m, a0.wrapping_add(OB_X)),
            BOSS_SBZ2_X.wrapping_add(0x110)
        );
        assert_eq!(
            rw(&mut m, a0.wrapping_add(OB_Y)),
            BOSS_SBZ2_Y.wrapping_add(0x94)
        );

        // Botao criado no slot livre
        let a1 = a0.wrapping_add(OBJECT_SIZE);
        assert_eq!(rb(&mut m, a1.wrapping_add(OB_ID)), 0x82);
        assert_eq!(rb(&mut m, a1.wrapping_add(OB_ROUTINE)), 4);
    }
}
