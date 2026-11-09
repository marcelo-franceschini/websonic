use sonic1_diferencial_sintese::RefOpn2;
fn main() {
    let mut r = RefOpn2::novo();
    let mut ant = 0;
    for c in 0..24 * 40 {
        r.ciclo();
        let m = r.estado(8);
        if m[4] != ant { println!("chamada {c} (cyc depois={}, amostra {}): eg_timer={}", m[0], c / 24, m[4]); ant = m[4]; }
    }
    // eg_cycle ao longo de um sample
    let v: Vec<i32> = (0..24).map(|_| { r.ciclo(); r.estado(8)[6] }).collect();
    println!("eg_cycle: {v:?}");
}
