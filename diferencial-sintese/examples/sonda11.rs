// Janela de captura: escreve o dado no instante j e troca o endereco d ciclos depois.
// Para cada j, imprime o menor d em que a escrita ainda vale (e se vale para todo d maior).
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
fn main() {
    // (nome, endereco, valor, camada, indice, endereco de troca)
    let regs: &[(&str, u8, u8, i32, usize, u8)] = &[
        ("A0 canal0", 0xA0, 0x70, 5, 0, 0x22), ("30 slot0", 0x30, 0x02, 5, 0, 0x22), ("38 slot12", 0x38, 0x02, 5, 12, 0x22),
        ("40 slot0", 0x40, 0x10, 2, 0, 0x22), ("30 troca p/ 34", 0x30, 0x02, 5, 0, 0x34), ("28 kon off", 0x28, 0x00, 6, 0, 0x22),
    ];
    for &(nome, a, d, camada, idx, troca) in regs {
        let mut res = Vec::new();
        for j in 0..24usize {
            let mut vale = Vec::new();
            for dd in 1..40usize {
                let mut r = prepara();
                while r.estado(8)[0] != 0 { r.ciclo(); }
                r.escreve(0, a);
                for _ in 0..48 + j { r.ciclo(); }
                let antes = r.estado(camada)[idx];
                r.escreve(1, d);
                for _ in 0..dd { r.ciclo(); }
                r.escreve(0, troca);
                for _ in 0..200 { r.ciclo(); }
                vale.push(r.estado(camada)[idx] != antes);
            }
            let primeiro = vale.iter().position(|&x| x).map(|x| x as i32 + 1).unwrap_or(-1);
            let estavel = primeiro > 0 && vale[(primeiro as usize - 1)..].iter().all(|&x| x);
            res.push(if estavel { primeiro } else { -primeiro - 100 });
        }
        println!("{nome}: {res:?}");
    }
}
