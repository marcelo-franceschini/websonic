//! Fase da demo, fatia 1: o trecho até o laço dos cartões de título e o VBlank deles. ROM
//! sintética escrita por nós, com tabelas de valores inventados; nada aqui vem da ROM do jogo.

use sonic1_core::abertura::fase::DepoisDosCartoes;
use sonic1_core::abertura::objetos::execute_objects;
use sonic1_core::abertura::ram::*;
use sonic1_core::abertura::{ErroAbertura, animacao, desenho, fase, hud, plc};
use sonic1_core::memoria::{Memoria, PortaVdp, Saida};

const TAMANHO_ROM: usize = 0x8_0000;

fn rom_com(tabelas: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let mut rom = vec![0u8; TAMANHO_ROM];
    for (a, bytes) in tabelas {
        let i = usize::try_from(*a).unwrap();
        rom[i..i + bytes.len()].copy_from_slice(bytes);
    }
    rom
}

fn indice(a: u32) -> usize {
    usize::try_from(a - 0xFF_0000).unwrap()
}

fn pb(m: &mut Memoria, a: u32, v: u8) {
    m.ram_mut()[indice(a)] = v;
}

fn pw(m: &mut Memoria, a: u32, v: u16) {
    let i = indice(a);
    m.ram_mut()[i..i + 2].copy_from_slice(&v.to_be_bytes());
}

fn pl(m: &mut Memoria, a: u32, v: u32) {
    let i = indice(a);
    m.ram_mut()[i..i + 4].copy_from_slice(&v.to_be_bytes());
}

fn lb(m: &Memoria, a: u32) -> u8 {
    m.ram()[indice(a)]
}

fn lw(m: &Memoria, a: u32) -> u16 {
    let i = indice(a);
    u16::from_be_bytes([m.ram()[i], m.ram()[i + 1]])
}

fn ll(m: &Memoria, a: u32) -> u32 {
    let i = indice(a);
    u32::from_be_bytes(m.ram()[i..i + 4].try_into().unwrap())
}

/// Escritas na VDP, na ordem, como (é controle, palavra).
fn vdp(m: &mut Memoria) -> Vec<(bool, u16)> {
    m.tirar_saidas()
        .into_iter()
        .filter_map(|s| match s {
            Saida::Vdp { porta, valor, .. } => Some((porta == PortaVdp::Controle, valor)),
            _ => None,
        })
        .collect()
}

fn palavras(ws: &[u16]) -> Vec<u8> {
    ws.iter().flat_map(|w| w.to_be_bytes()).collect()
}

#[test]
fn add_plc_soma_a_lista_depois_do_que_ja_esta_na_fila() {
    // Lista 1: duas entradas. Lista 2: contagem negativa (vazia).
    let mut cues = palavras(&[0x0000, 0x0010, 0x0030]);
    cues.resize(0x10, 0);
    cues.extend(palavras(&[
        0x0001, 0x0005, 0x0000, 0x2000, 0x0006, 0x0000, 0x3000,
    ]));
    cues.resize(0x30, 0);
    cues.extend(palavras(&[0xFFFF]));
    let mut m = Memoria::nova(rom_com(&[(ART_LOAD_CUES, cues)]));
    pl(&mut m, V_PLC_BUFFER, 0x0004_0000);
    pw(&mut m, V_PLC_BUFFER + 4, 0x1234);
    plc::add_plc(&mut m, 1).unwrap();
    assert_eq!(ll(&m, V_PLC_BUFFER), 0x0004_0000, "a entrada antiga fica");
    assert_eq!(lw(&m, V_PLC_BUFFER + 4), 0x1234);
    assert_eq!(ll(&m, V_PLC_BUFFER + 6), 0x0005_0000);
    assert_eq!(lw(&m, V_PLC_BUFFER + 10), 0x2000);
    assert_eq!(ll(&m, V_PLC_BUFFER + 12), 0x0006_0000);
    assert_eq!(lw(&m, V_PLC_BUFFER + 16), 0x3000);
    plc::add_plc(&mut m, 2).unwrap();
    assert_eq!(
        ll(&m, V_PLC_BUFFER + 18),
        0,
        "lista vazia não acrescenta nada"
    );
}

/// Arte dos tiles animados com bytes que dizem de onde vieram (o índice do byte na tabela).
fn rom_dos_tiles_animados() -> Vec<u8> {
    let sequencia = |n: usize| -> Vec<u8> { (0..n).map(|i| (i % 251) as u8).collect() };
    rom_com(&[
        (ART_GHZWATER, sequencia(2 * 8 * 32)),
        (ART_GHZFLOWER1, sequencia(2 * 16 * 32)),
        (ART_GHZFLOWER2, sequencia(3 * 12 * 32)),
    ])
}

#[test]
fn tiles_animados_da_ghz_um_por_quadro_na_ordem_cachoeira_flor_grande_flores() {
    let mut m = Memoria::nova(rom_dos_tiles_animados());
    // Quadro 1: a cachoeira (temporizador em zero), 8 tiles para $6F00.
    animacao::animate_level_gfx(&mut m).unwrap();
    let e = vdp(&mut m);
    assert_eq!(e.len(), 2 + 8 * 16);
    assert_eq!(&e[..2], &[(true, 0x6F00), (true, 0x0001)]);
    assert_eq!(e[2], (false, 0x0001));
    assert_eq!((lb(&m, V_LANI0_TIME), lb(&m, V_LANI0_FRAME)), (5, 1));
    // Quadro 2: a flor grande, 16 tiles para $6B80; a cachoeira ainda espera.
    animacao::animate_level_gfx(&mut m).unwrap();
    let e = vdp(&mut m);
    assert_eq!(e.len(), 2 + 16 * 16);
    assert_eq!(&e[..2], &[(true, 0x6B80), (true, 0x0001)]);
    assert_eq!(lb(&m, V_LANI1_TIME), 15);
    // Quadro 3: as flores pequenas, quadro de arte 0, que dura 128 quadros.
    animacao::animate_level_gfx(&mut m).unwrap();
    let e = vdp(&mut m);
    assert_eq!(e.len(), 2 + 12 * 16);
    assert_eq!(&e[..2], &[(true, 0x6D80), (true, 0x0001)]);
    assert_eq!(lb(&m, V_LANI2_TIME), 127);
    // Quadros 4 a 6: nada; no 7 a cachoeira volta, com a arte do segundo quadro (+$100).
    for _ in 0..3 {
        animacao::animate_level_gfx(&mut m).unwrap();
        assert!(vdp(&mut m).is_empty());
    }
    animacao::animate_level_gfx(&mut m).unwrap();
    let e = vdp(&mut m);
    assert_eq!(
        e[2],
        (false, u16::from_be_bytes([5, 6])),
        "começa no byte $100"
    );
}

#[test]
fn tiles_animados_param_com_o_jogo_pausado() {
    let mut m = Memoria::nova(rom_dos_tiles_animados());
    pw(&mut m, F_PAUSE, 1);
    animacao::animate_level_gfx(&mut m).unwrap();
    assert!(vdp(&mut m).is_empty());
    assert_eq!(lb(&m, V_LANI0_TIME), 0);
}

/// Posições inventadas dos quatro elementos para a zona 0: (início, alvo).
const POSICOES: [u16; 8] = [0x010, 0x0A0, 0xFF00, 0x0C0, 0x300, 0x0E0, 0x200, 0x0E0];
/// Altura, rotina e quadro inventados de cada elemento.
const ITENS: [u8; 16] = [
    0x00, 0xC0, 2, 0, 0x00, 0xC8, 2, 5, 0x00, 0xD0, 2, 7, 0x00, 0xD8, 2, 9,
];

fn cartoes_de_teste() -> Memoria {
    let mut m = Memoria::nova(rom_com(&[
        (CARD_CONDATA, palavras(&POSICOES)),
        (CARD_ITEMDATA, ITENS.to_vec()),
    ]));
    // Zona 0, ato 1.
    pw(&mut m, V_ZONE_ACT, 0x0001);
    pb(&mut m, V_TITLECARD + OB_ID, ID_TITLE_CARD);
    m
}

#[test]
fn cartoes_criam_quatro_elementos_que_entram_no_mesmo_quadro() {
    let mut m = cartoes_de_teste();
    execute_objects(&mut m).unwrap();
    let elementos = [V_TITLECARD, 0xFF_D0C0, V_TTLCARDACT, 0xFF_D140];
    // Cada um anda 16 pixels rumo ao alvo: dois vêm da esquerda e dois da direita.
    let x_esperado = [0x020u16, 0xFF10, 0x2F0, 0x1F0];
    let quadro_esperado = [0u8, 5, 8, 9];
    for (k, a) in elementos.iter().enumerate() {
        assert_eq!(lb(&m, a + OB_ID), ID_TITLE_CARD, "elemento {k}");
        assert_eq!(lb(&m, a + OB_ROUTINE), 2);
        assert_eq!(lw(&m, a + OB_X), x_esperado[k], "x do elemento {k}");
        assert_eq!(lw(&m, a + CARD_FINALX), POSICOES[2 * k]);
        assert_eq!(lw(&m, a + CARD_MAINX), POSICOES[2 * k + 1]);
        assert_eq!(
            lb(&m, a + OB_FRAME),
            quadro_esperado[k],
            "quadro do elemento {k}"
        );
        assert_eq!(ll(&m, a + OB_MAP), MAP_CARD);
        assert_eq!(lw(&m, a + OB_GFX), ARTTILE_TITLE_CARD | TILE_PRIO);
        assert_eq!(lb(&m, a + OB_ACTWID), 120);
        assert_eq!(lw(&m, a + OB_TIME_FRAME), 60);
    }
    // Na tela (X de 0 a $1FF) só o primeiro e o último: a fila de prioridade 0 tem os dois.
    assert_eq!(lw(&m, V_SPRITEQUEUE), 4);
    assert_eq!(lw(&m, V_SPRITEQUEUE + 2), 0xD080);
    assert_eq!(lw(&m, V_SPRITEQUEUE + 4), 0xD140);
}

#[test]
fn laco_dos_cartoes_acaba_com_o_act_no_alvo_e_a_fila_vazia() {
    let mut m = Memoria::nova(rom_com(&[]));
    pw(&mut m, V_TTLCARDACT + OB_X, 0x154);
    pw(&mut m, V_TTLCARDACT + CARD_MAINX, 0x154);
    assert_eq!(
        fase::laco_dos_cartoes(&mut m).unwrap(),
        DepoisDosCartoes::Fim
    );
    // Arte ainda na fila (e em curso, para o `RunPLC` não começar outra): o laço continua.
    pl(&mut m, V_PLC_BUFFER, 0x0004_0000);
    pw(&mut m, V_PLC_PATTERNSLEFT, 3);
    assert_eq!(
        fase::laco_dos_cartoes(&mut m).unwrap(),
        DepoisDosCartoes::Continua
    );
}

#[test]
fn hud_sem_flags_nao_escreve_e_com_flag_para() {
    let mut m = Memoria::nova(rom_com(&[]));
    hud::hud_update(&mut m).unwrap();
    assert!(vdp(&mut m).is_empty());
    pb(&mut m, F_SCORECOUNT, 1);
    assert_eq!(
        hud::hud_update(&mut m),
        Err(ErroAbertura::NaoImplementado("números do HUD"))
    );
}

#[test]
fn coluna_da_direita_da_frente_desenha_dezesseis_blocos_no_plano_de_c000() {
    let mut m = Memoria::nova(rom_com(&[]));
    pb(&mut m, V_FG_SCROLL_FLAGS_DUP, 0x08);
    desenho::load_tiles_as_you_move(&mut m, V_JPADHOLD1 + 4).unwrap();
    let e = vdp(&mut m);
    assert_eq!(e.len(), 16 * 8);
    // Primeiro bloco: Y $F0 e X $140 no plano, base $4000. O segundo desce $100 e dá a volta
    // no plano ($1050 & $FFF).
    assert_eq!(&e[..2], &[(true, 0x4F50), (true, 0x0003)]);
    assert_eq!(&e[8..10], &[(true, 0x4050), (true, 0x0003)]);
    assert_eq!(lb(&m, V_FG_SCROLL_FLAGS_DUP), 0, "a flag foi limpa");
}
