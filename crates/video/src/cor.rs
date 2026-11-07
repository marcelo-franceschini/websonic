//! Cor de 9 bits da VDP.
//!
//! Uma palavra da CRAM guarda a cor no formato `0000 BBB0 GGG0 RRR0`. O ponto do quadro guarda
//! os 9 bits úteis juntos: `BBBGGGRRR`, de 0 a 511. Fonte do formato: documentação pública da
//! CRAM da VDP (Plutiedev, página de cores; a confirmar na sub-etapa 4.0, ver `video.md`).

/// Número de cores distintas.
pub const CORES: u16 = 512;

/// Converte uma palavra da CRAM no valor de 9 bits do ponto. Bits fora do formato são ignorados.
pub const fn da_palavra_cram(palavra: u16) -> u16 {
    let r = (palavra >> 1) & 0x7;
    let g = (palavra >> 5) & 0x7;
    let b = (palavra >> 9) & 0x7;
    (b << 6) | (g << 3) | r
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn canais_nos_lugares() {
        assert_eq!(da_palavra_cram(0x0000), 0);
        assert_eq!(da_palavra_cram(0x000E), 0b000_000_111);
        assert_eq!(da_palavra_cram(0x00E0), 0b000_111_000);
        assert_eq!(da_palavra_cram(0x0E00), 0b111_000_000);
        assert_eq!(da_palavra_cram(0x0EEE), 511);
    }

    #[test]
    fn bits_fora_do_formato_sao_ignorados() {
        assert_eq!(da_palavra_cram(0xF111), 0);
        assert!(da_palavra_cram(0xFFFF) < CORES);
    }
}
