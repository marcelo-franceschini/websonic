//! HUD (`_inc/HUD Update.asm`). Por enquanto só o `HUD_Update` sem nada a atualizar: as flags de
//! contagem (pontos, anéis, tempo, vidas e bônus) só ligam depois dos cartões de título, na fatia
//! seguinte. Se alguma estiver ligada aqui, o núcleo para com `NaoImplementado`, em vez de pular
//! a escrita dos números em silêncio.

use super::ram::*;
use super::{ErroAbertura, rb, rw};
use crate::memoria::Memoria;

/// `HUD_Update`: no modo de depuração mostra as coordenadas; senão atualiza os números que as
/// flags pedem.
pub fn hud_update(m: &mut Memoria) -> Result<(), ErroAbertura> {
    if rw(m, F_DEBUGMODE) != 0 {
        return Err(ErroAbertura::NaoImplementado("HUD do modo de depuração"));
    }
    for flag in [
        F_SCORECOUNT,
        F_RINGCOUNT,
        F_TIMECOUNT,
        F_LIFECOUNT,
        F_ENDACTBONUS,
    ] {
        if rb(m, flag) != 0 {
            return Err(ErroAbertura::NaoImplementado("números do HUD"));
        }
    }
    Ok(())
}
