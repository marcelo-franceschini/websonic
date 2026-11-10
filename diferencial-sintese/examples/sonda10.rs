// Efeito do espaco entre a escrita do endereco e a do dado ($28) no instante em que a tecla chega ao envelope.
use sonic1_diferencial_sintese::RefOpn2;
fn main() {
    for &(canal, op) in &[(0u8, 0usize), (0, 1), (0, 2), (0, 3)] {
        let mut v = Vec::new();
        for esp in 1..=30usize {
            let mut r = RefOpn2::novo();
            let mut w = |p: u32, a: u8, d: u8| { r.escreve(p, a); for _ in 0..8 { r.ciclo(); } r.escreve(p + 1, d); for _ in 0..60 { r.ciclo(); } };
            for o in 0..4u8 { w(0, 0x50 + o * 4, 0x1F); w(0, 0x40 + o * 4, 0); }
            while r.estado(8)[0] != 0 { r.ciclo(); }
            r.escreve(0, 0x28);
            for _ in 0..esp { r.ciclo(); }
            let bit = [0x10u8, 0x40, 0x20, 0x80][op];
            let slot = op * 6 + canal as usize;
            r.escreve(1, bit | canal);
            // tempo absoluto do dado (apos esp ciclos): alvo = esp
            let mut hit = 0;
            for k in 1..200 { r.ciclo(); if r.estado(1)[slot] == 0 { hit = k; break; } }
            v.push(esp as i32 + hit as i32);
        }
        println!("canal{canal} grupo{op}: tempo absoluto do acerto (esp + k): {v:?}");
    }
}
