//! O núcleo contra o modelo independente do barramento do Z80, em sequências pseudoaleatórias
//! com semente fixa (gerador congruencial, sem dependências).

use sonic1_core::memoria::{Memoria, Tamanho};
use sonic1_core::som::z80::Banco;
use sonic1_teste_som::modelo_barramento::Modelo;

struct Lcg(u64);
impl Lcg {
    fn proximo(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        u32::try_from(self.0 >> 33).unwrap()
    }
}

#[test]
fn busack_e_visibilidade_da_ram_do_z80_concordam_com_o_modelo() {
    let mut rng = Lcg(0x1234_5678);
    for _ in 0..200 {
        let mut m = Memoria::nova(vec![0; 16]);
        let mut modelo = Modelo::novo();
        for _ in 0..60 {
            match rng.proximo() % 4 {
                0 => {
                    let pedir = rng.proximo() % 2 == 0;
                    m.escrever(0xA1_1100, Tamanho::Word, if pedir { 0x100 } else { 0 })
                        .unwrap();
                    modelo.busreq = pedir;
                }
                1 => {
                    let soltar = rng.proximo() % 2 == 0;
                    m.escrever(0xA1_1200, Tamanho::Word, if soltar { 0x100 } else { 0 })
                        .unwrap();
                    modelo.reset = !soltar;
                }
                2 => {
                    // Escreve um byte novo na RAM do Z80 e lê de volta: só vale com BUSACK.
                    let v = u32::from(u8::try_from(rng.proximo() % 255).unwrap() + 1);
                    let antes = m.ram_z80()[0x20];
                    m.escrever(0xA0_0020, Tamanho::Byte, v).unwrap();
                    let depois = m.ram_z80()[0x20];
                    if modelo.busack() {
                        assert_eq!(u32::from(depois), v);
                    } else {
                        assert_eq!(depois, antes);
                    }
                }
                _ => {}
            }
            assert_eq!(m.busack(), modelo.busack());
        }
    }
}

#[test]
fn registrador_de_banco_concorda_com_o_modelo() {
    let mut rng = Lcg(99);
    for _ in 0..500 {
        let mut b = Banco::default();
        let mut modelo = Modelo::novo();
        let n = rng.proximo() % 30;
        for _ in 0..n {
            let bit = rng.proximo() % 2 == 1;
            b.escrever_bit(bit);
            modelo.escrever_bit_de_banco(bit);
            assert_eq!(b.valor(), modelo.banco());
        }
    }
}
