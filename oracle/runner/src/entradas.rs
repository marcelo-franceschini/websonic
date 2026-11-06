//! Entradas de controle: so botoes, um byte por quadro (controle de 3 botoes na porta 1).

use genesis_config::GenesisController;
use genesis_core::{GenesisButton, GenesisInputs};

/// Botoes do controle de 3 botoes. O valor e o bit na mascara.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Botao {
    Cima = 0x01,
    Baixo = 0x02,
    Esquerda = 0x04,
    Direita = 0x08,
    B = 0x10,
    C = 0x20,
    A = 0x40,
    Start = 0x80,
}

impl Botao {
    const TODOS: [(Botao, GenesisButton); 8] = [
        (Botao::Cima, GenesisButton::Up),
        (Botao::Baixo, GenesisButton::Down),
        (Botao::Esquerda, GenesisButton::Left),
        (Botao::Direita, GenesisButton::Right),
        (Botao::B, GenesisButton::B),
        (Botao::C, GenesisButton::C),
        (Botao::A, GenesisButton::A),
        (Botao::Start, GenesisButton::Start),
    ];
}

/// Monta as entradas do jgenesis a partir da mascara de botoes da porta 1. A porta 2 fica vazia.
pub fn entradas_da_mascara(mascara: u8) -> GenesisInputs {
    let mut p1 = GenesisController::new(genesis_config::GenesisControllerType::ThreeButton);
    for (botao, original) in Botao::TODOS {
        p1.set_field(original, mascara & (botao as u8) != 0);
    }
    GenesisInputs {
        p1,
        p2: GenesisController::None,
    }
}

/// Lista de botoes por quadro. Depois do ultimo quadro listado, nenhum botao fica apertado.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Entradas {
    por_quadro: Vec<u8>,
}

impl Entradas {
    pub fn nova(por_quadro: Vec<u8>) -> Self {
        Self { por_quadro }
    }

    /// Texto: uma linha por quadro, com a mascara em hexadecimal (2 digitos). `#` comenta.
    pub fn de_texto(texto: &str) -> Result<Self, String> {
        let mut por_quadro = Vec::new();
        for (i, linha) in texto.lines().enumerate() {
            let linha = linha.trim();
            if linha.is_empty() || linha.starts_with('#') {
                continue;
            }
            let valor = u8::from_str_radix(linha, 16)
                .map_err(|_| format!("linha {}: mascara invalida", i + 1))?;
            por_quadro.push(valor);
        }
        Ok(Self { por_quadro })
    }

    /// Mascara do quadro `n`.
    pub fn do_quadro(&self, n: u32) -> u8 {
        usize::try_from(n)
            .ok()
            .and_then(|i| self.por_quadro.get(i))
            .copied()
            .unwrap_or(0)
    }

    pub fn len(&self) -> usize {
        self.por_quadro.len()
    }

    pub fn is_empty(&self) -> bool {
        self.por_quadro.is_empty()
    }

    /// SHA-1 da lista, para o cabecalho do trace.
    pub fn sha1(&self) -> [u8; 20] {
        sonic1_fidelidade::hash::sha1(&self.por_quadro)
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn texto_e_mascara() {
        let e = Entradas::de_texto("# x\n00\n80\n0c\n").unwrap();
        assert_eq!(e.len(), 3);
        assert_eq!(e.do_quadro(1), 0x80);
        assert_eq!(e.do_quadro(2), 0x0c);
        assert_eq!(e.do_quadro(99), 0);
        assert!(Entradas::de_texto("zz").is_err());
    }
}
