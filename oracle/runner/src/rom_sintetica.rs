//! ROM sintetica para testes. Escrita por nos, em palavras do 68000. Nao tem dado da ROM real.

/// Tamanho da ROM sintetica (128 KiB), para nao haver leitura fora dos limites.
pub const TAMANHO: usize = 0x2_0000;
/// Endereco do laco `bra.s *` usado como destino dos vetores nao definidos.
pub const ENDERECO_PARADO: u32 = 0x300;

/// Monta uma ROM sintetica: tabela de vetores, cabecalho minimo e codigo.
pub struct RomSintetica {
    bytes: Vec<u8>,
}

impl RomSintetica {
    /// Cria a ROM com SSP e PC iniciais. Os outros vetores apontam para um laco infinito.
    pub fn nova(ssp: u32, pc: u32) -> Self {
        let mut rom = Self {
            bytes: vec![0; TAMANHO],
        };
        for i in 2..64 {
            rom.vetor(i, ENDERECO_PARADO);
        }
        rom.vetor(0, ssp);
        rom.vetor(1, pc);
        // Cabecalho minimo: o nome do console e a regiao (J) nas posicoes que o jgenesis le.
        rom.bytes_em(0x100, b"SEGA MEGA DRIVE ");
        rom.bytes_em(0x120, b"WEBSONIC SYNTHETIC TEST ROM");
        rom.bytes_em(0x1F0, b"J  ");
        rom.palavras(ENDERECO_PARADO, &[0x60FE]); // bra.s *
        rom
    }

    /// Define o vetor `indice` (0 a 63).
    pub fn vetor(&mut self, indice: usize, endereco: u32) -> &mut Self {
        let pos = indice * 4;
        self.bytes[pos..pos + 4].copy_from_slice(&endereco.to_be_bytes());
        self
    }

    /// Escreve palavras de 16 bits (big-endian) a partir de `endereco`.
    pub fn palavras(&mut self, endereco: u32, palavras: &[u16]) -> &mut Self {
        let mut pos = endereco as usize;
        for p in palavras {
            self.bytes[pos..pos + 2].copy_from_slice(&p.to_be_bytes());
            pos += 2;
        }
        self
    }

    /// Escreve bytes a partir de `endereco`.
    pub fn bytes_em(&mut self, endereco: u32, dados: &[u8]) -> &mut Self {
        let pos = endereco as usize;
        self.bytes[pos..pos + dados.len()].copy_from_slice(dados);
        self
    }

    pub fn finalizar(&self) -> Vec<u8> {
        self.bytes.clone()
    }
}

/// SSP usado pelos programas sinteticos.
pub const SSP: u32 = 0x00FF_8000;

/// Programa 1: le o controle e guarda em $FF0000, em laco.
pub fn programa_entrada() -> Vec<u8> {
    let mut rom = RomSintetica::nova(SSP, 0x200);
    rom.palavras(
        0x200,
        &[
            0x46FC, 0x2700, // move.w #$2700,sr
            0x13F9, 0x00A1, 0x0003, 0x00FF, 0x0000, // move.b $A10003,$FF0000   (em $204)
            0x60F4, // bra.s $204  (deslocamento -12)
        ],
    );
    rom.finalizar()
}

/// Programa 2: liga a VDP e a interrupcao de VBlank, conta interrupcoes em $FF0100.
pub fn programa_vblank() -> Vec<u8> {
    let mut rom = RomSintetica::nova(SSP, 0x200);
    rom.palavras(
        0x200,
        &[
            0x46FC, 0x2700, // move.w #$2700,sr
            0x33FC, 0x8164, 0x00C0,
            0x0004, // move.w #$8164,$C00004  (reg 1: video e VBlank ligados)
            0x46FC, 0x2000, // move.w #$2000,sr      (libera as interrupcoes)
            0x60FE, // bra.s *   (em $210)
        ],
    );
    rom.vetor(30, 0x400); // interrupcao de nivel 6 (VBlank)
    rom.palavras(
        0x400,
        &[
            0x52B9, 0x00FF, 0x0100, // addq.l #1,$FF0100
            0x4E73, // rte
        ],
    );
    rom.finalizar()
}

/// Programa 3: carrega um programa no Z80 pelo 68000, solta o Z80 e escreve no YM2612.
pub fn programa_z80() -> Vec<u8> {
    // Programa do Z80, em Z80 (escrito por nos):
    //   ld hl,$4000 ; ld (hl),$22 ; inc hl ; ld (hl),$0F     (registrador $22 do YM)
    //   ld hl,$4000 ; ld (hl),$2A ; inc hl ; ld (hl),$80     (DAC)
    //   jr $
    let z80: [u8; 18] = [
        0x21, 0x00, 0x40, 0x36, 0x22, 0x23, 0x36, 0x0F, 0x21, 0x00, 0x40, 0x36, 0x2A, 0x23, 0x36,
        0x80, 0x18, 0xFE,
    ];
    let mut codigo: Vec<u16> = vec![
        0x46FC, 0x2700, // move.w #$2700,sr
        0x33FC, 0x0100, 0x00A1, 0x1200, // libera o reset do Z80
        0x33FC, 0x0100, 0x00A1, 0x1100, // pede o barramento
        0x0839, 0x0000, 0x00A1, 0x1100, // btst #0,$A11100   (em $214)
        0x66F6, // bne.s $214
    ];
    for (i, byte) in z80.iter().enumerate() {
        // move.b #byte,$A000xx
        codigo.extend_from_slice(&[0x13FC, u16::from(*byte), 0x00A0, i as u16]);
    }
    codigo.extend_from_slice(&[
        0x13FC, 0x0030, 0x00A0, 0x4000, // move.b #$30,$A04000   (YM, escrito pelo 68000)
        0x13FC, 0x0055, 0x00A0, 0x4001, // move.b #$55,$A04001
        0x33FC, 0x0000, 0x00A1, 0x1200, // reset do Z80 ligado
        0x33FC, 0x0100, 0x00A1, 0x1200, // reset solto: o Z80 comeca em $0000
        0x33FC, 0x0000, 0x00A1, 0x1100, // devolve o barramento
        0x60FE, // bra.s *
    ]);
    let mut rom = RomSintetica::nova(SSP, 0x200);
    rom.palavras(0x200, &codigo);
    rom.finalizar()
}

/// Programa 4: video. Liga o display em H40, pinta a cor 0 da CRAM de azul maximo ($0E00) e faz um
/// DMA de 4 palavras da ROM (em $1000) para a VRAM em $8000. Depois fica em laco.
pub fn programa_video() -> Vec<u8> {
    let mut rom = RomSintetica::nova(SSP, 0x200);
    rom.palavras(
        0x200,
        &[
            0x46FC, 0x2700, // move.w #$2700,sr
            0x33FC, 0x8F02, 0x00C0, 0x0004, // reg 15 = 2 (incremento automatico)
            0x33FC, 0x8004, 0x00C0, 0x0004, // reg 0 = $04 (cor de 9 bits)
            0x33FC, 0x8C81, 0x00C0, 0x0004, // reg 12 = $81 (40 colunas)
            0x33FC, 0x8700, 0x00C0, 0x0004, // reg 7 = 0 (fundo: cor 0 da paleta 0)
            0x33FC, 0x8174, 0x00C0, 0x0004, // reg 1 = $74 (video ligado, VBlank, DMA)
            0x23FC, 0xC000, 0x0000, 0x00C0, 0x0004, // CRAM, endereco 0
            0x33FC, 0x0E00, 0x00C0, 0x0000, // cor 0 = azul maximo
            0x33FC, 0x9304, 0x00C0, 0x0004, // reg 19 = 4 (comprimento do DMA)
            0x33FC, 0x9400, 0x00C0, 0x0004, // reg 20 = 0
            0x33FC, 0x9500, 0x00C0, 0x0004, // reg 21 = 0 (fonte, palavras)
            0x33FC, 0x9608, 0x00C0, 0x0004, // reg 22 = 8 (fonte = $1000 em bytes)
            0x33FC, 0x9700, 0x00C0, 0x0004, // reg 23 = 0 (68000 para VDP)
            0x23FC, 0x4000, 0x0082, 0x00C0,
            0x0004, // VRAM, endereco $8000 (longe do tile 0), com DMA
            0x60FE, // bra.s *
        ],
    );
    rom.palavras(0x1000, &PALAVRAS_DMA_VIDEO);
    rom.finalizar()
}

/// As 4 palavras que o DMA de `programa_video` transfere.
pub const PALAVRAS_DMA_VIDEO: [u16; 4] = [0x1111, 0x2222, 0x3333, 0x4444];
