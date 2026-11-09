//! Modo de fase e de demo (`GM_Level`): o começo até `PaletteFadeOut`, o trecho que prepara os
//! cartões de título e o laço dos cartões (`Level_TtlCardLoop`). A carga do resto da fase entra
//! nas fatias seguintes.

use super::boot::{rom_b, rom_l};
use super::objetos::{build_sprites, execute_objects};
use super::ram::*;
use super::sega::{clear_screen, pal_load, pal_load_para};
use super::titulo::{loc_vram, zerar};
use super::{ErroAbertura, cenario, desenho, hud, objpos, plc, rb, rl, rw, titulo, wb, wl, ww};
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

/// `LevelDataLoad`: os blocos 16x16 (Enigma), os chunks 256x256 (Kosinski), o layout, a paleta do
/// cabeçalho no buffer do desvanecimento e a segunda lista de arte. Devolve o `a0` que sobra, que
/// `LoadTilesFromStart` ainda usa num chunk vazio. Só as zonas com a paleta comum.
fn level_data_load(m: &mut Memoria) -> Result<u32, ErroAbertura> {
    let zona = rb(m, V_ZONE);
    let cabecalho = LEVEL_HEADERS.wrapping_add(u32::from(zona).wrapping_mul(0x10));
    let blocos = rom_l(m, cabecalho.wrapping_add(4)) & 0x00FF_FFFF;
    rotinas::eni_dec(m, blocos, V_16X16, 0).map_err(ErroAbertura::Rotina)?;
    let chunks = rom_l(m, cabecalho.wrapping_add(8)) & 0x00FF_FFFF;
    rotinas::kos_dec(m, chunks, V_256X256).map_err(ErroAbertura::Rotina)?;
    let a0 = cenario::level_layout_load(m)?;
    let zona_e_ato = rw(m, V_ZONE_ACT);
    if zona_e_ato == ID_LZ_ACT4 || zona_e_ato == ID_SBZ_ACT2 || zona_e_ato == ID_FZ {
        return Err(ErroAbertura::NaoImplementado("paleta especial da fase"));
    }
    let paleta = rom_b(m, cabecalho.wrapping_add(15));
    pal_load_para(m, u32::from(paleta), 0x80);
    let plc2 = rom_b(m, cabecalho.wrapping_add(4));
    if plc2 != 0 {
        plc::add_plc(m, u16::from(plc2))?;
    }
    Ok(a0)
}

/// `ColIndexLoad`: o ponteiro do índice de colisão da zona.
fn col_index_load(m: &mut Memoria) {
    let zona = u32::from(rb(m, V_ZONE));
    let ponteiro = rom_l(m, COLPOINTERS.wrapping_add(zona.wrapping_mul(4)));
    wl(m, V_COLLINDEX, ponteiro);
}

/// `OscillateNumInit`: os valores iniciais da tabela de números que oscilam.
fn oscillate_num_init(m: &mut Memoria) {
    const BASE: [u16; 33] = [
        0x007C, 0x80, 0, 0x80, 0, 0x80, 0, 0x80, 0, 0x80, 0, 0x80, 0, 0x80, 0, 0x80, 0, 0x80, 0,
        0x50F0, 0x11E, 0x2080, 0xB4, 0x3080, 0x10E, 0x5080, 0x1C2, 0x7080, 0x276, 0x80, 0, 0x80, 0,
    ];
    let mut a1 = V_OSCILLATE;
    for w in BASE {
        ww(m, a1, w);
        a1 = a1.wrapping_add(2);
    }
}

/// De `Level_SkipTtlCard` até `Level_Delay`: a arte do HUD, a paleta, os limites e a rolagem, os
/// blocos e o layout, o desenho inicial do cenário, a colisão, o Sonic, o HUD e os objetos da
/// partida, as variáveis de uma partida nova e o temporizador da demo. Roda de uma vez na marca
/// `Level_Delay` (D7.10); as interrupções que chegam no meio são quadros de lag (D7.25).
pub fn preparo_final(m: &mut Memoria) -> Result<(), ErroAbertura> {
    if rw(m, F_DEMO) & 0x8000 != 0 {
        return Err(ErroAbertura::NaoImplementado("demo do final"));
    }
    hud::hud_base(m);
    pal_load_para(m, PALID_SONIC, 0x80);
    cenario::level_size_load(m)?;
    cenario::deform_layers(m)?;
    let flags = rb(m, V_FG_SCROLL_FLAGS) | 4;
    wb(m, V_FG_SCROLL_FLAGS, flags);
    let a0 = level_data_load(m)?;
    desenho::load_tiles_from_start(m, a0)?;
    col_index_load(m);
    if rb(m, V_ZONE) == ID_LZ {
        return Err(ErroAbertura::NaoImplementado("água da LZ"));
    }
    wb(m, V_PLAYER.wrapping_add(OB_ID), ID_SONIC);
    wb(m, V_HUD.wrapping_add(OB_ID), ID_HUD);
    if rb(m, F_DEBUGCHEAT) != 0 && rb(m, V_JPADHOLD1) & 0x40 != 0 {
        wb(m, F_DEBUGMODE, 1);
    }
    ww(m, V_JPADHOLD2, 0);
    ww(m, V_JPADHOLD1, 0);
    objpos::obj_pos_load(m)?;
    objetos_faltando(m)?;
    execute_objects(m)?;
    build_sprites(m)?;
    if rb(m, V_LASTLAMP_FASE) == 0 {
        ww(m, V_RINGS, 0);
        wl(m, V_TIME, 0);
        wb(m, V_LIFECOUNT, 0);
    }
    for v in [F_TIMEOVER, V_SHIELD, V_INVINC, V_SHOES, V_UNUSED1] {
        wb(m, v, 0);
    }
    for v in [V_DEBUGUSE, F_RESTART, V_FRAMECOUNT] {
        ww(m, v, 0);
    }
    oscillate_num_init(m);
    for v in [F_SCORECOUNT, F_RINGCOUNT, F_TIMECOUNT] {
        wb(m, v, 1);
    }
    ww(m, V_BTNPUSHTIME1, 0);
    // O primeiro par de bytes da gravação da demo da zona: a duração do primeiro botão.
    let zona = u32::from(rb(m, V_ZONE));
    let demo = rom_l(m, DEMO_DATA_PTR.wrapping_add(zona.wrapping_mul(4)));
    let duracao = rom_b(m, demo.wrapping_add(1)).wrapping_sub(1);
    wb(m, V_BTNPUSHTIME2, duracao);
    ww(m, V_GENERICTIMER, 1800);
    Ok(())
}

/// `Level_StartGame`: os cartões começam a sair, e o bit 7 do modo se apaga (fim da sequência
/// antes da fase).
pub fn level_start_game(m: &mut Memoria) -> Result<(), ErroAbertura> {
    if rw(m, F_DEMO) & 0x8000 != 0 {
        return Err(ErroAbertura::NaoImplementado("demo do final"));
    }
    for (a, soma) in [
        (V_TITLECARD, 2u8),
        (0xFF_D0C0, 4),
        (V_TTLCARDACT, 4),
        (0xFF_D140, 4),
    ] {
        let r = rb(m, a.wrapping_add(OB_ROUTINE)).wrapping_add(soma);
        wb(m, a.wrapping_add(OB_ROUTINE), r);
    }
    let modo = rb(m, V_GAMEMODE) & 0x7F;
    wb(m, V_GAMEMODE, modo);
    Ok(())
}

/// Confere antes de executar os objetos criados: os IDs sem código entram todos numa máscara
/// (diagnóstico da rodada), em vez de parar no primeiro.
fn objetos_faltando(m: &mut Memoria) -> Result<(), ErroAbertura> {
    let mut mascara = [0u128; 2];
    let mut a = V_OBJSPACE;
    while a < V_OBJSPACE_END {
        let id = rb(m, a.wrapping_add(OB_ID));
        if id != 0 && !super::objetos::objeto_existe(id) {
            let palavra = usize::from(id >> 7);
            if let Some(p) = mascara.get_mut(palavra) {
                *p |= 1u128.wrapping_shl(u32::from(id & 0x7F));
            }
        }
        a = a.wrapping_add(OBJECT_SIZE);
    }
    if mascara == [0, 0] {
        Ok(())
    } else {
        Err(ErroAbertura::ObjetosNaoImplementados(mascara))
    }
}
