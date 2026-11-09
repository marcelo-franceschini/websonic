// Sonda do Nuked-OPN2: saida por amostra para escritas sinteticas, com o estado interno de cada amostra.
use sonic1_diferencial_sintese::RefOpn2;

/// Escreve registrador (parte 0 ou 1) esperando o chip aceitar (32+ ciclos).
fn w(r: &mut RefOpn2, parte: u32, a: u8, d: u8) {
    r.escreve(parte * 2, a);
    for _ in 0..4 { r.ciclo(); }
    r.escreve(parte * 2 + 1, d);
    for _ in 0..40 { r.ciclo(); }
}
fn amostra(r: &mut RefOpn2) -> (i16, i16) {
    // 24 ciclos; a saida do chip muda uma vez por amostra, guardo o ultimo.
    let mut s = (0, 0);
    for _ in 0..24 { s = r.ciclo(); }
    s
}
fn main() {
    let mut r = RefOpn2::novo();
    w(&mut r, 0, 0xB0, 0x07);
    w(&mut r, 0, 0xB4, 0xC0);
    w(&mut r, 0, 0x30, 0x01);
    w(&mut r, 0, 0x40, 0x00);
    w(&mut r, 0, 0x50, 0x1F);
    w(&mut r, 0, 0xA4, 0x22);
    w(&mut r, 0, 0xA0, 0x69);
    w(&mut r, 0, 0x28, 0xF0);
    // alinha em ciclo 0
    while r.estado(8)[0] != 0 { r.ciclo(); }
    for n in 0..12 {
        let mut v = Vec::new();
        for _ in 0..24 { v.push(r.ciclo().0); }
        println!("n{n} mol por ciclo: {v:?} ch_out={:?}", r.estado(4));
        continue;
    }
    for n in 0..0 {
        let s = amostra(&mut r);
        let ph = r.estado(0); let eg = r.estado(1);
        println!("n{n:2} lr={s:?} ph[0..4]={:?} eg[0..4]={:?} ch_out={:?}", &ph[..4], &eg[..4], r.estado(4));
    }
}
