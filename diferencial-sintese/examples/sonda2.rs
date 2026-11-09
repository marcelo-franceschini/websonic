// Sonda 2: relacao entre ch_out e o pino (mol), incluindo negativos, e ciclo do pico de cada canal.
use sonic1_diferencial_sintese::RefOpn2;
fn w(r: &mut RefOpn2, parte: u32, a: u8, d: u8) {
    r.escreve(parte * 2, a);
    for _ in 0..4 { r.ciclo(); }
    r.escreve(parte * 2 + 1, d);
    for _ in 0..40 { r.ciclo(); }
}
fn main() {
    let mut r = RefOpn2::novo();
    // todos os 6 canais, algoritmo 7, op1 audivel, panning L e R diferentes
    for ch in 0..6u8 {
        let (parte, off) = if ch < 3 { (0, ch) } else { (1, ch - 3) };
        w(&mut r, parte, 0xB0 + off, 0x07);
        w(&mut r, parte, 0xB4 + off, if ch % 2 == 0 { 0xC0 } else { 0x80 });
        w(&mut r, parte, 0x30 + off, 0x01);
        w(&mut r, parte, 0x40 + off, 0x10 * ch);
        w(&mut r, parte, 0x50 + off, 0x1F);
        w(&mut r, parte, 0xA4 + off, 0x22);
        w(&mut r, parte, 0xA0 + off, 0x69);
    }
    for ch in 0..6u8 { let k = if ch < 3 { ch } else { ch + 1 }; w(&mut r, 0, 0x28, 0x10 | k); }
    while r.estado(8)[0] != 0 { r.ciclo(); }
    let mut pares = std::collections::BTreeMap::new();
    for n in 0..300 {
        let mut v = Vec::new();
        for _ in 0..24 { v.push(r.ciclo()); }
        let co = r.estado(4);
        if n < 8 || n == 100 {
            let picos: Vec<(usize, i16, i16)> = v.iter().enumerate().filter(|(_, x)| x.0 != 3 || x.1 != 3).map(|(i, x)| (i, x.0, x.1)).collect();
            println!("n{n} ch_out={co:?} picos(ciclo,l,r)={picos:?}");
        }
        for (i, x) in v.iter().enumerate() {
            let _ = i;
            let _ = x;
        }
        for c in 0..6 { let _ = c; }
        if n > 4 {
            // canal 0, lado esquerdo
            if let Some(x) = v.iter().find(|x| x.0 != 3) { pares.insert(co[0], x.0); }
        }
    }
    println!("pares ch_out->mol (canal 0): {:?}", pares.iter().filter(|(k, _)| **k >= -8 && **k <= 8 || **k % 64 == 0).collect::<Vec<_>>());
}
