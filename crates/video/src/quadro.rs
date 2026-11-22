//! Quadro de saída: 320 por 224 pontos, cada um com a cor de 9 bits.

/// Largura da área ativa em modo de 40 colunas.
pub const LARGURA: usize = 320;
/// Largura máxima suportada no modo estendido widescreen/ultrawide.
pub const LARGURA_MAX: usize = 640;
/// Altura da área ativa em modo de 28 linhas de células.
pub const ALTURA: usize = 224;
/// Total de pontos no modo clássico 320x224.
pub const PONTOS: usize = LARGURA * ALTURA;

/// Normaliza uma largura pedida para o intervalo par `[320, 640]`.
pub fn normalizar_largura(largura: usize) -> usize {
    largura.clamp(LARGURA, LARGURA_MAX) & !1
}

/// Imagem de um quadro, linha após linha.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quadro {
    largura: usize,
    pontos: Vec<u16>,
}

impl Quadro {
    /// Quadro todo na cor 0 (320 x 224).
    pub fn novo() -> Self {
        Self::com_largura(LARGURA)
    }

    /// Quadro todo na cor 0 com largura entre 320 e 640 (arredondada para par).
    pub fn com_largura(largura: usize) -> Self {
        let l = normalizar_largura(largura);
        Self {
            largura: l,
            pontos: vec![0; l.saturating_mul(ALTURA)],
        }
    }

    /// Largura atual do quadro em pontos.
    pub fn largura(&self) -> usize {
        self.largura
    }

    /// Redimensiona o quadro para a nova largura (entre 320 e 640, par).
    pub fn redimensionar(&mut self, largura: usize) {
        let l = normalizar_largura(largura);
        if l == self.largura {
            return;
        }
        self.largura = l;
        self.pontos.resize(l.saturating_mul(ALTURA), 0);
        self.pontos.fill(0);
    }

    /// Cor do ponto (x, y). Fora da área devolve `None`.
    pub fn ponto(&self, x: usize, y: usize) -> Option<u16> {
        if x >= self.largura || y >= ALTURA {
            return None;
        }
        self.pontos
            .get(y.checked_mul(self.largura)?.checked_add(x)?)
            .copied()
    }

    /// Grava a cor (só os 9 bits baixos). Fora da área não faz nada e devolve `false`.
    pub fn definir(&mut self, x: usize, y: usize, cor: u16) -> bool {
        if x >= self.largura || y >= ALTURA {
            return false;
        }
        let Some(i) = y.checked_mul(self.largura).and_then(|b| b.checked_add(x)) else {
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

    /// Os pontos de uma linha, para escrita direta. Os valores já devem ter 9 bits.
    pub fn linha_mut(&mut self, y: usize) -> Option<&mut [u16]> {
        if y >= ALTURA {
            return None;
        }
        let ini = y.checked_mul(self.largura)?;
        let fim = ini.checked_add(self.largura)?;
        self.pontos.get_mut(ini..fim)
    }

    /// Os pontos, linha após linha.
    pub fn pontos(&self) -> &[u16] {
        &self.pontos
    }

    /// Serialização usada no hash da imagem: cada ponto em 2 bytes, little-endian.
    pub fn para_bytes(&self) -> Vec<u8> {
        let mut saida = Vec::with_capacity(self.pontos.len().saturating_mul(2));
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
