//! GHZ Bridge (`0x11`): ponte de troncos que verga com o peso do Sonic (`11 GHZ Bridge.asm`).
use super::boot::rom_b;
use super::objetos::{delete_object, display_sprite, fora_da_janela};
use super::ram::*;
use super::{ErroAbertura, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;

pub const MAP_BRI: u32 = 0x7FB2;
pub const ART_TILE_BRIDGE: u16 = 0x438E;
pub const BRI_CHILDREN: u32 = 0x28; // obSubtype
pub const BRI_CHILDREN_RAM: u32 = 0x29;
pub const BRI_ORIG_Y: u32 = 0x3C;
pub const BRI_NUDGE: u32 = 0x3E;
pub const BRI_CURRENTLOG: u32 = 0x3F;
pub const BRI_DATA_Y_MAX: u32 = 0x7D4A;
pub const BRI_DATA_ALIGN: u32 = 0x7E5A;

fn u16_de(i: i16) -> u16 {
    u16::from_be_bytes(i.to_be_bytes())
}

fn s16(w: u16) -> i16 {
    i16::from_be_bytes(w.to_be_bytes())
}

fn s8(b: u8) -> i8 {
    i8::from_be_bytes([b])
}

pub fn executar(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    match rb(m, a0.wrapping_add(OB_ROUTINE)) {
        0 => {
            bri_main(m, a0);
            bri_action(m, a0);
            Ok(())
        }
        2 => {
            bri_action(m, a0);
            Ok(())
        }
        4 => {
            bri_stood_on(m, a0);
            Ok(())
        }
        6 | 8 => {
            delete_object(m, a0);
            Ok(())
        }
        0x0A => {
            display_sprite(m, a0);
            Ok(())
        }
        _ => Err(ErroAbertura::NaoImplementado("rotina da Bridge")),
    }
}

fn bri_main(m: &mut Memoria, a0: u32) {
    wb(m, a0.wrapping_add(OB_ROUTINE), 2);
    wl(m, a0.wrapping_add(OB_MAP), MAP_BRI);
    ww(m, a0.wrapping_add(OB_GFX), ART_TILE_BRIDGE);
    wb(m, a0.wrapping_add(OB_RENDER), 4);
    wb(m, a0.wrapping_add(OB_PRIORITY), 3);
    wb(m, a0.wrapping_add(OB_ACTWID), 0x80);

    let d2_y = rw(m, a0.wrapping_add(OB_Y));
    let center_x = rw(m, a0.wrapping_add(OB_X));
    let id = rb(m, a0.wrapping_add(OB_ID));

    let subtype = rb(m, a0.wrapping_add(BRI_CHILDREN));
    wb(m, a0.wrapping_add(BRI_CHILDREN), 0);

    let d0_half = u16::from(subtype / 2).wrapping_mul(16);
    let mut d3_x = center_x.wrapping_sub(d0_half);

    if subtype <= 1 {
        return;
    }

    let total_to_create = subtype.saturating_sub(1);
    let mut children_count = 0u8;
    let mut a2_offset = BRI_CHILDREN_RAM;

    for _ in 0..total_to_create {
        let child_slot = find_free_obj(m);
        if child_slot.is_none() {
            break;
        }
        let a1 = child_slot.unwrap();

        if d3_x == center_x {
            d3_x = d3_x.wrapping_add(16);
            ww(m, a0.wrapping_add(OB_Y), d2_y);
            ww(m, a0.wrapping_add(BRI_ORIG_Y), d2_y);
            let parent_idx = u8::try_from((a0.wrapping_sub(V_OBJSPACE)) / OBJECT_SIZE).unwrap_or(0);
            wb(m, a0.wrapping_add(a2_offset), parent_idx);
            a2_offset = a2_offset.wrapping_add(1);
            children_count = children_count.wrapping_add(1);
        }

        let child_idx = u8::try_from((a1.wrapping_sub(V_OBJSPACE)) / OBJECT_SIZE).unwrap_or(0);
        wb(m, a0.wrapping_add(a2_offset), child_idx);
        a2_offset = a2_offset.wrapping_add(1);
        children_count = children_count.wrapping_add(1);

        wb(m, a1.wrapping_add(OB_ROUTINE), 0x0A);
        wb(m, a1.wrapping_add(OB_ID), id);
        ww(m, a1.wrapping_add(OB_Y), d2_y);
        ww(m, a1.wrapping_add(BRI_ORIG_Y), d2_y);
        ww(m, a1.wrapping_add(OB_X), d3_x);
        wl(m, a1.wrapping_add(OB_MAP), MAP_BRI);
        ww(m, a1.wrapping_add(OB_GFX), ART_TILE_BRIDGE);
        wb(m, a1.wrapping_add(OB_RENDER), 4);
        wb(m, a1.wrapping_add(OB_PRIORITY), 3);
        wb(m, a1.wrapping_add(OB_ACTWID), 8);

        d3_x = d3_x.wrapping_add(16);
    }

    wb(m, a0.wrapping_add(BRI_CHILDREN), children_count);
}

fn bri_action(m: &mut Memoria, a0: u32) {
    bri_check_on_bridge(m, a0);

    let nudge = rb(m, a0.wrapping_add(BRI_NUDGE));
    if nudge != 0 {
        let novo_nudge = nudge.saturating_sub(4);
        wb(m, a0.wrapping_add(BRI_NUDGE), novo_nudge);
        bri_bend(m, a0);
    }

    display_sprite(m, a0);
    bri_chk_del(m, a0);
}

fn bri_stood_on(m: &mut Memoria, a0: u32) {
    bri_walk_off(m, a0);
    display_sprite(m, a0);
    bri_chk_del(m, a0);
}

fn bri_check_on_bridge(m: &mut Memoria, a0: u32) {
    let num_logs = rb(m, a0.wrapping_add(BRI_CHILDREN));
    let d1 = (u16::from(num_logs).wrapping_mul(8)).wrapping_add(8);
    let d2 = u16::from(num_logs).wrapping_mul(16);

    let vy = s16(rw(m, V_PLAYER.wrapping_add(OB_VELY)));
    if vy < 0 {
        return;
    }

    let sonic_x = rw(m, V_PLAYER.wrapping_add(OB_X));
    let bridge_x = rw(m, a0.wrapping_add(OB_X));
    let d0 = sonic_x.wrapping_sub(bridge_x).wrapping_add(d1);
    if s16(d0) < 0 || d0 >= d2 {
        return;
    }

    let platform_top = rw(m, a0.wrapping_add(OB_Y)).wrapping_sub(8);
    let sonic_y = rw(m, V_PLAYER.wrapping_add(OB_Y));
    let sonic_height = u16::from(rb(m, V_PLAYER.wrapping_add(OB_HEIGHT)));
    let sonic_bottom = sonic_y.wrapping_add(sonic_height).wrapping_add(4);

    let d0_y = platform_top.wrapping_sub(sonic_bottom);
    let d0_signed = s16(d0_y);
    if !(-16..=0).contains(&d0_signed) {
        return;
    }

    if s8(rb(m, F_PLAYERCTRL)) < 0 || rb(m, V_PLAYER.wrapping_add(OB_ROUTINE)) >= 6 {
        return;
    }

    let new_sonic_y = sonic_y.wrapping_add(d0_y).wrapping_add(3);
    ww(m, V_PLAYER.wrapping_add(OB_Y), new_sonic_y);
    let rot = rb(m, a0.wrapping_add(OB_ROUTINE)).wrapping_add(2);
    wb(m, a0.wrapping_add(OB_ROUTINE), rot);

    let player_status = rb(m, V_PLAYER.wrapping_add(OB_STATUS));
    if player_status & STATUS_ON_OBJECT != 0 {
        let old_idx = rb(m, V_PLAYER.wrapping_add(SON_STANDONOBJECT));
        let old_addr = V_OBJSPACE.wrapping_add(u32::from(old_idx).wrapping_mul(OBJECT_SIZE));
        let old_status = rb(m, old_addr.wrapping_add(OB_STATUS)) & !STATUS_ON_OBJECT;
        wb(m, old_addr.wrapping_add(OB_STATUS), old_status);
        wb(m, old_addr.wrapping_add(OB_ROUTINE_2), 0);
        if rb(m, old_addr.wrapping_add(OB_ROUTINE)) == 4 {
            wb(m, old_addr.wrapping_add(OB_ROUTINE), 2);
        }
    }

    let bridge_idx = u8::try_from((a0.wrapping_sub(V_OBJSPACE)) / OBJECT_SIZE).unwrap_or(0);
    wb(m, V_PLAYER.wrapping_add(SON_STANDONOBJECT), bridge_idx);
    wb(m, V_PLAYER.wrapping_add(OB_ANGLE), 0);
    ww(m, V_PLAYER.wrapping_add(OB_VELY), 0);
    let vx = rw(m, V_PLAYER.wrapping_add(OB_VELX));
    ww(m, V_PLAYER.wrapping_add(OB_INERTIA), vx);

    if player_status & STATUS_IN_AIR != 0 {
        super::fisica::sonic_reset_on_floor(m, V_PLAYER);
    }

    let ps = rb(m, V_PLAYER.wrapping_add(OB_STATUS)) | STATUS_ON_OBJECT;
    wb(m, V_PLAYER.wrapping_add(OB_STATUS), ps);
    let bs = rb(m, a0.wrapping_add(OB_STATUS)) | STATUS_ON_OBJECT;
    wb(m, a0.wrapping_add(OB_STATUS), bs);
}

fn bri_walk_off(m: &mut Memoria, a0: u32) {
    let num_logs = rb(m, a0.wrapping_add(BRI_CHILDREN));
    let d1 = (u16::from(num_logs).wrapping_mul(8)).wrapping_add(8);
    let d2 = u16::from(num_logs).wrapping_mul(16);

    let player_status = rb(m, V_PLAYER.wrapping_add(OB_STATUS));
    let sonic_x = rw(m, V_PLAYER.wrapping_add(OB_X));
    let bridge_x = rw(m, a0.wrapping_add(OB_X));
    let d0 = sonic_x.wrapping_sub(bridge_x).wrapping_add(d1);

    if player_status & STATUS_IN_AIR != 0 || s16(d0) < 0 || d0 >= d2 {
        let ps = rb(m, V_PLAYER.wrapping_add(OB_STATUS)) & !STATUS_ON_OBJECT;
        wb(m, V_PLAYER.wrapping_add(OB_STATUS), ps);
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        let bs = rb(m, a0.wrapping_add(OB_STATUS)) & !STATUS_ON_OBJECT;
        wb(m, a0.wrapping_add(OB_STATUS), bs);
        return;
    }

    let log_idx = u8::try_from((d0 >> 4) & 0x7F).unwrap_or(0);
    let max_log = if num_logs > 0 {
        num_logs.wrapping_sub(1)
    } else {
        0
    };
    let clamped_log = if log_idx < num_logs { log_idx } else { max_log };
    wb(m, a0.wrapping_add(BRI_CURRENTLOG), clamped_log);

    let nudge = rb(m, a0.wrapping_add(BRI_NUDGE));
    if nudge < 0x40 {
        wb(m, a0.wrapping_add(BRI_NUDGE), nudge.wrapping_add(4));
    }

    bri_bend(m, a0);
    bri_move_sonic(m, a0);
}

fn bri_move_sonic(m: &mut Memoria, a0: u32) {
    let cur_log = rb(m, a0.wrapping_add(BRI_CURRENTLOG));
    let child_idx = rb(
        m,
        a0.wrapping_add(BRI_CHILDREN_RAM)
            .wrapping_add(u32::from(cur_log)),
    );
    let log_addr = V_OBJSPACE.wrapping_add(u32::from(child_idx).wrapping_mul(OBJECT_SIZE));

    let log_y = rw(m, log_addr.wrapping_add(OB_Y));
    let sonic_h = u16::from(rb(m, V_PLAYER.wrapping_add(OB_HEIGHT)));
    let new_y = log_y.wrapping_sub(8).wrapping_sub(sonic_h);
    ww(m, V_PLAYER.wrapping_add(OB_Y), new_y);
}

fn bri_bend(m: &mut Memoria, a0: u32) {
    let nudge = rb(m, a0.wrapping_add(BRI_NUDGE));
    let (seno, _) = super::fisica::calc_sine(m, nudge);
    let d4 = u32::from(u16_de(seno));

    let num_logs = rb(m, a0.wrapping_add(BRI_CHILDREN));
    let cur_log = rb(m, a0.wrapping_add(BRI_CURRENTLOG));
    let d0_idx = u32::from(num_logs).wrapping_mul(16);
    let d3_idx = u32::from(cur_log);
    let y_max_idx = BRI_DATA_Y_MAX.wrapping_add(d0_idx).wrapping_add(d3_idx);
    let d5 = u32::from(rom_b(m, y_max_idx));

    let align_row = (u32::from(cur_log) & 0xF).wrapping_mul(16);
    let mut align_ptr = BRI_DATA_ALIGN.wrapping_add(align_row);
    let num_left = cur_log;

    let mut children_ptr = a0.wrapping_add(BRI_CHILDREN_RAM);
    for _ in 0..=num_left {
        let child_idx = rb(m, children_ptr);
        children_ptr = children_ptr.wrapping_add(1);
        let child_addr = V_OBJSPACE.wrapping_add(u32::from(child_idx).wrapping_mul(OBJECT_SIZE));

        let align_val = u32::from(rom_b(m, align_ptr)).wrapping_add(1);
        align_ptr = align_ptr.wrapping_add(1);

        let prod = align_val.wrapping_mul(d5).wrapping_mul(d4);
        let bend = u16_de(i16::from_be_bytes(
            u16::try_from(prod >> 16).unwrap_or(0).to_be_bytes(),
        ));
        let orig_y = rw(m, child_addr.wrapping_add(BRI_ORIG_Y));
        ww(m, child_addr.wrapping_add(OB_Y), orig_y.wrapping_add(bend));
    }

    let d3_right = cur_log.wrapping_add(1);
    if d3_right < num_logs {
        let d2_count = num_logs.wrapping_sub(d3_right);
        let align_right_row = u32::from(d2_count).wrapping_mul(16);
        let mut right_align_ptr = BRI_DATA_ALIGN
            .wrapping_add(align_right_row)
            .wrapping_add(u32::from(d2_count));

        for _ in 0..d2_count {
            let child_idx = rb(m, children_ptr);
            children_ptr = children_ptr.wrapping_add(1);
            let child_addr =
                V_OBJSPACE.wrapping_add(u32::from(child_idx).wrapping_mul(OBJECT_SIZE));

            right_align_ptr = right_align_ptr.wrapping_sub(1);
            let align_val = u32::from(rom_b(m, right_align_ptr)).wrapping_add(1);

            let prod = align_val.wrapping_mul(d5).wrapping_mul(d4);
            let bend = u16_de(i16::from_be_bytes(
                u16::try_from(prod >> 16).unwrap_or(0).to_be_bytes(),
            ));
            let orig_y = rw(m, child_addr.wrapping_add(BRI_ORIG_Y));
            ww(m, child_addr.wrapping_add(OB_Y), orig_y.wrapping_add(bend));
        }
    }
}

fn bri_chk_del(m: &mut Memoria, a0: u32) {
    let x = rw(m, a0.wrapping_add(OB_X));
    if fora_da_janela(m, x) {
        let count = rb(m, a0.wrapping_add(BRI_CHILDREN));
        for i in 0..count {
            let child_idx = rb(
                m,
                a0.wrapping_add(BRI_CHILDREN_RAM).wrapping_add(u32::from(i)),
            );
            let child_addr =
                V_OBJSPACE.wrapping_add(u32::from(child_idx).wrapping_mul(OBJECT_SIZE));
            if child_addr != a0 {
                delete_object(m, child_addr);
            }
        }
        delete_object(m, a0);
    }
}

fn find_free_obj(m: &mut Memoria) -> Option<u32> {
    let mut a1 = V_LVLOBJSPACE;
    while a1 < V_LVLOBJEND {
        if rb(m, a1.wrapping_add(OB_ID)) == 0 {
            return Some(a1);
        }
        a1 = a1.wrapping_add(OBJECT_SIZE);
    }
    None
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::abertura::rl;

    fn memoria_teste() -> Memoria {
        let rom = vec![0u8; 0x10000];
        Memoria::nova(rom)
    }

    #[test]
    fn bridge_inicializa_e_cria_troncos() {
        let mut m = memoria_teste();
        let a0 = V_LVLOBJSPACE;
        wb(&mut m, a0.wrapping_add(OB_ID), ID_BRIDGE);
        wb(&mut m, a0.wrapping_add(BRI_CHILDREN), 8); // 8 troncos
        ww(&mut m, a0.wrapping_add(OB_X), 0x200);
        ww(&mut m, a0.wrapping_add(OB_Y), 0x180);
        ww(&mut m, V_SCREENPOSX, 0x180);

        executar(&mut m, a0).expect("ponte executa");

        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rl(&mut m, a0.wrapping_add(OB_MAP)), MAP_BRI);
        assert_eq!(rw(&mut m, a0.wrapping_add(OB_GFX)), ART_TILE_BRIDGE);
        assert_eq!(rb(&mut m, a0.wrapping_add(BRI_CHILDREN)), 8);
    }

    #[test]
    fn bridge_fora_da_janela_e_removida() {
        let mut m = memoria_teste();
        let a0 = V_LVLOBJSPACE;
        wb(&mut m, a0.wrapping_add(OB_ID), ID_BRIDGE);
        wb(&mut m, a0.wrapping_add(BRI_CHILDREN), 4);
        ww(&mut m, a0.wrapping_add(OB_X), 0x1000); // Longe da tela
        ww(&mut m, a0.wrapping_add(OB_Y), 0x180);
        ww(&mut m, V_SCREENPOSX, 0x180);

        executar(&mut m, a0).expect("ponte executa");

        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ID)), 0);
    }
}
