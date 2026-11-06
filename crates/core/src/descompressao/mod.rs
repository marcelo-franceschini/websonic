//! Descompressores dos três formatos de compressão do Sonic 1.
//!
//! Cada descompressor recebe os bytes da ROM do usuário, em tempo de execução, e devolve os
//! bytes descomprimidos. Nenhuma tabela nem conteúdo da ROM mora aqui. Os formatos foram
//! reimplementados a partir da descrição pública do formato (ver
//! `docs/conhecimento/descompressao.md`), não do código de máquina.
//!
//! Regras comuns:
//! - Nenhum descompressor entra em pânico, para qualquer entrada.
//! - Nenhum lê fora da entrada: o fim dela vira [`Erro::EntradaTruncada`].
//! - A saída nunca passa de `limite_saida` bytes.

pub mod enigma;
pub mod kosinski;
pub mod nemesis;

/// Motivo de uma descompressão ter falhado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Erro {
    /// A entrada acabou antes do fim do fluxo.
    EntradaTruncada,
    /// A saída passaria do limite recebido.
    SaidaExcedeLimite,
    /// O fluxo contradiz o formato.
    FormatoInvalido(&'static str),
}

impl core::fmt::Display for Erro {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EntradaTruncada => f.write_str("entrada truncada"),
            Self::SaidaExcedeLimite => f.write_str("saida passa do limite"),
            Self::FormatoInvalido(motivo) => write!(f, "formato invalido: {motivo}"),
        }
    }
}

impl std::error::Error for Erro {}

/// Resultado de uma descompressão que terminou.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resultado {
    /// Bytes descomprimidos.
    pub saida: Vec<u8>,
    /// Quantos bytes da entrada o descompressor usou (do primeiro ao último tocado).
    pub consumidos: usize,
}

/// Leitor de bytes que nunca passa do fim.
pub(crate) struct Bytes<'a> {
    dados: &'a [u8],
    pos: usize,
}

impl<'a> Bytes<'a> {
    pub(crate) fn novo(dados: &'a [u8]) -> Self {
        Self { dados, pos: 0 }
    }

    pub(crate) fn byte(&mut self) -> Result<u8, Erro> {
        let valor = *self.dados.get(self.pos).ok_or(Erro::EntradaTruncada)?;
        self.pos = self.pos.checked_add(1).ok_or(Erro::EntradaTruncada)?;
        Ok(valor)
    }

    pub(crate) fn palavra_be(&mut self) -> Result<u16, Erro> {
        let alto = self.byte()?;
        let baixo = self.byte()?;
        Ok(u16::from_be_bytes([alto, baixo]))
    }

    pub(crate) fn posicao(&self) -> usize {
        self.pos
    }
}

/// Leitor de bits, do mais significativo para o menos significativo de cada byte.
pub(crate) struct Bits<'a> {
    bytes: Bytes<'a>,
    atual: u8,
    restantes: u8,
}

impl<'a> Bits<'a> {
    pub(crate) fn novo(bytes: Bytes<'a>) -> Self {
        Self {
            bytes,
            atual: 0,
            restantes: 0,
        }
    }

    pub(crate) fn bit(&mut self) -> Result<u8, Erro> {
        if self.restantes == 0 {
            self.atual = self.bytes.byte()?;
            self.restantes = 8;
        }
        self.restantes = self.restantes.checked_sub(1).ok_or(Erro::EntradaTruncada)?;
        Ok(self.atual.wrapping_shr(u32::from(self.restantes)) & 1)
    }

    /// Lê `quantos` bits (no máximo 16), o primeiro lido é o mais significativo.
    pub(crate) fn bits(&mut self, quantos: u8) -> Result<u16, Erro> {
        let mut valor: u16 = 0;
        for _ in 0..quantos {
            valor = valor.wrapping_shl(1) | u16::from(self.bit()?);
        }
        Ok(valor)
    }

    /// Bytes da entrada tocados até agora (um byte parcial conta inteiro).
    pub(crate) fn posicao(&self) -> usize {
        self.bytes.posicao()
    }
}

/// Acrescenta um byte respeitando o limite.
pub(crate) fn empurrar(saida: &mut Vec<u8>, byte: u8, limite: usize) -> Result<(), Erro> {
    if saida.len() >= limite {
        return Err(Erro::SaidaExcedeLimite);
    }
    saida.push(byte);
    Ok(())
}
