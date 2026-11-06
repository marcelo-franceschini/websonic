//! Compressor Nemesis de teste, com uma tabela de códigos fixa:
//! - `00`: 8 nibbles 0; `01`: 1 nibble 0;
//! - `1000` a `1110`: um nibble de 1 a 7;
//! - `111100` a `111110`: um nibble de 8 a 10;
//! - nibbles 11 a 15 e corridas de 2 a 8 saem pelo código em linha `111111`.

struct Bits {
    bytes: Vec<u8>,
    n: u8,
}

impl Bits {
    fn campo(&mut self, valor: u16, quantos: u8) {
        for k in (0..quantos).rev() {
            if self.n.is_multiple_of(8) {
                self.bytes.push(0);
            }
            if (valor >> k) & 1 == 1 {
                let ultimo = self.bytes.len() - 1;
                self.bytes[ultimo] |= 0x80 >> (self.n % 8);
            }
            self.n = (self.n + 1) % 8;
        }
    }
}

/// Tabela fixa em bytes, terminada por 0xFF.
fn tabela() -> Vec<u8> {
    let mut t = vec![0x80]; // nibble 0
    t.extend_from_slice(&[(7 << 4) | 2, 0b00]); // 8 nibbles, codigo 00
    t.extend_from_slice(&[2, 0b01]); // 1 nibble, codigo 01
    for n in 1..=7u8 {
        t.push(0x80 | n);
        t.extend_from_slice(&[4, 0b1000 + (n - 1)]);
    }
    for n in 8..=10u8 {
        t.push(0x80 | n);
        t.extend_from_slice(&[6, 0b111100 + (n - 8)]);
    }
    t.push(0xFF);
    t
}

/// Comprime tiles (múltiplo de 32 bytes). Com `xor`, aplica o modo XOR.
pub fn comprimir(dados: &[u8], xor: bool) -> Vec<u8> {
    assert!(dados.len().is_multiple_of(32));
    let tiles = dados.len() / 32;
    let mut fonte = dados.to_vec();
    if xor {
        // Inverso do decodificador: cada linha de 4 bytes vira XOR da linha anterior original.
        for i in (1..fonte.len() / 4).rev() {
            for j in 0..4 {
                fonte[i * 4 + j] ^= dados[(i - 1) * 4 + j];
            }
        }
    }
    let cabecalho = (tiles as u16) | if xor { 0x8000 } else { 0 };
    let mut saida = cabecalho.to_be_bytes().to_vec();
    saida.extend_from_slice(&tabela());
    let nibbles: Vec<u8> = fonte.iter().flat_map(|b| [b >> 4, b & 0xF]).collect();
    let mut b = Bits {
        bytes: Vec::new(),
        n: 0,
    };
    let mut i = 0;
    while i < nibbles.len() {
        let n = nibbles[i];
        let mut corrida = 1;
        while i + corrida < nibbles.len() && corrida < 8 && nibbles[i + corrida] == n {
            corrida += 1;
        }
        if n == 0 && corrida == 8 {
            b.campo(0b00, 2);
            i += 8;
        } else if corrida >= 2 && n != 0 || (n == 0 && corrida >= 2) || n >= 11 {
            // Em linha: 111111, contagem-1 em 3 bits, nibble em 4 bits.
            let c = if n == 0 && corrida == 8 { 8 } else { corrida };
            b.campo(0b111111, 6);
            b.campo((c - 1) as u16, 3);
            b.campo(n as u16, 4);
            i += c;
        } else if n == 0 {
            b.campo(0b01, 2);
            i += 1;
        } else if n <= 7 {
            b.campo(0b1000 + (n - 1) as u16, 4);
            i += 1;
        } else {
            b.campo(0b111100 + (n - 8) as u16, 6);
            i += 1;
        }
    }
    saida.extend_from_slice(&b.bytes);
    saida
}
