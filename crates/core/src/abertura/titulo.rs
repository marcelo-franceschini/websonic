//! Tela do título (sub-etapa 7.5). Cada função cita o rótulo que reproduz.

use super::boot::{rom_b, rom_w};
use super::cenario;
use super::desenho;
use super::objetos::{build_sprites, execute_objects};
use super::ram::*;
use super::sega::{clear_screen, copy_tilemap, pal_load_para};
use super::{ErroAbertura, paleta, plc, rb, rw, wb, wl, ww};
use crate::memoria::Memoria;
use crate::rotinas;

fn rot(e: rotinas::ErroRotina) -> ErroAbertura {
    ErroAbertura::Rotina(e)
}

/// `locVRAM` para o endereço de uma tile (`tile` vezes o tamanho de 32 bytes).
pub(crate) fn loc_vram(tile: u32) -> u32 {
    let a = tile.wrapping_mul(32);
    0x4000_0000u32
        .wrapping_add((a & 0x3FFF) << 16)
        .wrapping_add((a & 0xC000) >> 14)
}

/// `clearRAM`: zera a RAM de `ini` até antes de `fim`, uma longa por vez.
pub(crate) fn zerar(m: &mut Memoria, ini: u32, fim: u32) {
    let mut a = ini;
    while a < fim {
        wl(m, a, 0);
        a = a.wrapping_add(4);
    }
}

/// `GM_Title`, da saída do primeiro desvanecimento até a chamada de `PaletteFadeIn`:
/// carrega o programa do Z80, a arte e os mapas do "SONIC TEAM PRESENTS" e cria o objeto.
pub fn gm_title_preparo_a(m: &mut Memoria) -> Result<(), ErroAbertura> {
    super::boot::dac_driver_load(m)?;
    for w in [0x8004u16, 0x8230, 0x8407, 0x9001, 0x9200, 0x8B03, 0x8720] {
        ww(m, VDP_CONTROLE, w);
    }
    wb(m, F_WTR_STATE, 0);
    clear_screen(m)?;
    zerar(m, V_OBJSPACE, V_OBJSPACE_END);
    wl(m, VDP_CONTROLE, loc_vram(0));
    rotinas::nem_dec(m, NEM_JAP_NAMES).map_err(rot)?;
    wl(
        m,
        VDP_CONTROLE,
        loc_vram(u32::from(ARTTILE_SONIC_TEAM_FONT)),
    );
    rotinas::nem_dec(m, NEM_CREDIT_TEXT).map_err(rot)?;
    rotinas::eni_dec(m, ENI_JAP_NAMES, V_RAM_START, 0).map_err(rot)?;
    copy_tilemap(m, V_RAM_START, 0xC000, 40, 28);
    zerar(m, V_PALETTE_FADING, V_PALETTE_FADING_END);
    pal_load_para(m, 3, 0x80);
    wb(m, V_SONICTEAM.wrapping_add(OB_ID), ID_CREDITS_TEXT);
    execute_objects(m)?;
    build_sprites(m)
}

/// `PaletteFadeIn` (começo): pinta de preto a paleta ativa e pede o primeiro VBlank do fade.
pub fn palette_fade_in_inicio(m: &mut Memoria) {
    ww(m, V_PFADE_START, 0x003F);
    palette_fade_in_alt(m);
}

/// `PalFadeIn_Alt`: começo do aparecimento com a posição e o tamanho já em `v_pfade_start`.
pub fn palette_fade_in_alt(m: &mut Memoria) {
    let inicio = u32::from(rb(m, V_PFADE_START));
    let n = u32::from(rb(m, V_PFADE_START.wrapping_add(1)));
    let mut a0 = V_PALETTE.wrapping_add(inicio);
    for _ in 0..=n {
        ww(m, a0, 0);
        a0 = a0.wrapping_add(2);
    }
    wb(m, V_VBLANK_ROUTINE, super::vblank::ROTINA_PALETTE_FADE);
}

/// Parte B do título, fatia 1: do fim do aparecimento do "SONIC TEAM PRESENTS" até a
/// `PaletteFadeOut` seguinte (linhas 1952 a 1995 de `GM_Title`). Carrega as artes do título na
/// VRAM, a fonte do menu, as variáveis de sessão e o cenário da GHZ (tamanhos, deformação, blocos
/// e layout). O `RunPLC` de dentro do desvanecimento não faz nada: a fila de arte está vazia.
pub fn parte_b1(m: &mut Memoria) -> Result<(), ErroAbertura> {
    for (tile, origem) in [
        (ARTTILE_TITLE_FOREGROUND, NEM_TITLE_FG),
        (ARTTILE_TITLE_SONIC, NEM_TITLE_SONIC),
        (ARTTILE_TITLE_TRADEMARK, NEM_TITLE_TM),
    ] {
        wl(m, VDP_CONTROLE, loc_vram(u32::from(tile)));
        rotinas::nem_dec(m, origem).map_err(rot)?;
    }
    // `Art_Text`: fonte do menu de fases, sem compressão, uma palavra por vez na porta de dados.
    wl(
        m,
        VDP_CONTROLE,
        loc_vram(u32::from(ARTTILE_LEVEL_SELECT_FONT)),
    );
    let mut a = ART_TEXT;
    while a < ART_TEXT_END {
        let w = rom_w(m, a);
        ww(m, VDP_DADOS, w);
        a = a.wrapping_add(2);
    }
    // Variáveis de sessão. O `UnusedOptimization` da referência é 0, então `v_unused2` também.
    wb(m, V_LASTLAMP, 0);
    ww(m, V_DEBUGUSE, 0);
    ww(m, F_DEMO, 0);
    ww(m, V_UNUSED2, 0);
    // `id_GHZ_act1` é zero: a fase é a GHZ1.
    ww(m, V_ZONE_ACT, 0);
    ww(m, V_PCYC_TIME, 0);
    cenario::level_size_load(m)?;
    cenario::deform_layers(m)?;
    // Blocos 16x16 (EniDec, tile inicial 0) e blocos 256x256 (KosDec) da GHZ.
    rotinas::eni_dec(m, BLK16_GHZ, V_16X16, 0).map_err(rot)?;
    rotinas::kos_dec(m, BLK256_GHZ, V_256X256).map_err(rot)?;
    cenario::level_layout_load(m).map(|_| ())
}

/// Parte C do título (linhas 1996 a 2070 de `GM_Title`): limpa a tela, desenha o fundo da GHZ,
/// carrega o emblema, a arte da GHZ e a paleta, toca a música, cria os objetos do título,
/// enfileira a arte de `plcid_Main` e liga a tela. Termina na chamada de `PaletteFadeIn`.
pub fn parte_c(m: &mut Memoria) -> Result<(), ErroAbertura> {
    clear_screen(m)?;
    // `DrawChunks` com a posição do fundo, o layout de fundo e a base $6000 do plano de fundo.
    desenho::draw_chunks(m, V_BGSCREENPOSX, V_LVLLAYOUT_BG, 0x6000);
    // Emblema: EniDec para o buffer, depois a cópia para o plano A em (34, 22), a partir de $206.
    rotinas::eni_dec(m, ENI_TITLE, V_RAM_START, 0).map_err(rot)?;
    copy_tilemap(m, V_RAM_START, 0xC000 + 0x206, 34, 22);
    wl(m, VDP_CONTROLE, loc_vram(0));
    rotinas::nem_dec(m, NEM_GHZ_1ST).map_err(rot)?;
    pal_load_para(m, PALID_TITLE, 0x80);
    // `QueueSound2`: a música do título vai para a fila do driver.
    wb(m, V_SOUNDQUEUE1, BGM_TITLE);
    wb(m, F_DEBUGMODE, 0);
    ww(m, V_GENERICTIMER, 376);
    // `clearRAM v_sonicteam,v_sonicteam+object_size/2`: só metade do objeto (FixBugs = 0).
    zerar(m, V_SONICTEAM, V_SONICTEAM.wrapping_add(0x20));
    wb(m, V_TITLESONIC, ID_TITLE_SONIC);
    wb(m, V_PRESSSTART, ID_PSBTM);
    // REV01: o "TM" só entra com o bit 7 de `v_megadrive` ligado (`bpl .isjap`).
    if rb(m, V_MEGADRIVE) & 0x80 != 0 {
        wb(m, V_TITLETM, ID_PSBTM);
        wb(m, V_TITLETM.wrapping_add(OB_FRAME), 3);
    }
    wb(m, V_TTLSONICHIDE, ID_PSBTM);
    wb(m, V_TTLSONICHIDE.wrapping_add(OB_FRAME), 2);
    execute_objects(m)?;
    cenario::deform_layers(m)?;
    build_sprites(m)?;
    super::plc::new_plc(m, PLCID_MAIN);
    ww(m, V_TITLE_DCOUNT, 0);
    ww(m, V_TITLE_CCOUNT, 0);
    // `enable_display`: liga o bit 6 do registrador 1 guardado.
    let b = rw(m, V_VDP_BUFFER1);
    ww(m, VDP_CONTROLE, (b & 0xFF00) | (b & 0xFF) | 0x40);
    Ok(())
}

/// Começo de `PaletteFadeOut`: a paleta inteira (a partir de 0) e o pedido do primeiro VBlank do
/// desvanecimento. Os passos de cada VBlank ficam em [`super::Abertura`].
pub fn palette_fade_out_inicio(m: &mut Memoria) {
    ww(m, V_PFADE_START, 0x003F);
    wb(m, V_VBLANK_ROUTINE, super::vblank::ROTINA_PALETTE_FADE);
}

/// `FadeIn_AddColor`: sobe um passo de um canal da cor em `a0` rumo à cor-alvo em `a1`.
fn fade_in_add_color(m: &mut Memoria, a0: u32, a1: u32) -> (u32, u32) {
    let alvo = rw(m, a1);
    let atual = rw(m, a0);
    let a1 = a1.wrapping_add(2);
    if alvo == atual {
        return (a0.wrapping_add(2), a1);
    }
    let azul = atual.wrapping_add(0x200);
    if azul <= alvo {
        ww(m, a0, azul);
        return (a0.wrapping_add(2), a1);
    }
    let verde = atual.wrapping_add(0x020);
    if verde <= alvo {
        ww(m, a0, verde);
        return (a0.wrapping_add(2), a1);
    }
    ww(m, a0, atual.wrapping_add(2));
    (a0.wrapping_add(2), a1)
}

/// `FadeIn_FromBlack`: um passo do aparecimento da paleta (e da de água na Labyrinth).
pub fn fade_in_from_black(m: &mut Memoria) {
    let inicio = u32::from(rb(m, V_PFADE_START));
    let n = u32::from(rb(m, V_PFADE_START.wrapping_add(1)));
    let mut a0 = V_PALETTE.wrapping_add(inicio);
    let mut a1 = V_PALETTE_FADING.wrapping_add(inicio);
    for _ in 0..=n {
        (a0, a1) = fade_in_add_color(m, a0, a1);
    }
    if rb(m, V_ZONE) == 1 {
        let mut a0 = V_PALETTE_WATER.wrapping_add(inicio);
        let mut a1 = V_PALETTE_WATER_FADING.wrapping_add(inicio);
        for _ in 0..=n {
            (a0, a1) = fade_in_add_color(m, a0, a1);
        }
    }
}

/// Para onde o laço do título vai depois de um quadro.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepoisDoTitulo {
    /// Pede o VBlank do título e repete o laço.
    Continua,
    /// O temporizador acabou: `GotoDemo`.
    GotoDemo,
    /// O "Sonic" do objeto 0 passou de `$1C00`: volta à Sega (não acontece com o temporizador).
    VoltaSega,
    /// Start apertado: `Tit_ChkLevSel` e o começo do jogo.
    Jogar,
}

/// `Tit_MainLoop`, depois da espera: objetos, fundo, sprites, ciclo de paleta, fila de arte, o
/// "Sonic" do objeto 0 que puxa a câmera, os códigos e o temporizador.
pub fn laco_do_titulo(m: &mut Memoria) -> Result<DepoisDoTitulo, ErroAbertura> {
    execute_objects(m)?;
    cenario::deform_layers(m)?;
    build_sprites(m)?;
    paleta::pal_cycle_title(m);
    plc::run_plc(m)?;
    if andar_o_sonic(m) {
        return Ok(DepoisDoTitulo::VoltaSega);
    }
    codigos(m);
    // `Tit_ChkStartOrDemo`.
    if rw(m, V_GENERICTIMER) == 0 {
        return Ok(DepoisDoTitulo::GotoDemo);
    }
    if start_apertado(m) {
        return Ok(DepoisDoTitulo::Jogar);
    }
    Ok(DepoisDoTitulo::Continua)
}

/// `move.w (v_player+obX).w,d0`, `addq.w #2,d0`, `cmpi.w #$1C00,d0` e `blo`: o objeto 0 anda 2
/// pixels para a direita, e a câmera o segue. Diz se ele passou de `$1C00`.
fn andar_o_sonic(m: &mut Memoria) -> bool {
    let x = rw(m, V_PLAYER.wrapping_add(OB_X)).wrapping_add(2);
    ww(m, V_PLAYER.wrapping_add(OB_X), x);
    x >= 0x1C00
}

/// Confere se o Start foi pressionado ou esta sendo segurado.
fn start_apertado(m: &mut Memoria) -> bool {
    let press = rb(m, V_JPADPRESS1) & BTN_START;
    let hold = rb(m, V_JPADHOLD1) & BTN_START;
    (press | hold) != 0
}

/// `adda.w`: a palavra entra com sinal na soma do endereço.
fn estender(w: u16) -> u32 {
    u32::from_be_bytes(i32::from(i16::from_be_bytes(w.to_be_bytes())).to_be_bytes())
}

/// `Tit_ChkRegion` a `Tit_CountC` (FixBugs = 0): a entrada do código pelo direcional e a
/// contagem do botão C.
fn codigos(m: &mut Memoria) {
    let codigo = if rb(m, V_MEGADRIVE) & 0x80 != 0 {
        LEVSELCODE_US
    } else {
        LEVSELCODE_J
    };
    let feitos = rw(m, V_TITLE_DCOUNT);
    let esperado = rom_b(m, codigo.wrapping_add(estender(feitos)));
    let direcional = rb(m, V_JPADPRESS1) & BTN_DIR;
    if direcional == esperado {
        ww(m, V_TITLE_DCOUNT, feitos.wrapping_add(1));
        // O zero no fim da sequência quer dizer que o código entrou inteiro.
        if direcional == 0 {
            ativar_codigo(m);
        }
    } else if direcional != 0 && feitos != 9 {
        // `Tit_ResetCheat`: um botão errado recomeça a sequência.
        ww(m, V_TITLE_DCOUNT, 0);
    }
    if rb(m, V_JPADPRESS1) & BTN_C != 0 {
        let c = rw(m, V_TITLE_CCOUNT).wrapping_add(1);
        ww(m, V_TITLE_CCOUNT, c);
    }
}

/// `Tit_ActivateCheat` e `Tit_PlayRing`: o código liga a trapaça que a contagem do C escolhe (fora
/// do Japão, duas vezes C ou mais liga a câmera lenta e o modo de depuração) e toca o anel.
fn ativar_codigo(m: &mut Memoria) {
    let mut d1 = (rw(m, V_TITLE_CCOUNT) >> 1) & 3;
    if d1 != 0 && rb(m, V_MEGADRIVE) & 0x80 != 0 {
        d1 = 1;
        // `move.b d1,1(a0,d1.w)`: liga o byte depois do escolhido.
        wb(m, F_LEVSELCHEAT.wrapping_add(2), 1);
    }
    wb(m, F_LEVSELCHEAT.wrapping_add(u32::from(d1)), 1);
    wb(m, V_SOUNDQUEUE1, SFX_RING);
}

/// Para onde o laço de espera do `GotoDemo` vai depois de um quadro.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepoisDoGotoDemo {
    /// Pede o VBlank do título e repete o laço.
    Continua,
    /// A demo foi preparada: o `rts` volta ao `MainGameLoop`, que chama o modo novo.
    Demo,
    /// O "Sonic" do objeto 0 passou de `$1C00`: volta à Sega.
    VoltaSega,
    /// Start apertado durante a espera: `Tit_ChkLevSel`.
    Jogar,
}

/// `GotoDemo_PreDelayLoop`, depois da espera: fundo, ciclo de paleta da zona, fila de arte e o
/// objeto 0. No fim do temporizador de 30 quadros, prepara a demo.
pub fn goto_demo_passo(m: &mut Memoria) -> Result<DepoisDoGotoDemo, ErroAbertura> {
    cenario::deform_layers(m)?;
    paleta::palette_cycle(m)?;
    plc::run_plc(m)?;
    if andar_o_sonic(m) {
        return Ok(DepoisDoGotoDemo::VoltaSega);
    }
    if start_apertado(m) {
        return Ok(DepoisDoGotoDemo::Jogar);
    }
    if rw(m, V_GENERICTIMER) != 0 {
        return Ok(DepoisDoGotoDemo::Continua);
    }
    preparar_demo(m);
    Ok(DepoisDoGotoDemo::Demo)
}

/// Fim de `GotoDemo` (REV01): apaga a música, escolhe a fase da demo pela `Demo_Levels`, liga o
/// modo de demo e zera vidas, anéis, tempo e pontos.
fn preparar_demo(m: &mut Memoria) {
    wb(m, V_SOUNDQUEUE1, BGM_FADE);
    let numero = rw(m, V_DEMONUM) & 7;
    let fase = rom_w(
        m,
        DEMO_LEVELS.wrapping_add(u32::from(numero).wrapping_mul(2)),
    );
    ww(m, V_ZONE_ACT, fase);
    let proximo = rw(m, V_DEMONUM).wrapping_add(1);
    ww(m, V_DEMONUM, if proximo < 4 { proximo } else { 0 });
    ww(m, F_DEMO, 1);
    wb(m, V_GAMEMODE, ID_DEMO);
    // `$600` é o valor de mentira que leva à demo do estágio especial.
    if fase == 0x600 {
        wb(m, V_GAMEMODE, ID_SPECIAL);
        ww(m, V_ZONE_ACT, 0);
        wb(m, V_LASTSPECIAL, 0);
    }
    wb(m, V_LIVES, 3);
    ww(m, V_RINGS, 0);
    wl(m, V_TIME, 0);
    wl(m, V_SCORE, 0);
    // REV01: a vida extra seguinte vem nos 50000 pontos.
    wl(m, V_SCORELIFE, 5000);
}

/// `PlayLevel`: prepara o jogo para iniciar com controle normal do jogador.
pub fn play_level(m: &mut Memoria) {
    wb(m, V_GAMEMODE, ID_LEVEL);
    wb(m, V_LIVES, 3);
    ww(m, V_RINGS, 0);
    wl(m, V_TIME, 0);
    wl(m, V_SCORE, 0);
    wb(m, V_LASTSPECIAL, 0);
    wb(m, V_EMERALDS, 0);
    wl(m, V_EMLDLIST, 0);
    wl(m, V_EMLDLIST.wrapping_add(4), 0);
    wb(m, V_CONTINUES, 0);
    wl(m, V_SCORELIFE, 5000);
    ww(m, F_DEMO, 0);
    wb(m, V_SOUNDQUEUE1, BGM_FADE);
}

#[cfg(test)]
mod testes_play_level {
    use super::super::rl;
    use super::*;

    #[test]
    fn play_level_inicializa_parametros_do_jogo_normal() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        ww(&mut m, F_DEMO, 1);
        wb(&mut m, V_LIVES, 1);
        wl(&mut m, V_SCORE, 9999);
        play_level(&mut m);

        assert_eq!(rb(&mut m, V_GAMEMODE), ID_LEVEL);
        assert_eq!(rb(&mut m, V_LIVES), 3);
        assert_eq!(rw(&mut m, V_RINGS), 0);
        assert_eq!(rl(&mut m, V_TIME), 0);
        assert_eq!(rl(&mut m, V_SCORE), 0);
        assert_eq!(rb(&mut m, V_LASTSPECIAL), 0);
        assert_eq!(rb(&mut m, V_EMERALDS), 0);
        assert_eq!(rl(&mut m, V_EMLDLIST), 0);
        assert_eq!(rl(&mut m, V_EMLDLIST + 4), 0);
        assert_eq!(rb(&mut m, V_CONTINUES), 0);
        assert_eq!(rl(&mut m, V_SCORELIFE), 5000);
        assert_eq!(rw(&mut m, F_DEMO), 0);
        assert_eq!(rb(&mut m, V_SOUNDQUEUE1), BGM_FADE);
    }
}
