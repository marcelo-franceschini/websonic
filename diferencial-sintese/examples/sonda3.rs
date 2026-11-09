use sonic1_diferencial_sintese::RefOpn2;
fn w(r: &mut RefOpn2, parte: u32, a: u8, d: u8) {
    r.escreve(parte * 2, a);
    for _ in 0..4 { r.ciclo(); }
    r.escreve(parte * 2 + 1, d);
    for _ in 0..40 { r.ciclo(); }
}
fn main() {
    let mut r = RefOpn2::novo();
    for ch in 0..6u8 {
        let (parte, off) = if ch < 3 { (0, ch) } else { (1, ch - 3) };
        w(&mut r, parte, 0xB0 + off, 0x07);
        w(&mut r, parte, 0xB4 + off, 0xC0);
        for op in 0..4u8 { w(&mut r, parte, 0x30 + off + 4 * op, 0x01); }
        // so op1 audivel (TL 0), os outros mudos (TL 127)
        w(&mut r, parte, 0x40 + off, 0x08 * ch);
        for op in 1..4u8 { w(&mut r, parte, 0x40 + off + 4 * op, 0x7F); }
        for op in 0..4u8 { w(&mut r, parte, 0x50 + off + 4 * op, 0x1F); }
        w(&mut r, parte, 0xA4 + off, 0x22);
        w(&mut r, parte, 0xA0 + off, 0x69 + ch);
    }
    for ch in 0..6u8 { let k = if ch < 3 { ch } else { ch + 1 }; w(&mut r, 0, 0x28, 0xF0 | k); }
    for _ in 0..24 * 40 { r.ciclo(); }
    while r.estado(8)[0] != 0 { r.ciclo(); }
    for c in 0..24 {
        r.ciclo();
        let ph: Vec<String> = r.estado(0).iter().enumerate().filter(|(_, v)| **v != 0).map(|(i, v)| format!("{i}:{v}")).collect();
        let fo: Vec<String> = r.estado(3).iter().enumerate().filter(|(_, v)| **v != 0).map(|(i, v)| format!("{i}:{v}")).collect();
        println!("c{c:2} cyc={} ph[{}]\n      fm_out[{}] ch_out={:?}", r.estado(8)[0], ph.join(" "), fo.join(" "), r.estado(4));
    }
}
