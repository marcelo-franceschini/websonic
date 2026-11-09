//! A ponte com saídas sintéticas: DMA da memória para a CRAM e escrita direta na VRAM.
use sonic1_console::Ponte;
use sonic1_core::memoria::{Memoria, PortaVdp, Saida, Tamanho};

fn ctrl(v: u16) -> Saida {
    Saida::Vdp {
        porta: PortaVdp::Controle,
        valor: v,
        tamanho: Tamanho::Word,
    }
}

#[test]
fn dma_da_ram_para_a_cram_chega_a_cram() {
    let mut mem = Memoria::nova(vec![0; 0x100]);
    // Paleta na RAM em $FFFB00: a cor 1 vale $0EEE (branco), a cor 2 vale $000E (vermelho).
    mem.ram_mut()[0xFB02..0xFB04].copy_from_slice(&0x0EEEu16.to_be_bytes());
    mem.ram_mut()[0xFB04..0xFB06].copy_from_slice(&0x000Eu16.to_be_bytes());
    let mut ponte = Ponte::nova();
    // Registrador 1: DMA e display ligados ($34 mais o bit 6).
    let escritas = [
        ctrl(0x8174),
        ctrl(0x8F02),
        // Comprimento $0040 palavras, origem $FFFB00 (>>1 = $7FFD80).
        ctrl(0x9440),
        ctrl(0x9300),
        ctrl(0x96FD),
        ctrl(0x9580),
        ctrl(0x977F),
        // Escrita na CRAM no endereço 0 com DMA: $C000 e depois $0080.
        ctrl(0xC000),
        ctrl(0x0080),
    ];
    // Os registradores 19 e 20 vêm como $9340 e $9400 (comprimento baixo, alto).
    let escritas = {
        let mut e = escritas.to_vec();
        e[2] = ctrl(0x9340);
        e[3] = ctrl(0x9400);
        e
    };
    for s in &escritas {
        ponte.aplicar(&mut mem, s, 224, 0);
    }
    let _ = ponte.fechar_quadro();
    let cram = ponte.renderizador().vdp().cram();
    assert_eq!(cram.get(1).copied(), Some(0x0EEE), "cor 1 na CRAM");
    assert_eq!(cram.get(2).copied(), Some(0x000E), "cor 2 na CRAM");
}
