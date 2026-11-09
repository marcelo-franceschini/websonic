//! Fatia 3 do título (7.5): o desenho do fundo durante a rolagem, o ciclo de paleta, o laço do
//! título com a entrada do código e o `GotoDemo`. ROM sintética escrita por nós, com tabelas de
//! valores inventados; nada aqui vem da ROM do jogo.

use sonic1_core::abertura::ram::*;
use sonic1_core::abertura::titulo::{DepoisDoGotoDemo, DepoisDoTitulo};
use sonic1_core::abertura::{desenho, paleta, titulo};
use sonic1_core::memoria::{Memoria, PortaVdp, Saida};

const TAMANHO_ROM: usize = 0x8_0000;

fn rom_com(tabelas: &[(u32, &[u8])]) -> Vec<u8> {
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

/// Fundo com o bloco 2 em x = 16 e y = 0, e o chunk 1 na coluna de chunks 1 da primeira linha.
/// Os blocos da coluna x = 336 (320 + 16) nas linhas 112, 128 e 144 são 5, 6 e 7.
fn fundo_de_teste() -> Memoria {
    let mut m = Memoria::nova(rom_com(&[]));
    pw(&mut m, V_BG2SCREENPOSX, 16);
    pw(&mut m, V_BG2SCREENPOSX + 4, 0);
    pb(&mut m, V_LVLLAYOUT_BG + 1, 1);
    // Palavra do bloco no chunk: `(y * 2) & $1E0` mais `(x >> 3) & $1E`.
    for (dy, id) in [(0xE0u32, 5u16), (0x100, 6), (0x120, 7)] {
        pw(&mut m, V_256X256 + dy + 0x0A, id);
    }
    for (id, topo, base) in [
        (5u32, 0x1111_2222u32, 0x3333_4444u32),
        (6, 0x5555_6666, 0x7777_8888),
        (7, 0x9999_AAAA, 0xBBBB_CCCC),
    ] {
        pl(&mut m, V_16X16 + id * 8, topo);
        pl(&mut m, V_16X16 + id * 8 + 4, base);
    }
    m
}

#[test]
fn coluna_da_direita_do_bloco_2_desenha_tres_blocos_e_limpa_a_flag() {
    let mut m = fundo_de_teste();
    pb(&mut m, V_BG2_SCROLL_FLAGS, 0x02);
    desenho::load_tiles_as_you_move_bg_only(&mut m, V_JPADHOLD1 + 4).unwrap();
    // Plano em ($70 << 4) + ($150 >> 2) = $754, com a base $6000 e o comando trocado de palavra.
    // Cada bloco desce $100 no plano; a segunda linha do bloco soma $80 na palavra alta.
    let esperado = vec![
        (true, 0x6754),
        (true, 0x0003),
        (false, 0x1111),
        (false, 0x2222),
        (true, 0x67D4),
        (true, 0x0003),
        (false, 0x3333),
        (false, 0x4444),
        (true, 0x6854),
        (true, 0x0003),
        (false, 0x5555),
        (false, 0x6666),
        (true, 0x68D4),
        (true, 0x0003),
        (false, 0x7777),
        (false, 0x8888),
        (true, 0x6954),
        (true, 0x0003),
        (false, 0x9999),
        (false, 0xAAAA),
        (true, 0x69D4),
        (true, 0x0003),
        (false, 0xBBBB),
        (false, 0xCCCC),
    ];
    assert_eq!(vdp(&mut m), esperado);
    assert_eq!(lb(&m, V_BG2_SCROLL_FLAGS), 0, "a flag foi limpa");
}

#[test]
fn sem_flags_o_fundo_nao_desenha_nada() {
    let mut m = fundo_de_teste();
    desenho::load_tiles_as_you_move_bg_only(&mut m, V_JPADHOLD1 + 4).unwrap();
    assert!(vdp(&mut m).is_empty());
}

#[test]
fn chunk_zero_usa_as_flags_do_a0_que_o_chamador_deu() {
    // Sem chunk no layout, o bloco é o 0 de `v_16x16` e as flags vêm de `(a0)`. Aqui o byte em
    // `a0` liga o espelhamento horizontal: os dois tiles trocam de lugar e ganham o bit $800.
    let mut m = fundo_de_teste();
    pb(&mut m, V_LVLLAYOUT_BG + 1, 0);
    pl(&mut m, V_16X16, 0x0001_0002);
    pl(&mut m, V_16X16 + 4, 0x0003_0004);
    pb(&mut m, V_JPADHOLD1 + 4, 0x08);
    pb(&mut m, V_BG2_SCROLL_FLAGS, 0x02);
    desenho::load_tiles_as_you_move_bg_only(&mut m, V_JPADHOLD1 + 4).unwrap();
    let escritas = vdp(&mut m);
    assert_eq!(&escritas[2..4], &[(false, 0x0802), (false, 0x0801)]);
    assert_eq!(&escritas[6..8], &[(false, 0x0804), (false, 0x0803)]);
}

#[test]
fn ciclo_da_agua_do_titulo_anda_um_passo_a_cada_seis_quadros() {
    let tabela: Vec<u8> = (0u8..32).collect();
    let mut m = Memoria::nova(rom_com(&[(PAL_TITLECYC_WATER, &tabela)]));
    let destino = V_PALETTE_LINE_3 + 0x10;
    // Contador em zero: o primeiro quadro já passa a -1, copia o passo 0 e volta a 5.
    paleta::pal_cycle_title(&mut m);
    assert_eq!(ll(&m, destino), 0x0001_0203);
    assert_eq!(ll(&m, destino + 4), 0x0405_0607);
    assert_eq!(lw(&m, V_PCYC_TIME), 5);
    assert_eq!(lw(&m, V_PCYC_NUM), 1);
    for _ in 0..5 {
        paleta::pal_cycle_title(&mut m);
    }
    assert_eq!(ll(&m, destino), 0x0001_0203, "ainda não é hora");
    paleta::pal_cycle_title(&mut m);
    assert_eq!(ll(&m, destino), 0x0809_0A0B);
    // O número do passo só usa os dois bits baixos: depois do passo 3 volta ao 0.
    pw(&mut m, V_PCYC_NUM, 7);
    pw(&mut m, V_PCYC_TIME, 0);
    paleta::pal_cycle_title(&mut m);
    assert_eq!(ll(&m, destino), 0x1819_1A1B);
}

/// Sequência de código inventada para o teste: direita, esquerda, baixo, cima, fim.
const CODIGO: [u8; 6] = [0x08, 0x04, 0x02, 0x01, 0x00, 0xFF];

fn titulo_de_teste(regiao: u8) -> Memoria {
    let mut m = Memoria::nova(rom_com(&[
        (LEVSELCODE_US, &CODIGO),
        (LEVSELCODE_J, &CODIGO),
    ]));
    pb(&mut m, V_MEGADRIVE, regiao);
    pw(&mut m, V_GENERICTIMER, 100);
    m
}

fn quadro_do_titulo(m: &mut Memoria, botoes: u8) -> DepoisDoTitulo {
    pb(m, V_JPADPRESS1, botoes);
    titulo::laco_do_titulo(m).unwrap()
}

#[test]
fn laco_do_titulo_anda_o_objeto_0_e_continua() {
    let mut m = titulo_de_teste(0x80);
    pw(&mut m, V_PLAYER + OB_X, 0x50);
    assert_eq!(quadro_do_titulo(&mut m, 0), DepoisDoTitulo::Continua);
    assert_eq!(lw(&m, V_PLAYER + OB_X), 0x52);
}

#[test]
fn codigo_inteiro_liga_a_selecao_de_fases_e_toca_o_anel() {
    let mut m = titulo_de_teste(0x80);
    for (i, b) in CODIGO[..4].iter().enumerate() {
        assert_eq!(quadro_do_titulo(&mut m, *b), DepoisDoTitulo::Continua);
        assert_eq!(usize::from(lw(&m, V_TITLE_DCOUNT)), i + 1);
    }
    // Um quadro sem direcional casa com o zero do fim da sequência.
    assert_eq!(quadro_do_titulo(&mut m, 0), DepoisDoTitulo::Continua);
    assert_eq!(lw(&m, V_TITLE_DCOUNT), 5);
    assert_eq!(lb(&m, F_LEVSELCHEAT), 1);
    assert_eq!(lb(&m, V_SOUNDQUEUE1), SFX_RING);
}

#[test]
fn botao_errado_recomeca_o_codigo_e_quadro_sem_botao_nao() {
    let mut m = titulo_de_teste(0x80);
    quadro_do_titulo(&mut m, CODIGO[0]);
    quadro_do_titulo(&mut m, 0);
    assert_eq!(lw(&m, V_TITLE_DCOUNT), 1, "sem direcional a sequência fica");
    quadro_do_titulo(&mut m, CODIGO[2]);
    assert_eq!(lw(&m, V_TITLE_DCOUNT), 0, "direcional errado recomeça");
}

#[test]
fn fora_do_japao_codigo_com_c_duas_vezes_liga_camera_lenta_e_depuracao() {
    let mut m = titulo_de_teste(0x80);
    pw(&mut m, V_TITLE_CCOUNT, 2);
    for b in CODIGO[..5].iter() {
        quadro_do_titulo(&mut m, *b);
    }
    assert_eq!(lb(&m, F_LEVSELCHEAT), 0);
    assert_eq!(lb(&m, F_LEVSELCHEAT + 1), 1);
    assert_eq!(lb(&m, F_LEVSELCHEAT + 2), 1);
}

#[test]
fn botao_c_conta_e_start_comeca_o_jogo() {
    let mut m = titulo_de_teste(0x00);
    quadro_do_titulo(&mut m, BTN_C);
    assert_eq!(lw(&m, V_TITLE_CCOUNT), 1);
    assert_eq!(quadro_do_titulo(&mut m, BTN_START), DepoisDoTitulo::Jogar);
}

#[test]
fn temporizador_zerado_vai_para_a_demo() {
    let mut m = titulo_de_teste(0x80);
    pw(&mut m, V_GENERICTIMER, 0);
    assert_eq!(quadro_do_titulo(&mut m, 0), DepoisDoTitulo::GotoDemo);
}

/// Fases das demos, inventadas, com o `$600` que leva ao estágio especial no fim.
const FASES_DAS_DEMOS: [u8; 8] = [0x00, 0x01, 0x01, 0x02, 0x02, 0x03, 0x06, 0x00];

fn goto_demo_de_teste(numero: u16, tempo: u16) -> Memoria {
    let mut m = Memoria::nova(rom_com(&[(DEMO_LEVELS, &FASES_DAS_DEMOS)]));
    pw(&mut m, V_DEMONUM, numero);
    pw(&mut m, V_GENERICTIMER, tempo);
    pw(&mut m, V_PLAYER + OB_X, 0x100);
    m
}

#[test]
fn espera_do_goto_demo_continua_enquanto_ha_tempo() {
    let mut m = goto_demo_de_teste(0, 5);
    assert_eq!(
        titulo::goto_demo_passo(&mut m).unwrap(),
        DepoisDoGotoDemo::Continua
    );
    assert_eq!(lw(&m, V_PLAYER + OB_X), 0x102);
    assert_eq!(lb(&m, V_GAMEMODE), 0, "a demo ainda não foi preparada");
}

#[test]
fn fim_do_goto_demo_prepara_a_demo() {
    let mut m = goto_demo_de_teste(0, 0);
    assert_eq!(
        titulo::goto_demo_passo(&mut m).unwrap(),
        DepoisDoGotoDemo::Demo
    );
    assert_eq!(lb(&m, V_SOUNDQUEUE1), BGM_FADE);
    assert_eq!(lw(&m, V_ZONE_ACT), 0x0001);
    assert_eq!(lw(&m, V_DEMONUM), 1);
    assert_eq!(lw(&m, F_DEMO), 1);
    assert_eq!(lb(&m, V_GAMEMODE), ID_DEMO);
    assert_eq!(lb(&m, V_LIVES), 3);
    assert_eq!(ll(&m, V_SCORELIFE), 5000);
}

#[test]
fn ultima_demo_e_o_estagio_especial_e_o_numero_volta_a_zero() {
    let mut m = goto_demo_de_teste(3, 0);
    titulo::goto_demo_passo(&mut m).unwrap();
    assert_eq!(lb(&m, V_GAMEMODE), ID_SPECIAL);
    assert_eq!(lw(&m, V_ZONE_ACT), 0);
    assert_eq!(lw(&m, V_DEMONUM), 0);
}
