//! Objeto 36: Espinhos (`36 Spikes.asm`).

use super::boot::rom_b;
use super::objetos::{delete_object, display_sprite, fora_da_janela, solid_object};
use super::ram::*;
use super::{ErroAbertura, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

pub const MAP_SPIKE: u32 = 0x00D676;
pub const ARTTILE_SPIKES: u16 = 0x051B;

const SPIKES_CONFIG: u32 = 0xD4C8;

fn ext_l(w: u16) -> u32 {
    u32::from_be_bytes(i32::from(i16::from_be_bytes(w.to_be_bytes())).to_be_bytes())
}

/// Executa o objeto `Spikes` (`0x36`).
pub fn executar(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    if rb(m, a0.wrapping_add(OB_ROUTINE)) == 0 {
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        wl(m, a0.wrapping_add(OB_MAP), MAP_SPIKE);
        ww(m, a0.wrapping_add(OB_GFX), ARTTILE_SPIKES);
        let render = rb(m, a0.wrapping_add(OB_RENDER)) | RENDER_CAM_FIELD;
        wb(m, a0.wrapping_add(OB_RENDER), render);
        wb(m, a0.wrapping_add(OB_PRIORITY), 4);

        let subtype = rb(m, a0.wrapping_add(OB_SUBTYPE));
        wb(m, a0.wrapping_add(OB_SUBTYPE), subtype & 0x0F);
        let idx = u32::from((subtype >> 4) & 0x0F).wrapping_mul(2);
        let frame = rom_b(m, SPIKES_CONFIG.wrapping_add(idx));
        let actwid = rom_b(m, SPIKES_CONFIG.wrapping_add(idx).wrapping_add(1));
        wb(m, a0.wrapping_add(OB_FRAME), frame);
        wb(m, a0.wrapping_add(OB_ACTWID), actwid);

        let x = rw(m, a0.wrapping_add(OB_X));
        let y = rw(m, a0.wrapping_add(OB_Y));
        ww(m, a0.wrapping_add(0x30), x); // spikes_origX
        ww(m, a0.wrapping_add(0x32), y); // spikes_origY
    }

    spikes_move(m, a0);

    let frame = rb(m, a0.wrapping_add(OB_FRAME));
    let (d1, d2, d3) = if frame == 1 || frame == 5 {
        let d2 = if frame == 5 { 4u16 } else { 20u16 };
        (26u16, d2, d2.wrapping_add(1))
    } else {
        let actwid = u16::from(rb(m, a0.wrapping_add(OB_ACTWID)));
        (actwid.wrapping_add(10), 16u16, 17u16)
    };

    let d4 = solid_object(m, a0, d1, d2, d3)?;

    let hurt = if frame == 1 || frame == 5 {
        if rb(m, a0.wrapping_add(OB_STATUS)) & STATUS_ON_OBJECT != 0 {
            false
        } else {
            d4 == 1
        }
    } else if rb(m, a0.wrapping_add(OB_STATUS)) & STATUS_ON_OBJECT != 0 {
        true
    } else {
        d4 < 0
    };

    if hurt {
        spikes_hurt(m, a0)?;
    }

    display_sprite(m, a0);
    let orig_x = rw(m, a0.wrapping_add(0x30));
    if fora_da_janela(m, orig_x) {
        delete_object(m, a0);
    }

    Ok(())
}

fn spikes_move(m: &mut Memoria, a0: u32) {
    let sub = rb(m, a0.wrapping_add(OB_SUBTYPE)) & 0x0F;
    match sub {
        1 => {
            spikes_wait_and_move(m, a0);
            let pos_high = u16::from(rb(m, a0.wrapping_add(0x34)));
            let orig_y = rw(m, a0.wrapping_add(0x32));
            ww(m, a0.wrapping_add(OB_Y), orig_y.wrapping_add(pos_high));
        }
        2 => {
            spikes_wait_and_move(m, a0);
            let pos_high = u16::from(rb(m, a0.wrapping_add(0x34)));
            let orig_x = rw(m, a0.wrapping_add(0x30));
            ww(m, a0.wrapping_add(OB_X), orig_x.wrapping_add(pos_high));
        }
        _ => {}
    }
}

fn spikes_wait_and_move(m: &mut Memoria, a0: u32) {
    let delay = rw(m, a0.wrapping_add(0x38));
    if delay != 0 {
        let n_delay = delay.wrapping_sub(1);
        ww(m, a0.wrapping_add(0x38), n_delay);
        if n_delay == 0 && rb(m, a0.wrapping_add(OB_RENDER)) & 0x80 != 0 {
            wb(m, V_SOUNDQUEUE1, 0xB6); // sfx_SpikesMove
        }
        return;
    }

    let dir = rw(m, a0.wrapping_add(0x36));
    if dir == 0 {
        let pos = rw(m, a0.wrapping_add(0x34)).wrapping_add(0x800);
        ww(m, a0.wrapping_add(0x34), pos);
        if pos >= 0x2000 {
            ww(m, a0.wrapping_add(0x34), 0x2000);
            ww(m, a0.wrapping_add(0x36), 1);
            ww(m, a0.wrapping_add(0x38), 60);
        }
    } else {
        let (pos, borrow) = rw(m, a0.wrapping_add(0x34)).overflowing_sub(0x800);
        if borrow {
            ww(m, a0.wrapping_add(0x34), 0);
            ww(m, a0.wrapping_add(0x36), 0);
            ww(m, a0.wrapping_add(0x38), 60);
        } else {
            ww(m, a0.wrapping_add(0x34), pos);
        }
    }
}

fn spikes_hurt(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    if rb(m, V_INVINC) != 0 {
        return Ok(());
    }
    if rb(m, V_PLAYER.wrapping_add(OB_ROUTINE)) >= 4 {
        return Ok(());
    }
    let y = rl(m, V_PLAYER.wrapping_add(OB_Y));
    let vy = ext_l(rw(m, V_PLAYER.wrapping_add(OB_VELY))).wrapping_shl(8);
    wl(m, V_PLAYER.wrapping_add(OB_Y), y.wrapping_sub(vy));
    super::fisica::hurt_sonic(m, a0)
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::abertura::ram::{V_LVLOBJSPACE, V_SCREENPOSX};

    fn memoria() -> Memoria {
        let mut rom = vec![0; 0x10000];
        rom[0xD4C9] = 20;
        Memoria::nova(rom)
    }

    #[test]
    fn spikes_inicializa_e_configura_frame_e_largura() {
        let mut m = memoria();
        let a = V_LVLOBJSPACE;
        wb(&mut m, a + OB_ID, 0x36);
        wb(&mut m, a + OB_SUBTYPE, 0x00); // 3 espinhos verticais estaticos
        ww(&mut m, a + OB_X, 300);
        ww(&mut m, a + OB_Y, 200);
        ww(&mut m, V_SCREENPOSX, 300);

        executar(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a + OB_ROUTINE), 2);
        assert_eq!(rl(&mut m, a + OB_MAP), MAP_SPIKE);
        assert_eq!(rw(&mut m, a + OB_GFX), ARTTILE_SPIKES);
        assert_eq!(rb(&mut m, a + OB_FRAME), 0);
        assert_eq!(rb(&mut m, a + OB_ACTWID), 20);
        assert_eq!(rw(&mut m, a + 0x30), 300);
        assert_eq!(rw(&mut m, a + 0x32), 200);
    }
}
