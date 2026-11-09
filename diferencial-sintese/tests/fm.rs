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
    let mut g = Rng(0);
    let mut e = Vec::new();
    for s in 1..=20u64 { g = Rng(s.wrapping_mul(0x9E3779B97F4A7C15) | 1); e = aleatorio(&mut g, 1); }
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
