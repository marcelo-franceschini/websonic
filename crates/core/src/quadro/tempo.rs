//! Arquivo de tempo gravado (`.s1tempo`): o que o oráculo viu de tempo em cada quadro.
//!
//! É o "tempo gravado" do nível 1: orçamento do quadro, ponto de cada interrupção e valor de
//! cada leitura de estado de hardware, na ordem. Tamanho fixo, little-endian, sem relógio de
//! parede nem ponteiros. O escritor do oráculo e o leitor do núcleo seguem este formato.
//!
//! ```text
//! cabeçalho: "S1TEMPO\0" (8) versão u16
//! quadro:    'Q' numero u32, mclk_inicio u64, mclk_fim u64, n_eventos u32, eventos (24 bytes cada)
//! evento:    tipo u8, aux u8, linha u16, coluna u16, reservado u16, valor u32, mclk u64, extra u32
//! ```
//! Tipos: 1 interrupção (aux = nível, valor = PC interrompido, extra = PC do tratador);
//! 2 leitura de hardware (aux = classe, 0 BUSACK, 1 estado da VDP, 2 estado do YM; valor = o que
//! o 68000 leu); 3 espera do Z80 consumida (valor = ciclos do 68000).

use crate::memoria::ClasseLeitura;

pub const MAGIA: &[u8; 8] = b"S1TEMPO\0";
pub const VERSAO: u16 = 1;
const TAMANHO_EVENTO: usize = 24;

/// Um evento de tempo dentro de um quadro.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventoTempo {
    Interrupcao {
        nivel: u8,
        pc_interrompido: u32,
        pc_tratador: u32,
        mclk: u64,
        linha: u16,
        coluna: u16,
    },
    Leitura {
        classe: ClasseLeitura,
        valor: u16,
        mclk: u64,
    },
    EsperaZ80 {
        ciclos: u32,
        mclk: u64,
    },
}

/// O tempo de um quadro.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TempoQuadro {
    pub numero: u32,
    pub mclk_inicio: u64,
    pub mclk_fim: u64,
    pub eventos: Vec<EventoTempo>,
}

impl TempoQuadro {
    /// Ciclos mestres do quadro (o orçamento gravado).
    pub fn orcamento(&self) -> u64 {
        self.mclk_fim.saturating_sub(self.mclk_inicio)
    }
}

/// Erros de leitura do arquivo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErroTempo {
    MagiaInvalida,
    VersaoDesconhecida,
    Truncado,
    MarcaInvalida,
    TipoInvalido,
}

/// O arquivo inteiro.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TempoGravado {
    pub quadros: Vec<TempoQuadro>,
}

fn classe_para_u8(c: ClasseLeitura) -> u8 {
    match c {
        ClasseLeitura::Busack => 0,
        ClasseLeitura::VdpStatus => 1,
        ClasseLeitura::YmStatus => 2,
    }
}

fn classe_de_u8(v: u8) -> Option<ClasseLeitura> {
    match v {
        0 => Some(ClasseLeitura::Busack),
        1 => Some(ClasseLeitura::VdpStatus),
        2 => Some(ClasseLeitura::YmStatus),
        _ => None,
    }
}

impl TempoGravado {
    pub fn para_bytes(&self) -> Vec<u8> {
        let mut s = Vec::new();
        s.extend_from_slice(MAGIA);
        s.extend_from_slice(&VERSAO.to_le_bytes());
        for q in &self.quadros {
            s.push(b'Q');
            s.extend_from_slice(&q.numero.to_le_bytes());
            s.extend_from_slice(&q.mclk_inicio.to_le_bytes());
            s.extend_from_slice(&q.mclk_fim.to_le_bytes());
            s.extend_from_slice(
                &u32::try_from(q.eventos.len())
                    .unwrap_or(u32::MAX)
                    .to_le_bytes(),
            );
            for e in &q.eventos {
                let (tipo, aux, linha, coluna, valor, mclk, extra) = match *e {
                    EventoTempo::Interrupcao {
                        nivel,
                        pc_interrompido,
                        pc_tratador,
                        mclk,
                        linha,
                        coluna,
                    } => (
                        1u8,
                        nivel,
                        linha,
                        coluna,
                        pc_interrompido,
                        mclk,
                        pc_tratador,
                    ),
                    EventoTempo::Leitura {
                        classe,
                        valor,
                        mclk,
                    } => (2, classe_para_u8(classe), 0, 0, u32::from(valor), mclk, 0),
                    EventoTempo::EsperaZ80 { ciclos, mclk } => (3, 0, 0, 0, ciclos, mclk, 0),
                };
                s.push(tipo);
                s.push(aux);
                s.extend_from_slice(&linha.to_le_bytes());
                s.extend_from_slice(&coluna.to_le_bytes());
                s.extend_from_slice(&[0, 0]);
                s.extend_from_slice(&valor.to_le_bytes());
                s.extend_from_slice(&mclk.to_le_bytes());
                s.extend_from_slice(&extra.to_le_bytes());
            }
        }
        s
    }

    pub fn de_bytes(b: &[u8]) -> Result<Self, ErroTempo> {
        let mut l = Leitor { b, pos: 0 };
        if l.pegar(8)? != MAGIA {
            return Err(ErroTempo::MagiaInvalida);
        }
        if l.u16()? != VERSAO {
            return Err(ErroTempo::VersaoDesconhecida);
        }
        let mut quadros = Vec::new();
        while l.pos < b.len() {
            if l.pegar(1)? != b"Q" {
                return Err(ErroTempo::MarcaInvalida);
            }
            let numero = l.u32()?;
            let mclk_inicio = l.u64()?;
            let mclk_fim = l.u64()?;
            let n = l.u32()?;
            let mut eventos = Vec::new();
            for _ in 0..n {
                let reg = l.pegar(TAMANHO_EVENTO)?;
                let mut r = Leitor { b: reg, pos: 0 };
                let tipo = r.pegar(1)?[0];
                let aux = r.pegar(1)?[0];
                let linha = r.u16()?;
                let coluna = r.u16()?;
                r.u16()?;
                let valor = r.u32()?;
                let mclk = r.u64()?;
                let extra = r.u32()?;
                eventos.push(match tipo {
                    1 => EventoTempo::Interrupcao {
                        nivel: aux,
                        pc_interrompido: valor,
                        pc_tratador: extra,
                        mclk,
                        linha,
                        coluna,
                    },
                    2 => EventoTempo::Leitura {
                        classe: classe_de_u8(aux).ok_or(ErroTempo::TipoInvalido)?,
                        valor: u16::try_from(valor & 0xFFFF).unwrap_or(0),
                        mclk,
                    },
                    3 => EventoTempo::EsperaZ80 {
                        ciclos: valor,
                        mclk,
                    },
                    _ => return Err(ErroTempo::TipoInvalido),
                });
            }
            quadros.push(TempoQuadro {
                numero,
                mclk_inicio,
                mclk_fim,
                eventos,
            });
        }
        Ok(Self { quadros })
    }
}

struct Leitor<'a> {
    b: &'a [u8],
    pos: usize,
}

impl<'a> Leitor<'a> {
    fn pegar(&mut self, n: usize) -> Result<&'a [u8], ErroTempo> {
        let fim = self.pos.checked_add(n).ok_or(ErroTempo::Truncado)?;
        let s = self.b.get(self.pos..fim).ok_or(ErroTempo::Truncado)?;
        self.pos = fim;
        Ok(s)
    }
    fn u16(&mut self) -> Result<u16, ErroTempo> {
        let s = self.pegar(2)?;
        Ok(u16::from_le_bytes([s[0], s[1]]))
    }
    fn u32(&mut self) -> Result<u32, ErroTempo> {
        let s = self.pegar(4)?;
        Ok(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    }
    fn u64(&mut self) -> Result<u64, ErroTempo> {
        let s = self.pegar(8)?;
        Ok(u64::from_le_bytes([
            s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7],
        ]))
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn exemplo() -> TempoGravado {
        TempoGravado {
            quadros: vec![
                TempoQuadro {
                    numero: 0,
                    mclk_inicio: 0,
                    mclk_fim: 896_040,
                    eventos: vec![
                        EventoTempo::Interrupcao {
                            nivel: 6,
                            pc_interrompido: 0x388,
                            pc_tratador: 0x1000,
                            mclk: 700_000,
                            linha: 224,
                            coluna: 12,
                        },
                        EventoTempo::Leitura {
                            classe: ClasseLeitura::VdpStatus,
                            valor: 0x3400,
                            mclk: 700_100,
                        },
                        EventoTempo::EsperaZ80 {
                            ciclos: 14,
                            mclk: 700_200,
                        },
                    ],
                },
                TempoQuadro {
                    numero: 1,
                    mclk_inicio: 896_040,
                    mclk_fim: 1_792_080,
                    eventos: vec![],
                },
            ],
        }
    }

    #[test]
    fn ida_e_volta() {
        let t = exemplo();
        assert_eq!(TempoGravado::de_bytes(&t.para_bytes()), Ok(t));
    }

    #[test]
    fn arquivo_ruim_e_recusado() {
        let bom = exemplo().para_bytes();
        assert_eq!(
            TempoGravado::de_bytes(b"xxxxxxxxxx"),
            Err(ErroTempo::MagiaInvalida)
        );
        let mut v = bom.clone();
        v[8] = 9;
        assert_eq!(
            TempoGravado::de_bytes(&v),
            Err(ErroTempo::VersaoDesconhecida)
        );
        for corte in [11usize, 20, bom.len() - 1] {
            assert_eq!(
                TempoGravado::de_bytes(&bom[..corte]),
                Err(ErroTempo::Truncado)
            );
        }
        let mut v = bom.clone();
        v[10] = b'X';
        assert_eq!(TempoGravado::de_bytes(&v), Err(ErroTempo::MarcaInvalida));
        let mut v = bom;
        // Tipo do primeiro evento (depois do cabecalho de 10 bytes e do quadro de 25 bytes).
        v[10 + 25] = 9;
        assert_eq!(TempoGravado::de_bytes(&v), Err(ErroTempo::TipoInvalido));
    }

    #[test]
    fn orcamento_e_a_diferenca() {
        assert_eq!(exemplo().quadros[0].orcamento(), 896_040);
    }
}
