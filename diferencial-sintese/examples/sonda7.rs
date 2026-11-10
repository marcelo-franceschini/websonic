// Latencia da tecla por slot e por fase da escrita: ciclos da escrita do dado de $28 ate o nivel do
// envelope do slot ir a zero (ataque instantaneo). Imprime o minimo e a fase j do salto.
use sonic1_diferencial_sintese::RefOpn2;
fn main() {
    for op in 0..4usize {
        let mut linha = String::new();
        for canal in 0..6u8 {
            let mut v = Vec::new();
            for j in 0..24usize {
                let mut r = RefOpn2::novo();
                let (p, off) = if canal < 3 { (0u32, canal) } else { (2, canal - 3) };
                let mut w = |p: u32, a: u8, d: u8| { r.escreve(p, a); for _ in 0..8 { r.ciclo(); } r.escreve(p + 1, d); for _ in 0..60 { r.ciclo(); } };
                for o in 0..4u8 { w(p, 0x50 + o * 4 + off, 0x1F); w(p, 0x40 + o * 4 + off, 0); }
                while r.estado(8)[0] != 0 { r.ciclo(); }
                r.escreve(0, 0x28);
                for _ in 0..j + 24 { r.ciclo(); }
                let bit = [0x10u8, 0x40, 0x20, 0x80][op];
                let slot = op * 6 + canal as usize;
                r.escreve(1, bit | (if canal < 3 { canal } else { canal + 1 }));
                let mut hit = 0;
                for k in 1..200 { r.ciclo(); if r.estado(1)[slot] == 0 { hit = k; break; } }
                v.push(hit);
            }
            let min = *v.iter().min().unwrap();
            let salto = v.iter().position(|&x| x == min).unwrap();
            linha += &format!(" c{canal}:min{min}@j{salto}");
        }
        println!("grupo{op}:{linha}");
    }
}
