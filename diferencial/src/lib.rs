//! Testes diferenciais entre o oráculo (jgenesis modificado) e o núcleo.
//!
//! Licença GPLv3, como o oráculo. Nunca entra no build do site nem do núcleo.
//!
//! Uma [`Bancada`] monta uma ROM sintética (escrita por nós, sem dado da ROM do jogo) com um
//! trecho de código do 68000, e o roda no oráculo a partir de registradores definidos. Quem usa
//! compara o resultado com a função equivalente do núcleo.

pub mod rotina;

use genesis_core::api::oracle::Regs68;
use sonic1_fidelidade::{Evento, Tipo};
use websonic_oracle::oraculo::ChamadaRotina;
use websonic_oracle::rom_sintetica::{RomSintetica, SSP};
use websonic_oracle::{Entradas, Oraculo};

/// Onde o trecho de teste fica na ROM sintética.
pub const ENDERECO_TRECHO: u32 = 0x400;
/// Endereço de retorno falso.
pub const SENTINELA: u32 = 0x1_F000;
/// RTS.
pub const RTS: u16 = 0x4E75;

/// Registradores de dados, de endereço (A0 a A6) e SR.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Regs {
    pub d: [u32; 8],
    pub a: [u32; 7],
    pub sr: u16,
}

/// Resultado de uma execução.
#[derive(Clone, Debug)]
pub struct Resultado {
    pub regs: Regs,
    pub eventos: Vec<Evento>,
    pub mudancas_ram: Vec<(u32, u8, u8)>,
}

/// Oráculo com um trecho de código carregado.
pub struct Bancada {
    oraculo: Oraculo,
    base: Regs68,
}

impl Bancada {
    /// `rom` já montada pelo chamador, com o trecho em [`ENDERECO_TRECHO`].
    pub fn com_rom(rom: Vec<u8>) -> Self {
        let oraculo = Oraculo::novo(rom, Entradas::default()).expect("oraculo");
        let base = oraculo.emulador().oracle_regs68();
        Self { oraculo, base }
    }

    /// Monta uma ROM sintética com `palavras` seguidas de RTS.
    pub fn nova(palavras: &[u16]) -> Self {
        Self::com_rom(rom_com_trecho(palavras))
    }

    /// Roda o trecho. A pilha é a do supervisor. As interrupções ficam mascaradas.
    /// Falha se o trecho não retornar (por exemplo, exceção de divisão por zero).
    pub fn rodar(&mut self, entrada: &Regs, pokes: &[(u32, u8)]) -> Resultado {
        self.tentar(entrada, pokes).expect("rotina")
    }

    /// Como [`Bancada::rodar`], mas devolve o erro se o trecho não retornar.
    pub fn tentar(&mut self, entrada: &Regs, pokes: &[(u32, u8)]) -> Result<Resultado, String> {
        let mut regs = self.base;
        regs.d = entrada.d;
        regs.a = entrada.a;
        regs.sr = entrada.sr | 0x2000;
        regs.pc = ENDERECO_TRECHO;
        regs.ssp = SSP;
        let chamada = ChamadaRotina {
            regs,
            pokes_ram68: pokes.to_vec(),
            sentinela: SENTINELA,
            mascarar_interrupcoes: true,
            max_passos: 10_000,
        };
        let r = self.oraculo.chamar_rotina(&chamada)?;
        Ok(Resultado {
            regs: Regs {
                d: r.regs.d,
                a: r.regs.a,
                sr: r.regs.sr,
            },
            eventos: r.eventos,
            mudancas_ram: r.mudancas_ram68,
        })
    }

    pub fn oraculo(&mut self) -> &mut Oraculo {
        &mut self.oraculo
    }
}

/// ROM sintética com o trecho (e um RTS no fim) em [`ENDERECO_TRECHO`].
pub fn rom_com_trecho(palavras: &[u16]) -> Vec<u8> {
    let mut rom = RomSintetica::nova(SSP, 0x200);
    rom.palavras(0x200, &[0x60FE]); // bra.s *
    let mut todas = palavras.to_vec();
    todas.push(RTS);
    rom.palavras(ENDERECO_TRECHO, &todas);
    rom.finalizar()
}

/// Eventos de um dos tipos dados, na ordem.
pub fn eventos_do_tipo(eventos: &[Evento], tipos: &[Tipo]) -> Vec<Evento> {
    eventos
        .iter()
        .filter(|e| tipos.contains(&e.tipo))
        .copied()
        .collect()
}
