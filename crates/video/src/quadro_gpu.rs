//! Dados de composição por ponto, preservando as alterações de paleta durante a linha.
use crate::camadas::FonteDeTile;
use crate::quadro::{ALTURA, LARGURA};

/// Candidatos de planos e sprites, índices de paleta e paletas em cor de 9 bits.
pub struct QuadroGpu {
    largura: usize,
    altura: usize,
    camadas: Vec<u8>,
    sprites: Vec<u8>,
    contagens: Vec<u8>,
    fundos: Vec<u8>,
    seletores: Vec<u16>,
    paletas: Vec<u16>,
}

impl Default for QuadroGpu {
    fn default() -> Self {
        Self::novo()
    }
}

impl QuadroGpu {
    pub fn novo() -> Self {
        Self::com_dimensoes(LARGURA, ALTURA)
    }

    pub fn com_dimensoes(largura: usize, altura: usize) -> Self {
        let pontos = largura.saturating_mul(altura);
        Self {
            largura,
            altura,
            camadas: vec![0; pontos.saturating_mul(4)],
            sprites: vec![0; pontos.saturating_mul(20).saturating_mul(2)],
            contagens: vec![0; altura],
            fundos: vec![0; pontos],
            seletores: vec![0; pontos],
            paletas: vec![0; 64],
        }
    }

    pub fn widescreen_16_9() -> Self {
        Self::com_dimensoes(426, ALTURA)
    }

    pub fn largura(&self) -> usize {
        self.largura
    }

    pub fn altura(&self) -> usize {
        self.altura
    }

    pub fn pontos_total(&self) -> usize {
        self.largura.saturating_mul(self.altura)
    }

    pub fn redimensionar(&mut self, largura: usize, altura: usize) {
        if self.largura == largura && self.altura == altura {
            return;
        }
        self.largura = largura;
        self.altura = altura;
        let pontos = largura.saturating_mul(altura);
        self.camadas = vec![0; pontos.saturating_mul(4)];
        self.sprites = vec![0; pontos.saturating_mul(20).saturating_mul(2)];
        self.contagens = vec![0; altura];
        self.fundos = vec![0; pontos];
        self.seletores = vec![0; pontos];
        self.paletas = vec![0; 64];
    }

    pub fn iniciar_quadro(&mut self) {
        self.paletas.clear();
        self.paletas.resize(64, 0);
        self.contagens.fill(0);
    }

    /// Retorna None se o quadro exceder a capacidade de índices de paleta.
    pub fn paleta(&mut self, cores: &[u16; 64]) -> Option<u16> {
        if self.paletas.ends_with(cores) {
            return u16::try_from(self.paletas.len().checked_div(64)?.checked_sub(1)?).ok();
        }
        let indice = self.paletas.len().checked_div(64)?;
        if indice >= 0x8000 {
            return None;
        }
        self.paletas.extend_from_slice(cores);
        u16::try_from(indice).ok()
    }

    pub fn ponto(
        &mut self,
        x: usize,
        y: usize,
        a: FonteDeTile,
        b: FonteDeTile,
        fundo: u8,
        paleta: u16,
    ) {
        if x >= self.largura || y >= self.altura {
            return;
        }
        let i = y.saturating_mul(self.largura).saturating_add(x);
        let inicio = i.saturating_mul(4);
        if let Some(bytes) = self.camadas.get_mut(inicio..inicio.saturating_add(4)) {
            bytes.copy_from_slice(&[a.byte, a.atributos, b.byte, b.atributos]);
        }
        if let Some(f) = self.fundos.get_mut(i) {
            *f = fundo;
        }
        if let Some(seletor) = self.seletores.get_mut(i) {
            *seletor = paleta;
        }
    }

    pub fn sprites_da_linha(
        &mut self,
        y: usize,
        inicio: usize,
        quantidade: usize,
        fontes: &[FonteDeTile],
    ) {
        if y >= self.altura || self.largura == 0 {
            return;
        }
        let anterior = usize::from(self.contagens.get(y).copied().unwrap_or(0));
        let quantidade = quantidade.min(20);
        let total = if inicio == 0 {
            quantidade
        } else {
            anterior.max(quantidade)
        };
        if let Some(c) = self.contagens.get_mut(y) {
            *c = u8::try_from(total).unwrap_or(20);
        }
        for (i, fonte) in fontes
            .iter()
            .enumerate()
            .take(self.largura.saturating_mul(total))
        {
            if let Some(resto) = i.checked_rem(self.largura)
                && resto < inicio
            {
                continue;
            }
            let base = y
                .saturating_mul(self.largura)
                .saturating_mul(20)
                .saturating_add(i)
                .saturating_mul(2);
            if let Some(d) = self.sprites.get_mut(base..base.saturating_add(2)) {
                d.copy_from_slice(&[fonte.byte, fonte.atributos]);
            }
        }
    }

    /// Sobrepõe a cor de um ponto da CRAM ou um ponto com o vídeo desligado.
    pub fn cor_direta(&mut self, x: usize, y: usize, cor: u16) {
        if x >= self.largura || y >= self.altura {
            return;
        }
        let i = y.saturating_mul(self.largura).saturating_add(x);
        if let Some(seletor) = self.seletores.get_mut(i) {
            *seletor = 0x8000 | (cor & 0x1ff);
        }
    }

    pub fn camadas(&self) -> &[u8] {
        &self.camadas
    }
    pub fn seletores(&self) -> &[u16] {
        &self.seletores
    }
    pub fn sprites(&self) -> &[u8] {
        &self.sprites
    }

    /// Empacota o mesmo número de candidatos por linha reutilizando o vetor de destino.
    pub fn sprites_compactos_em(&self, saida: &mut Vec<u8>) {
        let quantidade = self.contagens.iter().copied().max().unwrap_or(0).max(1);
        let n = usize::from(quantidade);
        let tamanho_linha = self.largura.saturating_mul(n).saturating_mul(2);
        let total = self.altura.saturating_mul(tamanho_linha);
        saida.resize(total, 0);
        saida.fill(0);
        for (y, contagem) in self.contagens.iter().copied().enumerate() {
            let origem = y
                .saturating_mul(self.largura)
                .saturating_mul(20)
                .saturating_mul(2);
            let destino = y.saturating_mul(tamanho_linha);
            let tamanho = usize::from(contagem)
                .saturating_mul(self.largura)
                .saturating_mul(2);
            if let (Some(a), Some(b)) = (
                self.sprites.get(origem..origem.saturating_add(tamanho)),
                saida.get_mut(destino..destino.saturating_add(tamanho)),
            ) {
                b.copy_from_slice(a);
            }
        }
    }

    /// Empacota o mesmo número de candidatos por linha; linhas menores são zeradas.
    pub fn sprites_compactos(&self) -> Vec<u8> {
        let mut saida = Vec::new();
        self.sprites_compactos_em(&mut saida);
        saida
    }
    pub fn fundos(&self) -> &[u8] {
        &self.fundos
    }
    pub fn paletas(&self) -> &[u16] {
        &self.paletas
    }
}
