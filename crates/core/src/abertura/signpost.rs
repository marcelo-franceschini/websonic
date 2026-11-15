//! Objeto 0D: Placa de fim de ato (Signpost).
//!
//! Gira ao toque do Sonic, solta faiscas (objetos de anel) e trava a camera.
//! Ao termino do giro, faz o Sonic correr para a direita e aciona os cartoes
//! de passagem de ato (Objeto 3A, GotThroughCard).

use super::boot::{rom_b, rom_w};
use super::objetos::{
    animate_sprite, delete_object, display_sprite, find_free_obj, fora_da_janela,
};
use super::ram::*;
use super::{ErroAbertura, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;

fn s16(w: u16) -> i16 {
    i16::from_be_bytes(w.to_be_bytes())
}

fn u16_de(i: i16) -> u16 {
    u16::from_be_bytes(i.to_be_bytes())
}

const SIGN_SPARK_POS: u32 = 0xF2D0;
const TIME_BONUSES: u32 = 0xF37C;

/// Executa o objeto `Signpost` (ID 0x0D).
pub fn executar(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    match rotina {
        0 => sign_main(m, a0),
        2 => sign_touch(m, a0),
        4 => sign_spin(m, a0),
        6 => sign_sonic_run(m, a0),
        8 => sign_exit(m, a0),
        _ => Ok(()),
    }?;
    animate_sprite(m, a0, ANI_SIGN)?;
    display_sprite(m, a0);
    let x_pos = rw(m, a0.wrapping_add(OB_X));
    if fora_da_janela(m, x_pos) {
        delete_object(m, a0);
    }
    Ok(())
}

fn sign_main(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    wb(m, a0.wrapping_add(OB_ROUTINE), 2);
    wl(m, a0.wrapping_add(OB_MAP), MAP_SIGN);
    ww(m, a0.wrapping_add(OB_GFX), ARTTILE_SIGNPOST);
    wb(m, a0.wrapping_add(OB_RENDER), RENDER_CAM_FIELD);
    wb(m, a0.wrapping_add(OB_ACTWID), 24); // 48 / 2
    wb(m, a0.wrapping_add(OB_PRIORITY), 4);
    // Sign_Main continua diretamente em Sign_Touch no mesmo quadro.
    sign_touch(m, a0)
}

fn sign_touch(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let sonic_x = rw(m, V_PLAYER.wrapping_add(OB_X));
    let sign_x = rw(m, a0.wrapping_add(OB_X));
    if sonic_x < sign_x || sonic_x.wrapping_sub(sign_x) >= 32 {
        return Ok(());
    }

    // Tocou na placa
    wb(m, V_SOUNDQUEUE0, SFX_SIGNPOST);
    wb(m, F_TIMECOUNT, 0);
    let limite_dir = rw(m, V_LIMITRIGHT2);
    ww(m, V_LIMITLEFT2, limite_dir);
    wb(m, a0.wrapping_add(OB_ROUTINE), 4);
    Ok(())
}

fn sign_spin(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let spintime = rw(m, a0.wrapping_add(OB_OFF30));
    let novo_spintime = spintime.wrapping_sub(1);
    ww(m, a0.wrapping_add(OB_OFF30), novo_spintime);
    if s16(novo_spintime) < 0 {
        ww(m, a0.wrapping_add(OB_OFF30), 60);
        let anim = rb(m, a0.wrapping_add(OB_ANIM)).wrapping_add(1);
        wb(m, a0.wrapping_add(OB_ANIM), anim);
        if anim >= 3 {
            wb(m, a0.wrapping_add(OB_ROUTINE), 6);
        }
    }

    // Faiscas
    let sparkletime = rw(m, a0.wrapping_add(OB_OFF32));
    let novo_sparkle = sparkletime.wrapping_sub(1);
    ww(m, a0.wrapping_add(OB_OFF32), novo_sparkle);
    if s16(novo_sparkle) < 0 {
        ww(m, a0.wrapping_add(OB_OFF32), 11);
        let id = rb(m, a0.wrapping_add(OB_OFF34));
        let prox_id = (id.wrapping_add(2)) & 0x0E;
        wb(m, a0.wrapping_add(OB_OFF34), prox_id);

        let pos = SIGN_SPARK_POS.wrapping_add(u32::from(id & 0x0E));
        let dx = i8::from_be_bytes([rom_b(m, pos)]);
        let dy = i8::from_be_bytes([rom_b(m, pos.wrapping_add(1))]);

        if let Some(a1) = find_free_obj(m) {
            wb(m, a1.wrapping_add(OB_ID), ID_RINGS);
            wb(m, a1.wrapping_add(OB_ROUTINE), 6); // Ring_Sparkle
            let px = rw(m, a0.wrapping_add(OB_X)).wrapping_add(u16_de(i16::from(dx)));
            let py = rw(m, a0.wrapping_add(OB_Y)).wrapping_add(u16_de(i16::from(dy)));
            ww(m, a1.wrapping_add(OB_X), px);
            ww(m, a1.wrapping_add(OB_Y), py);
            wl(m, a1.wrapping_add(OB_MAP), MAP_RING);
            ww(m, a1.wrapping_add(OB_GFX), ARTTILE_RING | 0x2000); // Tile_Pal2
            wb(m, a1.wrapping_add(OB_RENDER), RENDER_CAM_FIELD);
            wb(m, a1.wrapping_add(OB_PRIORITY), 2);
            wb(m, a1.wrapping_add(OB_ACTWID), 8);
        }
    }
    Ok(())
}

fn sign_sonic_run(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    if rw(m, V_DEBUGUSE) != 0 {
        return Ok(());
    }

    let status = rb(m, V_PLAYER.wrapping_add(OB_STATUS));
    if status & STATUS_IN_AIR == 0 {
        wb(m, F_LOCKCTRL, 1);
        // btnR << 8: faz o Sonic correr para a direita
        ww(m, V_JPADHOLD2, 0x0800);
    }

    let sonic_ativo = rb(m, V_PLAYER.wrapping_add(OB_ID)) != 0;
    let sonic_x = rw(m, V_PLAYER.wrapping_add(OB_X));
    let limite_x = rw(m, V_LIMITRIGHT2).wrapping_add(320 - 24);

    if !sonic_ativo || sonic_x >= limite_x {
        wb(m, a0.wrapping_add(OB_ROUTINE), 8);
        got_through_act(m);
    }
    Ok(())
}

fn sign_exit(_m: &mut Memoria, _a0: u32) -> Result<(), ErroAbertura> {
    Ok(())
}

/// `GotThroughAct`: inicializa bonus de tempo, aneis e carrega os cartoes de passagem de ato.
pub fn got_through_act(m: &mut Memoria) {
    if rb(m, V_ENDCARD.wrapping_add(OB_ID)) != 0 {
        return;
    }

    let limite_dir = rw(m, V_LIMITRIGHT2);
    ww(m, V_LIMITLEFT2, limite_dir);
    wb(m, V_INVINC, 0);
    wb(m, F_TIMECOUNT, 0);
    wb(m, V_ENDCARD.wrapping_add(OB_ID), ID_GOT_THROUGH_CARD);
    super::plc::new_plc(m, PLCID_TITLECARD);
    wb(m, F_ENDACTBONUS, 1);

    // Os valores da tabela são lidos da ROM fornecida pelo jogador.
    let min = u32::from(rb(m, V_TIMEMIN));
    let seg = u32::from(rb(m, V_TIMESEC));
    let total_seg = min.wrapping_mul(60).wrapping_add(seg);
    let indice = total_seg.checked_div(15).unwrap_or(0).min(20);
    let time_bonus = rom_w(m, TIME_BONUSES.wrapping_add(indice.wrapping_mul(2)));
    ww(m, V_TIMEBONUS, time_bonus);

    let rings = rw(m, V_RINGS);
    let ring_bonus = rings.wrapping_mul(10);
    ww(m, V_RINGBONUS, ring_bonus);

    wb(m, V_SOUNDQUEUE1, BGM_GOT_THROUGH);
}

#[cfg(test)]
mod testes_signpost {
    use super::super::rl;
    use super::*;

    #[test]
    fn signpost_inicializa_com_rotina_2_e_propriedades() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        let a0 = V_LVLOBJSPACE;
        wb(&mut m, a0.wrapping_add(OB_ID), ID_SIGNPOST);
        ww(&mut m, a0.wrapping_add(OB_X), 0x100);
        sign_main(&mut m, a0).unwrap();
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 2);
        assert_eq!(rl(&mut m, a0.wrapping_add(OB_MAP)), MAP_SIGN);
        assert_eq!(rw(&mut m, a0.wrapping_add(OB_GFX)), ARTTILE_SIGNPOST);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_RENDER)), RENDER_CAM_FIELD);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ACTWID)), 24);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_PRIORITY)), 4);
    }

    #[test]
    fn signpost_detecta_toque_ao_inicializar() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        let a0 = V_LVLOBJSPACE;
        ww(&mut m, a0.wrapping_add(OB_X), 0x1000);
        ww(&mut m, V_PLAYER.wrapping_add(OB_X), 0x1000);
        wb(&mut m, F_TIMECOUNT, 1);
        sign_main(&mut m, a0).unwrap();
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 4);
        assert_eq!(rb(&mut m, F_TIMECOUNT), 0);
        assert_eq!(rb(&mut m, V_SOUNDQUEUE0), SFX_SIGNPOST);
    }

    #[test]
    fn signpost_nao_detecta_toque_com_subtracao_transbordada() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        let a0 = V_LVLOBJSPACE;
        ww(&mut m, a0.wrapping_add(OB_X), 0xffff);
        ww(&mut m, V_PLAYER.wrapping_add(OB_X), 0);
        sign_touch(&mut m, a0).unwrap();
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 0);
        assert_eq!(rb(&mut m, V_SOUNDQUEUE0), 0);
    }

    #[test]
    fn signpost_detecta_toque_do_sonic_e_inicia_giro() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        let a0 = V_LVLOBJSPACE;
        ww(&mut m, a0.wrapping_add(OB_X), 0x1000);
        ww(&mut m, V_PLAYER.wrapping_add(OB_X), 0x1010);
        ww(&mut m, V_LIMITRIGHT2, 0x1200);
        wb(&mut m, F_TIMECOUNT, 1);

        sign_touch(&mut m, a0).unwrap();
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 4);
        assert_eq!(rb(&mut m, F_TIMECOUNT), 0);
        assert_eq!(rw(&mut m, V_LIMITLEFT2), 0x1200);
        assert_eq!(rb(&mut m, V_SOUNDQUEUE0), SFX_SIGNPOST);
    }

    #[test]
    fn signpost_sonic_run_trava_controles_e_chama_got_through() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        let a0 = V_LVLOBJSPACE;
        wb(&mut m, V_PLAYER.wrapping_add(OB_ID), ID_SONIC);
        wb(&mut m, V_PLAYER.wrapping_add(OB_STATUS), 0); // No chao
        ww(&mut m, V_LIMITRIGHT2, 0x1000);
        ww(&mut m, V_PLAYER.wrapping_add(OB_X), 0x1000);

        sign_sonic_run(&mut m, a0).unwrap();
        assert_eq!(rb(&mut m, F_LOCKCTRL), 1);
        assert_eq!(rw(&mut m, V_JPADHOLD2), 0x0800);
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 0); // Ainda nao cruzou limite

        // Ao cruzar o limite direito
        ww(&mut m, V_PLAYER.wrapping_add(OB_X), 0x1000 + 320);
        sign_sonic_run(&mut m, a0).unwrap();
        assert_eq!(rb(&mut m, a0.wrapping_add(OB_ROUTINE)), 8);
        assert_eq!(
            rb(&mut m, V_ENDCARD.wrapping_add(OB_ID)),
            ID_GOT_THROUGH_CARD
        );
    }
}
