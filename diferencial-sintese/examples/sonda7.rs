// Latencia da tecla em funcao da fase da escrita: tempo (em ciclos) da escrita do dado ate o nivel do
// envelope de um slot mudar, para cada fase j da escrita (0..23).
use sonic1_diferencial_sintese::RefOpn2;
fn main() {
    // (nome, canal, grupo)
    for &(canal, op) in &[(0u8, 0usize), (1, 0), (0, 1), (1, 1), (2, 1), (5, 1), (0, 2), (0, 3)] {
        let mut v = Vec::new();
        for j in 0..24usize {
            let mut r = RefOpn2::novo();
            let mut w = |p: u32, a: u8, d: u8| { r.escreve(p, a); for _ in 0..8 { r.ciclo(); } r.escreve(p + 1, d); for _ in 0..60 { r.ciclo(); } };
            for o in 0..4u8 { w(0, 0x50 + o * 4, 0x1F); w(0, 0x40 + o * 4, 0); w(0, 0x51 + o * 4, 0x1F); w(0, 0x41 + o * 4, 0); w(0, 0x52 + o * 4, 0x1F); w(0, 0x42 + o * 4, 0); w(2, 0x50 + o * 4 + 2, 0x1F); w(2, 0x40 + o * 4 + 2, 0); }
            while r.estado(8)[0] != 0 { r.ciclo(); }
            r.escreve(0, 0x28);
            for _ in 0..8 { r.ciclo(); }
            for _ in 0..j { r.ciclo(); }
            let bit = [0x10u8, 0x40, 0x20, 0x80][op];
            let slot = op * 6 + canal as usize;
            r.escreve(1, bit | (if canal < 3 { canal } else { canal + 1 }));
            let mut hit = 0;
            for k in 1..200 { r.ciclo(); if r.estado(1)[slot] == 0 { hit = k; break; } }
            v.push(hit);
        }
        println!("canal{canal} grupo{op}: {v:?}");
    }
}
