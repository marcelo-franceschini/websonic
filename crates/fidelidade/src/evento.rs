//! Eventos registrados pelo oraculo. Cada evento tem tamanho fixo (34 bytes).
//!
//! O significado dos campos `a`, `b`, `c` e `d` depende do tipo. A tabela fica em
//! `Tipo::campos`, e `docs/conhecimento/oraculo.md` a repete com exemplos.

use crate::formato::Erro;

/// Tamanho de um evento serializado.
pub const TAMANHO_EVENTO: usize = 34;

/// Tipo do evento. O numero e parte do formato e nao pode mudar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Tipo {
    /// Interrupcao aceita pelo 68000.
    Int68 = 1,
    /// Desvio executado pelo 68000, tomado ou nao.
    Desvio68 = 2,
    /// Interrupcao aceita pelo Z80.
    Int80 = 3,
    /// Desvio executado pelo Z80, tomado ou nao.
    Desvio80 = 4,
    /// Escrita na VDP (porta de dados ou de controle).
    Vdp = 5,
    /// Escrita no YM2612 (endereco ou dado).
    Ym = 6,
    /// Escrita no PSG.
    Psg = 7,
    /// Mudanca no BUSREQ ou no RESET do Z80.
    BarramentoZ80 = 8,
    /// Ciclos de espera do 68000 acumulados, com a causa.
    EsperaAcumulada = 9,
    /// Ciclos de espera do 68000 consumidos.
    EsperaConsumida = 10,
    /// Leitura do registrador de BUSACK pelo 68000.
    LeituraBusAck = 11,
}

impl Tipo {
    /// Converte o byte do formato em tipo.
    pub fn de_u8(valor: u8) -> Option<Self> {
        Some(match valor {
            1 => Self::Int68,
            2 => Self::Desvio68,
            3 => Self::Int80,
            4 => Self::Desvio80,
            5 => Self::Vdp,
            6 => Self::Ym,
            7 => Self::Psg,
            8 => Self::BarramentoZ80,
            9 => Self::EsperaAcumulada,
            10 => Self::EsperaConsumida,
            11 => Self::LeituraBusAck,
            _ => return None,
        })
    }

    /// Nome curto, para relatorios.
    pub fn nome(self) -> &'static str {
        match self {
            Self::Int68 => "int68",
            Self::Desvio68 => "desvio68",
            Self::Int80 => "int80",
            Self::Desvio80 => "desvio80",
            Self::Vdp => "vdp",
            Self::Ym => "ym",
            Self::Psg => "psg",
            Self::BarramentoZ80 => "barramento_z80",
            Self::EsperaAcumulada => "espera_acumulada",
            Self::EsperaConsumida => "espera_consumida",
            Self::LeituraBusAck => "leitura_busack",
        }
    }

    /// Todos os tipos, na ordem do numero.
    pub const TODOS: [Tipo; 11] = [
        Self::Int68,
        Self::Desvio68,
        Self::Int80,
        Self::Desvio80,
        Self::Vdp,
        Self::Ym,
        Self::Psg,
        Self::BarramentoZ80,
        Self::EsperaAcumulada,
        Self::EsperaConsumida,
        Self::LeituraBusAck,
    ];

    /// Significado de `a`, `b`, `c` e `d` para cada tipo.
    pub fn campos(self) -> [&'static str; 4] {
        match self {
            Self::Int68 => ["nivel", "pc_interrompido", "endereco_tratador", "-"],
            Self::Desvio68 => ["pc", "destino_se_tomado", "classe", "tomado"],
            Self::Int80 => [
                "tipo(0=int,1=nmi)",
                "pc_interrompido",
                "endereco_tratador",
                "-",
            ],
            Self::Desvio80 => ["pc", "destino_se_tomado", "classe", "tomado"],
            Self::Vdp => ["porta(0=dados,1=controle)", "valor", "tamanho(1,2)", "-"],
            Self::Ym => ["porta(0..3)", "valor", "-", "-"],
            Self::Psg => ["valor", "-", "-", "-"],
            Self::BarramentoZ80 => ["sinal(0=busreq,1=reset)", "ativo(0,1)", "-", "-"],
            Self::EsperaAcumulada => ["causa(1=68k_z80,2=z80_68k)", "ciclos_68k", "-", "-"],
            Self::EsperaConsumida => ["ciclos_68k", "-", "-", "-"],
            Self::LeituraBusAck => ["bit_busack(1=Z80 ainda no barramento)", "-", "-", "-"],
        }
    }
}

/// Quem causou o evento.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Origem {
    Cpu68 = 0,
    Z80 = 1,
    Hardware = 2,
}

impl Origem {
    pub fn de_u8(valor: u8) -> Option<Self> {
        match valor {
            0 => Some(Self::Cpu68),
            1 => Some(Self::Z80),
            2 => Some(Self::Hardware),
            _ => None,
        }
    }
}

/// Um evento do trace.
///
/// `mclk` conta ciclos do relogio mestre desde o ligar. O ciclo do 68000 e `mclk / 7`.
/// Para eventos do 68000, `mclk` e o inicio da instrucao que causou o evento. Para eventos do
/// Z80, e o relogio do Z80 no inicio da instrucao.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Evento {
    pub mclk: u64,
    /// Numero de sequencia dentro do quadro, a partir de zero.
    pub seq: u32,
    pub tipo: Tipo,
    pub origem: Origem,
    /// Linha da tela da VDP quando o evento foi aplicado.
    pub linha: u16,
    /// Ciclos do relogio mestre desde o inicio da linha.
    pub coluna: u16,
    pub a: u32,
    pub b: u32,
    pub c: u32,
    pub d: u32,
}

impl Evento {
    /// Serializa em 34 bytes, little-endian.
    pub fn para_bytes(&self) -> [u8; TAMANHO_EVENTO] {
        let mut saida = [0u8; TAMANHO_EVENTO];
        saida[0..8].copy_from_slice(&self.mclk.to_le_bytes());
        saida[8..12].copy_from_slice(&self.seq.to_le_bytes());
        saida[12] = self.tipo as u8;
        saida[13] = self.origem as u8;
        saida[14..16].copy_from_slice(&self.linha.to_le_bytes());
        saida[16..18].copy_from_slice(&self.coluna.to_le_bytes());
        saida[18..22].copy_from_slice(&self.a.to_le_bytes());
        saida[22..26].copy_from_slice(&self.b.to_le_bytes());
        saida[26..30].copy_from_slice(&self.c.to_le_bytes());
        saida[30..34].copy_from_slice(&self.d.to_le_bytes());
        saida
    }

    /// Le um evento de 34 bytes.
    pub fn de_bytes(bytes: &[u8; TAMANHO_EVENTO]) -> Result<Self, Erro> {
        let u16_em = |i: usize| u16::from_le_bytes([bytes[i], bytes[i + 1]]);
        let u32_em =
            |i: usize| u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
        let mut mclk = [0u8; 8];
        mclk.copy_from_slice(&bytes[0..8]);
        Ok(Self {
            mclk: u64::from_le_bytes(mclk),
            seq: u32_em(8),
            tipo: Tipo::de_u8(bytes[12]).ok_or(Erro::TipoDesconhecido(bytes[12]))?,
            origem: Origem::de_u8(bytes[13]).ok_or(Erro::OrigemDesconhecida(bytes[13]))?,
            linha: u16_em(14),
            coluna: u16_em(16),
            a: u32_em(18),
            b: u32_em(22),
            c: u32_em(26),
            d: u32_em(30),
        })
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn exemplo() -> Evento {
        Evento {
            mclk: 0x0102_0304_0506_0708,
            seq: 9,
            tipo: Tipo::Vdp,
            origem: Origem::Z80,
            linha: 223,
            coluna: 3419,
            a: 1,
            b: 0xC000,
            c: 2,
            d: 0,
        }
    }

    #[test]
    fn ida_e_volta() {
        let e = exemplo();
        assert_eq!(Evento::de_bytes(&e.para_bytes()).unwrap(), e);
    }

    #[test]
    fn little_endian_fixo() {
        let b = exemplo().para_bytes();
        assert_eq!(&b[0..8], &[8, 7, 6, 5, 4, 3, 2, 1]);
        assert_eq!(b[12], 5);
        assert_eq!(b[13], 1);
    }

    #[test]
    fn tipo_invalido_falha() {
        let mut b = exemplo().para_bytes();
        b[12] = 99;
        assert_eq!(Evento::de_bytes(&b), Err(Erro::TipoDesconhecido(99)));
    }

    #[test]
    fn todos_os_tipos_voltam() {
        for t in Tipo::TODOS {
            assert_eq!(Tipo::de_u8(t as u8), Some(t));
        }
    }
}
