//! Modo de fase e de demo (`GM_Level`). Por enquanto só o começo, até a chamada de
//! `PaletteFadeOut`: a carga da fase da demo entra nas fatias seguintes.

use super::ram::*;
use super::{plc, rb, rw, titulo, wb};
use crate::memoria::Memoria;

/// `GM_Level` até `PaletteFadeOut`: marca o modo com o bit 7 (sequência antes da fase), apaga a
/// música (menos na demo do final, com `f_demo` negativo), esvazia a fila de arte e começa o
/// desvanecimento para preto.
pub fn gm_level_inicio(m: &mut Memoria) {
    let modo = rb(m, V_GAMEMODE) | 0x80;
    wb(m, V_GAMEMODE, modo);
    if rw(m, F_DEMO) & 0x8000 == 0 {
        wb(m, V_SOUNDQUEUE1, BGM_FADE);
    }
    plc::clear_plc(m);
    titulo::palette_fade_out_inicio(m);
}
