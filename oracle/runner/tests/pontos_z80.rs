//! Pontos do Z80 no arquivo de tempo (Etapa 6, patch 0005). ROM sintética escrita por nós.
//!
//! O programa 3 da ROM sintética carrega um programa no Z80 pelo 68000 (18 escritas na janela),
//! escreve duas vezes no YM2612 pela janela, liga e solta o reset e devolve o barramento.

use oracle_trace::{TempoRaw, acesso_z80};
use websonic_oracle::rom_sintetica::programa_z80;
use websonic_oracle::{Entradas, Oraculo};

#[derive(Clone, Copy, Debug)]
struct Ponto {
    acesso: u8,
    pc: u16,
    espera: u16,
    ativos: u32,
    banco: u16,
    endereco: u16,
    byte: u8,
}

fn pontos() -> Vec<Ponto> {
    oracle_trace::tempo_ligar();
    let mut o = Oraculo::novo(programa_z80(), Entradas::default()).unwrap();
    let mut saida = Vec::new();
    for _ in 0..6 {
        o.executar_quadro().unwrap();
        let mut brutos = Vec::new();
        oracle_trace::drain_tempo(&mut brutos);
        for r in brutos {
            if let TempoRaw::Z80Ponto {
                acesso,
                pc,
                espera,
                ativos,
                banco,
                endereco,
                byte,
                ..
            } = r
            {
                saida.push(Ponto {
                    acesso,
                    pc,
                    espera,
                    ativos,
                    banco,
                    endereco,
                    byte,
                });
            }
        }
    }
    saida
}

#[test]
fn cada_acesso_do_68000_ao_z80_vira_um_ponto_na_ordem() {
    let p = pontos();
    let tipos: Vec<u8> = p.iter().map(|x| x.acesso).collect();
    // reset solto, busreq pedido, 18 escritas do programa, 2 escritas no YM, reset ligado,
    // reset solto, busreq liberado.
    let mut esperado = vec![acesso_z80::RESET_SOLTO, acesso_z80::BUSREQ_PEDIDO];
    esperado.extend(std::iter::repeat_n(acesso_z80::ESCRITA_JANELA, 20));
    esperado.extend([
        acesso_z80::RESET_LIGADO,
        acesso_z80::RESET_SOLTO,
        acesso_z80::BUSREQ_LIBERADO,
    ]);
    assert_eq!(tipos, esperado);
}

#[test]
fn escritas_da_janela_levam_endereco_e_byte() {
    let p = pontos();
    let janela: Vec<&Ponto> = p
        .iter()
        .filter(|x| x.acesso == acesso_z80::ESCRITA_JANELA)
        .collect();
    // O programa do Z80 (18 bytes) vai de $0000 a $0011; depois $30 em $4000 e $55 em $4001.
    assert_eq!(janela[0].endereco, 0);
    assert_eq!(janela[0].byte, 0x21);
    assert_eq!(janela[17].endereco, 17);
    assert_eq!(janela[17].byte, 0xFE);
    assert_eq!((janela[18].endereco, janela[18].byte), (0x4000, 0x30));
    assert_eq!((janela[19].endereco, janela[19].byte), (0x4001, 0x55));
    // Nenhum programa escreveu no registrador de banco.
    assert!(p.iter().all(|x| x.banco == 0));
}

#[test]
fn z80_parado_pelo_barramento_nao_anda_mas_os_ciclos_correm() {
    let p = pontos();
    let janela: Vec<&Ponto> = p
        .iter()
        .filter(|x| x.acesso == acesso_z80::ESCRITA_JANELA)
        .collect();
    // Com o barramento cedido o Z80 fica parado: o PC e os ciclos que faltam não mudam.
    let primeiro = janela[0];
    assert!(janela.iter().all(|x| x.pc == primeiro.pc));
    // O Z80 rodou NOPs desde que o reset foi solto: o PC andou.
    assert!(primeiro.pc > 0, "pc {}", primeiro.pc);
    // Os ciclos continuam sendo contados enquanto ele espera (o oráculo conta o tick parado).
    assert!(janela.iter().skip(1).all(|x| x.ativos > 0));
}

#[test]
fn depois_do_reset_o_z80_volta_ao_endereco_zero() {
    let p = pontos();
    let ligado = p
        .iter()
        .position(|x| x.acesso == acesso_z80::RESET_LIGADO)
        .unwrap();
    // O ponto do reset solto logo depois: o oráculo zerou o PC durante o reset.
    let solto = p[ligado + 1];
    assert_eq!(solto.acesso, acesso_z80::RESET_SOLTO);
    assert_eq!(solto.pc, 0);
    assert!(solto.ativos > 0);
    // E nenhum ponto mostra ciclo faltante maior que o maior tempo de instrução do Z80 (23).
    assert!(p.iter().all(|x| x.espera <= 23), "{p:?}");
}

#[test]
fn a_soma_dos_ciclos_dos_pontos_nao_passa_do_tempo_decorrido() {
    oracle_trace::tempo_ligar();
    let mut o = Oraculo::novo(programa_z80(), Entradas::default()).unwrap();
    let mut mclk_fim = 0;
    let mut soma: u64 = 0;
    for _ in 0..6 {
        let q = o.executar_quadro().unwrap();
        mclk_fim = q.mclk_fim;
        let mut brutos = Vec::new();
        oracle_trace::drain_tempo(&mut brutos);
        soma += brutos
            .iter()
            .filter_map(|r| match r {
                TempoRaw::Z80Ponto { ativos, .. } => Some(u64::from(*ativos)),
                _ => None,
            })
            .sum::<u64>();
    }
    // Um ciclo do Z80 são 15 ciclos mestres.
    assert!(soma <= mclk_fim / 15 + 1, "soma {soma}, mclk {mclk_fim}");
    assert!(soma > 0);
}
