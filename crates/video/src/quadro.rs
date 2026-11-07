//! Quadro de saída: 320 por 224 pontos, cada um com a cor de 9 bits.

/// Largura da área ativa em modo de 40 colunas.
pub const LARGURA: usize = 320;
/// Altura da área ativa em modo de 28 linhas de células.
pub const ALTURA: usize = 224;
/// Total de pontos.
pub const PONTOS: usize = LARGURA * ALTURA;

/// Imagem de um quadro, linha após linha.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quadro {
    pontos: Vec<u16>,
}

impl Quadro {
    /// Quadro todo na cor 0.
    pub fn novo() -> Self {
        Self {
            pontos: vec![0; PONTOS],
        }
    }

    /// Cor do ponto (x, y). Fora da área devolve `None`.
    pub fn ponto(&self, x: usize, y: usize) -> Option<u16> {
        if x >= LARGURA || y >= ALTURA {
            return None;
        }
        self.pontos
            .get(y.checked_mul(LARGURA)?.checked_add(x)?)
            .copied()
    }

    /// Grava a cor (só os 9 bits baixos). Fora da área não faz nada e devolve `false`.
    pub fn definir(&mut self, x: usize, y: usize, cor: u16) -> bool {
        if x >= LARGURA || y >= ALTURA {
            return false;
        }
        let Some(i) = y.checked_mul(LARGURA).and_then(|b| b.checked_add(x)) else {
            return false;
        };
        match self.pontos.get_mut(i) {
            Some(p) => {
                *p = cor & 0x1ff;
                true
            }
            None => false,
        }
    }

    /// Os pontos, linha após linha.
    pub fn pontos(&self) -> &[u16] {
        &self.pontos
    }

    /// Serialização usada no hash da imagem: cada ponto em 2 bytes, little-endian.
    pub fn para_bytes(&self) -> Vec<u8> {
        let mut saida = Vec::with_capacity(PONTOS.saturating_mul(2));
        for p in &self.pontos {
            saida.extend_from_slice(&p.to_le_bytes());
        }
        saida
    }
}

impl Default for Quadro {
    fn default() -> Self {
        Self::novo()
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn fora_da_area_nao_escreve() {
        let mut q = Quadro::novo();
        assert!(!q.definir(LARGURA, 0, 1));
        assert!(!q.definir(0, ALTURA, 1));
        assert!(!q.definir(usize::MAX, usize::MAX, 1));
        assert_eq!(q, Quadro::novo());
        assert_eq!(q.ponto(LARGURA, 0), None);
    }

    #[test]
    fn cor_e_cortada_em_9_bits() {
        let mut q = Quadro::novo();
        assert!(q.definir(319, 223, 0xffff));
        assert_eq!(q.ponto(319, 223), Some(0x1ff));
        assert_eq!(q.ponto(0, 0), Some(0));
    }

    #[test]
    fn bytes_tem_o_tamanho_certo() {
        let mut q = Quadro::novo();
        q.definir(1, 0, 0x101);
        let b = q.para_bytes();
        assert_eq!(b.len(), PONTOS * 2);
        assert_eq!(&b[0..4], &[0, 0, 1, 1]);
    }
}
