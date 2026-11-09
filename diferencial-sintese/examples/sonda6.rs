// Latencia da tecla por slot: ciclos entre a escrita do dado de $28 e o nivel do envelope mudar.
use sonic1_diferencial_sintese::RefOpn2;
fn main() {
    for ch in 0..6u8 {
        let mut linha = String::new();
        for op in 0..4u8 {
            let mut r = RefOpn2::novo();
            let (parte, off) = if ch < 3 { (0u32, ch) } else { (2, ch - 3) };
            let mut w = |p: u32, a: u8, d: u8| { r.escreve(p, a); for _ in 0..8 { r.ciclo(); } r.escreve(p + 1, d); for _ in 0..60 { r.ciclo(); } };
            for o in 0..4u8 { w(parte, 0x50 + off + 4 * o, 0x1F); w(parte, 0x40 + off + 4 * o, 0); }
            while r.estado(8)[0] != 0 { r.ciclo(); }
            // bit do $28: S1=0x10, S2=0x20, S3=0x40, S4=0x80; endereco o: S1=0,S3=1,S2=2,S4=3
            let bit = [0x10u8, 0x40, 0x20, 0x80][op as usize];
            let canal = if ch < 3 { ch } else { ch + 1 };
            r.escreve(0, 0x28);
            for _ in 0..8 { r.ciclo(); }
            r.escreve(1, bit | canal);
            let mut hit = None;
            let slot = (op as usize) * 6 + ch as usize;
            for k in 1..200 { r.ciclo(); if hit.is_none() && r.estado(1)[slot] == 0 { hit = Some(k); } }
            linha += &format!(" op{op}:{:?}", hit);
        }
        println!("ch{ch}:{linha}");
    }
}
