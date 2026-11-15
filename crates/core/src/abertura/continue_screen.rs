//! Tela de continue, com objetos 0x80 e 0x81 da REV01.
use super::boot::{rom_l, rom_w};
use super::ram::*;
use super::{ErroAbertura, fisica, objetos, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;
use crate::rotinas;

const MAP_CONT: u32 = 0x5062;
const ANI_CONT: u32 = 0x5054;
const CONT_TEXT: u32 = 0xFF_D040;
const CONT_LIGHT: u32 = 0xFF_D080;
const CONT_ICON: u32 = 0xFF_D0C0;
const CSI_MIN_SON_X_POS: u32 = 0x4E9A;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resultado {
    Aguardar,
    Sega,
    Fase,
}

fn contador(m: &mut Memoria, valor: u16) {
    wl(m, VDP_CONTROLE, super::titulo::loc_vram(0x6FC));
    for digito in [valor.checked_div(10).unwrap_or(0), valor % 10] {
        let mut fonte = ART_HUD.wrapping_add(u32::from(digito).wrapping_mul(64));
        for _ in 0..16 {
            let dados = rom_l(m, fonte);
            wl(m, VDP_DADOS, dados);
            fonte = fonte.wrapping_add(4);
        }
    }
}

/// Prepara VRAM, objetos e paleta depois do fade-out. A máquina efetua o fade-in.
pub fn preparar(m: &mut Memoria) -> Result<(), ErroAbertura> {
    let reg = rw(m, V_VDP_BUFFER1);
    ww(m, VDP_CONTROLE, reg & 0xFFBF);
    ww(m, VDP_CONTROLE, 0x8004);
    ww(m, VDP_CONTROLE, 0x8700);
    super::sega::clear_screen(m)?;
    super::titulo::zerar(m, V_OBJSPACE, V_OBJSPACE_END);
    for (tile, arte) in [(0x580, NEM_TITLECARD), (0x500, 0x3_B39A), (0x551, 0x3_B64A)] {
        wl(m, VDP_CONTROLE, super::titulo::loc_vram(tile));
        rotinas::nem_dec(m, arte).map_err(ErroAbertura::Rotina)?;
    }
    contador(m, 10);
    super::sega::pal_load_para(m, 0x12, 0x80);
    wb(m, V_SOUNDQUEUE0, 0x90);
    ww(m, V_GENERICTIMER, 659);
    wl(m, V_SCREENPOSX, 0);
    wl(m, V_SCREENPOSY, 0x0100_0000);
    wb(m, V_PLAYER.wrapping_add(OB_ID), 0x81);
    wb(m, CONT_TEXT.wrapping_add(OB_ID), 0x80);
    wb(m, CONT_LIGHT.wrapping_add(OB_ID), 0x80);
    wb(m, CONT_LIGHT.wrapping_add(OB_PRIORITY), 3);
    wb(m, CONT_LIGHT.wrapping_add(OB_FRAME), 4);
    wb(m, CONT_ICON.wrapping_add(OB_ID), 0x80);
    wb(m, CONT_ICON.wrapping_add(OB_ROUTINE), 4);
    objetos::execute_objects(m)?;
    objetos::build_sprites(m)?;
    ww(m, VDP_CONTROLE, reg | 0x40);
    Ok(())
}

/// Avança após o VBlank 0x16. A saída indica o próximo modo ao controlador principal.
pub fn passo(m: &mut Memoria) -> Result<Resultado, ErroAbertura> {
    if rb(m, V_PLAYER.wrapping_add(OB_ROUTINE)) < 6 {
        let tempo = rw(m, V_GENERICTIMER);
        contador(m, (tempo.checked_div(60).unwrap_or(0)) & 0xF);
    }
    objetos::execute_objects(m)?;
    objetos::build_sprites(m)?;
    if rw(m, V_PLAYER.wrapping_add(OB_X)) >= 384 {
        wb(m, V_GAMEMODE, ID_LEVEL);
        wb(m, V_LIVES, 3);
        ww(m, V_RINGS, 0);
        wl(m, V_TIME, 0);
        wl(m, V_SCORE, 0);
        wb(m, V_LASTLAMP, 0);
        let credito = rb(m, V_CONTINUES).wrapping_sub(1);
        wb(m, V_CONTINUES, credito);
        return Ok(Resultado::Fase);
    }
    if rb(m, V_PLAYER.wrapping_add(OB_ROUTINE)) < 6 && rw(m, V_GENERICTIMER) == 0 {
        wb(m, V_GAMEMODE, 0);
        return Ok(Resultado::Sega);
    }
    Ok(Resultado::Aguardar)
}

pub fn item(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    match rb(m, a.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, a.wrapping_add(OB_ROUTINE), 2);
            wl(m, a.wrapping_add(OB_MAP), MAP_CONT);
            ww(m, a.wrapping_add(OB_GFX), 0x8500);
            wb(m, a.wrapping_add(OB_RENDER), 0);
            wb(m, a.wrapping_add(OB_ACTWID), 60);
            ww(m, a.wrapping_add(OB_X), 288);
            ww(m, a.wrapping_add(OB_SCREENY), 192);
            ww(m, V_RINGS, 0);
            objetos::display_sprite(m, a);
        }
        2 => objetos::display_sprite(m, a),
        4 => {
            let continua = rb(m, V_CONTINUES);
            if continua < 2 {
                objetos::delete_object(m, a);
                return Ok(());
            }
            let quantidade = continua.wrapping_sub(1).min(15);
            let deslocar = quantidade.wrapping_sub(1) & 1 != 0;
            let mut slot = a;
            for i in 0..quantidade {
                let x = rom_w(
                    m,
                    CSI_MIN_SON_X_POS.wrapping_add(u32::from(i).wrapping_mul(2)),
                )
                .wrapping_sub(if deslocar { 10 } else { 0 });
                wb(m, slot.wrapping_add(OB_ID), 0x80);
                ww(m, slot.wrapping_add(OB_X), x);
                ww(m, slot.wrapping_add(OB_SCREENY), 208);
                wb(m, slot.wrapping_add(OB_FRAME), 6);
                wb(m, slot.wrapping_add(OB_ROUTINE), 6);
                wl(m, slot.wrapping_add(OB_MAP), MAP_CONT);
                ww(m, slot.wrapping_add(OB_GFX), 0x8551);
                wb(m, slot.wrapping_add(OB_RENDER), 0);
                if i == quantidade.wrapping_sub(1) && continua <= 15 {
                    wb(m, slot.wrapping_add(OB_SUBTYPE), 1);
                }
                slot = slot.wrapping_add(OBJECT_SIZE);
            }
            mini(m, a);
        }
        6 => mini(m, a),
        _ => return Err(ErroAbertura::NaoImplementado("rotina do item de continue")),
    }
    Ok(())
}
fn mini(m: &mut Memoria, a: u32) {
    let tempo = rb(m, V_VBLANK_BYTE);
    if rb(m, a.wrapping_add(OB_SUBTYPE)) != 0
        && rb(m, V_PLAYER.wrapping_add(OB_ROUTINE)) >= 6
        && tempo & 1 == 0
    {
        if rw(m, V_PLAYER.wrapping_add(OB_VELX)) != 0 {
            objetos::delete_object(m, a);
        }
        return;
    }
    if tempo & 15 == 0 {
        let frame = rb(m, a.wrapping_add(OB_FRAME));
        wb(m, a.wrapping_add(OB_FRAME), frame ^ 1);
    }
    objetos::display_sprite(m, a);
}

pub fn sonic(m: &mut Memoria, a: u32) -> Result<(), ErroAbertura> {
    let mut rotina = rb(m, a.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        wb(m, a.wrapping_add(OB_ROUTINE), 2);
        ww(m, a.wrapping_add(OB_X), 0xA0);
        ww(m, a.wrapping_add(OB_Y), 0xC0);
        wl(m, a.wrapping_add(OB_MAP), MAP_SONIC);
        ww(m, a.wrapping_add(OB_GFX), ARTTILE_SONIC);
        wb(m, a.wrapping_add(OB_RENDER), 4);
        wb(m, a.wrapping_add(OB_PRIORITY), 2);
        wb(m, a.wrapping_add(OB_ANIM), 0x1D);
        ww(m, a.wrapping_add(OB_VELY), 0x400);
        rotina = 2;
    }
    if rotina == 2 && rw(m, a.wrapping_add(OB_Y)) == 0x1A0 {
        wb(m, a.wrapping_add(OB_ROUTINE), 4);
        ww(m, a.wrapping_add(OB_VELY), 0);
        wl(m, a.wrapping_add(OB_MAP), MAP_CONT);
        ww(m, a.wrapping_add(OB_GFX), 0x8500);
        wb(m, a.wrapping_add(OB_ANIM), 0);
        rotina = 4;
    }
    if rotina == 4 {
        if rb(m, V_JPADPRESS1) & BTN_START == 0 {
            objetos::animate_sprite(m, a, ANI_CONT)?;
            objetos::display_sprite(m, a);
            return Ok(());
        }
        wb(m, a.wrapping_add(OB_ROUTINE), 6);
        wl(m, a.wrapping_add(OB_MAP), MAP_SONIC);
        ww(m, a.wrapping_add(OB_GFX), ARTTILE_SONIC);
        wb(m, a.wrapping_add(OB_ANIM), 0x1E);
        ww(m, a.wrapping_add(OB_INERTIA), 0);
        let y = rw(m, a.wrapping_add(OB_Y)).wrapping_sub(8);
        ww(m, a.wrapping_add(OB_Y), y);
        wb(m, V_SOUNDQUEUE1, BGM_FADE);
        rotina = 6;
    }
    if rotina == 6 {
        let inercia = rw(m, a.wrapping_add(OB_INERTIA));
        if inercia == 0x800 {
            ww(m, a.wrapping_add(OB_VELX), 0x1000);
        } else {
            ww(m, a.wrapping_add(OB_INERTIA), inercia.wrapping_add(0x20));
        }
    }
    if rotina != 2 && rotina != 6 {
        return Err(ErroAbertura::NaoImplementado("rotina do Sonic em continue"));
    }
    fisica::speed_to_pos(m, a);
    fisica::sonic_animate(m, a)?;
    fisica::sonic_load_gfx(m, a);
    objetos::display_sprite(m, a);
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    #[ignore = "requer SONIC1_ROM com a ROM REV01"]
    fn continue_com_rom_real_ate_o_retorno_a_fase() {
        let caminho = std::env::var_os("SONIC1_ROM").expect("SONIC1_ROM");
        let rom = std::fs::read(caminho).expect("leitura da ROM");
        let mut m = Memoria::nova(rom);
        wb(&mut m, V_CONTINUES, 2);
        preparar(&mut m).unwrap();
        let mut resultado = Resultado::Aguardar;
        for quadro in 0..300 {
            wb(
                &mut m,
                V_JPADPRESS1,
                if quadro == 100 { BTN_START } else { 0 },
            );
            resultado = passo(&mut m).unwrap();
            if resultado != Resultado::Aguardar {
                break;
            }
        }
        assert_eq!(resultado, Resultado::Fase);
        assert_eq!(rb(&mut m, V_CONTINUES), 1);
        assert_eq!(rb(&mut m, V_LIVES), 3);
    }

    #[test]
    fn continua_retorna_com_tres_vidas_e_consume_um_credito() {
        let mut m = Memoria::nova(vec![0; 0x40000]);
        wb(&mut m, V_PLAYER.wrapping_add(OB_ID), 0x81);
        wb(&mut m, V_PLAYER.wrapping_add(OB_ROUTINE), 6);
        ww(&mut m, V_PLAYER.wrapping_add(OB_X), 384);
        wb(&mut m, V_CONTINUES, 2);
        wb(&mut m, V_LIVES, 0);
        ww(&mut m, V_RINGS, 99);
        assert_eq!(passo(&mut m).unwrap(), Resultado::Fase);
        assert_eq!(rb(&mut m, V_CONTINUES), 1);
        assert_eq!(rb(&mut m, V_LIVES), 3);
        assert_eq!(rw(&mut m, V_RINGS), 0);
    }

    #[test]
    fn countdown_expirado_retorna_a_sega() {
        let mut m = Memoria::nova(vec![0; 0x40000]);
        wb(&mut m, V_PLAYER.wrapping_add(OB_ID), 0x81);
        wb(&mut m, V_PLAYER.wrapping_add(OB_ROUTINE), 4);
        assert_eq!(passo(&mut m).unwrap(), Resultado::Sega);
        assert_eq!(rb(&mut m, V_GAMEMODE), 0);
    }

    #[test]
    fn start_dispara_saida_e_impede_expiracao() {
        let mut m = Memoria::nova(vec![0; 0x40000]);
        wb(&mut m, V_PLAYER.wrapping_add(OB_ID), 0x81);
        wb(&mut m, V_PLAYER.wrapping_add(OB_ROUTINE), 4);
        wb(&mut m, V_JPADPRESS1, BTN_START);
        assert_eq!(passo(&mut m).unwrap(), Resultado::Aguardar);
        assert_eq!(rb(&mut m, V_PLAYER.wrapping_add(OB_ROUTINE)), 6);
        assert_eq!(rw(&mut m, V_PLAYER.wrapping_add(OB_INERTIA)), 0x20);
    }
}
