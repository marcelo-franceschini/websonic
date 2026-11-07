//! Ajudantes dos testes do renderizador: aritmética explícita, sorteio e a cena aleatória.

use sonic1_video::renderizador::{Escrita, Renderizador};

// ---- Aritmética explícita (os lints do núcleo valem nos testes) ----
pub fn u(i: i32) -> usize {
    usize::try_from(i).unwrap_or(0)
}

/// Gerador pseudoaleatório xorshift com semente fixa.
pub struct Sorteio(pub u64);

impl Sorteio {
    pub fn proximo(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    pub fn byte(&mut self) -> u8 {
        self.proximo().to_le_bytes()[0]
    }
    pub fn palavra(&mut self) -> u16 {
        u16::from_le_bytes([
            self.proximo().to_le_bytes()[1],
            self.proximo().to_le_bytes()[2],
        ])
    }
    pub fn ate(&mut self, n: u16) -> u16 {
        u16::try_from(self.proximo().checked_rem(u64::from(n)).unwrap_or(0)).unwrap_or(0)
    }
}

pub fn reg(r: &mut Renderizador, linha: u16, n: u8, v: u8) {
    let w = 0x8000u16 | (u16::from(n) << 8) | u16::from(v);
    r.aplicar(linha, 0, Escrita::Controle(w));
}

pub fn comando(r: &mut Renderizador, linha: u16, codigo_alto: u16, endereco: u32) {
    // Primeira palavra: CD1 CD0 e A13..A0. Segunda: CD5..CD2 e A16..A14.
    let primeira = (codigo_alto & 3) << 14 | u16::try_from(endereco & 0x3fff).unwrap_or(0);
    let segunda =
        (((codigo_alto >> 2) & 0xf) << 4) | u16::try_from((endereco >> 14) & 7).unwrap_or(0);
    r.aplicar(linha, 0, Escrita::Controle(primeira));
    r.aplicar(linha, 0, Escrita::Controle(segunda));
}

pub fn dados(r: &mut Renderizador, linha: u16, v: u16) {
    r.aplicar(
        linha,
        0,
        Escrita::Dados {
            valor: v,
            tamanho: 2,
        },
    );
}

/// Cena aleatória estática. Devolve o renderizador depois de desenhar um quadro completo.
pub fn cena(semente: u64) -> Renderizador {
    let mut s = Sorteio(semente | 1);
    let mut r = Renderizador::novo();
    let l = 240; // dentro do VBlank do quadro anterior
    r.avancar_ate(l, 0);
    // Modo 5, 40 colunas, imagem ligada, DMA permitido, incremento 2.
    reg(&mut r, l, 1, 0x74);
    reg(&mut r, l, 12, 0x81);
    reg(&mut r, l, 0, 0x04);
    reg(&mut r, l, 15, 2);
    let nome_a: u8 = [0x30, 0x20, 0x10][u(i32::from(s.ate(3)))];
    let nome_b: u8 = [0x07, 0x05, 0x06][u(i32::from(s.ate(3)))];
    reg(&mut r, l, 2, nome_a);
    reg(&mut r, l, 4, nome_b);
    reg(&mut r, l, 5, 0x6c); // tabela de sprites em $D800
    reg(&mut r, l, 13, 0x37); // tabela de rolagem em $DC00
    reg(
        &mut r,
        l,
        16,
        [0x00, 0x01, 0x10, 0x11][u(i32::from(s.ate(4)))],
    );
    let modo_h = [0x00u8, 0x02, 0x03][u(i32::from(s.ate(3)))];
    let modo_v = if s.ate(2) == 0 { 0x00 } else { 0x04 };
    reg(&mut r, l, 11, modo_h | modo_v);
    reg(&mut r, l, 7, s.byte() & 0x3f);
    // VRAM inteira aleatória (por palavras), exceto que mantemos zeros em partes para haver transparência.
    comando(&mut r, l, 0b0001, 0);
    for i in 0..32768u32 {
        let mut w = s.palavra();
        if i % 3 == 0 {
            w = 0;
        }
        dados(&mut r, l, w);
    }
    // Tabela de sprites: 24 sprites encadeados, posições espalhadas.
    comando(&mut r, l, 0b0001, 0xd800);
    for i in 0..80u16 {
        let v = 0x80u16.wrapping_add(s.ate(224)).wrapping_sub(8);
        dados(&mut r, l, v & 0x1ff);
        let tam = u16::from(s.byte() & 0x0f);
        let elo = if i < 23 { i.wrapping_add(1) } else { 0 };
        dados(&mut r, l, (tam << 8) | elo);
        dados(&mut r, l, s.palavra() & 0xe7ff);
        let h = 0x80u16.wrapping_add(s.ate(340)).wrapping_sub(10);
        dados(&mut r, l, if i == 5 { 0 } else { h & 0x1ff });
    }
    // CRAM e VSRAM.
    comando(&mut r, l, 0b0011, 0);
    for _ in 0..64 {
        dados(&mut r, l, s.palavra() & 0x0eee);
    }
    comando(&mut r, l, 0b0101, 0);
    for _ in 0..40 {
        dados(&mut r, l, s.palavra() & 0x03ff);
    }
    r.drenar_vdp();
    // Um quadro inteiro para tudo assentar, depois outro para medir.
    r.avancar_ate(232, 0);
    r.avancar_ate(l, 0);
    r.avancar_ate(232, 0);
    r
}
