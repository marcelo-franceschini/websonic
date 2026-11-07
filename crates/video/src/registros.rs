//! Registradores da VDP (24 registradores de 8 bits) e os campos que o renderizador usa.
//!
//! Fonte do mapa de bits: documentação pública dos registradores (Plutiedev, página
//! vdp-registers, consultada em 06/11/2026) e o comportamento do oráculo (ver `video.md`).

/// Quantidade de registradores.
pub const QUANTIDADE: usize = 24;

/// Tamanho de um plano de rolagem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TamanhoPlano {
    T32,
    T64,
    T128,
    /// Valor proibido (bits 10). O oráculo trata como 32 de largura e altura especial.
    Invalido,
}

impl TamanhoPlano {
    fn dos_bits(bits: u8) -> Self {
        match bits & 3 {
            0 => Self::T32,
            1 => Self::T64,
            2 => Self::Invalido,
            _ => Self::T128,
        }
    }

    /// Tamanho em células (e em pontos, multiplicando por 8) quando válido.
    pub fn celulas(self) -> u16 {
        match self {
            Self::T32 | Self::Invalido => 32,
            Self::T64 => 64,
            Self::T128 => 128,
        }
    }
}

/// Modo de rolagem horizontal (registrador 11, bits 1 e 0).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RolagemH {
    Tela,
    Invalida,
    Celula,
    Linha,
}

/// Modo de rolagem vertical (registrador 11, bit 2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RolagemV {
    Tela,
    DuasCelulas,
}

/// Os 24 registradores brutos e as leituras decodificadas.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Registros {
    pub brutos: [u8; QUANTIDADE],
}

impl Default for Registros {
    fn default() -> Self {
        Self::novo()
    }
}

impl Registros {
    pub const fn novo() -> Self {
        Self {
            brutos: [0; QUANTIDADE],
        }
    }

    fn bit(&self, reg: usize, bit: u8) -> bool {
        self.brutos.get(reg).is_some_and(|v| (v >> bit) & 1 == 1)
    }

    fn valor(&self, reg: usize) -> u8 {
        self.brutos.get(reg).copied().unwrap_or(0)
    }

    /// Registrador 0, bit 4: interrupção de linha ligada.
    pub fn interrupcao_de_linha(&self) -> bool {
        self.bit(0, 4)
    }

    /// Registrador 1, bit 6: imagem ligada.
    pub fn imagem_ligada(&self) -> bool {
        self.bit(1, 6)
    }

    /// Registrador 1, bit 4: DMA permitido.
    pub fn dma_permitido(&self) -> bool {
        self.bit(1, 4)
    }

    /// Registrador 1, bit 3: 30 linhas de células.
    pub fn v30(&self) -> bool {
        self.bit(1, 3)
    }

    /// Registrador 1, bit 2 desligado: modo 4 do Master System (escritas nos registradores acima
    /// de 10 são ignoradas, como no oráculo).
    pub fn modo4(&self) -> bool {
        !self.bit(1, 2)
    }

    /// Registrador 1, bit 7: VRAM de 128 KiB (não suportado).
    pub fn vram128(&self) -> bool {
        self.bit(1, 7)
    }

    /// Registrador 12: 40 colunas quando o bit 7 ou o bit 0 está ligado.
    pub fn h40(&self) -> bool {
        self.bit(12, 7) || self.bit(12, 0)
    }

    /// Registrador 12, bit 3: sombra e realce.
    pub fn sombra_realce(&self) -> bool {
        self.bit(12, 3)
    }

    /// Registrador 12, bits 2 e 1: entrelaçamento ligado.
    pub fn entrelacado(&self) -> bool {
        self.bit(12, 1) || self.bit(12, 2)
    }

    /// Endereço da tabela de nomes do plano A (registrador 2).
    pub fn nome_a(&self) -> u16 {
        (u16::from(self.valor(2) & 0x38)) << 10
    }

    /// Endereço da tabela de nomes da janela (registrador 3).
    pub fn nome_janela(&self) -> u16 {
        (u16::from(self.valor(3) & 0x3e)) << 10
    }

    /// Endereço da tabela de nomes do plano B (registrador 4).
    pub fn nome_b(&self) -> u16 {
        (u16::from(self.valor(4) & 0x07)) << 13
    }

    /// Endereço da tabela de sprites, com o bit A9 ignorado em 40 colunas (registrador 5).
    pub fn tabela_de_sprites(&self) -> u32 {
        let base = u32::from(self.valor(5)) << 9;
        if self.h40() { base & !0x3ff } else { base }
    }

    /// Paleta e índice da cor de fundo (registrador 7).
    pub fn fundo(&self) -> (u8, u8) {
        let v = self.valor(7);
        ((v >> 4) & 3, v & 0x0f)
    }

    /// Índice de 6 bits da cor de fundo na CRAM.
    pub fn indice_do_fundo(&self) -> u8 {
        let (p, c) = self.fundo();
        (p << 4) | c
    }

    /// Contador da interrupção de linha (registrador 10).
    pub fn contador_de_linha(&self) -> u8 {
        self.valor(10)
    }

    pub fn rolagem_h(&self) -> RolagemH {
        match self.valor(11) & 3 {
            0 => RolagemH::Tela,
            1 => RolagemH::Invalida,
            2 => RolagemH::Celula,
            _ => RolagemH::Linha,
        }
    }

    pub fn rolagem_v(&self) -> RolagemV {
        if self.bit(11, 2) {
            RolagemV::DuasCelulas
        } else {
            RolagemV::Tela
        }
    }

    /// Endereço da tabela de rolagem horizontal (registrador 13).
    pub fn tabela_de_rolagem_h(&self) -> u16 {
        (u16::from(self.valor(13) & 0x3f)) << 10
    }

    /// Incremento automático do endereço (registrador 15).
    pub fn incremento(&self) -> u16 {
        u16::from(self.valor(15))
    }

    /// Largura do plano (registrador 16, bits 1 e 0).
    pub fn largura_do_plano(&self) -> TamanhoPlano {
        TamanhoPlano::dos_bits(self.valor(16))
    }

    /// Altura do plano (registrador 16, bits 5 e 4).
    pub fn altura_do_plano(&self) -> TamanhoPlano {
        TamanhoPlano::dos_bits(self.valor(16) >> 4)
    }

    /// Registradores 17 e 18 diferentes de zero: a janela cobre alguma área (uso fora do inventário).
    pub fn janela_ativa(&self) -> bool {
        (self.valor(17) & 0x9f) != 0 || (self.valor(18) & 0x9f) != 0
    }

    /// Comprimento do DMA (registradores 19 e 20). Zero vale 65536.
    pub fn comprimento_dma(&self) -> u32 {
        let v = u32::from(self.valor(19)) | (u32::from(self.valor(20)) << 8);
        if v == 0 { 0x1_0000 } else { v }
    }

    /// Modo do DMA pelos bits 7 e 6 do registrador 23.
    pub fn modo_dma(&self) -> ModoDma {
        match self.valor(23) >> 6 {
            0 | 1 => ModoDma::MemoriaParaVdp,
            2 => ModoDma::PreenchimentoDeVram,
            _ => ModoDma::CopiaDeVram,
        }
    }
}

/// Modo do DMA.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModoDma {
    MemoriaParaVdp,
    PreenchimentoDeVram,
    CopiaDeVram,
}

#[cfg(test)]
mod testes {
    use super::*;

    fn com(reg: usize, v: u8) -> Registros {
        let mut r = Registros::novo();
        r.brutos[reg] = v;
        r
    }

    #[test]
    fn campos_dos_valores_do_corpus() {
        assert!(com(12, 0x81).h40());
        assert!(!com(12, 0x00).h40());
        assert!(!com(12, 0x81).sombra_realce());
        assert!(!com(12, 0x81).entrelacado());
        assert!(com(1, 0x74).imagem_ligada());
        assert!(com(1, 0x74).dma_permitido());
        assert!(!com(1, 0x74).v30());
        assert!(!com(1, 0x74).modo4());
        assert!(com(1, 0x00).modo4());
        assert_eq!(com(11, 0x03).rolagem_h(), RolagemH::Linha);
        assert_eq!(com(11, 0x00).rolagem_h(), RolagemH::Tela);
        assert_eq!(com(16, 0x01).largura_do_plano(), TamanhoPlano::T64);
        assert_eq!(com(16, 0x01).altura_do_plano(), TamanhoPlano::T32);
    }

    #[test]
    fn enderecos_das_tabelas() {
        assert_eq!(com(2, 0x30).nome_a(), 0xc000);
        assert_eq!(com(4, 0x07).nome_b(), 0xe000);
        assert_eq!(com(13, 0x37).tabela_de_rolagem_h(), 0xdc00);
        let mut r = com(5, 0x6c);
        r.brutos[12] = 0x81;
        // 0x6c << 9 = 0xd800; em 40 colunas o bit A9 e ignorado (ja zero aqui).
        assert_eq!(r.tabela_de_sprites(), 0xd800);
        assert_eq!(com(5, 0x7d).tabela_de_sprites() & 0x200, 0x200);
    }

    #[test]
    fn fundo_e_dma() {
        assert_eq!(com(7, 0x20).fundo(), (2, 0));
        assert_eq!(com(7, 0x3f).indice_do_fundo(), 0x3f);
        assert_eq!(com(19, 0).comprimento_dma(), 0x1_0000);
        let mut r = com(19, 0x40);
        r.brutos[20] = 0x01;
        assert_eq!(r.comprimento_dma(), 0x140);
        assert_eq!(com(23, 0x7f).modo_dma(), ModoDma::MemoriaParaVdp);
        assert_eq!(com(23, 0x80).modo_dma(), ModoDma::PreenchimentoDeVram);
        assert_eq!(com(23, 0xc0).modo_dma(), ModoDma::CopiaDeVram);
    }

    #[test]
    fn janela_zerada_nao_cobre_nada() {
        assert!(!Registros::novo().janela_ativa());
        assert!(com(17, 0x04).janela_ativa());
        assert!(com(18, 0x80).janela_ativa());
    }
}
