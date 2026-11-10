// DAC: saida do canal 6 e do pino para varios valores de $2A.
use sonic1_diferencial_sintese::RefOpn2;
fn main() {
    let mut r = RefOpn2::novo();
    let mut w = |r: &mut RefOpn2, p: u32, a: u8, d: u8| { r.escreve(p, a); for _ in 0..8 { r.ciclo(); } r.escreve(p + 1, d); for _ in 0..60 { r.ciclo(); } };
    w(&mut r, 2, 0xB4, 0xC0);
    w(&mut r, 0, 0x2B, 0x80);
    for v in [0u8, 1, 2, 0x7F, 0x80, 0x81, 0xFE, 0xFF] {
        w(&mut r, 0, 0x2A, v);
        for _ in 0..48 { r.ciclo(); }
        let mut pino = Vec::new();
        for _ in 0..24 { pino.push(r.ciclo().0); }
        println!("dac {v:02X}: ch_out5={} pino={:?}", r.estado(4)[5], pino);
    }
}
