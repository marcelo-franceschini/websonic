// Gera as tabelas log-seno e exponencial a partir da FORMULA PUBLICADA (nao do Nuked), em f64.
// Formula (ROMs do OPL2 decapsuladas, usadas tambem na familia OPN; ver Nuked-OPN2/README e o
// manual do YM2608): logsin[i] = round(-log2(sin((i + 0.5) * pi / 512)) * 256), i = 0..255
//                     exp[i]    = round((2^((255 - i) / 256) - 1) * 1024)  (mantissa de 10 bits)
fn main() {
    let pi = std::f64::consts::PI;
    let ls: Vec<u32> = (0..256).map(|i| (-(((i as f64 + 0.5) * pi / 512.0).sin()).log2() * 256.0).round() as u32).collect();
    let ex: Vec<u32> = (0..256).map(|i| ((2f64.powf((255 - i) as f64 / 256.0) - 1.0) * 1024.0).round() as u32).collect();
    println!("pub const LOGSIN: [u16; 256] = {:?};", ls);
    println!("pub const EXP: [u16; 256] = {:?};", ex);
}
