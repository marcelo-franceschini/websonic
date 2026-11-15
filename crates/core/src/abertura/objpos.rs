//! Carga dos objetos da fase (`_inc/ObjPosLoad.asm`): a janela de objetos ao redor da câmera, a
//! lista de lembrados e a criação de cada objeto no primeiro espaço livre. Só a partida da fase
//! (câmera parada à direita do último ponto) e os movimentos para os dois lados.

use super::boot::{rom_b, rom_w};
use super::ram::*;
use super::{ErroAbertura, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

/// Bytes de uma entrada da lista de objetos: X, Y com os giros, ID com o bit de lembrar e subtipo.
const ENTRADA: u32 = 6;

/// `ext.l` de uma palavra (`adda.w`).
fn ext_l(w: u16) -> u32 {
    u32::from_be_bytes(i32::from(i16::from_be_bytes(w.to_be_bytes())).to_be_bytes())
}

fn com_sinal(w: u16) -> i16 {
    i16::from_be_bytes(w.to_be_bytes())
}

/// `ObjPosLoad`: despacha pela rotina guardada em `v_opl_routine`.
pub fn obj_pos_load(m: &mut Memoria) -> Result<(), ErroAbertura> {
    match rb(m, V_OPL_ROUTINE) {
        0 => {
            opl_main(m);
            opl_next(m)
        }
        2 => opl_next(m),
        _ => Err(ErroAbertura::NaoImplementado("rotina do ObjPosLoad")),
    }
}

/// `OPL_Main`: acha as listas da fase, zera a lista de lembrados e posiciona a janela.
fn opl_main(m: &mut Memoria) {
    let r = rb(m, V_OPL_ROUTINE).wrapping_add(2);
    wb(m, V_OPL_ROUTINE, r);
    // `lsl.b #6` age só no byte baixo (o ato); `lsr.w #4` junta com a zona: zona * 16 + ato * 4.
    let za = rw(m, V_ZONE_ACT);
    let baixo = (za & 0xFF).wrapping_shl(6) & 0xFF;
    let d0 = ((za & 0xFF00) | baixo) >> 4;
    let indice = OBJPOS_INDEX.wrapping_add(u32::from(d0));
    let lista = OBJPOS_INDEX.wrapping_add(ext_l(rom_w(m, indice)));
    wl(m, V_OPL_DATA, lista);
    wl(m, V_OPL_DATA.wrapping_add(4), lista);
    let secundaria = OBJPOS_INDEX.wrapping_add(ext_l(rom_w(m, indice.wrapping_add(2))));
    wl(m, V_OPL_DATA.wrapping_add(8), secundaria);
    wl(m, V_OPL_DATA.wrapping_add(12), secundaria);
    // A lista de lembrados começa com o contador em 1 e limpa $17C bytes (o original conta
    // palavras num laço de longas, FixBugs = 0).
    ww(m, V_OBJSTATE, 0x0101);
    let mut a2 = V_OBJSTATE.wrapping_add(2);
    let palavras = (V_OBJSTATE_END.wrapping_sub(V_OBJSTATE).wrapping_sub(2)) / 2;
    for _ in 0..palavras {
        wl(m, a2, 0);
        a2 = a2.wrapping_add(4);
    }
    let a2 = V_OBJSTATE;
    let mut d6 = rw(m, V_SCREENPOSX).saturating_sub(128) & 0xFF80;
    let mut a0 = rl(m, V_OPL_DATA);
    // `.loop_find_right_init`: pula os objetos antes da borda direita da janela inicial.
    while d6 > rw(m, a0) {
        if rb(m, a0.wrapping_add(4)) & 0x80 != 0 {
            let n = rb(m, a2).wrapping_add(1);
            wb(m, a2, n);
        }
        a0 = a0.wrapping_add(ENTRADA);
    }
    wl(m, V_OPL_DATA, a0);
    let mut a0 = rl(m, V_OPL_DATA.wrapping_add(4));
    if let Some(esquerda) = d6.checked_sub(128) {
        d6 = esquerda;
        while d6 > rw(m, a0) {
            if rb(m, a0.wrapping_add(4)) & 0x80 != 0 {
                let n = rb(m, a2.wrapping_add(1)).wrapping_add(1);
                wb(m, a2.wrapping_add(1), n);
            }
            a0 = a0.wrapping_add(ENTRADA);
        }
    }
    wl(m, V_OPL_DATA.wrapping_add(4), a0);
    ww(m, V_OPL_SCREEN, 0xFFFF);
}

/// `OPL_Next`: se a câmera andou meio bloco de objetos, cria os que entram na janela.
fn opl_next(m: &mut Memoria) -> Result<(), ErroAbertura> {
    let a2 = V_OBJSTATE;
    let mut d6 = rw(m, V_SCREENPOSX) & 0xFF80;
    let anterior = rw(m, V_OPL_SCREEN);
    if d6 == anterior {
        return Ok(());
    }
    if com_sinal(d6.wrapping_sub(anterior)) < 0 {
        return opl_moved_left(m, d6);
    }
    ww(m, V_OPL_SCREEN, d6);
    let mut a0 = rl(m, V_OPL_DATA);
    d6 = d6.wrapping_add(320 + 320);
    let mut d2 = 0u8;
    // `.loop_find_right`: cria os objetos até a borda direita da janela.
    while d6 > rw(m, a0) {
        if rb(m, a0.wrapping_add(4)) & 0x80 != 0 {
            d2 = rb(m, a2);
            wb(m, a2, d2.wrapping_add(1));
        }
        let (criou, proximo) = opl_spawn_obj(m, a0, a2, d2)?;
        a0 = proximo;
        if !criou {
            break;
        }
    }
    wl(m, V_OPL_DATA, a0);
    let mut a0 = rl(m, V_OPL_DATA.wrapping_add(4));
    if let Some(esquerda) = d6.checked_sub(320 + 320 + 128) {
        while esquerda > rw(m, a0) {
            if rb(m, a0.wrapping_add(4)) & 0x80 != 0 {
                let n = rb(m, a2.wrapping_add(1)).wrapping_add(1);
                wb(m, a2.wrapping_add(1), n);
            }
            a0 = a0.wrapping_add(ENTRADA);
        }
    }
    wl(m, V_OPL_DATA.wrapping_add(4), a0);
    Ok(())
}

/// `OPL_MovedLeft`: recua os dois cursores e reconstitui os contadores de objetos lembrados.
fn opl_moved_left(m: &mut Memoria, posicao: u16) -> Result<(), ErroAbertura> {
    let a2 = V_OBJSTATE;
    ww(m, V_OPL_SCREEN, posicao);
    let (esquerda, sob_fluxo) = posicao.overflowing_sub(128);
    let mut a0 = rl(m, V_OPL_DATA.wrapping_add(4));
    let mut d2 = 0u8;
    if !sob_fluxo {
        while com_sinal(esquerda) < com_sinal(rw(m, a0.wrapping_sub(ENTRADA))) {
            a0 = a0.wrapping_sub(ENTRADA);
            let lembrar = rb(m, a0.wrapping_add(4)) & 0x80 != 0;
            if lembrar {
                d2 = rb(m, a2.wrapping_add(1)).wrapping_sub(1);
                wb(m, a2.wrapping_add(1), d2);
            }
            let (criou, seguinte) = opl_spawn_obj(m, a0, a2, d2)?;
            if !criou {
                if lembrar {
                    let n = rb(m, a2.wrapping_add(1)).wrapping_add(1);
                    wb(m, a2.wrapping_add(1), n);
                }
                a0 = seguinte.wrapping_add(ENTRADA);
                break;
            }
            a0 = seguinte.wrapping_sub(ENTRADA);
        }
    }
    wl(m, V_OPL_DATA.wrapping_add(4), a0);
    let direita = esquerda.wrapping_add(128 + 320 + 320);
    let mut a0 = rl(m, V_OPL_DATA);
    while com_sinal(direita) <= com_sinal(rw(m, a0.wrapping_sub(ENTRADA))) {
        if rb(m, a0.wrapping_sub(2)) & 0x80 != 0 {
            let n = rb(m, a2).wrapping_sub(1);
            wb(m, a2, n);
        }
        a0 = a0.wrapping_sub(ENTRADA);
    }
    wl(m, V_OPL_DATA, a0);
    Ok(())
}

/// `OPL_SpawnObj`: cria o objeto da entrada `a0` (ou pula, se já foi destruído). Devolve se deu
/// certo (`Z`) e o `a0` seguinte.
fn opl_spawn_obj(m: &mut Memoria, a0: u32, a2: u32, d2: u8) -> Result<(bool, u32), ErroAbertura> {
    if rb(m, a0.wrapping_add(4)) & 0x80 != 0 {
        // `bset #7,2(a2,d2.w)`: marca como carregado e vê se já estava marcado.
        let marca = a2.wrapping_add(2).wrapping_add(u32::from(d2));
        let antes = rb(m, marca);
        wb(m, marca, antes | 0x80);
        if antes & 0x80 != 0 {
            return Ok((true, a0.wrapping_add(ENTRADA)));
        }
    }
    // `FindFreeObj`: o primeiro espaço de objeto livre.
    let mut a1 = V_LVLOBJSPACE;
    while a1 < V_LVLOBJEND && rb(m, a1.wrapping_add(OB_ID)) != 0 {
        a1 = a1.wrapping_add(OBJECT_SIZE);
    }
    if a1 >= V_LVLOBJEND {
        return Ok((false, a0));
    }
    let x = rw(m, a0);
    ww(m, a1.wrapping_add(OB_X), x);
    let yw = rw(m, a0.wrapping_add(2));
    ww(m, a1.wrapping_add(OB_Y), yw & 0xFFF);
    let giros = u8::try_from(yw.rotate_left(2) & 3).unwrap_or(0);
    wb(m, a1.wrapping_add(OB_RENDER), giros);
    wb(m, a1.wrapping_add(OB_STATUS), giros);
    let mut id = rom_b(m, a0.wrapping_add(4));
    if id & 0x80 != 0 {
        id &= 0x7F;
        wb(m, a1.wrapping_add(OB_RESPAWNNO), d2);
    }
    wb(m, a1.wrapping_add(OB_ID), id);
    let sub = rom_b(m, a0.wrapping_add(5));
    wb(m, a1.wrapping_add(OB_SUBTYPE), sub);
    Ok((true, a0.wrapping_add(ENTRADA)))
}

#[cfg(test)]
mod testes_esquerda {
    use super::*;

    fn memoria() -> Memoria {
        let mut rom = vec![0; 0x5000];
        for (i, x) in [100u16, 200, 300, 500, 800, 0xFFFF].iter().enumerate() {
            let pos = 0x4000usize.wrapping_add(i.wrapping_mul(6));
            rom[pos..pos.wrapping_add(2)].copy_from_slice(&x.to_be_bytes());
            if *x != 0xFFFF {
                rom[pos.wrapping_add(3)] = 64;
                rom[pos.wrapping_add(4)] = if *x == 200 { 0xA5 } else { 0x25 };
            }
        }
        let mut m = Memoria::nova(rom);
        ww(&mut m, V_OPL_SCREEN, 384);
        ww(&mut m, V_SCREENPOSX, 256);
        wl(&mut m, V_OPL_DATA, 0x401E);
        wl(&mut m, V_OPL_DATA + 4, 0x400C);
        wb(&mut m, V_OBJSTATE + 1, 2);
        m
    }

    #[test]
    fn ao_recuar_recria_objeto_e_atualiza_cursor_e_contador() {
        let mut m = memoria();
        opl_next(&mut m).unwrap();
        assert_eq!(rw(&mut m, V_OPL_SCREEN), 256);
        assert_eq!(rl(&mut m, V_OPL_DATA + 4), 0x4006);
        assert_eq!(rb(&mut m, V_OBJSTATE + 1), 1);
        assert_eq!(rb(&mut m, V_LVLOBJSPACE + OB_ID), 0x25);
        assert_eq!(rw(&mut m, V_LVLOBJSPACE + OB_X), 200);
        assert_eq!(rb(&mut m, V_LVLOBJSPACE + OB_RESPAWNNO), 1);
    }

    #[test]
    fn ao_faltar_espaco_restaura_contador_e_cursor() {
        let mut m = memoria();
        let mut objeto = V_LVLOBJSPACE;
        while objeto < V_LVLOBJEND {
            wb(&mut m, objeto + OB_ID, 1);
            objeto += OBJECT_SIZE;
        }
        opl_next(&mut m).unwrap();
        assert_eq!(rb(&mut m, V_OBJSTATE + 1), 2);
        assert_eq!(rl(&mut m, V_OPL_DATA + 4), 0x400C);
    }
}
