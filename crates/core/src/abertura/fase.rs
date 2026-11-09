//! Modo de fase e de demo (`GM_Level`): o começo até `PaletteFadeOut`, o trecho que prepara os
//! cartões de título e o laço dos cartões (`Level_TtlCardLoop`). A carga do resto da fase entra
//! nas fatias seguintes.

use super::boot::rom_b;
use super::objetos::{build_sprites, execute_objects};
use super::ram::*;
use super::sega::{clear_screen, pal_load};
use super::titulo::{loc_vram, zerar};
use super::{ErroAbertura, plc, rb, rl, rw, titulo, wb, wl, ww};
use crate::memoria::Memoria;
use crate::rotinas;

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

/// Do fim do `PaletteFadeOut` de `GM_Level` até o laço dos cartões de título: a arte dos cartões,
/// as filas de arte da zona, a limpeza da RAM (`Level_ClrRam`), os registradores da VDP, a paleta
/// do Sonic, a música da fase e o objeto dos cartões. Roda de uma vez na marca
/// `Level_TtlCardLoop` (D7.10). A demo do final e a LZ (água) devolvem `NaoImplementado`.
pub fn preparo_dos_cartoes(m: &mut Memoria) -> Result<(), ErroAbertura> {
    if rw(m, F_DEMO) & 0x8000 != 0 {
        return Err(ErroAbertura::NaoImplementado("demo do final"));
    }
    // A arte dos cartões vai direto para a VRAM (com as interrupções desligadas).
    wl(m, VDP_CONTROLE, loc_vram(u32::from(ARTTILE_TITLE_CARD)));
    rotinas::nem_dec(m, NEM_TITLECARD).map_err(ErroAbertura::Rotina)?;
    // A primeira lista de PLC do cabeçalho da zona (`LevelHeaders`, $10 bytes por zona) e a
    // `plcid_Main2`, somadas ao fim da fila.
    let zona = rb(m, V_ZONE);
    let cabecalho = LEVEL_HEADERS.wrapping_add(u32::from(zona).wrapping_mul(0x10));
    let primeira = rom_b(m, cabecalho);
    if primeira != 0 {
        plc::add_plc(m, u16::from(primeira))?;
    }
    plc::add_plc(m, PLCID_MAIN2)?;
    // `Level_ClrRam`: objetos, variáveis diversas, variáveis da fase e as de tempo e tela.
    zerar(m, V_OBJSPACE, V_OBJSPACE_END);
    zerar(m, V_MISC_VARIABLES, V_MISC_VARIABLES_END);
    zerar(m, V_LEVELVARIABLES, V_LEVELVARIABLES_END);
    zerar(
        m,
        V_TIMINGANDSCREENVARIABLES,
        V_TIMINGANDSCREENVARIABLES_END,
    );
    clear_screen(m)?;
    // Rolagem por linha, tabelas em $C000, $E000 e $F800, plano de 64 células, cor de fundo da
    // linha 3 e a linha do HBlank da água (223).
    for w in [0x8B03u16, 0x8230, 0x8407, 0x857C, 0x9001, 0x8004, 0x8720] {
        ww(m, VDP_CONTROLE, w);
    }
    ww(m, V_HBLANK_HREG, 0x8ADF);
    let h = rw(m, V_HBLANK_HREG);
    ww(m, VDP_CONTROLE, h);
    if zona == ID_LZ {
        return Err(ErroAbertura::NaoImplementado("água da LZ"));
    }
    // `Level_LoadPal`: 30 segundos de ar e a paleta do Sonic (para os cartões).
    ww(m, V_AIR, 30);
    pal_load(m, PALID_SONIC);
    // `Level_GetBgm`: a música da zona pela `MusicList` (SBZ3 usa a da SBZ; a FZ, a sétima).
    let zona_e_ato = rw(m, V_ZONE_ACT);
    let indice = if zona_e_ato == ID_FZ {
        6
    } else if zona_e_ato == ID_LZ_ACT4 {
        5
    } else {
        u32::from(zona)
    };
    let musica = rom_b(m, MUSIC_LIST.wrapping_add(indice));
    wb(m, V_SOUNDQUEUE0, musica);
    wb(m, V_TITLECARD.wrapping_add(OB_ID), ID_TITLE_CARD);
    Ok(())
}

/// Para onde o laço dos cartões vai depois de um quadro.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepoisDosCartoes {
    /// Pede o VBlank dos cartões e repete.
    Continua,
    /// O "ACT" chegou ao alvo e a fila de arte esvaziou.
    Fim,
}

/// `Level_TtlCardLoop`, depois da espera: os cartões andam, os sprites saem e a fila de arte anda.
/// O laço acaba quando o elemento "ACT" chegou ao alvo e a fila esvaziou (FixBugs = 0: só o "ACT"
/// é conferido).
pub fn laco_dos_cartoes(m: &mut Memoria) -> Result<DepoisDosCartoes, ErroAbertura> {
    execute_objects(m)?;
    build_sprites(m)?;
    plc::run_plc(m)?;
    let x = rw(m, V_TTLCARDACT.wrapping_add(OB_X));
    let alvo = rw(m, V_TTLCARDACT.wrapping_add(CARD_MAINX));
    if x != alvo || rl(m, V_PLC_BUFFER) != 0 {
        return Ok(DepoisDosCartoes::Continua);
    }
    Ok(DepoisDosCartoes::Fim)
}
