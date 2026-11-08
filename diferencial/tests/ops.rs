//! Operações do 68000 do núcleo contra o oráculo (sub-etapa 5.2), com ROM sintética.
//!
//! Cada caso roda UMA instrução (codificada por nós a partir do manual) no oráculo, com os
//! operandos em D1 (fonte) e D0 (destino) e o CCR de entrada no SR, e compara o D0 inteiro e os
//! 5 bits do CCR com a função do núcleo. Não usa a ROM do jogo.
//!
//! Tamanho da amostra: `WEBSONIC_OPS_CASOS` (padrão 6000 por operação e tamanho; byte usa os
//! 65536 pares quando o valor passa de 65536).

use sonic1_core::memoria::Tamanho;
use sonic1_core::ops68k::{self, Ccr, Condicao, Deslocamento, OpBit};
use sonic1_diferencial::{Bancada, Regs};

fn casos_por_operacao() -> usize {
    std::env::var("WEBSONIC_OPS_CASOS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(6000)
}

/// Gerador xorshift64 determinístico.
struct Sorteio(u64);
impl Sorteio {
    fn proximo(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn u32(&mut self) -> u32 {
        (self.proximo() >> 16) as u32
    }
}

fn mascara(t: Tamanho) -> u32 {
    match t {
        Tamanho::Byte => 0xFF,
        Tamanho::Word => 0xFFFF,
        Tamanho::Long => 0xFFFF_FFFF,
    }
}

fn bordas(t: Tamanho) -> Vec<u32> {
    let m = mascara(t);
    let alto = (m >> 1) + 1;
    let mut v = vec![
        0,
        1,
        2,
        3,
        7,
        8,
        9,
        0x0F,
        0x10,
        0x7F,
        0x80,
        0x81,
        0xFE,
        0xFF,
        m,
        m - 1,
        alto,
        alto - 1,
        alto + 1,
        m >> 4,
        0x99,
        0x09,
        0x90,
    ];
    v.iter_mut().for_each(|x| *x &= m);
    v.sort_unstable();
    v.dedup();
    v
}

/// Pares (fonte, destino): todos os 65536 de byte se pedido, senão bordas x bordas e sorteio.
fn pares(t: Tamanho, n: usize, semente: u64) -> Vec<(u32, u32)> {
    if t == Tamanho::Byte && n >= 65536 {
        return (0..=0xFFFFu32).map(|p| (p >> 8, p & 0xFF)).collect();
    }
    let b = bordas(t);
    let mut v: Vec<(u32, u32)> = b
        .iter()
        .flat_map(|s| b.iter().map(move |d| (*s, *d)))
        .collect();
    let mut r = Sorteio(semente);
    let m = mascara(t);
    while v.len() < n {
        v.push((r.u32() & m, r.u32() & m));
    }
    v.truncate(n.max(b.len() * b.len()));
    v
}

type Nucleo = Box<dyn Fn(u32, u32, Ccr) -> Option<(u32, Ccr)> + Send + Sync>;

struct Caso {
    nome: String,
    trecho: Vec<u16>,
    t: Tamanho,
    nucleo: Nucleo,
}

/// Roda todos os pares num caso e devolve as divergências (no máximo 8 por caso).
fn rodar_caso(c: &Caso, pares: &[(u32, u32)], semente: u64) -> Vec<String> {
    let mut banco = Bancada::nova(&c.trecho);
    let mut sorteio = Sorteio(semente);
    let m = mascara(c.t);
    let mut ruins = Vec::new();
    for (i, (s, d)) in pares.iter().enumerate() {
        let ccr_bits = u16::try_from(i % 32).unwrap();
        let ccr = Ccr::do_sr(ccr_bits);
        let alto_d = sorteio.u32() & !m;
        let alto_s = sorteio.u32() & !m;
        let mut e = Regs::default();
        e.d[0] = alto_d | d;
        e.d[1] = alto_s | s;
        e.sr = ccr_bits;
        let o = banco.tentar(&e, &[]);
        let n = (c.nucleo)(*s, *d, ccr);
        match (o, n) {
            (Err(_), None) => {}
            (Ok(o), Some((valor, ccr_n))) => {
                let esperado_d0 = alto_d | (valor & m);
                let d0_ok =
                    o.regs.d[0] == esperado_d0 || (c.t == Tamanho::Long && o.regs.d[0] == valor);
                let ccr_o = o.regs.sr & 0x1F;
                if !d0_ok || ccr_o != ccr_n.para_bits() {
                    ruins.push(format!(
                        "{} s={s:X} d={d:X} ccr_in={ccr_bits:02X}: oraculo D0={:X} ccr={ccr_o:02X}, nucleo D0={esperado_d0:X} ccr={:02X}",
                        c.nome,
                        o.regs.d[0],
                        ccr_n.para_bits()
                    ));
                }
            }
            (o, n) => ruins.push(format!(
                "{} s={s:X} d={d:X}: oraculo {:?}, nucleo {:?}",
                c.nome,
                o.map(|r| r.regs.d[0]),
                n
            )),
        }
        if ruins.len() >= 8 {
            break;
        }
    }
    ruins
}

fn tam3() -> [Tamanho; 3] {
    [Tamanho::Byte, Tamanho::Word, Tamanho::Long]
}

fn idx(t: Tamanho) -> u16 {
    match t {
        Tamanho::Byte => 0,
        Tamanho::Word => 1,
        Tamanho::Long => 2,
    }
}

fn com_r(f: impl Fn(u32, u32, Ccr) -> ops68k::Resultado + Send + Sync + 'static) -> Nucleo {
    Box::new(move |s, d, c| {
        let r = f(s, d, c);
        Some((r.valor, r.ccr))
    })
}

fn casos_de_tamanho() -> Vec<Caso> {
    let mut v = Vec::new();
    for t in tam3() {
        let i = idx(t);
        let dobra = i << 6; // bits 7-6 do opcode
        let nome = |n: &str| format!("{n}.{t:?}");
        v.push(Caso {
            nome: nome("ADD"),
            trecho: vec![0xD001 | dobra],
            t,
            nucleo: com_r(move |s, d, c| ops68k::add(t, s, d, c)),
        });
        v.push(Caso {
            nome: nome("SUB"),
            trecho: vec![0x9001 | dobra],
            t,
            nucleo: com_r(move |s, d, c| ops68k::sub(t, s, d, c)),
        });
        v.push(Caso {
            nome: nome("ADDX"),
            trecho: vec![0xD101 | dobra],
            t,
            nucleo: com_r(move |s, d, c| ops68k::addx(t, s, d, c)),
        });
        v.push(Caso {
            nome: nome("SUBX"),
            trecho: vec![0x9101 | dobra],
            t,
            nucleo: com_r(move |s, d, c| ops68k::subx(t, s, d, c)),
        });
        v.push(Caso {
            nome: nome("AND"),
            trecho: vec![0xC001 | dobra],
            t,
            nucleo: com_r(move |s, d, c| ops68k::and(t, s, d, c)),
        });
        v.push(Caso {
            nome: nome("OR"),
            trecho: vec![0x8001 | dobra],
            t,
            nucleo: com_r(move |s, d, c| ops68k::or(t, s, d, c)),
        });
        v.push(Caso {
            nome: nome("EOR"),
            trecho: vec![0xB300 | dobra],
            t,
            nucleo: com_r(move |s, d, c| ops68k::eor(t, s, d, c)),
        });
        v.push(Caso {
            nome: nome("NOT"),
            trecho: vec![0x4600 | dobra],
            t,
            nucleo: com_r(move |_, d, c| ops68k::not(t, d, c)),
        });
        v.push(Caso {
            nome: nome("NEG"),
            trecho: vec![0x4400 | dobra],
            t,
            nucleo: com_r(move |_, d, c| ops68k::neg(t, d, c)),
        });
        v.push(Caso {
            nome: nome("NEGX"),
            trecho: vec![0x4000 | dobra],
            t,
            nucleo: com_r(move |_, d, c| ops68k::negx(t, d, c)),
        });
        v.push(Caso {
            nome: nome("CLR"),
            trecho: vec![0x4200 | dobra],
            t,
            nucleo: com_r(move |_, _, c| ops68k::clr(c)),
        });
        v.push(Caso {
            nome: nome("TST"),
            trecho: vec![0x4A00 | dobra],
            t,
            nucleo: Box::new(move |_, d, c| Some((d, ops68k::tst(t, d, c)))),
        });
        v.push(Caso {
            nome: nome("CMP"),
            trecho: vec![0xB001 | dobra],
            t,
            nucleo: Box::new(move |s, d, c| Some((d, ops68k::cmp(t, s, d, c)))),
        });
        v.push(Caso {
            nome: nome("MOVE"),
            trecho: vec![[0x1001, 0x3001, 0x2001][usize::from(i)]],
            t,
            nucleo: com_r(move |s, _, c| ops68k::mover(t, s, c)),
        });
    }
    v
}

/// Roda `f` em cada item, em varias threads (cada uma com a sua Bancada, porque o estado de
/// observacao do oraculo e por thread). Junta as divergencias na ordem dos itens.
fn paralelo<T: Sync>(itens: &[T], f: impl Fn(&T) -> Vec<String> + Sync) -> Vec<String> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let proximo = AtomicUsize::new(0);
    let fios = std::thread::available_parallelism()
        .map_or(2, |n| n.get())
        .min(itens.len().max(1));
    let mut por_item: Vec<(usize, Vec<String>)> = std::thread::scope(|sc| {
        let hs: Vec<_> = (0..fios)
            .map(|_| {
                sc.spawn(|| {
                    let mut feitos = Vec::new();
                    loop {
                        let i = proximo.fetch_add(1, Ordering::SeqCst);
                        let Some(item) = itens.get(i) else { break };
                        feitos.push((i, f(item)));
                    }
                    feitos
                })
            })
            .collect();
        hs.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    por_item.sort_by_key(|(i, _)| *i);
    por_item.into_iter().flat_map(|(_, v)| v).collect()
}

fn verificar(ruins: Vec<String>) {
    assert!(
        ruins.is_empty(),
        "{} divergencias:\n{}",
        ruins.len(),
        ruins[..ruins.len().min(30)].join("\n")
    );
}

#[test]
fn aritmetica_e_logica_iguais_ao_oraculo() {
    let n = casos_por_operacao();
    let casos: Vec<(usize, Caso)> = casos_de_tamanho().into_iter().enumerate().collect();
    verificar(paralelo(&casos, |(k, c)| {
        let p = pares(c.t, n, 0x9E37_79B9 + *k as u64);
        rodar_caso(c, &p, 77 + *k as u64)
    }));
}

#[test]
fn extensao_troca_e_multiplicacao_iguais_ao_oraculo() {
    let n = casos_por_operacao();
    let casos: Vec<Caso> = vec![
        Caso {
            nome: "EXT.W".into(),
            trecho: vec![0x4880],
            t: Tamanho::Word,
            nucleo: com_r(|_, d, c| ops68k::ext(Tamanho::Word, d, c)),
        },
        Caso {
            nome: "EXT.L".into(),
            trecho: vec![0x48C0],
            t: Tamanho::Long,
            nucleo: com_r(|_, d, c| ops68k::ext(Tamanho::Long, d, c)),
        },
        Caso {
            nome: "SWAP".into(),
            trecho: vec![0x4840],
            t: Tamanho::Long,
            nucleo: com_r(|_, d, c| ops68k::swap(d, c)),
        },
        Caso {
            nome: "MULU".into(),
            trecho: vec![0xC0C1],
            t: Tamanho::Long,
            nucleo: com_r(ops68k::mulu),
        },
        Caso {
            nome: "MULS".into(),
            trecho: vec![0xC1C1],
            t: Tamanho::Long,
            nucleo: com_r(ops68k::muls),
        },
    ];
    let itens: Vec<(usize, Caso)> = casos.into_iter().enumerate().collect();
    verificar(paralelo(&itens, |(k, c)| {
        let tam = if c.nome.starts_with("MUL") {
            Tamanho::Word
        } else {
            c.t
        };
        let mut p = pares(tam, n, 5 + *k as u64);
        if c.nome.starts_with("EXT") || c.nome == "SWAP" {
            // O destino e o D0 inteiro: sorteia valores de 32 bits (16 para EXT.W).
            let mut r = Sorteio(99 + *k as u64);
            let m = if c.nome == "EXT.W" {
                Tamanho::Word
            } else {
                Tamanho::Long
            };
            for par in &mut p {
                par.1 = r.u32() & mascara(m);
            }
        }
        rodar_caso(c, &p, 11 + *k as u64)
    }));
}

#[test]
fn divisao_com_estouro_e_por_zero_iguais_ao_oraculo() {
    let n = casos_por_operacao();
    let ops = [("DIVU", 0x80C1u16, false), ("DIVS", 0x81C1u16, true)];
    verificar(paralelo(&ops, |(nome, opcode, sinal)| {
        let mut r = Sorteio(4242);
        let mut ruins = Vec::new();
        let mut banco = Bancada::nova(&[*opcode]);
        let mut pares: Vec<(u32, u32)> = Vec::new();
        let divisores = [0u32, 1, 2, 3, 0x7FFF, 0x8000, 0xFFFF, 0xFFFE, 10, 100];
        let dividendos = [
            0u32,
            1,
            0xFFFF,
            0x1_0000,
            0x7FFF_FFFF,
            0x8000_0000,
            0xFFFF_FFFF,
            0xFFFF_0000,
            0x0001_0001,
            0x8000_0001,
        ];
        for d in divisores {
            for dd in dividendos {
                pares.push((d, dd));
            }
        }
        while pares.len() < n {
            let divisor = r.u32() & 0xFFFF;
            // Dividendos que cabem e que estouram, em proporcoes parecidas.
            let dividendo = if r.u32() & 1 == 0 {
                r.u32()
            } else {
                r.u32() >> (r.u32() & 15)
            };
            pares.push((divisor, dividendo));
        }
        for (i, (divisor, dividendo)) in pares.iter().enumerate() {
            let bits = u16::try_from(i % 32).unwrap();
            let mut e = Regs::default();
            e.d[0] = *dividendo;
            e.d[1] = 0xABCD_0000 | divisor;
            e.sr = bits;
            // O jgenesis em build de depuracao entra em panico em i32::MIN / -1 (overflow da
            // divisao). O quociente nao cabe em word de qualquer jeito: caso fora da comparacao.
            if *sinal && *dividendo == 0x8000_0000 && *divisor == 0xFFFF {
                continue;
            }
            let o = banco.tentar(&e, &[]);
            let f = if *sinal { ops68k::divs } else { ops68k::divu };
            let n = f(*divisor, *dividendo, Ccr::do_sr(bits));
            match (o, n) {
                (Err(_), Err(_)) => {}
                (Ok(o), Ok(n)) => {
                    if o.regs.d[0] != n.valor || o.regs.sr & 0x1F != n.ccr.para_bits() {
                        ruins.push(format!(
                            "{nome} {dividendo:X}/{divisor:X} ccr={bits:02X}: oraculo D0={:X} ccr={:02X}, nucleo D0={:X} ccr={:02X}",
                            o.regs.d[0], o.regs.sr & 0x1F, n.valor, n.ccr.para_bits()
                        ));
                    }
                }
                (o, n) => ruins.push(format!(
                    "{nome} {dividendo:X}/{divisor:X}: oraculo {:?} nucleo {n:?}",
                    o.map(|x| x.regs.d[0])
                )),
            }
            if ruins.len() > 8 {
                break;
            }
        }
        ruins
    }));
}

#[test]
fn bcd_igual_ao_oraculo_nos_65536_pares() {
    // Itens: (operacao, faixa do byte de origem). Cada faixa e um item para dividir o trabalho.
    let mut itens = Vec::new();
    for op in 0..3u8 {
        for faixa in 0..16u32 {
            itens.push((op, faixa));
        }
    }
    verificar(paralelo(&itens, |(op, faixa)| {
        let mut ruins = Vec::new();
        let trecho = match op {
            0 => 0xC101u16, // ABCD D1,D0
            1 => 0x8101,    // SBCD D1,D0
            _ => 0x4800,    // NBCD D0
        };
        let mut banco = Bancada::nova(&[trecho]);
        for s in (faixa * 16)..(faixa * 16 + 16) {
            for d in 0..=0xFFu32 {
                let bits = u16::try_from((s + d) % 32).unwrap();
                let mut e = Regs::default();
                e.d[0] = 0x1234_5600 | d;
                e.d[1] = s;
                e.sr = bits;
                let o = banco.rodar(&e, &[]);
                let ccr = Ccr::do_sr(bits);
                let n = match op {
                    0 => ops68k::abcd(s as u8, d as u8, ccr),
                    1 => ops68k::sbcd(s as u8, d as u8, ccr),
                    _ => ops68k::nbcd(d as u8, ccr),
                };
                if o.regs.d[0] != (0x1234_5600 | n.valor) || o.regs.sr & 0x1F != n.ccr.para_bits() {
                    ruins.push(format!(
                        "BCD{op} s={s:X} d={d:X} ccr={bits:02X}: oraculo {:X}/{:02X} nucleo {:X}/{:02X}",
                        o.regs.d[0], o.regs.sr & 0x1F, n.valor, n.ccr.para_bits()
                    ));
                    if ruins.len() > 8 {
                        return ruins;
                    }
                }
                // NBCD nao usa a fonte: um unico s basta.
                if *op == 2 && s != faixa * 16 {
                    break;
                }
            }
        }
        ruins
    }));
}

#[test]
fn deslocamentos_e_rotacoes_com_contagem_0_a_70() {
    use Deslocamento::*;
    let tipos = [
        (AsrDireita, 0xE220u16, "ASR"),
        (AslEsquerda, 0xE320, "ASL"),
        (LsrDireita, 0xE228, "LSR"),
        (LslEsquerda, 0xE328, "LSL"),
        (RoxrDireita, 0xE230, "ROXR"),
        (RoxlEsquerda, 0xE330, "ROXL"),
        (RorDireita, 0xE238, "ROR"),
        (RolEsquerda, 0xE338, "ROL"),
    ];
    let mut itens = Vec::new();
    for t in tipos {
        for tam in tam3() {
            itens.push((t, tam));
        }
    }
    verificar(paralelo(&itens, |((tipo, base, nome), t)| {
        let mut ruins = Vec::new();
        let mut banco = Bancada::nova(&[*base | (idx(*t) << 6)]);
        let m = mascara(*t);
        let mut valores = bordas(*t);
        let mut r = Sorteio(31337);
        for _ in 0..40 {
            valores.push(r.u32() & m);
        }
        for contagem in 0..=70u32 {
            for (k, v) in valores.iter().enumerate() {
                let bits = u16::try_from((k + contagem as usize) % 32).unwrap();
                let mut e = Regs::default();
                e.d[0] = (0xA5A5_A5A5 & !m) | v;
                // A contagem fica nos 6 bits baixos de D1; o resto do registrador e lixo.
                e.d[1] = contagem | (r.u32() & !0x3F);
                e.sr = bits;
                let o = banco.rodar(&e, &[]);
                // A contagem em registrador vale modulo 64.
                let n = ops68k::deslocar(*tipo, *t, contagem & 63, *v, Ccr::do_sr(bits));
                let esperado = (0xA5A5_A5A5 & !m) | (n.valor & m);
                if o.regs.d[0] != esperado || o.regs.sr & 0x1F != n.ccr.para_bits() {
                    ruins.push(format!(
                        "{nome}.{t:?} v={v:X} cont={contagem} ccr={bits:02X}: oraculo {:X}/{:02X} nucleo {esperado:X}/{:02X}",
                        o.regs.d[0], o.regs.sr & 0x1F, n.ccr.para_bits()
                    ));
                    if ruins.len() > 8 {
                        return ruins;
                    }
                }
            }
        }
        ruins
    }));
}

#[test]
fn operacoes_de_bit_condicoes_e_registradores_de_endereco() {
    let mut ruins = Vec::new();
    // BTST/BCHG/BCLR/BSET Dn,Dm: o numero do bit vem de D1 (modulo 32).
    for (op, opcode, nome) in [
        (OpBit::Testar, 0x0300u16, "BTST"),
        (OpBit::Inverter, 0x0340, "BCHG"),
        (OpBit::Desligar, 0x0380, "BCLR"),
        (OpBit::Ligar, 0x03C0, "BSET"),
    ] {
        let mut banco = Bancada::nova(&[opcode]);
        let mut r = Sorteio(8);
        for k in 0..2000u32 {
            let bits = u16::try_from(k % 32).unwrap();
            let mut e = Regs::default();
            e.d[0] = r.u32();
            e.d[1] = if k < 70 { k } else { r.u32() };
            e.sr = bits;
            let o = banco.rodar(&e, &[]);
            let n = ops68k::operar_bit(op, Tamanho::Long, e.d[1], e.d[0], Ccr::do_sr(bits));
            if o.regs.d[0] != n.valor || o.regs.sr & 0x1F != n.ccr.para_bits() {
                ruins.push(format!("{nome} d0={:X} bit={:X}", e.d[0], e.d[1]));
            }
        }
    }
    // Scc: as 16 condicoes com os 32 CCRs.
    for cc in 0..16u16 {
        let mut banco = Bancada::nova(&[0x50C0 | (cc << 8)]);
        for bits in 0..32u16 {
            let mut e = Regs::default();
            e.d[0] = 0x1234_5600;
            e.sr = bits;
            let o = banco.rodar(&e, &[]);
            let esperado = 0x1234_5600
                | u32::from(ops68k::scc(Condicao::do_codigo(cc as u8), Ccr::do_sr(bits)));
            if o.regs.d[0] != esperado || o.regs.sr & 0x1F != bits {
                ruins.push(format!("Scc cc={cc} ccr={bits:02X}"));
            }
        }
    }
    // DBcc D0,+6 ; LEA 1(A1),A1 ; RTS. LEA nao mexe nas flags; A1 so muda se o desvio NAO foi tomado.
    for cc in 0..16u16 {
        let mut banco = Bancada::nova(&[0x50C8 | (cc << 8), 0x0006, 0x43E9, 0x0001]);
        for bits in 0..32u16 {
            for contador in [0u32, 1, 2, 0xFFFF, 0x1_0000, 0xFFFF_FFFF] {
                let mut e = Regs::default();
                e.d[0] = contador;
                e.sr = bits;
                let o = banco.rodar(&e, &[]);
                let (novo, tomado) = ops68k::dbcc(
                    Condicao::do_codigo(cc as u8),
                    Ccr::do_sr(bits),
                    contador as u16,
                );
                let esperado_d0 = (contador & 0xFFFF_0000) | u32::from(novo);
                if o.regs.d[0] != esperado_d0
                    || (o.regs.a[1] == 0) != tomado
                    || o.regs.sr & 0x1F != bits
                {
                    ruins.push(format!("DBcc cc={cc} ccr={bits:02X} cont={contador:X}"));
                }
            }
        }
    }
    // ADDA, SUBA e CMPA: A0 e o destino, D1 a fonte.
    for (nome, opcode, t) in [
        ("ADDA.W", 0xD0C1u16, Tamanho::Word),
        ("ADDA.L", 0xD1C1, Tamanho::Long),
        ("SUBA.W", 0x90C1, Tamanho::Word),
        ("SUBA.L", 0x91C1, Tamanho::Long),
        ("CMPA.W", 0xB0C1, Tamanho::Word),
        ("CMPA.L", 0xB1C1, Tamanho::Long),
    ] {
        let mut banco = Bancada::nova(&[opcode]);
        let mut r = Sorteio(12);
        for k in 0..1500u32 {
            let bits = u16::try_from(k % 32).unwrap();
            let mut e = Regs::default();
            e.a[0] = r.u32();
            e.d[1] = r.u32() >> (k % 20);
            e.sr = bits;
            let o = banco.rodar(&e, &[]);
            let src = if t == Tamanho::Word {
                e.d[1] & 0xFFFF
            } else {
                e.d[1]
            };
            let (a0, ccr) = if nome.starts_with("ADDA") {
                (ops68k::adda(t, src, e.a[0]), Ccr::do_sr(bits))
            } else if nome.starts_with("SUBA") {
                (ops68k::suba(t, src, e.a[0]), Ccr::do_sr(bits))
            } else {
                (e.a[0], ops68k::cmpa(t, src, e.a[0], Ccr::do_sr(bits)))
            };
            if o.regs.a[0] != a0 || o.regs.sr & 0x1F != ccr.para_bits() {
                ruins.push(format!(
                    "{nome} a0={:X} d1={:X} ccr={bits:02X}",
                    e.a[0], e.d[1]
                ));
            }
        }
    }
    verificar(ruins);
}
