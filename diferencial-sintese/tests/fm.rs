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
    let passo = |a: &mut Fm, r: &mut RefOpn2, ciclo: &mut usize| -> Result<(), String> {
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
        r.ciclo();
        let cur = (r.estado(0)[..6].to_vec(), r.estado(1)[..6].to_vec(), r.estado(3)[..6].to_vec(), r.estado(4));
        if cur != ant { println!("+{k} (cyc {}): phase{:?} eg{:?} fm_out{:?} ch_out{:?}", r.estado(8)[0], cur.0, cur.1, cur.2, cur.3); ant = cur; }
    }
}

struct Rng(u64);
impl Rng {
    fn prox(&mut self) -> u64 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; self.0 }
    fn ate(&mut self, n: u64) -> u64 { self.prox() % n }
}

/// Configuracao estatica aleatoria: todos os canais, operadores audiveis, envelope instantaneo.
fn aleatorio(g: &mut Rng, canais: usize) -> Vec<Escrita> {
    let mut e = Vec::new();
    for ch in 0..canais {
        let (p, off) = if ch < 3 { (0u32, ch as u8) } else { (2, (ch - 3) as u8) };
        let w = |e: &mut Vec<Escrita>, a: u8, d: u8| { e.push((p, a)); e.push((p + 1, d)); };
        w(&mut e, 0xB0 + off, (g.ate(8) as u8) | ((g.ate(8) as u8) << 3));
        w(&mut e, 0xB4 + off, 0xC0);
        for o in 0..4u8 {
            w(&mut e, 0x30 + off + 4 * o, ((g.ate(8) as u8) << 4) | g.ate(16) as u8);
            w(&mut e, 0x40 + off + 4 * o, g.ate(40) as u8);
            w(&mut e, 0x50 + off + 4 * o, 0x1F);
        }
        w(&mut e, 0xA4 + off, ((g.ate(8) as u8) << 3) | g.ate(8) as u8);
        w(&mut e, 0xA0 + off, g.ate(256) as u8);
    }
    for ch in 0..canais { let k = if ch < 3 { ch } else { ch + 1 }; e.push((0, 0x28)); e.push((1, 0xF0 | k as u8)); }
    e
}

#[test]
fn estatico_aleatorio_um_canal() {
    for s in 1..=30u64 {
        let mut g = Rng(s.wrapping_mul(0x9E3779B97F4A7C15) | 1);
        let e = aleatorio(&mut g, 1);
        compara(&e, 400).unwrap_or_else(|m| panic!("semente {s}: {m}"));
    }
}

/// Compara em camadas no fim de cada amostra e diz a primeira camada que difere.
fn camadas(escritas: &[Escrita], amostras: usize) -> Result<(), String> {
    let mut a = Fm::novo();
    let mut r = RefOpn2::novo();
    let mut n = 0usize;
    let total = escritas.len() * 48 + amostras * 24;
    let mut it = escritas.iter();
    let mut prox = Some(it.next());
    let mut folga = 0;
    for _ in 0..total {
        if folga == 0 { if let Some(Some(&(p, v))) = prox { a.escreve(p, v); r.escreve(p, v); prox = Some(it.next()); folga = 48; } }
        if folga > 0 { folga -= 1; }
        let (x, y) = (a.ciclo(), r.ciclo());
        n += 1;
        if n % 24 == 0 {
            let ph_r = r.estado(0);
            let fo_r = r.estado(3);
            let co_r = r.estado(4);
            let ph_a = a.diag_fase();
            let fo_a = a.diag_saida_op();
            let co_a = a.diag_saida_canal();
            for i in 0..24 { if ph_r[i] as u32 != ph_a[i] && false { return Err(format!("amostra {}: fase slot {i}: ref {} nucleo {}", n/24, ph_r[i], ph_a[i])); } }
            for i in 0..24 { if fo_r[i] != fo_a[i] { return Err(format!("amostra {}: fm_out slot {i}: ref {} nucleo {}", n/24, fo_r[i], fo_a[i])); } }
            for i in 0..6 { if co_r[i] != co_a[i] { return Err(format!("amostra {}: ch_out {i}: ref {} nucleo {}", n/24, co_r[i], co_a[i])); } }
        }
        let _ = (x, y);
    }
    Ok(())
}

#[test]
#[ignore]
fn diag_camadas() {
    for s in 1..=30u64 {
        let mut g = Rng(s.wrapping_mul(0x9E3779B97F4A7C15) | 1);
        let e = aleatorio(&mut g, 1);
        if let Err(m) = camadas(&e, 400) { println!("semente {s}: {m}"); println!("{e:02X?}"); return; }
    }
}

#[test]
#[ignore]
fn diag_soma() {
    let mut g = Rng(3u64.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let e = aleatorio(&mut g, 1);
    let mut a = Fm::novo();
    let mut r = RefOpn2::novo();
    let mut n = 0;
    for &(p, v) in &e { a.escreve(p, v); r.escreve(p, v); for _ in 0..48 { a.ciclo(); r.ciclo(); n += 1; } }
    for _ in 0..400 * 24 {
        a.ciclo(); r.ciclo(); n += 1;
        if n % 24 == 7 && n / 24 >= 388 && n / 24 <= 392 {
            let fo = r.estado(3);
            println!("amostra {}: ref fm_out ch0 [{},{},{},{}] ch_out {} | mine [{:?}] ch_out {}", n / 24, fo[0], fo[6], fo[12], fo[18], r.estado(4)[0], a.diag_saida_op().iter().step_by(6).collect::<Vec<_>>(), a.diag_saida_canal()[0]);
        }
    }
}

/// Descobre quais amostras (atual ou anterior) de cada operador entram no canal.
#[test]
#[ignore]
fn diag_combinacoes() {
    for semente in [1u64, 3, 4, 5] {
        let mut g = Rng(semente.wrapping_mul(0x9E3779B97F4A7C15) | 1);
        let e = aleatorio(&mut g, 1);
        let mut r = RefOpn2::novo();
        let mut hist: Vec<[i32; 4]> = Vec::new();
        let mut chs: Vec<i32> = Vec::new();
        let mut n = 0;
        for &(p, v) in &e { r.escreve(p, v); for _ in 0..48 { r.ciclo(); n += 1; } }
        for _ in 0..400 * 24 {
            r.ciclo(); n += 1;
            if n % 24 == 7 { let fo = r.estado(3); hist.push([fo[0], fo[6], fo[12], fo[18]]); chs.push(r.estado(4)[0]); }
        }
        let algo = (e[1].1 & 7) as usize;
        let mut melhor: Vec<(usize, u32)> = Vec::new();
        for mask in 0..(1u32 << 9) {
            // bit k (0..3): usa atraso 1 para o operador k; bit 4+k: atraso 2
            let mut ok = 0;
            for t in 380..hist.len() {
                let pick = |k: usize| -> i32 {
                    let d = if mask >> (4 + k) & 1 == 1 { 2 } else if mask >> k & 1 == 1 { 1 } else { 0 };
                    hist[t - d][k]
                };
                let (s1, s3, s2, s4) = (pick(0), pick(1), pick(2), pick(3));
                let f = |x: i32| x >> 5;
                let (a, b) = match algo { 0..=3 => (f(s4), s4 >> 5), 4 => (f(s2) + f(s4), (s2 + s4) >> 5), 5 | 6 => (f(s2) + f(s3) + f(s4), (s2 + s3 + s4) >> 5), _ => (f(s1) + f(s2) + f(s3) + f(s4), (s1 + s2 + s3 + s4) >> 5) };
                let ok_a = a.clamp(-256, 255) == chs[t];
                let ok_b = b.clamp(-256, 255) == chs[t];
                if (mask >> 8) == 0 { if ok_a { ok += 1; } } else if ok_b { ok += 1; }
            }
            melhor.push((mask as usize, ok));
        }
        melhor.sort_by_key(|x| std::cmp::Reverse(x.1));
        println!("semente {semente} algo {algo}: {:?} de {}", &melhor[..4], hist.len() - 4);
    }
}

#[test]
#[ignore]
fn diag_sat() {
    let mut g = Rng(3u64.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let e = aleatorio(&mut g, 1);
    let mut r = RefOpn2::novo();
    let mut n = 0;
    let mut hist = Vec::new();
    for &(p, v) in &e { r.escreve(p, v); for _ in 0..48 { r.ciclo(); n += 1; } }
    for _ in 0..400 * 24 {
        r.ciclo(); n += 1;
        if n % 24 == 7 { let fo = r.estado(3); hist.push(([fo[0], fo[6], fo[12], fo[18]], r.estado(4)[0])); }
    }
    for t in 316..324 {
        let h = hist[t].0; let p = hist[t - 1].0;
        let f = |x: i32| x >> 5;
        println!("t{t}: ref {} | cur {:?} prev {:?} | A(s1 prev)={} | sum_raw={}", hist[t].1, h, p, f(p[0]) + f(h[1]) + f(h[2]) + f(h[3]), (p[0] + h[1] + h[2] + h[3]) >> 5);
    }
}

#[test]
#[ignore]
fn diag_slot() {
    let mut e = Vec::new();
    for s in 1..=20u64 {
        let mut g = Rng(s.wrapping_mul(0x9E3779B97F4A7C15) | 1);
        e = aleatorio(&mut g, 1);
    }
    let mut a = Fm::novo();
    let mut r = RefOpn2::novo();
    let mut n = 0;
    for &(p, v) in &e { a.escreve(p, v); r.escreve(p, v); for _ in 0..48 { a.ciclo(); r.ciclo(); n += 1; } }
    for _ in 0..40 * 24 {
        a.ciclo(); r.ciclo(); n += 1;
        if n % 24 == 7 && n / 24 >= 86 && n / 24 <= 95 {
            println!("amostra {}: fase ref {} nucleo {} | fm_out ref {} nucleo {} | s1 ref {} nucleo {}", n / 24, r.estado(0)[12], a.diag_fase()[12], r.estado(3)[12], a.diag_saida_op()[12], r.estado(3)[0], a.diag_saida_op()[0]);
        }
    }
}

#[test]
fn estatico_aleatorio_seis_canais() {
    for s in 1..=30u64 {
        let mut g = Rng(s.wrapping_mul(0x9E3779B97F4A7C15) | 1);
        let e = aleatorio(&mut g, 6);
        compara(&e, 300).unwrap_or_else(|m| panic!("semente {s}: {m}"));
    }
}

#[test]
#[ignore]
fn diag_camadas6() {
    for s in 1..=30u64 {
        let mut g = Rng(s.wrapping_mul(0x9E3779B97F4A7C15) | 1);
        let e = aleatorio(&mut g, 6);
        if let Err(m) = camadas(&e, 300) { println!("semente {s}: {m}"); return; }
    }
}

#[test]
#[ignore]
fn diag_slot7() {
    let mut g = Rng(1u64.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let e = aleatorio(&mut g, 6);
    let mut a = Fm::novo();
    let mut r = RefOpn2::novo();
    let mut n = 0;
    for &(p, v) in &e { a.escreve(p, v); r.escreve(p, v); for _ in 0..48 { a.ciclo(); r.ciclo(); n += 1;
        if n % 24 == 7 && n / 24 >= 396 && n / 24 <= 404 { println!("amostra {}: eg ref {:?} nucleo {:?} fase ref {} nucleo {} s3 ref {} nucleo {}", n / 24, &r.estado(1)[0..24], &a.diag_eg()[0..24], r.estado(0)[7], a.diag_fase()[7], r.estado(3)[7], a.diag_saida_op()[7]); } } }
    let regs: Vec<String> = e.chunks(2).filter(|c| matches!(c[0].1 & 3, 1)).map(|c| format!("{:02X}={:02X}", c[0].1, c[1].1)).collect();
    println!("regs canal1: {regs:?}");
    for _ in 0..400 * 24 {
        a.ciclo(); r.ciclo(); n += 1;
        if n % 24 == 7 && n / 24 >= 386 && n / 24 <= 394 {
            println!("amostra {}: fase ref {} nucleo {} | fm_out s3 ref {} nucleo {} | s1 ref {} nucleo {} | eg ref {} nucleo {}", n / 24, r.estado(0)[7], a.diag_fase()[7], r.estado(3)[7], a.diag_saida_op()[7], r.estado(3)[1], a.diag_saida_op()[1], r.estado(1)[7], a.diag_eg()[7]);
        }
    }
}

#[test]
#[ignore]
fn diag_pino3() {
    let mut g = Rng(1u64.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let e = aleatorio(&mut g, 6);
    let mut a = Fm::novo();
    let mut r = RefOpn2::novo();
    let mut n = 0;
    for &(p, v) in &e { a.escreve(p, v); r.escreve(p, v); for _ in 0..48 { 
        let (x, y) = (a.ciclo(), r.ciclo()); n += 1;
        if n / 24 >= 400 && n / 24 <= 403 && x != y { println!("n={} amostra {} pos {}: nucleo {x:?} ref {y:?} ch_out ref {:?} nucleo {:?}", n, n / 24, n % 24, r.estado(4), a.diag_saida_canal()); }
    } }
}

/// Canal 0, algoritmo 7, envelopes aleatorios, tecla ligada e depois desligada.
fn envelope_aleatorio(g: &mut Rng, espera: usize) -> Vec<Escrita> {
    let mut e = vec![(0, 0xB0), (1, 0x07), (0, 0xB4), (1, 0xC0)];
    for o in 0..4u8 {
        e.extend([(0, 0x30 + 4 * o), (1, 0x01)]);
        e.extend([(0, 0x40 + 4 * o), (1, g.ate(30) as u8)]);
        e.extend([(0, 0x50 + 4 * o), (1, ((g.ate(4) as u8) << 6) | (1 + g.ate(31) as u8))]);
        e.extend([(0, 0x60 + 4 * o), (1, g.ate(32) as u8)]);
        e.extend([(0, 0x70 + 4 * o), (1, g.ate(32) as u8)]);
        e.extend([(0, 0x80 + 4 * o), (1, ((g.ate(16) as u8) << 4) | g.ate(16) as u8)]);
    }
    e.extend([(0, 0xA4), (1, ((g.ate(8) as u8) << 3) | g.ate(8) as u8), (0, 0xA0), (1, g.ate(256) as u8)]);
    e.extend([(0, 0x28), (1, 0xF0)]);
    for _ in 0..espera { e.push((0, 0xFE)); }
    e.extend([(0, 0x28), (1, 0x00)]);
    for _ in 0..espera { e.push((0, 0xFE)); }
    e
}

#[test]
fn envelopes_aleatorios() {
    for s in 1..=20u64 {
        let mut g = Rng(s.wrapping_mul(0x9E3779B97F4A7C15) | 1);
        let e = envelope_aleatorio(&mut g, 150);
        compara(&e, 10).unwrap_or_else(|m| panic!("semente {s}: {m}"));
    }
}

#[test]
#[ignore]
fn diag_eg() {
    for s in 1..=20u64 {
        let mut g = Rng(s.wrapping_mul(0x9E3779B97F4A7C15) | 1);
        let e = envelope_aleatorio(&mut g, 150);
        let mut a = Fm::novo();
        let mut r = RefOpn2::novo();
        let mut n = 0usize;
        for &(p, v) in &e {
            a.escreve(p, v); r.escreve(p, v);
            for _ in 0..48 {
                a.ciclo(); r.ciclo(); n += 1;
                if n % 24 == 7 {
                    let (x, y) = (a.diag_eg(), r.estado(1));
                    for i in 0..24 { if i32::from(x[i]) != y[i] { println!("semente {s} amostra {}: eg slot {i} ref {} nucleo {}", n / 24, y[i], x[i]); println!("regs: {:02X?}", &e[..e.iter().position(|q| q.1 == 0xA4).unwrap() + 4]); return; } }
                }
            }
        }
    }
}

#[test]
#[ignore]
fn diag_eg_serie() {
    let mut g = Rng(2u64.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let e = envelope_aleatorio(&mut g, 150);
    let mut a = Fm::novo();
    let mut r = RefOpn2::novo();
    let mut n = 0usize;
    for &(p, v) in &e {
        a.escreve(p, v); r.escreve(p, v);
        for _ in 0..48 {
            a.ciclo(); r.ciclo(); n += 1;
            if n % 24 == 7 && n / 24 >= 108 && n / 24 <= 130 { println!("amostra {}: slot6 ref {} nucleo {}", n / 24, r.estado(1)[6], a.diag_eg()[6]); }
        }
    }
}

/// Decaimento 1 por taxa: um operador, ataque instantaneo, SL=15, compara o nivel do envelope.
fn rodar_taxa(d1r: u8, ks: u8, bloco: u8, fnum: u16, amostras: usize) -> Option<(usize, i32, i32)> { rodar_taxa_o(0, d1r, ks, bloco, fnum, amostras) }

fn rodar_taxa_o(o: u8, d1r: u8, ks: u8, bloco: u8, fnum: u16, amostras: usize) -> Option<(usize, i32, i32)> {
    let bit = [0x10u8, 0x40, 0x20, 0x80][o as usize];
    let od = 4 * o;
    let e = vec![(0, 0xB0), (1, 0x07), (0, 0x40 + od), (1, 0), (0, 0x50 + od), (1, (ks << 6) | 0x1F), (0, 0x60 + od), (1, d1r), (0, 0x80 + od), (1, 0xF0),
        (0, 0xA4), (1, (bloco << 3) | (fnum >> 8) as u8), (0, 0xA0), (1, fnum as u8), (0, 0x28), (1, bit)];
    let mut a = Fm::novo();
    let mut r = RefOpn2::novo();
    let mut n = 0usize;
    for &(p, v) in &e { a.escreve(p, v); r.escreve(p, v); for _ in 0..48 { a.ciclo(); r.ciclo(); n += 1; } }
    let slot = [0usize, 6, 12, 18][o as usize];
    for _ in 0..amostras * 24 {
        a.ciclo(); r.ciclo(); n += 1;
        if n % 24 == 7 {
            let (x, y) = (i32::from(a.diag_eg()[slot]), r.estado(1)[slot]);
            if x != y { return Some((n / 24, y, x)); }
        }
    }
    None
}

#[test]
#[ignore]
fn diag_taxas() {
    for d1r in 1..32u8 {
        println!("d1r {d1r}: {:?}", rodar_taxa(d1r, 0, 0, 0x020, 4500));
    }
}

#[test]
#[ignore]
fn diag_taxas_ks() {
    let mut falhas = Vec::new();
    for ks in 0..4u8 { for bloco in [0u8, 3, 5, 7] { for fnum in [0x020u16, 0x3A0, 0x420, 0x4A0, 0x7FF] { for d1r in [1u8, 2, 5, 9, 14, 20, 26, 31] {
        if let Some(m) = rodar_taxa(d1r, ks, bloco, fnum, 1500) { falhas.push((ks, bloco, fnum, d1r, m)); }
    } } } }
    println!("falhas: {} de {}", falhas.len(), 4 * 4 * 5 * 8);
    for f in falhas.iter().take(12) { println!("{f:?}"); }
}

#[test]
#[ignore]
fn diag_serie_taxa() {
    let (ks, bloco, fnum, d1r) = (0u8, 7u8, 0x020u16, 26u8);
    let e = vec![(0, 0xB0), (1, 0x07), (0, 0x40), (1, 0), (0, 0x50), (1, (ks << 6) | 0x1F), (0, 0x60), (1, d1r), (0, 0x80), (1, 0xF0),
        (0, 0xA4), (1, (bloco << 3) | (fnum >> 8) as u8), (0, 0xA0), (1, fnum as u8), (0, 0x28), (1, 0x10)];
    let mut a = Fm::novo();
    let mut r = RefOpn2::novo();
    let mut n = 0usize;
    let mut s = String::new();
    for &(p, v) in &e { a.escreve(p, v); r.escreve(p, v); for _ in 0..48 { a.ciclo(); r.ciclo(); n += 1; } }
    for _ in 0..80 * 24 {
        a.ciclo(); r.ciclo(); n += 1;
        if n % 24 == 7 { s += &format!("{}:{}/{} ", n / 24, r.estado(1)[0], a.diag_eg()[0]); }
    }
    println!("{s}");
}

#[test]
#[ignore]
fn diag_taxas_op() {
    for o in 0..4u8 { for d1r in [1u8, 2, 3, 6, 12, 26] { for bloco in [0u8, 4] {
        let r = rodar_taxa_o(o, d1r, 0, bloco, 0x020, 1500);
        println!("op{o} d1r{d1r} bloco{bloco}: {r:?}");
    } } }
}

/// Calibracao da regra de gravacao por classe de registrador.
struct Cenario { classe: usize, base: Vec<(u8, u8)>, alvo: u8, valor: u8 }

fn cenarios() -> Vec<Cenario> {
    // Escritas comuns (por canal; os enderecos sao somados ao canal depois).
    let comum = |tl0: u8, ar: u8, d1r: u8, d2r: u8, slrr: u8, fb_algo: u8| -> Vec<(u8, u8)> {
        let mut v = vec![(0xB0, fb_algo), (0xB4, 0xC0)];
        for o in 0..4u8 {
            let ativo = o == 0;
            v.push((0x30 + 4 * o, 0x01));
            v.push((0x40 + 4 * o, if ativo { tl0 } else { 0x7F }));
            v.push((0x50 + 4 * o, ar));
            v.push((0x60 + 4 * o, d1r));
            v.push((0x70 + 4 * o, d2r));
            v.push((0x80 + 4 * o, slrr));
        }
        v.push((0xA4, 0x22));
        v.push((0xA0, 0x69));
        v
    };
    vec![
        Cenario { classe: 0, base: comum(0, 0x1F, 0, 0, 0x0F, 7), alvo: 0x30, valor: 0x03 },
        Cenario { classe: 1, base: comum(0, 0x1F, 0, 0, 0x0F, 7), alvo: 0x40, valor: 0x28 },
        Cenario { classe: 2, base: comum(0, 0x0A, 0, 0, 0x0F, 7), alvo: 0x50, valor: 0x15 },
        Cenario { classe: 3, base: comum(0, 0x1F, 0x0A, 0, 0xF0, 7), alvo: 0x60, valor: 0x14 },
        Cenario { classe: 4, base: comum(0, 0x1F, 0, 0x0A, 0x00, 7), alvo: 0x70, valor: 0x14 },
        Cenario { classe: 5, base: comum(0, 0x1F, 0x0A, 0, 0xF0, 7), alvo: 0x80, valor: 0x20 },
        Cenario { classe: 6, base: comum(0, 0x1F, 0, 0, 0x0F, 7), alvo: 0xA0, valor: 0xC0 },
        Cenario { classe: 7, base: comum(0, 0x1F, 0, 0, 0x0F, 7), alvo: 0xA4, valor: 0x1A },
        Cenario { classe: 8, base: comum(0, 0x1F, 0, 0, 0x0F, 7), alvo: 0xB0, valor: 0x2F },
        Cenario { classe: 9, base: comum(0, 0x1F, 0, 0, 0x0F, 7), alvo: 0xB4, valor: 0x80 },
        Cenario { classe: 12, base: { let mut b = comum(0x7F, 0x1F, 0, 0, 0x0F, 7); b.extend([(0x2B, 0x80), (0x2A, 0x20)]); b }, alvo: 0x2A, valor: 0xE0 },
        Cenario { classe: 13, base: { let mut b = comum(0x7F, 0x1F, 0, 0, 0x0F, 7); b.extend([(0x2A, 0xE0), (0x2B, 0x00)]); b }, alvo: 0x2B, valor: 0x80 },
    ]
}

/// Roda uma cena em um chip qualquer (via fechamentos) e devolve o fluxo do pino.
fn fluxo_cena<F: FnMut() -> (i16, i16), W: FnMut(u32, u8)>(c: &Cenario, ch: u8, j: usize, mut ciclo: F, mut escreve: W) -> Vec<(i16, i16)> {
    let (parte, off) = if ch < 3 { (0u32, ch) } else { (2, ch - 3) };
    for &(a, d) in &c.base { escreve(parte, if a < 0x30 { a } else { a + off }); for _ in 0..4 { ciclo(); } escreve(parte + 1, d); for _ in 0..48 { ciclo(); } }
    escreve(0, 0x28); for _ in 0..4 { ciclo(); } escreve(1, 0xF0 | if ch < 3 { ch } else { ch + 1 }); 
    for _ in 0..24 * 60 { ciclo(); }
    // Alinha a 24 ciclos pelo contador de chamadas: a cena inteira conta chamadas em `ciclo`.
    escreve(parte, if c.alvo < 0x30 { c.alvo } else { c.alvo + off });
    for _ in 0..48 { ciclo(); }
    for _ in 0..j { ciclo(); }
    escreve(parte + 1, c.valor);
    (0..24 * 50).map(|_| ciclo()).collect()
}

#[test]
#[ignore]
fn diag_calibra_classes() {
    let cs = cenarios();
    let so: Vec<usize> = std::env::var("CLASSES").unwrap_or_default().split(',').filter_map(|x| x.parse().ok()).collect();
    for c in &cs {
        if !so.is_empty() && !so.contains(&c.classe) { continue; }
        let mut refs: Vec<(u8, usize, Vec<(i16, i16)>)> = Vec::new();
        for &ch in &[0u8, 2] { for j in 0..24usize {
            let mut r = RefOpn2::novo();
            let rp = std::cell::RefCell::new(&mut r);
            let f = fluxo_cena(c, ch, j, || rp.borrow_mut().ciclo(), |p, v| rp.borrow_mut().escreve(p, v));
            refs.push((ch, j, f));
        } }
        let mut boas = Vec::new();
        for dp in 0..1u32 { for fase in 0..25u64 { for atraso in 0..40u64 {
            let mut ok = true;
            for (ch, j, ref_f) in &refs {
                let mut a = Fm::novo();

                let _ = (dp, fase, atraso);
                let ap = std::cell::RefCell::new(&mut a);
                let f = fluxo_cena(c, *ch, *j, || ap.borrow_mut().ciclo(), |p, v| ap.borrow_mut().escreve(p, v));
                if &f != ref_f { ok = false; break; }
            }
            if ok { boas.push((dp, fase, atraso)); }
        } } }
        println!("classe {} ({:02X}): combinacoes boas {:?}", c.classe, c.alvo, &boas[..boas.len().min(8)]);
    }
}

#[test]
#[ignore]
fn diag_pan() {
    let cs = cenarios();
    let c = &cs[10];
    for j in [0usize, 6, 12, 18] {
        let mut r = RefOpn2::novo();
        let rp = std::cell::RefCell::new(&mut r);
        let fr = fluxo_cena(c, 0, j, || rp.borrow_mut().ciclo(), |p, v| rp.borrow_mut().escreve(p, v));
        let mut a = Fm::novo();
        let ap = std::cell::RefCell::new(&mut a);
        let fa = fluxo_cena(c, 0, j, || ap.borrow_mut().ciclo(), |p, v| ap.borrow_mut().escreve(p, v));
        let d = fr.iter().zip(fa.iter()).position(|(x, y)| x != y);
        println!("j{j}: primeira diferenca {:?} ref {:?} nucleo {:?}", d, d.map(|k| fr[k]), d.map(|k| fa[k]));
        if let Some(k) = d { println!("  ref {:?}\n  mine {:?}", &fr[k.saturating_sub(2)..k + 6], &fa[k.saturating_sub(2)..k + 6]); }
    }
}

/// Escritas aleatorias em instantes aleatorios, so nos registradores que o jogo usa, mais o DAC.
fn dinamico(g: &mut Rng, n: usize) -> Vec<(usize, u32, u8)> {
    // (ciclos de espera antes, porta, valor)
    let mut e: Vec<(usize, u32, u8)> = Vec::new();
    let w = |e: &mut Vec<(usize, u32, u8)>, gap: usize, parte: u32, a: u8, d: u8| { e.push((gap, parte, a)); e.push((4 + gap % 3, parte + 1, d)); };
    // configuracao inicial
    for ch in 0..6u8 {
        let (p, off) = if ch < 3 { (0u32, ch) } else { (2, ch - 3) };
        w(&mut e, 10, p, 0xB0 + off, (g.ate(8) as u8) | ((g.ate(4) as u8) << 3));
        w(&mut e, 10, p, 0xB4 + off, 0xC0);
        for o in 0..4u8 {
            w(&mut e, 10, p, 0x30 + off + 4 * o, ((g.ate(8) as u8) << 4) | g.ate(16) as u8);
            w(&mut e, 10, p, 0x40 + off + 4 * o, g.ate(60) as u8);
            w(&mut e, 10, p, 0x50 + off + 4 * o, ((g.ate(4) as u8) << 6) | (10 + g.ate(22) as u8));
            w(&mut e, 10, p, 0x60 + off + 4 * o, g.ate(20) as u8);
            w(&mut e, 10, p, 0x70 + off + 4 * o, g.ate(20) as u8);
            w(&mut e, 10, p, 0x80 + off + 4 * o, ((g.ate(16) as u8) << 4) | g.ate(16) as u8);
        }
        w(&mut e, 10, p, 0xA4 + off, ((g.ate(8) as u8) << 3) | g.ate(8) as u8);
        w(&mut e, 10, p, 0xA0 + off, g.ate(256) as u8);
    }
    for _ in 0..n {
        let gap = 1 + g.ate(300) as usize;
        let ch = g.ate(6) as u8;
        let (p, off) = if ch < 3 { (0u32, ch) } else { (2, ch - 3) };
        match g.ate(8) {
            0 => { let k = if ch < 3 { ch } else { ch + 1 }; w(&mut e, gap, 0, 0x28, ((g.ate(16) as u8) << 4) | k); }
            1 => w(&mut e, gap, p, 0x40 + off + 4 * g.ate(4) as u8, g.ate(70) as u8),
            2 => { w(&mut e, gap, p, 0xA4 + off, ((g.ate(8) as u8) << 3) | g.ate(8) as u8); w(&mut e, 20, p, 0xA0 + off, g.ate(256) as u8); }
            3 => w(&mut e, gap, p, 0xB4 + off, ((g.ate(2) as u8) << 7) | ((g.ate(2) as u8) << 6)),
            4 => w(&mut e, gap, p, 0x30 + off + 4 * g.ate(4) as u8, ((g.ate(8) as u8) << 4) | g.ate(16) as u8),
            5 => w(&mut e, gap, p, 0xB0 + off, (g.ate(8) as u8) | ((g.ate(4) as u8) << 3)),
            6 => w(&mut e, gap, 0, 0x2A, g.ate(256) as u8),
            _ => w(&mut e, gap, 0, 0x2B, (g.ate(2) as u8) << 7),
        }
    }
    e
}

fn compara_dinamico(e: &[(usize, u32, u8)], cauda: usize) -> Result<(), String> {
    let mut a = Fm::novo();
    let mut r = RefOpn2::novo();
    let mut c = 0usize;
    for &(gap, p, v) in e {
        for _ in 0..gap {
            let (x, y) = (a.ciclo(), r.ciclo());
            c += 1;
            if x != y { return Err(format!("ciclo {c} (amostra {}, pos {}): nucleo {x:?} ref {y:?}", c / 24, c % 24)); }
        }
        a.escreve(p, v); r.escreve(p, v);
    }
    for _ in 0..cauda {
        let (x, y) = (a.ciclo(), r.ciclo());
        c += 1;
        if x != y { return Err(format!("ciclo {c} (amostra {}, pos {}): nucleo {x:?} ref {y:?}", c / 24, c % 24)); }
    }
    Ok(())
}

#[test]
fn dinamico_aleatorio() {
    let mut falhas = Vec::new();
    for s in 1..=25u64 {
        let mut g = Rng(s.wrapping_mul(0x9E3779B97F4A7C15) | 1);
        let e = dinamico(&mut g, 150);
        if let Err(m) = compara_dinamico(&e, 24 * 100) { falhas.push(format!("semente {s}: {m}")); }
    }
    assert!(falhas.is_empty(), "{} falhas:\n{}", falhas.len(), falhas.join("\n"));
}


#[test]
#[ignore]
fn diag_dinamico() {
    for s in 1..=25u64 {
        let mut g = Rng(s.wrapping_mul(0x9E3779B97F4A7C15) | 1);
        let e = dinamico(&mut g, 150);
        let mut a = Fm::novo();
        let mut r = RefOpn2::novo();
        let mut c = 0usize;
        'fim: for (k, &(gap, p, v)) in e.iter().enumerate() {
            for _ in 0..gap {
                a.ciclo(); r.ciclo(); c += 1;
                if c % 24 == 7 {
                    let (fa, fr) = (a.diag_saida_op(), r.estado(3));
                    let (ca, cr) = (a.diag_saida_canal(), r.estado(4));
                    let (ea, er) = (a.diag_eg(), r.estado(1));
                    let ult = &e[k.saturating_sub(5)..k];
                    for i in 0..24 { if i32::from(ea[i]) != er[i] { println!("s{s} amostra {}: EG slot {i} ref {} nucleo {} | {:02X?}", c / 24, er[i], ea[i], ult); break 'fim; } }
                    for i in 0..24 { if fa[i] != fr[i] { println!("s{s} amostra {}: fm_out slot {i} ref {} nucleo {} | {:02X?}", c / 24, fr[i], fa[i], ult); break 'fim; } }
                    for i in 0..6 { if ca[i] != cr[i] && i != 5 { println!("s{s} amostra {}: ch_out {i} ref {} nucleo {} | {:02X?}", c / 24, cr[i], ca[i], ult); break 'fim; } }
                }
            }
            a.escreve(p, v); r.escreve(p, v);
        }
    }
}

#[test]
#[ignore]
fn diag_kon_serie() {
    let s = 2u64; let slot = 9usize; let canal_alvo = 3usize;
    let mut g = Rng(s.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let e = dinamico(&mut g, 150);
    let mut a = Fm::novo();
    let mut r = RefOpn2::novo();
    let mut c = 0usize;
    let mut ultimo_eg = (0i32, 0i32);
    for &(gap, p, v) in &e {
        for _ in 0..gap {
            a.ciclo(); r.ciclo(); c += 1;
            if c % 24 == 7 && c / 24 >= 90 && c / 24 <= 130 {
                let cur = (r.estado(1)[slot], i32::from(a.diag_eg()[slot]));
                if cur != ultimo_eg { println!("amostra {} (ciclo {c}): slot{slot} ref {} nucleo {}", c / 24, cur.0, cur.1); ultimo_eg = cur; }
            }
        }
        if p == 1 && false { }
        a.escreve(p, v); r.escreve(p, v);
        if p == 1 && e.iter().any(|_| true) { }
    }
    let _ = canal_alvo;
}

#[test]
#[ignore]
fn diag_kon_escritas() {
    let mut g = Rng(2u64.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let e = dinamico(&mut g, 150);
    let mut c = 0usize; let mut ult_addr = 0u8;
    for &(gap, p, v) in &e {
        c += gap;
        if p == 0 { ult_addr = v; }
        if p == 1 && ult_addr == 0x28 && c / 24 >= 60 && c / 24 <= 130 { println!("kon em ciclo {c} (amostra {}, fase {}): valor {v:02X}", c / 24, c % 24); }
    }
}

/// Serie de uma camada em torno de uma amostra, para uma semente do teste dinamico.
fn serie(s: u64, de: usize, ate: usize, f: &dyn Fn(&Fm, &RefOpn2) -> String) {
    let mut g = Rng(s.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let e = dinamico(&mut g, 150);
    let mut a = Fm::novo();
    let mut r = RefOpn2::novo();
    let mut c = 0usize;
    let mut ult = 0u8;
    for &(gap, p, v) in &e {
        for _ in 0..gap {
            a.ciclo(); r.ciclo(); c += 1;
            if c % 24 == 7 && c / 24 >= de && c / 24 <= ate { println!("amostra {}: {}", c / 24, f(&a, &r)); }
        }
        if c / 24 + 3 >= de && c / 24 <= ate { if p & 1 == 0 { ult = v; } else { println!("  escrita ciclo {c} (pos {}): reg {ult:02X} = {v:02X} porta {p}", c % 24); } }
        a.escreve(p, v); r.escreve(p, v);
    }
}

#[test]
#[ignore]
fn diag_serie_a() {
    serie(11, 558, 568, &|a, r| format!("ch_out0 ref {} nucleo {} | fm_out ref {:?} nucleo {:?}", r.estado(4)[0], a.diag_saida_canal()[0],
        [r.estado(3)[0], r.estado(3)[6], r.estado(3)[12], r.estado(3)[18]], [a.diag_saida_op()[0], a.diag_saida_op()[6], a.diag_saida_op()[12], a.diag_saida_op()[18]]));
}

#[test]
#[ignore]
fn diag_ciclo_a_ciclo() {
    let mut g = Rng(5u64.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let e = dinamico(&mut g, 150);
    let mut a = Fm::novo();
    let mut r = RefOpn2::novo();
    let mut c = 0usize;
    let mut prim_div = None;
    for &(gap, p, v) in &e {
        for _ in 0..gap {
            let x = a.ciclo();
            let y = r.ciclo();
            c += 1;
            let eg_r = r.estado(1);
            let eg_n = a.diag_eg();
            let mut igual_eg = true;
            for i in 0..24 {
                if eg_r[i] != eg_n[i] as i32 {
                    igual_eg = false;
                    break;
                }
            }
            if !igual_eg && prim_div.is_none() {
                prim_div = Some((c, x, y));
                println!("EG DIVERGIU no ciclo {c} (amostra {}, pos {}):", (c-1)/24, (c-1)%24);
                for i in 0..24 {
                    if eg_r[i] != eg_n[i] as i32 {
                        println!("  slot {i}: ref {} nuc {} (diff {})", eg_r[i], eg_n[i], eg_r[i] - (eg_n[i] as i32));
                    }
                }
            }
            let fm_r = r.estado(3);
            let fm_n = a.diag_saida_op();
            let mut igual_fm = true;
            for i in 0..24 {
                if fm_r[i] != fm_n[i] {
                    igual_fm = false;
                    break;
                }
            }
            if !igual_fm && prim_div.is_none() {
                prim_div = Some((c, x, y));
                println!("FM_OUT DIVERGIU no ciclo {c} (amostra {}, pos {}):", (c-1)/24, (c-1)%24);
                for i in 0..24 {
                    if fm_r[i] != fm_n[i] {
                        println!("  slot {i}: ref {} nuc {} (diff {})", fm_r[i], fm_n[i], fm_r[i] - fm_n[i]);
                    }
                }
            }
            let f_r = r.estado(0);
            let f_n = a.diag_fase();
            let mut igual_f = true;
            for i in 0..24 {
                if f_r[i] != f_n[i] as i32 {
                    igual_f = false;
                    break;
                }
            }
            if !igual_f && prim_div.is_none() {
                prim_div = Some((c, x, y));
                println!("FASE DIVERGIU no ciclo {c} (amostra {}, pos {}):", (c-1)/24, (c-1)%24);
                for i in 0..24 {
                    if f_r[i] != f_n[i] as i32 {
                        println!("  slot {i}: ref {} nuc {} (diff {})", f_r[i], f_n[i], f_r[i] - (f_n[i] as i32));
                    }
                }
            }
            if x != y && prim_div.is_none() {
                prim_div = Some((c, x, y));
                println!("PINO DIVERGIU no ciclo {c} (amostra {}, pos {}): nucleo {x:?} ref {y:?}", (c-1)/24, (c-1)%24);
            }
        }
        a.escreve(p, v); r.escreve(p, v);
    }
    assert!(prim_div.is_none(), "Divergiu: {:?}", prim_div);
}
