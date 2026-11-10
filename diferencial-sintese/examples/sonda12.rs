// Troca de algoritmo (0 -> 7 e 7 -> 0) na fase j: quais operadores entram na soma do canal em cada
// uma das amostras seguintes. So usa o chip de referencia: compara ch_out com as somas possiveis.
use sonic1_diferencial_sintese::RefOpn2;
fn main() {
    for (de, para) in [(0u8, 7u8), (7, 0)] {
        println!("algoritmo {de} -> {para} (canal 0). Por fase j: mascara S1,S3,S2,S4 em cada latch seguinte");
        for j in 0..24usize {
            let mut r = RefOpn2::novo();
            let mut w = |r: &mut RefOpn2, a: u8, d: u8| { r.escreve(0, a); for _ in 0..48 { r.ciclo(); } r.escreve(1, d); for _ in 0..48 { r.ciclo(); } };
            w(&mut r, 0xB0, de); w(&mut r, 0xB4, 0xC0);
            for (o, (mul, tl)) in [(1u8, 0x1Au8), (3, 0x1E), (5, 0x22), (7, 0x26)].iter().enumerate() {
                let o = o as u8;
                w(&mut r, 0x30 + 4 * o, *mul); w(&mut r, 0x40 + 4 * o, *tl); w(&mut r, 0x50 + 4 * o, 0x1F);
            }
            w(&mut r, 0xA4, 0x1B); w(&mut r, 0xA0, 0x55); w(&mut r, 0x28, 0xF0);
            for _ in 0..24 * 30 { r.ciclo(); }
            while r.estado(8)[0] != 0 { r.ciclo(); }
            r.escreve(0, 0xB0);
            for _ in 0..48 + j { r.ciclo(); }
            r.escreve(1, para);
            // chamadas j.. ; latch do canal 0 acontece na chamada de indice 6 (cycles 6)
            let mut ant = [0i32; 4];
            let mut linha = String::new();
            let mut n = j;
            let mut latches = 0;
            while latches < 4 {
                // estado antes da chamada n: guarda fm_out do grupo 0 antes do slot ser reprocessado (chamada 5)
                if n % 24 == 5 { ant[0] = r.estado(3)[0]; }
                r.ciclo();
                if n % 24 == 6 {
                    let fo = r.estado(3);
                    let v = [ant[0] >> 5, fo[6] >> 5, fo[12] >> 5, fo[18] >> 5];
                    let alvo = r.estado(4)[0];
                    let ms: Vec<u32> = (0..16u32).filter(|m| (0..4).filter(|k| m >> k & 1 == 1).map(|k| v[k]).sum::<i32>() == alvo).collect();
                    linha += &format!(" {:?}", ms.iter().map(|m| format!("{:04b}", m.reverse_bits() >> 28)).collect::<Vec<_>>());
                    latches += 1;
                }
                n += 1;
            }
            println!("j{j:2}:{linha}");
        }
    }
}
