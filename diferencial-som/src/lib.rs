//! Som do núcleo contra o oráculo (Etapa 6). Licença GPLv3, como o oráculo.
//!
//! Um [`Roteiro`] é uma lista de acessos do 68000 ao Z80. Ele é compilado em código do 68000
//! numa ROM sintética (escrita por nós, sem dado da ROM do jogo), rodado no oráculo com o tempo
//! gravado ligado, e repetido no núcleo com os pontos do Z80 gravados. Quem usa compara as
//! escritas no YM2612, a RAM do Z80 e as divergências de posição.

pub mod harness;

use oracle_trace::TempoRaw;
use sonic1_core::memoria::{Memoria, Saida, Tamanho};
use sonic1_core::quadro::tempo::AcessoZ80;
use sonic1_core::som::z80::{DivergenciaZ80, PontoZ80};
use sonic1_fidelidade::{Origem, Tipo};
use websonic_oracle::rom_sintetica::{RomSintetica, SSP};
use websonic_oracle::{Entradas, Oraculo};

/// Um acesso do 68000 que toca o Z80, ou uma espera.
#[derive(Clone, Copy, Debug)]
pub enum Op {
    /// Escreve um byte na janela do Z80 (endereço do Z80).
    Escrever(u16, u8),
    /// Lê um byte da janela do Z80.
    Ler(u16),
    /// Pede (`true`) ou devolve o barramento.
    Barramento(bool),
    /// `true` solta o reset do Z80; `false` liga o reset.
    SoltarReset(bool),
    /// Espera `n` voltas de um laço de 14 ciclos do 68000.
    Esperar(u16),
}

pub type Roteiro = Vec<Op>;

/// Código do 68000 do roteiro, seguido de um laço infinito.
pub fn compilar(roteiro: &[Op]) -> Vec<u16> {
    let mut c: Vec<u16> = vec![0x46FC, 0x2700]; // move.w #$2700,sr
    for op in roteiro {
        match *op {
            Op::Escrever(a, v) => {
                // move.b #v,$A0xxxx.l
                c.extend_from_slice(&[0x13FC, u16::from(v), 0x00A0, a]);
            }
            Op::Ler(a) => {
                // move.b $A0xxxx.l,d0
                c.extend_from_slice(&[0x1039, 0x00A0, a]);
            }
            Op::Barramento(pedir) => {
                c.extend_from_slice(&[0x33FC, if pedir { 0x0100 } else { 0 }, 0x00A1, 0x1100]);
            }
            Op::SoltarReset(soltar) => {
                c.extend_from_slice(&[0x33FC, if soltar { 0x0100 } else { 0 }, 0x00A1, 0x1200]);
            }
            Op::Esperar(n) => {
                // move.w #n,d1 ; nop ; dbf d1,*-2
                c.extend_from_slice(&[0x323C, n, 0x4E71, 0x51C9, 0xFFFC]);
            }
        }
    }
    c.push(0x60FE); // bra.s *
    c
}

pub fn rom_do_roteiro(roteiro: &[Op]) -> Vec<u8> {
    let mut rom = RomSintetica::nova(SSP, 0x200);
    rom.palavras(0x200, &compilar(roteiro));
    rom.finalizar()
}

pub fn acesso_de(codigo: u8) -> Option<AcessoZ80> {
    use oracle_trace::acesso_z80::*;
    Some(match codigo {
        BUSREQ_PEDIDO => AcessoZ80::BusReqPedido,
        BUSREQ_LIBERADO => AcessoZ80::BusReqLiberado,
        RESET_LIGADO => AcessoZ80::ResetLigado,
        RESET_SOLTO => AcessoZ80::ResetSolto,
        LEITURA_JANELA => AcessoZ80::LeituraJanela,
        ESCRITA_JANELA => AcessoZ80::EscritaJanela,
        _ => return None,
    })
}

/// O que o oráculo produziu para o roteiro.
pub struct SaidaOraculo {
    /// Escritas do Z80 no YM2612: (porta, valor).
    pub ym_z80: Vec<(u8, u8)>,
    pub pontos: Vec<PontoZ80>,
    pub ram_z80: Vec<u8>,
}

/// Roda a ROM do roteiro no oráculo por `quadros` quadros, com o tempo gravado ligado.
pub fn rodar_oraculo(rom: Vec<u8>, quadros: u32) -> SaidaOraculo {
    oracle_trace::tempo_ligar();
    let mut o = Oraculo::novo(rom, Entradas::default()).expect("oraculo");
    let (mut ym_z80, mut pontos) = (Vec::new(), Vec::new());
    for _ in 0..quadros {
        let q = o.executar_quadro().expect("quadro");
        for e in &q.eventos {
            if e.tipo == Tipo::Ym && e.origem == Origem::Z80 {
                ym_z80.push((u8::try_from(e.a).unwrap(), u8::try_from(e.b).unwrap()));
            }
        }
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
                mclk,
            } = r
            {
                pontos.push(PontoZ80 {
                    acesso: acesso_de(acesso).expect("acesso"),
                    pc,
                    espera,
                    ativos,
                    banco,
                    endereco,
                    byte,
                    mclk,
                });
            }
        }
    }
    SaidaOraculo {
        ym_z80,
        pontos,
        ram_z80: o.ram_z80(),
    }
}

/// O que o núcleo produziu para o mesmo roteiro e os mesmos pontos.
pub struct SaidaNucleo {
    pub ym_z80: Vec<(u8, u8)>,
    pub divergencias: Vec<DivergenciaZ80>,
    pub pontos_sobrando: usize,
    pub ram_z80: Vec<u8>,
}

/// Repete o roteiro no núcleo, com os pontos do Z80 do oráculo.
pub fn rodar_nucleo(rom: Vec<u8>, roteiro: &[Op], pontos: &[PontoZ80]) -> SaidaNucleo {
    let mut m = Memoria::nova(rom);
    for p in pontos {
        m.enfileirar_ponto_z80(*p);
    }
    let janela = |a: u16| 0xA0_0000 + u32::from(a);
    for op in roteiro {
        match *op {
            Op::Escrever(a, v) => {
                m.escrever(janela(a), Tamanho::Byte, u32::from(v)).unwrap();
            }
            Op::Ler(a) => {
                m.ler(janela(a), Tamanho::Byte).unwrap();
            }
            Op::Barramento(pedir) => {
                let v = if pedir { 0x0100 } else { 0 };
                m.escrever(0xA1_1100, Tamanho::Word, v).unwrap();
            }
            Op::SoltarReset(soltar) => {
                let v = if soltar { 0x0100 } else { 0 };
                m.escrever(0xA1_1200, Tamanho::Word, v).unwrap();
            }
            Op::Esperar(_) => {}
        }
    }
    let ym_z80 = m
        .saidas()
        .iter()
        .filter_map(|s| match *s {
            Saida::YmZ80 { porta, valor } => Some((porta, valor)),
            _ => None,
        })
        .collect();
    SaidaNucleo {
        ym_z80,
        divergencias: m.divergencias_z80().to_vec(),
        pontos_sobrando: m.pontos_z80_pendentes(),
        ram_z80: m.ram_z80().to_vec(),
    }
}
