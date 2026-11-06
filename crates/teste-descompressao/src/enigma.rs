//! Compressor Enigma de teste. Recebe palavras "cruas" (sem o tile inicial) e escolhe o tipo
//! de pacote pelo que encontra. O descompressor soma o tile inicial na saída.

struct Bits {
    bytes: Vec<u8>,
    n: u8,
}

impl Bits {
    fn bit(&mut self, b: bool) {
        if self.n.is_multiple_of(8) {
            self.bytes.push(0);
        }
        if b {
            let ultimo = self.bytes.len() - 1;
            self.bytes[ultimo] |= 0x80 >> (self.n % 8);
        }
        self.n = (self.n + 1) % 8;
    }

    fn campo(&mut self, valor: u16, quantos: u8) {
        for k in (0..quantos).rev() {
            self.bit((valor >> k) & 1 == 1);
        }
    }
}

/// Parâmetros escolhidos a partir das palavras.
pub struct Parametros {
    pub bits_por_valor: u8,
    pub mascara: u8,
    pub incremental: u16,
    pub comum: u16,
}

pub fn parametros_para(palavras: &[u16], incremental: u16, comum: u16) -> Parametros {
    let mut mascara = 0u8;
    let mut maior = 0u16;
    for &p in palavras.iter().chain([&incremental, &comum]) {
        for k in 0..5 {
            if p & (1 << (11 + k)) != 0 {
                mascara |= 1 << k;
            }
        }
        maior = maior.max(p & 0x7FF);
    }
    let bits_por_valor = (16 - maior.leading_zeros()) as u8;
    Parametros {
        bits_por_valor,
        mascara,
        incremental,
        comum,
    }
}

fn valor(b: &mut Bits, p: &Parametros, palavra: u16) {
    for k in (0..5).rev() {
        if p.mascara >> k & 1 == 1 {
            b.bit(palavra & (1 << (11 + k)) != 0);
        }
    }
    b.campo(palavra & 0x7FF, p.bits_por_valor);
}

/// Comprime as palavras. O cabeçalho leva `bits_por_valor` e a máscara de `p`.
pub fn comprimir(palavras: &[u16], p: &Parametros) -> Vec<u8> {
    let mut saida = vec![p.bits_por_valor, p.mascara];
    saida.extend_from_slice(&p.incremental.to_be_bytes());
    saida.extend_from_slice(&p.comum.to_be_bytes());
    let mut b = Bits {
        bytes: Vec::new(),
        n: 0,
    };
    let mut incremental = p.incremental;
    let mut i = 0;
    while i < palavras.len() {
        let w = palavras[i];
        let resto = palavras.len() - i;
        if w == incremental {
            let mut k = 1;
            while k < resto.min(16) && palavras[i + k] == incremental.wrapping_add(k as u16) {
                k += 1;
            }
            b.bit(false);
            b.bit(false);
            b.campo((k - 1) as u16, 4);
            incremental = incremental.wrapping_add(k as u16);
            i += k;
        } else if w == p.comum {
            let mut k = 1;
            while k < resto.min(16) && palavras[i + k] == w {
                k += 1;
            }
            b.bit(false);
            b.bit(true);
            b.campo((k - 1) as u16, 4);
            i += k;
        } else {
            // Corrida em linha: igual, subindo ou descendo.
            let mut melhor = (0u16, 1usize);
            for modo in 0..3u16 {
                let mut k = 1;
                while k < resto.min(16) {
                    let esperado = match modo {
                        0 => w,
                        1 => w.wrapping_add(k as u16),
                        _ => w.wrapping_sub(k as u16),
                    };
                    if palavras[i + k] != esperado {
                        break;
                    }
                    k += 1;
                }
                if k > melhor.1 {
                    melhor = (modo, k);
                }
            }
            if melhor.1 >= 2 {
                b.bit(true);
                b.campo(melhor.0, 2);
                b.campo((melhor.1 - 1) as u16, 4);
                valor(&mut b, p, w);
                i += melhor.1;
            } else {
                // Lista de literais (1 a 15 palavras).
                let k = resto.min(15);
                b.bit(true);
                b.campo(3, 2);
                b.campo((k - 1) as u16, 4);
                for &x in &palavras[i..i + k] {
                    valor(&mut b, p, x);
                }
                i += k;
            }
        }
    }
    // Fim: 1, modo 3, contagem 15.
    b.bit(true);
    b.campo(3, 2);
    b.campo(15, 4);
    saida.extend_from_slice(&b.bytes);
    saida
}
