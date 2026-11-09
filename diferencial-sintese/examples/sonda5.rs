// Mede a tabela de detune no Nuked-OPN2: deslocamento do incremento de fase por DT e codigo de tecla.
use sonic1_diferencial_sintese::RefOpn2;
fn inc(dt: u8, bloco: u8, fnum: u16) -> i32 {
    let mut r = RefOpn2::novo();
    let mut w = |a: u8, d: u8| { r.escreve(0, a); for _ in 0..8 { r.ciclo(); } r.escreve(1, d); for _ in 0..40 { r.ciclo(); } };
    w(0x30, (dt << 4) | 1);
    w(0xA4, (bloco << 3) | (fnum >> 8) as u8);
    w(0xA0, fnum as u8);
    for _ in 0..24 * 6 { r.ciclo(); }
    r.estado(5)[0]
}
fn main() {
    let mut tab = vec![vec![0i32; 32]; 4];
    for bloco in 0..8u8 {
        for (k, f) in [0x020u16, 0x3A0, 0x420, 0x4A0].iter().enumerate() {
            let base = inc(0, bloco, *f);
            for dt in 1..4u8 {
                tab[dt as usize][(bloco as usize) * 4 + k] = inc(dt, bloco, *f) - base;
            }
        }
    }
    for dt in 1..4 { println!("dt{dt}: {:?}", tab[dt]); }
}
