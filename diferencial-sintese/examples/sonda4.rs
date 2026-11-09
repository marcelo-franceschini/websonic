// Latencia de escrita: para cada deslocamento j (0..47) do instante da escrita do dado do registrador,
// em que ciclo absoluto o estado interno observado muda.
use sonic1_diferencial_sintese::RefOpn2;
fn prepara() -> RefOpn2 {
    let mut r = RefOpn2::novo();
    let mut w = |a: u8, d: u8| { r.escreve(0, a); for _ in 0..48 { r.ciclo(); } r.escreve(1, d); for _ in 0..48 { r.ciclo(); } };
    w(0xB0, 0x07); w(0xB4, 0xC0);
    for o in 0..4u8 { w(0x30 + 4 * o, 0x01); w(0x40 + 4 * o, 0x00); w(0x50 + 4 * o, 0x1F); }
    w(0xA4, 0x22); w(0xA0, 0x69); w(0x28, 0xF0);
    for _ in 0..24 * 10 { r.ciclo(); }
    r
}
// registrador a testar: (nome, endereco, valor novo, camada observada, indice)
fn main() {
    let regs: &[(&str, u8, u8, i32, usize)] = &[
        ("fnum_baixo A0", 0xA0, 0x70, 5, 0),   // pg_inc[0]
        ("bloco A4", 0xA4, 0x2A, 5, 0),
        ("mul 30", 0x30, 0x02, 5, 0),
        ("dt 30", 0x30, 0x31, 5, 0),
        ("tl 40", 0x40, 0x10, 2, 0),            // eg_out[0]
        ("ar 50", 0x50, 0x1E, 2, 0),
    ];
    for &(nome, a, d, camada, idx) in regs {
        let mut res = Vec::new();
        for j in 0..48usize {
            let mut r = prepara();
            // alinha: cycles == 0
            while r.estado(8)[0] != 0 { r.ciclo(); }
            r.escreve(0, a);
            for _ in 0..48 { r.ciclo(); }
            while r.estado(8)[0] != 0 { r.ciclo(); }
            for _ in 0..j { r.ciclo(); }
            let antes = r.estado(camada)[idx];
            r.escreve(1, d);
            let mut v = None;
            for k in 0..200 { r.ciclo(); if v.is_none() && r.estado(camada)[idx] != antes { v = Some(k + 1); } }
            res.push(v.map(|x| x as i32).unwrap_or(-1));
        }
        println!("{nome}: {res:?}");
    }
}
