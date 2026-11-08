//! Inventario de instrucoes e de barramento, e arquivo de tempo (Etapa 5). ROM sintetica.

use sonic1_core::quadro::tempo::EventoTempo;
use websonic_oracle::rom_sintetica::{RomSintetica, SSP, programa_vblank};
use websonic_oracle::{Entradas, Oraculo};

fn contar(mapa: &[(String, String, u64)], nome: &str, tamanho: &str) -> u64 {
    mapa.iter()
        .find(|(n, t, _)| n == nome && t == tamanho)
        .map_or(0, |(_, _, c)| *c)
}

#[test]
fn histograma_de_instrucoes_conta_classe_e_tamanho() {
    oracle_trace::instrucoes_ligar();
    let mut rom = RomSintetica::nova(SSP, 0x200);
    rom.palavras(
        0x200,
        &[
            0x7005, // moveq #5,d0
            0xD041, // add.w d1,d0
            0xD001, // add.b d1,d0
            0xD001, // add.b d1,d0
            0x60FE, // bra.s *   (repete ate o fim do quadro)
        ],
    );
    let mut o = Oraculo::novo(rom.finalizar(), Entradas::default()).unwrap();
    o.executar_quadro().unwrap();
    let mapa = oracle_trace::mapa_instrucoes();
    assert_eq!(contar(&mapa, "Add", "Word"), 1);
    assert_eq!(contar(&mapa, "Add", "Byte"), 2);
    assert!(
        contar(&mapa, "BranchAlways", "-") + contar(&mapa, "Branch", "-") > 100,
        "{mapa:?}"
    );
    // Nenhum operando nem valor aparece nas linhas.
    assert!(
        mapa.iter()
            .all(|(n, t, _)| n.chars().all(|c| c.is_ascii_alphanumeric()) && t.len() < 12)
    );
}

#[test]
fn barramento_conta_regiao_e_excecao() {
    oracle_trace::barramento_ligar();
    let mut rom = RomSintetica::nova(SSP, 0x200);
    rom.palavras(
        0x200,
        &[
            0x13FC, 0x0001, 0x00FF, 0x0000, // move.b #1,$FF0000      (ram, escrita)
            0x1039, 0x00A1, 0x0001, // move.b $A10001,d0              (io, leitura)
            0x33FC, 0x8F02, 0x00C0, 0x0004, // move.w #$8F02,$C00004  (vdp, escrita)
            0x1039, 0x0050, 0x0000, // move.b $500000,d0              (sem dispositivo)
            0x33FC, 0x0001, 0x0000, 0x1000, // move.w #1,$1000       (rom, escrita)
            0x60FE,
        ],
    );
    let mut o = Oraculo::novo(rom.finalizar(), Entradas::default()).unwrap();
    o.executar_quadro().unwrap();
    let mapa = oracle_trace::mapa_barramento();
    let tem = |r: &str, op: &str, b: &str| {
        mapa.iter()
            .any(|(rr, oo, bb, _)| rr == r && oo == op && bb == b)
    };
    assert!(tem("ram", "escrita", "1"), "{mapa:?}");
    assert!(tem("io", "leitura", "1"));
    assert!(tem("vdp", "escrita", "2"));
    assert!(tem("sem_dispositivo", "leitura", "1"));
    assert!(tem("rom", "escrita", "2"));
}

#[test]
fn arquivo_de_tempo_tem_interrupcao_e_orcamento_por_quadro() {
    oracle_trace::tempo_ligar();
    let mut o = Oraculo::novo(programa_vblank(), Entradas::default()).unwrap();
    let mut anterior = 0;
    let mut ints = 0;
    for n in 0..4u32 {
        let q = o.executar_quadro().unwrap();
        let mut brutos = Vec::new();
        oracle_trace::drain_tempo(&mut brutos);
        let nivel6 = brutos
            .iter()
            .filter(|r| matches!(r, oracle_trace::TempoRaw::Interrupcao { nivel: 6, .. }))
            .count();
        ints += nivel6;
        assert!(q.mclk_fim > anterior, "quadro {n}");
        anterior = q.mclk_fim;
    }
    // O VBlank chega uma vez por quadro depois que a VDP e ligada.
    assert!(ints >= 3, "interrupcoes de nivel 6: {ints}");
    // O tipo usado no arquivo existe no formato do nucleo.
    let _ = EventoTempo::EsperaZ80 { ciclos: 0, mclk: 0 };
}
