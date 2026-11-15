//! Modo de fase e de demo (`GM_Level`): o começo até `PaletteFadeOut`, o trecho que prepara os
//! cartões de título e o laço dos cartões (`Level_TtlCardLoop`). A carga do resto da fase entra
//! nas fatias seguintes.

use super::boot::{rom_b, rom_l, rom_w};
use super::objetos::{build_sprites, execute_objects};
use super::ram::*;
use super::sega::{clear_screen, pal_load, pal_load_para};
use super::titulo::{loc_vram, zerar};
use super::{
    ErroAbertura, cenario, desenho, hud, objpos, paleta, plc, rb, rl, rw, titulo, wb, wl, ww,
};
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
/// `Level_TtlCardLoop` (D7.10). A demo do final devolve `NaoImplementado`.
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
        super::labirinto::iniciar_agua(m);
    }
    // `Level_LoadPal`: 30 segundos de ar e a paleta do Sonic (para os cartões).
    ww(m, V_AIR, 30);
    pal_load(m, PALID_SONIC);
    if zona == ID_LZ {
        let id = if rb(m, V_ACT) == 3 { 0x10 } else { 0x0F };
        pal_load_para(m, id, 0xFFFF_FF80);
        if rb(m, V_LASTLAMP) != 0 {
            let estado = rb(m, 0xFF_FE53);
            wb(m, F_WTR_STATE, estado);
        }
    }
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
    let paleta = paleta_da_fase(zona_e_ato, rom_b(m, cabecalho.wrapping_add(15)));
    pal_load_para(m, u32::from(paleta), 0x80);
    let plc2 = rom_b(m, cabecalho.wrapping_add(4));
    if plc2 != 0 {
        plc::add_plc(m, u16::from(plc2))?;
    }
    Ok(a0)
}

/// Seleciona as exceções de paleta do `LevelDataLoad`.
fn paleta_da_fase(zona_e_ato: u16, padrao: u8) -> u8 {
    match zona_e_ato {
        ID_LZ_ACT4 => 0x0C,
        ID_SBZ_ACT2 | ID_FZ => 0x0E,
        _ => padrao,
    }
}

/// `ColIndexLoad`: o ponteiro do índice de colisão da zona.
fn col_index_load(m: &mut Memoria) {
    let zona = u32::from(rb(m, V_ZONE));
    let ponteiro = rom_l(m, COLPOINTERS.wrapping_add(zona.wrapping_mul(4)));
    wl(m, V_COLLINDEX, ponteiro);
}

/// `OscillateNumInit`: os valores iniciais da tabela de números que oscilam.
fn oscillate_num_init(m: &mut Memoria) {
    let mut a1 = V_OSCILLATE;
    for i in 0..33u32 {
        let valor = rom_w(m, 0x412E_u32.wrapping_add(i.wrapping_mul(2)));
        ww(m, a1, valor);
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
    super::labirinto::efeitos(m);
    wb(m, V_PLAYER.wrapping_add(OB_ID), ID_SONIC);
    wb(m, V_HUD.wrapping_add(OB_ID), ID_HUD);
    if rb(m, F_DEBUGCHEAT) != 0 && rb(m, V_JPADHOLD1) & 0x40 != 0 {
        wb(m, F_DEBUGMODE, 1);
    }
    ww(m, V_JPADHOLD2, 0);
    ww(m, V_JPADHOLD1, 0);
    if rb(m, V_ZONE) == ID_LZ {
        for (objeto, x) in [(0xFF_D780, 0x60), (0xFF_D7C0, 0x120)] {
            wb(m, objeto, 0x1B);
            ww(m, objeto.wrapping_add(OB_X), x);
        }
    }
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
    if rb(m, V_ZONE) == ID_LZ {
        let id = if rb(m, V_ACT) == 3 { 0x0D } else { 0x0B };
        pal_load_para(m, id, 0xFFFF_FF00);
    }
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

/// `MoveSonicInDemo` da REV01, inclusive o comportamento original do byte pressionado.
pub fn move_sonic_in_demo(m: &mut Memoria) -> Result<(), ErroAbertura> {
    let demo = rw(m, F_DEMO);
    if demo == 0 {
        return Ok(());
    }
    if demo & 0x8000 != 0 {
        return Err(ErroAbertura::NaoImplementado("demo dos créditos"));
    }
    if rb(m, V_JPADHOLD1) & BTN_START != 0 {
        wb(m, V_GAMEMODE, GM_TITLE_ID);
    }
    let indice = u32::from(rb(m, V_ZONE)).wrapping_mul(4);
    let base = rom_l(m, DEMO_DATA_PTR.wrapping_add(indice));
    let pos = base.wrapping_add(u32::from(rw(m, V_BTNPUSHTIME1)));
    let botoes = rom_b(m, pos);
    wb(m, V_JPADHOLD1, botoes);
    wb(m, V_JPADPRESS1, botoes);
    let anterior = rb(m, V_BTNPUSHTIME2);
    wb(m, V_BTNPUSHTIME2, anterior.wrapping_sub(1));
    if anterior == 0 {
        wb(m, V_BTNPUSHTIME2, rom_b(m, pos.wrapping_add(3)));
        let deslocamento = rw(m, V_BTNPUSHTIME1).wrapping_add(2);
        ww(m, V_BTNPUSHTIME1, deslocamento);
    }
    Ok(())
}

/// `Level_EndDemo`: começa o escurecimento lento após o término de uma demo regular.
pub fn demo_fade_inicio(m: &mut Memoria) {
    wb(m, V_GAMEMODE, 0);
    ww(m, V_GENERICTIMER, 60);
    ww(m, V_PFADE_START, 0x003F);
    ww(m, 0xFF_F794, 0);
}

/// `Level_FDLoop`: o VBlank já ocorreu; retorna verdadeiro quando o temporizador terminou.
pub fn demo_fade_passo(m: &mut Memoria) -> Result<bool, ErroAbertura> {
    move_sonic_in_demo(m)?;
    execute_objects(m)?;
    build_sprites(m)?;
    objpos::obj_pos_load(m)?;
    let intervalo = rw(m, 0xFF_F794).wrapping_sub(1);
    ww(m, 0xFF_F794, intervalo);
    if i16::from_be_bytes(intervalo.to_be_bytes()) < 0 {
        ww(m, 0xFF_F794, 2);
        super::sega::fade_out_to_black(m);
    }
    Ok(rw(m, V_GENERICTIMER) == 0)
}

/// `OscillateNumDo`: parâmetros lidos da ROM, na ordem e com os bits do original.
pub fn oscillate_num_do(m: &mut Memoria) {
    if rb(m, V_PLAYER.wrapping_add(OB_ROUTINE)) >= 6 {
        return;
    }
    let mut direcao = rw(m, V_OSCILLATE);
    for i in 0..16u32 {
        let valor_addr = V_OSCILLATE.wrapping_add(2).wrapping_add(i.wrapping_mul(4));
        let taxa_addr = valor_addr.wrapping_add(2);
        let ajuste = rom_w(m, OSCILLATE_SETTINGS.wrapping_add(i.wrapping_mul(4)));
        let limite = rom_b(
            m,
            OSCILLATE_SETTINGS
                .wrapping_add(i.wrapping_mul(4))
                .wrapping_add(3),
        );
        let mascara = 1u16.wrapping_shl(15u32.wrapping_sub(i));
        let descendo = direcao & mascara != 0;
        let taxa = if descendo {
            rw(m, taxa_addr).wrapping_sub(ajuste)
        } else {
            rw(m, taxa_addr).wrapping_add(ajuste)
        };
        ww(m, taxa_addr, taxa);
        let valor = rw(m, valor_addr).wrapping_add(taxa);
        ww(m, valor_addr, valor);
        let alto = valor.to_be_bytes()[0];
        if descendo && alto < limite {
            direcao &= !mascara;
        } else if !descendo && alto >= limite {
            direcao |= mascara;
        }
    }
    ww(m, V_OSCILLATE, direcao);
}

/// `SynchroAnimate`: atualiza os quatro relógios comuns de animação.
pub fn synchro_animate(m: &mut Memoria) {
    for (tempo, quadro, recarga, modulo, regressiva) in [
        (V_ANI0_TIME, V_ANI0_FRAME, 11u8, 8u8, true),
        (V_ANI1_TIME, V_ANI1_FRAME, 7, 4, false),
        (V_ANI2_TIME, V_ANI2_FRAME, 7, 6, false),
    ] {
        let t = rb(m, tempo).wrapping_sub(1);
        wb(m, tempo, t);
        if t & 0x80 != 0 {
            wb(m, tempo, recarga);
            let q = rb(m, quadro);
            let proximo = if regressiva {
                q.wrapping_sub(1) & 7
            } else {
                q.wrapping_add(1)
            };
            wb(m, quadro, if proximo >= modulo { 0 } else { proximo });
        }
    }
    let tempo = rb(m, V_ANI3_TIME);
    if tempo != 0 {
        let soma = rw(m, V_ANI3_BUF).wrapping_add(u16::from(tempo));
        ww(m, V_ANI3_BUF, soma);
        wb(m, V_ANI3_FRAME, (soma.rotate_left(7) & 3).to_be_bytes()[1]);
        wb(m, V_ANI3_TIME, tempo.wrapping_sub(1));
    }
}

/// `PauseGame`: pausa o jogo quando Start e pressionado, e retoma no Start seguinte.
pub fn pause_game(m: &mut Memoria) {
    if rb(m, V_LIVES) == 0 {
        ww(m, F_PAUSE, 0);
        return;
    }
    if rw(m, F_PAUSE) != 0 {
        if rb(m, V_JPADPRESS1) & BTN_START != 0 {
            ww(m, F_PAUSE, 0);
            wb(m, V_SOUNDQUEUE0, 0x80);
        }
        return;
    }
    if rb(m, V_JPADPRESS1) & BTN_START != 0 {
        ww(m, F_PAUSE, 1);
        wb(m, V_SOUNDQUEUE0, 0x7E);
    }
}

/// Um passo de `Level_MainLoop`, depois da interrupção de VBlank.
pub fn level_main_loop(m: &mut Memoria) -> Result<(), ErroAbertura> {
    pause_game(m);
    if rw(m, F_PAUSE) != 0 {
        build_sprites(m)?;
        return Ok(());
    }
    let quadros = rw(m, V_FRAMECOUNT).wrapping_add(1);
    ww(m, V_FRAMECOUNT, quadros);
    move_sonic_in_demo(m)?;
    super::labirinto::efeitos(m);
    execute_objects(m)?;
    if rw(m, F_RESTART) != 0 {
        return Ok(());
    }
    if rw(m, V_DEBUGUSE) != 0 || rb(m, V_PLAYER.wrapping_add(OB_ROUTINE)) < 6 {
        cenario::deform_layers(m)?;
    }
    build_sprites(m)?;
    objpos::obj_pos_load(m)?;
    paleta::palette_cycle(m)?;
    plc::run_plc(m)?;
    oscillate_num_do(m);
    synchro_animate(m);
    signpost_art_load(m);
    Ok(())
}

/// `SignpostArtLoad`: ao se aproximar do fim do ato, trava a borda e prepara a arte.
fn signpost_art_load(m: &mut Memoria) {
    if rw(m, V_DEBUGUSE) != 0 || rb(m, V_ZONE_ACT.wrapping_add(1)) == 2 {
        return;
    }
    let borda = rw(m, V_LIMITRIGHT2).wrapping_sub(0x100);
    let camera = rw(m, V_SCREENPOSX);
    if i16::from_be_bytes(camera.to_be_bytes()) < i16::from_be_bytes(borda.to_be_bytes())
        || rb(m, F_TIMECOUNT) == 0
        || rw(m, V_LIMITLEFT2) == borda
    {
        return;
    }
    ww(m, V_LIMITLEFT2, borda);
    plc::new_plc(m, 0x12);
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

#[cfg(test)]
mod testes_laco {
    use super::*;

    #[test]
    #[ignore = "requer SONIC1_ROM com a ROM REV01"]
    fn cartoes_labyrinth_inicializam_agua_e_paleta() {
        let caminho = std::env::var_os("SONIC1_ROM").expect("SONIC1_ROM");
        let mut m = Memoria::nova(std::fs::read(caminho).unwrap());
        wb(&mut m, V_ZONE, ID_LZ);
        preparo_dos_cartoes(&mut m).unwrap();
        assert_eq!(rb(&mut m, 0xFF_F64C), 1);
        assert_eq!(rw(&mut m, 0xFF_F646), rom_w(&mut m, 0x3C2C));
        let entrada = PAL_INDEX.wrapping_add(0x0F * 8);
        let fonte = rom_l(&mut m, entrada);
        let destino =
            (u32::from(rom_w(&mut m, entrada.wrapping_add(4))) | 0xFFFF_0000).wrapping_sub(0x80);
        assert_eq!(
            rl(&mut m, destino.wrapping_add(4)),
            rom_l(&mut m, fonte.wrapping_add(4))
        );
    }

    #[test]
    fn fade_da_demo_mantem_o_tempo_e_escurece_cada_tres_quadros() {
        let mut m = Memoria::nova(vec![0; 0x10000]);
        wb(&mut m, V_GAMEMODE, ID_DEMO);
        ww(&mut m, V_OPL_ROUTINE, 2);
        ww(&mut m, V_PALETTE, 0x0EEE);
        demo_fade_inicio(&mut m);
        assert_eq!(rb(&mut m, V_GAMEMODE), 0);
        assert_eq!(rw(&mut m, V_GENERICTIMER), 60);
        assert!(!demo_fade_passo(&mut m).unwrap());
        assert_eq!(rw(&mut m, V_PALETTE), 0x0EEC);
        demo_fade_passo(&mut m).unwrap();
        demo_fade_passo(&mut m).unwrap();
        assert_eq!(rw(&mut m, V_PALETTE), 0x0EEC);
        demo_fade_passo(&mut m).unwrap();
        assert_eq!(rw(&mut m, V_PALETTE), 0x0EEA);
        ww(&mut m, V_GENERICTIMER, 0);
        assert!(demo_fade_passo(&mut m).unwrap());
    }

    #[test]
    fn demo_rev01_atualiza_botoes_e_avanca_a_lista() {
        let mut rom = vec![0; 0x5000];
        let ptr = usize::try_from(DEMO_DATA_PTR).unwrap();
        rom[ptr..ptr + 4].copy_from_slice(&0x0000_4500u32.to_be_bytes());
        rom[0x4500..0x4504].copy_from_slice(&[8, 2, 0x10, 5]);
        let mut m = Memoria::nova(rom);
        ww(&mut m, F_DEMO, 1);
        wb(&mut m, V_BTNPUSHTIME2, 1);
        move_sonic_in_demo(&mut m).unwrap();
        assert_eq!(rb(&mut m, V_JPADHOLD1), 8);
        assert_eq!(rb(&mut m, V_JPADPRESS1), 8);
        assert_eq!(rb(&mut m, V_BTNPUSHTIME2), 0);
        move_sonic_in_demo(&mut m).unwrap();
        assert_eq!(rw(&mut m, V_BTNPUSHTIME1), 2);
        assert_eq!(rb(&mut m, V_BTNPUSHTIME2), 5);
        move_sonic_in_demo(&mut m).unwrap();
        assert_eq!(rb(&mut m, V_JPADHOLD1), 0x10);
        assert_eq!(rb(&mut m, V_JPADPRESS1), 0x10);
    }

    #[test]
    fn oscilacao_usa_parametros_da_rom_e_inverte_direcao() {
        let mut rom = vec![0; 0x5000];
        let i = usize::try_from(OSCILLATE_SETTINGS).unwrap();
        rom[i..i + 4].copy_from_slice(&[0, 2, 0, 1]);
        let mut m = Memoria::nova(rom);
        ww(&mut m, V_OSCILLATE + 2, 0x00FF);
        oscillate_num_do(&mut m);
        assert_eq!(rw(&mut m, V_OSCILLATE + 2), 0x0101);
        assert_eq!(rw(&mut m, V_OSCILLATE + 4), 2);
        assert_ne!(rw(&mut m, V_OSCILLATE) & 0x8000, 0);
        oscillate_num_do(&mut m);
        assert_eq!(rw(&mut m, V_OSCILLATE + 4), 0);
    }

    #[test]
    fn animacoes_sincronas_respeitam_periodos_e_limites() {
        let mut m = Memoria::nova(vec![]);
        synchro_animate(&mut m);
        assert_eq!(rb(&mut m, V_ANI0_FRAME), 7);
        assert_eq!(rb(&mut m, V_ANI1_FRAME), 1);
        assert_eq!(rb(&mut m, V_ANI2_FRAME), 1);
        for _ in 0..7 {
            synchro_animate(&mut m);
        }
        assert_eq!(rb(&mut m, V_ANI1_FRAME), 1);
        synchro_animate(&mut m);
        assert_eq!(rb(&mut m, V_ANI1_FRAME), 2);
    }

    #[test]
    fn jogo_normal_preserva_botoes_do_jogador_e_permite_pausa() {
        let mut m = Memoria::nova(vec![]);
        ww(&mut m, F_DEMO, 0);
        wb(&mut m, V_JPADHOLD1, 0x08);
        wb(&mut m, V_JPADPRESS1, 0x08);
        move_sonic_in_demo(&mut m).unwrap();
        assert_eq!(rb(&mut m, V_JPADHOLD1), 0x08);
        assert_eq!(rb(&mut m, V_JPADPRESS1), 0x08);

        // Teste de pausa
        wb(&mut m, V_LIVES, 3);
        wb(&mut m, V_JPADPRESS1, BTN_START);
        pause_game(&mut m);
        assert_eq!(rw(&mut m, F_PAUSE), 1);
        assert_eq!(rb(&mut m, V_SOUNDQUEUE0), 0x7E);

        // Sem start continua pausado
        wb(&mut m, V_JPADPRESS1, 0);
        pause_game(&mut m);
        assert_eq!(rw(&mut m, F_PAUSE), 1);

        // Com start despausa
        wb(&mut m, V_JPADPRESS1, BTN_START);
        pause_game(&mut m);
        assert_eq!(rw(&mut m, F_PAUSE), 0);
        assert_eq!(rb(&mut m, V_SOUNDQUEUE0), 0x80);
    }

    #[test]
    fn f_restart_sinaliza_reinicio_sem_falha() {
        let mut m = Memoria::nova(vec![]);
        ww(&mut m, F_RESTART, 1);
        assert!(level_main_loop(&mut m).is_ok());
    }
}
