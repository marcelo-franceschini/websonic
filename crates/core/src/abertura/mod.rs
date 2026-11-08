//! Abertura do jogo (Etapa 7): do ligar do console à primeira demo.
//!
//! Cada modo do jogo é uma máquina de estados. O laço principal do original roda como trechos
//! que terminam numa **marca**: um endereço de rótulo que o oráculo viu o 68000 alcançar. O
//! tempo gravado (nível 1) diz em que quadro cada marca chegou, e o trecho roda nesse quadro.
//! Interrupções gravadas rodam o tratador correspondente. Isto é infraestrutura e não tem par no
//! original. Cada rotina de `boot` cita o rótulo do disassembly que reproduz.
//!
//! Limite (D7.10 do plano): um trecho é atômico. Seu efeito aparece de uma vez, no quadro da marca
//! final. O comparador separa os quadros dentro de um trecho atômico dos demais.

pub mod boot;
pub mod ram;

use crate::memoria::{ClasseLeitura, ErroAcesso, Memoria, Tamanho};
use crate::som::z80::PontoZ80;

/// Pontos do original que o oráculo marca (primeira ou qualquer chegada do PC).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Marca {
    /// `GameProgram`: fim da preparação do console e começo da verificação da ROM.
    GameProgram,
    /// `CheckSumOk`: fim da verificação da ROM.
    ChecksumOk,
}

/// Rótulo e endereço de cada marca (metadado de `refs/rotulos.txt`, conferido por teste).
pub const MARCAS: [(Marca, &str, u32); 2] = [
    (Marca::GameProgram, "GameProgram", 0x300),
    (Marca::ChecksumOk, "CheckSumOk", 0x33E),
];

/// Um fato do tempo gravado dentro de um quadro, na ordem em que aconteceu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventoQuadro {
    Interrupcao { nivel: u8, pc: u32 },
    Marca(Marca),
    Leitura { classe: ClasseLeitura, valor: u16 },
    PontoZ80(PontoZ80),
}

/// Erros que param a abertura.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErroAbertura {
    /// A soma de verificação da ROM não bate com o cabeçalho (`CheckSumError`).
    ChecksumErrado,
    /// Um laço de espera não terminou: faltou valor gravado.
    EsperaSemFim,
    Acesso(ErroAcesso),
    Rotina(crate::rotinas::ErroRotina),
    /// O trecho pedido não existe nesta versão do núcleo.
    NaoImplementado(&'static str),
}

impl From<ErroAcesso> for ErroAbertura {
    fn from(e: ErroAcesso) -> Self {
        Self::Acesso(e)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Estado {
    /// Antes de `GameProgram`: o 68000 está em `EntryPoint`.
    Inicio,
    /// Dentro do laço de soma de verificação.
    Checksum,
    /// `GM_Sega` ainda não implementado: o núcleo para aqui.
    Sega,
}

/// O console no trecho da abertura.
pub struct Abertura {
    pub mem: Memoria,
    estado: Estado,
    quadro: u32,
    pub erro: Option<ErroAbertura>,
    /// Quantas vezes cada tipo de evento chegou sem uso (diagnóstico do relatório).
    pub marcas_ignoradas: u32,
}

impl Abertura {
    pub fn nova(rom: Vec<u8>) -> Self {
        Self {
            mem: Memoria::nova(rom),
            estado: Estado::Inicio,
            quadro: 0,
            erro: None,
            marcas_ignoradas: 0,
        }
    }

    pub fn quadro(&self) -> u32 {
        self.quadro
    }

    /// Está dentro de um trecho atômico? (a marca final do trecho ainda não chegou)
    pub fn em_trecho_atomico(&self) -> bool {
        matches!(self.estado, Estado::Inicio | Estado::Checksum)
    }

    /// Roda um quadro com os fatos gravados dele.
    pub fn executar_quadro(&mut self, eventos: &[EventoQuadro]) {
        // O tempo gravado do quadro inteiro fica disponível antes de qualquer trecho rodar.
        for e in eventos {
            match *e {
                EventoQuadro::Leitura { classe, valor } => {
                    self.mem.enfileirar_leitura(classe, valor)
                }
                EventoQuadro::PontoZ80(p) => self.mem.enfileirar_ponto_z80(p),
                _ => {}
            }
        }
        for e in eventos {
            if self.erro.is_some() {
                break;
            }
            if let EventoQuadro::Marca(m) = *e
                && let Err(erro) = self.marca(m)
            {
                self.erro = Some(erro);
            }
        }
        self.quadro = self.quadro.saturating_add(1);
    }

    fn marca(&mut self, m: Marca) -> Result<(), ErroAbertura> {
        match (self.estado, m) {
            (Estado::Inicio, Marca::GameProgram) => {
                boot::entry_point(&mut self.mem)?;
                boot::game_program(&mut self.mem)?;
                self.estado = Estado::Checksum;
            }
            (Estado::Checksum, Marca::ChecksumOk) => {
                boot::check_sum_ok(&mut self.mem)?;
                boot::game_init(&mut self.mem)?;
                self.estado = Estado::Sega;
            }
            _ => self.marcas_ignoradas = self.marcas_ignoradas.saturating_add(1),
        }
        Ok(())
    }
}

/// Atalhos de acesso à memória no estilo do 68000: erro de alinhamento nunca acontece nos
/// endereços fixos que as rotinas usam.
pub(crate) fn wb(m: &mut Memoria, a: u32, v: u8) {
    let _ = m.escrever(a, Tamanho::Byte, u32::from(v));
}
pub(crate) fn ww(m: &mut Memoria, a: u32, v: u16) {
    let _ = m.escrever(a, Tamanho::Word, u32::from(v));
}
pub(crate) fn wl(m: &mut Memoria, a: u32, v: u32) {
    let _ = m.escrever(a, Tamanho::Long, v);
}
pub(crate) fn rb(m: &mut Memoria, a: u32) -> u8 {
    m.ler(a, Tamanho::Byte).map_or(0, |v| v.to_le_bytes()[0])
}
pub(crate) fn rw(m: &mut Memoria, a: u32) -> u16 {
    m.ler(a, Tamanho::Word).map_or(0, |v| {
        let b = v.to_le_bytes();
        u16::from_le_bytes([b[0], b[1]])
    })
}
