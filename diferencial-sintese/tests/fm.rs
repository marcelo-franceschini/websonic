//! FM do jogo contra o Nuked-OPN2 (modo YM2612), com escritas sinteticas (sem ROM).
use sonic1_audio::fm::Fm;
use sonic1_diferencial_sintese::RefOpn2;

type Escrita = (u32, u8);

/// Aplica as escritas nos dois chips, com 48 ciclos de folga entre elas, e compara o pino de cada
/// ciclo ate `amostras` amostras depois. Devolve a primeira divergencia.
fn compara(escritas: &[Escrita], amostras: usize) -> Result<(), String> {
    let mut a = Fm::novo();
    let mut r = RefOpn2::novo();
    let mut ciclo = 0usize;
    let mut passo = |a: &mut Fm, r: &mut RefOpn2, ciclo: &mut usize| -> Result<(), String> {
        let x = a.ciclo();
        let y = r.ciclo();
        *ciclo += 1;
        if x != y { Err(format!("ciclo {} (amostra {}, pos {}): nucleo {x:?} referencia {y:?}", *ciclo, *ciclo / 24, *ciclo % 24)) } else { Ok(()) }
    };
    for &(porta, v) in escritas {
        a.escreve(porta, v);
        r.escreve(porta, v);
        for _ in 0..48 { passo(&mut a, &mut r, &mut ciclo)?; }
    }
    for _ in 0..amostras * 24 { passo(&mut a, &mut r, &mut ciclo)?; }
    Ok(())
}

fn canal0(algo: u8, fb: u8, tl: [u8; 4], fnum: u16, bloco: u8) -> Vec<Escrita> {
    let mut e = vec![(0, 0xB0), (1, (fb << 3) | algo), (0, 0xB4), (1, 0xC0)];
    for o in 0..4u8 {
        e.extend([(0, 0x30 + 4 * o), (1, 0x01), (0, 0x40 + 4 * o), (1, tl[o as usize]), (0, 0x50 + 4 * o), (1, 0x1F)]);
    }
    e.extend([(0, 0xA4), (1, (bloco << 3) | ((fnum >> 8) as u8)), (0, 0xA0), (1, fnum as u8), (0, 0x28), (1, 0xF0)]);
    e
}

#[test]
fn um_operador_algoritmo_7() {
    compara(&canal0(7, 0, [0, 127, 127, 127], 0x269, 4), 300).unwrap();
}

/// Diagnostico: imprime os dois fluxos de pino (so os ciclos que nao sao o valor de repouso).
#[test]
#[ignore]
fn diag_um_operador() {
    let e = canal0(7, 0, [0, 127, 127, 127], 0x269, 4);
    let mut a = Fm::novo();
    let mut r = RefOpn2::novo();
    let mut c = 0;
    for &(p, v) in &e { a.escreve(p, v); r.escreve(p, v); for _ in 0..48 { a.ciclo(); r.ciclo(); c += 1; } }
    for k in 0..12 * 24 {
        let (x, y) = (a.ciclo(), r.ciclo());
        c += 1;
        if x != (3, 3) || y != (3, 3) { println!("c={c} amostra {} pos {}: nucleo {x:?} ref {y:?}", c / 24, c % 24); }
        let _ = k;
    }
}

/// Diagnostico: quando cada estagio da referencia muda depois da escrita da tecla.
#[test]
#[ignore]
fn diag_latencia_tecla() {
    let e = canal0(7, 0, [0, 127, 127, 127], 0x269, 4);
    let mut r = RefOpn2::novo();
    let mut c = 0usize;
    let n = e.len();
    for (j, &(p, v)) in e.iter().enumerate() {
        r.escreve(p, v);
        if j == n - 1 { println!("escrita final no ciclo {c} (cycles={})", r.estado(8)[0]); }
        for _ in 0..48 { r.ciclo(); c += 1; if j == n - 1 { break; } }
    }
    let mut ant = (vec![], vec![], vec![], vec![]);
    for k in 0..8 * 24 {
        r.ciclo(); c += 1;
        let cur = (r.estado(0)[..6].to_vec(), r.estado(1)[..6].to_vec(), r.estado(3)[..6].to_vec(), r.estado(4));
        if cur != ant { println!("+{k} (cyc {}): phase{:?} eg{:?} fm_out{:?} ch_out{:?}", r.estado(8)[0], cur.0, cur.1, cur.2, cur.3); ant = cur; }
    }
}
