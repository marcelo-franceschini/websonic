//! Uma rotina que atravessa o fim de vários quadros devolve todos os eventos, não só os do
//! último quadro (falha encontrada no bloco B da Etapa 5: sequências de desvio truncadas).

use sonic1_diferencial::{ENDERECO_TRECHO, SENTINELA, rom_com_trecho};
use sonic1_fidelidade::Tipo;
use websonic_oracle::oraculo::ChamadaRotina;
use websonic_oracle::rom_sintetica::SSP;
use websonic_oracle::{Entradas, Oraculo};

#[test]
fn rotina_longa_devolve_os_eventos_de_todos_os_quadros() {
    // move.w #39999,d0 ; dbf d0,* ; rts. Dura mais de dois quadros.
    let rom = rom_com_trecho(&[0x303C, 39_999, 0x51C8, 0xFFFE]);
    let mut o = Oraculo::novo(rom, Entradas::default()).expect("oraculo");
    let mut regs = o.emulador().oracle_regs68();
    regs.pc = ENDERECO_TRECHO;
    regs.ssp = SSP;
    regs.sr |= 0x2000;
    let r = o
        .chamar_rotina(&ChamadaRotina {
            regs,
            pokes_ram68: Vec::new(),
            sentinela: SENTINELA,
            mascarar_interrupcoes: true,
            max_passos: 1_000_000,
        })
        .expect("rotina");
    let dbcc = r
        .eventos
        .iter()
        .filter(|e| e.tipo == Tipo::Desvio68 && e.c == 4)
        .count();
    // 39999 voltas tomadas e uma final nao tomada.
    assert_eq!(dbcc, 40_000);
}
