//! Operações do 68000: vetores do manual, propriedades e gabarito por fórmulas booleanas.
//! O teste diferencial com o oráculo está em `diferencial/tests/ops.rs`.

use sonic1_core::memoria::Tamanho;
use sonic1_core::memoria::Tamanho::{Byte, Long, Word};
use sonic1_core::ops68k::{self, Ccr, Condicao, Deslocamento, OpBit};
use sonic1_teste_nucleo::ops::{gabarito_add, gabarito_neg, gabarito_sub, mascara};

const LIMPO: Ccr = Ccr {
    x: false,
    n: false,
    z: false,
    v: false,
    c: false,
};

fn flags(x: bool, n: bool, z: bool, v: bool, c: bool) -> Ccr {
    Ccr { x, n, z, v, c }
}

// ---- Vetores calculados à mão com as definições do manual ----

#[test]
fn vetores_de_soma() {
    // 0x7F + 0x01 = 0x80: estouro com sinal.
    let r = ops68k::add(Byte, 0x01, 0x7F, LIMPO);
    assert_eq!(
        (r.valor, r.ccr),
        (0x80, flags(false, true, false, true, false))
    );
    // 0xFF + 0x01 = 0x00: vai-um, zero, X acompanha C.
    let r = ops68k::add(Byte, 0x01, 0xFF, LIMPO);
    assert_eq!(
        (r.valor, r.ccr),
        (0x00, flags(true, false, true, false, true))
    );
    // 0x8000 + 0x8000 = 0 (word): vai-um e estouro.
    let r = ops68k::add(Word, 0x8000, 0x8000, LIMPO);
    assert_eq!((r.valor, r.ccr), (0, flags(true, false, true, true, true)));
    // ADDX: Z só é zerado. Z ligado na entrada e resultado zero: continua ligado.
    let z_in = Ccr { z: true, ..LIMPO };
    let r = ops68k::addx(Byte, 0, 0, z_in);
    assert!(r.ccr.z);
    // Resultado diferente de zero apaga Z. Soma o X de entrada.
    let r = ops68k::addx(
        Byte,
        1,
        1,
        Ccr {
            x: true,
            z: true,
            ..LIMPO
        },
    );
    assert_eq!(r.valor, 3);
    assert!(!r.ccr.z);
}

#[test]
fn vetores_de_subtracao() {
    // 0x00 - 0x01 = 0xFF: empresta.
    let r = ops68k::sub(Byte, 0x01, 0x00, LIMPO);
    assert_eq!(
        (r.valor, r.ccr),
        (0xFF, flags(true, true, false, false, true))
    );
    // 0x80 - 0x01 = 0x7F: estouro com sinal.
    let r = ops68k::sub(Byte, 0x01, 0x80, LIMPO);
    assert_eq!(
        (r.valor, r.ccr),
        (0x7F, flags(false, false, false, true, false))
    );
    // NEG 0x80 = 0x80: estouro e vai-um.
    let r = ops68k::neg(Byte, 0x80, LIMPO);
    assert_eq!(
        (r.valor, r.ccr),
        (0x80, flags(true, true, false, true, true))
    );
    // NEG 0 = 0: sem vai-um.
    let r = ops68k::neg(Byte, 0, LIMPO);
    assert_eq!(
        (r.valor, r.ccr),
        (0, flags(false, false, true, false, false))
    );
}

#[test]
fn cmp_nao_mexe_em_x_e_tst_clr_idem() {
    let x = Ccr { x: true, ..LIMPO };
    assert!(ops68k::cmp(Word, 5, 3, x).x);
    assert!(!ops68k::cmp(Word, 5, 3, LIMPO).x);
    assert!(ops68k::tst(Long, 0, x).x);
    assert!(ops68k::clr(x).ccr.x);
    // CMP 5 com 5 liga Z; CMP 3 com 5 (dst 3 < src 5) liga N e C.
    let c = ops68k::cmp(Byte, 5, 5, LIMPO);
    assert!(c.z && !c.c);
    let c = ops68k::cmp(Byte, 5, 3, LIMPO);
    assert!(c.c && c.n && !c.z);
}

#[test]
fn vetores_de_deslocamento() {
    use Deslocamento::*;
    // ASL.B #1 de 0x40 = 0x80: o bit de sinal mudou, V ligado.
    let r = ops68k::deslocar(AslEsquerda, Byte, 1, 0x40, LIMPO);
    assert_eq!(
        (r.valor, r.ccr),
        (0x80, flags(false, true, false, true, false))
    );
    // LSR.B #1 de 0x01 = 0: C e X ligados, Z ligado.
    let r = ops68k::deslocar(LsrDireita, Byte, 1, 0x01, LIMPO);
    assert_eq!((r.valor, r.ccr), (0, flags(true, false, true, false, true)));
    // ASR mantem o sinal.
    let r = ops68k::deslocar(AsrDireita, Byte, 1, 0x80, LIMPO);
    assert_eq!(r.valor, 0xC0);
    // Contagem 0: C zerado, X intacto (menos em ROX, onde C recebe X).
    let x = Ccr {
        x: true,
        c: true,
        ..LIMPO
    };
    let r = ops68k::deslocar(LslEsquerda, Word, 0, 0x1234, x);
    assert_eq!((r.ccr.x, r.ccr.c), (true, false));
    let r = ops68k::deslocar(RoxlEsquerda, Word, 0, 0x1234, Ccr { x: true, ..LIMPO });
    assert_eq!((r.ccr.x, r.ccr.c), (true, true));
    // ROL.B #1 de 0x80 = 0x01 com C ligado e X intacto.
    let r = ops68k::deslocar(RolEsquerda, Byte, 1, 0x80, LIMPO);
    assert_eq!(
        (r.valor, r.ccr),
        (0x01, flags(false, false, false, false, true))
    );
}

#[test]
fn vetores_de_multiplicacao_e_divisao() {
    let r = ops68k::mulu(0xFFFF, 0xFFFF, LIMPO);
    assert_eq!(r.valor, 0xFFFE_0001);
    assert!(r.ccr.n);
    let r = ops68k::muls(0xFFFF, 0x0002, LIMPO); // -1 * 2 = -2
    assert_eq!(r.valor, 0xFFFF_FFFE);
    // DIVU 100 / 7 = 14 resto 2.
    let r = ops68k::divu(7, 100, LIMPO).unwrap();
    assert_eq!(r.valor, (2 << 16) | 14);
    // Estouro: quociente nao cabe em 16 bits. O destino nao muda e V liga.
    let r = ops68k::divu(1, 0x1_0000, LIMPO).unwrap();
    assert!(r.ccr.v);
    assert_eq!(r.valor, 0x1_0000);
    // DIVS -7 / 2 = -3 resto -1 (o resto tem o sinal do dividendo).
    let r = ops68k::divs(2, 0xFFFF_FFF9, LIMPO).unwrap();
    assert_eq!(r.valor, (0xFFFF << 16) | 0xFFFD);
    assert_eq!(
        ops68k::divu(0, 5, LIMPO),
        Err(ops68k::Excecao::DivisaoPorZero)
    );
    assert_eq!(
        ops68k::divs(0, 5, LIMPO),
        Err(ops68k::Excecao::DivisaoPorZero)
    );
}

#[test]
fn vetores_de_bcd() {
    // 0x19 + 0x01 = 0x20 em BCD, sem vai-um.
    let r = ops68k::abcd(0x01, 0x19, LIMPO);
    assert_eq!(r.valor, 0x20);
    assert!(!r.ccr.c);
    // 0x99 + 0x01 = 0x00 com vai-um; Z continua como estava (zerado na entrada).
    let r = ops68k::abcd(0x01, 0x99, LIMPO);
    assert_eq!(r.valor, 0x00);
    assert!(r.ccr.c && r.ccr.x && !r.ccr.z);
    // 0x20 - 0x01 = 0x19.
    let r = ops68k::sbcd(0x01, 0x20, LIMPO);
    assert_eq!(r.valor, 0x19);
    // NBCD de 0x01 = 0x99 com empresta.
    let r = ops68k::nbcd(0x01, LIMPO);
    assert_eq!(r.valor, 0x99);
    assert!(r.ccr.c);
}

#[test]
fn condicoes_e_dbcc() {
    let z = Ccr { z: true, ..LIMPO };
    assert!(Condicao::Igual.avaliar(z));
    assert!(!Condicao::Diferente.avaliar(z));
    assert!(Condicao::MenorOuIgual.avaliar(z));
    assert!(Condicao::Verdadeira.avaliar(LIMPO) && !Condicao::Falsa.avaliar(LIMPO));
    // N diferente de V: menor.
    assert!(Condicao::Menor.avaliar(Ccr { n: true, ..LIMPO }));
    assert!(Condicao::MaiorOuIgual.avaliar(Ccr {
        n: true,
        v: true,
        ..LIMPO
    }));
    assert_eq!(ops68k::scc(Condicao::Igual, z), 0xFF);
    assert_eq!(ops68k::scc(Condicao::Igual, LIMPO), 0x00);
    // DBcc: condicao falsa decrementa; chega a -1 e para.
    assert_eq!(ops68k::dbcc(Condicao::Falsa, LIMPO, 2), (1, true));
    assert_eq!(ops68k::dbcc(Condicao::Falsa, LIMPO, 0), (0xFFFF, false));
    // Condicao verdadeira: nao mexe no contador e nao desvia.
    assert_eq!(ops68k::dbcc(Condicao::Verdadeira, LIMPO, 5), (5, false));
}

#[test]
fn operacoes_de_bit() {
    let r = ops68k::operar_bit(OpBit::Ligar, Long, 33, 0, LIMPO); // 33 mod 32 = 1
    assert_eq!(r.valor, 2);
    assert!(r.ccr.z); // o bit estava em zero
    let r = ops68k::operar_bit(OpBit::Inverter, Byte, 9, 0xFF, LIMPO); // 9 mod 8 = 1
    assert_eq!(r.valor, 0xFD);
    assert!(!r.ccr.z);
}

// ---- Gabarito por fórmulas booleanas ----

fn amostras(t: Tamanho) -> Vec<u32> {
    if t == Byte {
        return (0..=0xFF).collect();
    }
    let m = mascara(t);
    let alto = (m >> 1) + 1;
    let mut v = vec![
        0,
        1,
        2,
        0x7F,
        0x80,
        0xFF,
        0x100,
        alto - 1,
        alto,
        alto + 1,
        m - 1,
        m,
        m >> 4,
        0x1234 & m,
        0xA5A5_A5A5 & m,
    ];
    let mut x = 0x1234_5678u32;
    for _ in 0..40 {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        v.push(x & m);
    }
    v.sort_unstable();
    v.dedup();
    v
}

fn confere(nome: &str, obtido: (u32, Ccr), esperado: (u32, Ccr)) {
    assert_eq!(obtido, esperado, "{nome}");
}

#[test]
fn soma_e_subtracao_batem_com_as_formulas_do_manual() {
    for t in [Byte, Word, Long] {
        let a = amostras(t);
        for s in &a {
            for d in &a {
                for x in [false, true] {
                    for z_in in [false, true] {
                        let c = Ccr {
                            x,
                            n: false,
                            z: z_in,
                            v: false,
                            c: false,
                        };
                        let r = ops68k::add(t, *s, *d, c);
                        confere(
                            "ADD",
                            (r.valor, r.ccr),
                            gabarito_add(t, *s, *d, x, false, z_in, false),
                        );
                        let r = ops68k::addx(t, *s, *d, c);
                        confere(
                            "ADDX",
                            (r.valor, r.ccr),
                            gabarito_add(t, *s, *d, x, true, z_in, true),
                        );
                        let r = ops68k::sub(t, *s, *d, c);
                        confere(
                            "SUB",
                            (r.valor, r.ccr),
                            gabarito_sub(t, *s, *d, x, false, z_in, false),
                        );
                        let r = ops68k::subx(t, *s, *d, c);
                        confere(
                            "SUBX",
                            (r.valor, r.ccr),
                            gabarito_sub(t, *s, *d, x, true, z_in, true),
                        );
                        // CMP: as mesmas flags de SUB, mas X fica como entrou.
                        let (_, g) = gabarito_sub(t, *s, *d, x, false, z_in, false);
                        assert_eq!(ops68k::cmp(t, *s, *d, c), Ccr { x, ..g }, "CMP");
                    }
                }
            }
            let (gv, gc) = gabarito_neg(t, *s);
            let r = ops68k::neg(t, *s, LIMPO);
            confere("NEG", (r.valor, r.ccr), (gv, gc));
        }
    }
}

// ---- Propriedades ----

#[test]
fn propriedades_de_ida_e_volta() {
    for t in [Byte, Word, Long] {
        let a = amostras(t);
        for s in &a {
            for d in &a {
                let soma = ops68k::add(t, *s, *d, LIMPO).valor;
                assert_eq!(
                    ops68k::sub(t, *s, soma, LIMPO).valor,
                    *d,
                    "ADD seguido de SUB"
                );
                // Z e N coerentes com o resultado.
                let r = ops68k::and(t, *s, *d, LIMPO);
                assert_eq!(r.ccr.z, r.valor == 0);
                assert_eq!(r.ccr.n, r.valor & (mascara(t) ^ (mascara(t) >> 1)) != 0);
                assert!(!r.ccr.v && !r.ccr.c);
            }
            let n = ops68k::neg(t, *s, LIMPO).valor;
            assert_eq!(ops68k::neg(t, n, LIMPO).valor, *s, "NEG duas vezes");
            assert_eq!(
                ops68k::not(t, ops68k::not(t, *s, LIMPO).valor, LIMPO).valor,
                *s
            );
        }
    }
}

#[test]
fn propriedades_de_deslocamento() {
    use Deslocamento::*;
    for t in [Byte, Word, Long] {
        let bits = match t {
            Byte => 8,
            Word => 16,
            Long => 32,
        };
        for v in amostras(t) {
            // Rotacionar pelo tamanho inteiro devolve o valor.
            assert_eq!(ops68k::deslocar(RolEsquerda, t, bits, v, LIMPO).valor, v);
            assert_eq!(ops68k::deslocar(RorDireita, t, bits, v, LIMPO).valor, v);
            // ROXL por tamanho + 1 devolve o valor (o bit X entra na volta).
            assert_eq!(
                ops68k::deslocar(RoxlEsquerda, t, bits + 1, v, LIMPO).valor,
                v
            );
            for n in 0..bits {
                // ASL e LSL dao o mesmo valor, e ROL(n) seguido de ROR(n) volta.
                let a = ops68k::deslocar(AslEsquerda, t, n, v, LIMPO);
                let l = ops68k::deslocar(LslEsquerda, t, n, v, LIMPO);
                assert_eq!(a.valor, l.valor);
                let r = ops68k::deslocar(RolEsquerda, t, n, v, LIMPO).valor;
                assert_eq!(ops68k::deslocar(RorDireita, t, n, r, LIMPO).valor, v);
                // LSL(n) iguala a multiplicacao por 2^n no tamanho.
                assert_eq!(l.valor, (u64::from(v) << n) as u32 & mascara(t));
            }
        }
    }
}

#[test]
fn multiplicacao_e_divisao_contra_aritmetica_de_64_bits() {
    let a = amostras(Word);
    for s in &a {
        for d in &a {
            assert_eq!(ops68k::mulu(*s, *d, LIMPO).valor, s * d);
            let ms = i64::from(*s as u16 as i16) * i64::from(*d as u16 as i16);
            assert_eq!(ops68k::muls(*s, *d, LIMPO).valor, ms as u32);
        }
    }
    for divisor in &a {
        if *divisor == 0 {
            continue;
        }
        for dividendo in amostras(Long) {
            let r = ops68k::divu(*divisor, dividendo, LIMPO).unwrap();
            let q = u64::from(dividendo) / u64::from(*divisor);
            if q > 0xFFFF {
                assert!(r.ccr.v);
            } else {
                assert_eq!(
                    r.valor,
                    ((u64::from(dividendo) % u64::from(*divisor)) << 16) as u32 | q as u32
                );
            }
        }
    }
}
