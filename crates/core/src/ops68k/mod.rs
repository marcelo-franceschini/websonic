//! Operações aritméticas e lógicas do 68000, com resultado e flags idênticos aos do processador
//! (sub-etapa 5.2).
//!
//! São funções puras sobre inteiros: não há interpretador, nem registradores como estado. Cada
//! função recebe os operandos já no tamanho da operação (byte, word ou long, nos bits baixos),
//! o CCR de entrada e devolve o valor e o CCR de saída. Flags que a instrução não afeta saem
//! iguais às de entrada.
//!
//! Fonte do comportamento: Motorola, M68000 Family Programmer's Reference Manual (seção de cada
//! instrução e o resumo de códigos de condição). Flags que o manual deixa indefinidas seguem o
//! que o oráculo produz, medido pelo teste diferencial (`diferencial/tests/ops.rs`).
//! Estes nomes não espelham rotinas do jogo: são o conjunto de instruções do processador.

use crate::memoria::Tamanho;

mod bcd;
mod deslocamento;

pub use bcd::{abcd, nbcd, sbcd};
pub use deslocamento::{Deslocamento, deslocar};

/// Códigos de condição: X (estende), N (negativo), Z (zero), V (estouro com sinal), C (vai-um).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ccr {
    pub x: bool,
    pub n: bool,
    pub z: bool,
    pub v: bool,
    pub c: bool,
}

impl Ccr {
    /// Do byte baixo do SR: C bit 0, V bit 1, Z bit 2, N bit 3, X bit 4.
    pub const fn do_sr(sr: u16) -> Self {
        Self {
            c: sr & 1 != 0,
            v: sr & 2 != 0,
            z: sr & 4 != 0,
            n: sr & 8 != 0,
            x: sr & 16 != 0,
        }
    }

    /// Os 5 bits do CCR no formato do SR.
    pub const fn para_bits(self) -> u16 {
        (self.c as u16)
            | ((self.v as u16) << 1)
            | ((self.z as u16) << 2)
            | ((self.n as u16) << 3)
            | ((self.x as u16) << 4)
    }
}

/// Valor e CCR depois de uma operação.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Resultado {
    pub valor: u32,
    pub ccr: Ccr,
}

/// Erros que o processador trata como exceção.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Excecao {
    /// DIVU ou DIVS com divisor zero (vetor 5).
    DivisaoPorZero,
}

pub(crate) const fn bits(t: Tamanho) -> u32 {
    match t {
        Tamanho::Byte => 8,
        Tamanho::Word => 16,
        Tamanho::Long => 32,
    }
}

pub(crate) const fn mascara(t: Tamanho) -> u64 {
    match t {
        Tamanho::Byte => 0xFF,
        Tamanho::Word => 0xFFFF,
        Tamanho::Long => 0xFFFF_FFFF,
    }
}

pub(crate) const fn bit_alto(t: Tamanho) -> u64 {
    match t {
        Tamanho::Byte => 0x80,
        Tamanho::Word => 0x8000,
        Tamanho::Long => 0x8000_0000,
    }
}

/// Interpreta `v` (nos bits baixos do tamanho) como inteiro com sinal.
pub(crate) fn com_sinal(v: u64, t: Tamanho) -> i64 {
    let v = v & mascara(t);
    if v & bit_alto(t) != 0 {
        i64::try_from(v)
            .unwrap_or(0)
            .wrapping_sub(i64::try_from(mascara(t)).unwrap_or(0).wrapping_add(1))
    } else {
        i64::try_from(v).unwrap_or(0)
    }
}

/// O valor com sinal cabe no tamanho?
pub(crate) fn cabe(v: i64, t: Tamanho) -> bool {
    let metade = i64::try_from(bit_alto(t)).unwrap_or(0);
    v >= metade.wrapping_neg() && v < metade
}

fn u32_de(v: u64) -> u32 {
    u32::try_from(v & 0xFFFF_FFFF).unwrap_or(0)
}

/// N e Z de um resultado.
fn nz(valor: u64, t: Tamanho) -> (bool, bool) {
    (valor & bit_alto(t) != 0, valor & mascara(t) == 0)
}

fn montar(valor: u64, t: Tamanho, ccr: Ccr) -> Resultado {
    Resultado {
        valor: u32_de(valor & mascara(t)),
        ccr,
    }
}

/// ADD, ADDI, ADDQ (em registrador ou memória): `dst + src`.
pub fn add(t: Tamanho, src: u32, dst: u32, ccr: Ccr) -> Resultado {
    somar(t, src, dst, false, ccr, false)
}

/// ADDX: `dst + src + X`. Z só é zerado, nunca ligado.
pub fn addx(t: Tamanho, src: u32, dst: u32, ccr: Ccr) -> Resultado {
    somar(t, src, dst, true, ccr, true)
}

fn somar(t: Tamanho, src: u32, dst: u32, com_x: bool, ccr: Ccr, z_acumula: bool) -> Resultado {
    let (s, d) = (u64::from(src) & mascara(t), u64::from(dst) & mascara(t));
    let x = u64::from(com_x && ccr.x);
    let largo = s.wrapping_add(d).wrapping_add(x);
    let r = largo & mascara(t);
    let soma_com_sinal = com_sinal(s, t)
        .wrapping_add(com_sinal(d, t))
        .wrapping_add(i64::try_from(x).unwrap_or(0));
    let (n, z) = nz(r, t);
    let carry = largo > mascara(t);
    montar(
        r,
        t,
        Ccr {
            x: carry,
            n,
            z: if z_acumula { ccr.z && z } else { z },
            v: !cabe(soma_com_sinal, t),
            c: carry,
        },
    )
}

/// SUB, SUBI, SUBQ: `dst - src`.
pub fn sub(t: Tamanho, src: u32, dst: u32, ccr: Ccr) -> Resultado {
    subtrair(t, src, dst, false, ccr, false)
}

/// SUBX: `dst - src - X`.
pub fn subx(t: Tamanho, src: u32, dst: u32, ccr: Ccr) -> Resultado {
    subtrair(t, src, dst, true, ccr, true)
}

fn subtrair(t: Tamanho, src: u32, dst: u32, com_x: bool, ccr: Ccr, z_acumula: bool) -> Resultado {
    let (s, d) = (u64::from(src) & mascara(t), u64::from(dst) & mascara(t));
    let x = u64::from(com_x && ccr.x);
    let r = d.wrapping_sub(s).wrapping_sub(x) & mascara(t);
    let emprestimo = d < s.wrapping_add(x);
    let dif_com_sinal = com_sinal(d, t)
        .wrapping_sub(com_sinal(s, t))
        .wrapping_sub(i64::try_from(x).unwrap_or(0));
    let (n, z) = nz(r, t);
    montar(
        r,
        t,
        Ccr {
            x: emprestimo,
            n,
            z: if z_acumula { ccr.z && z } else { z },
            v: !cabe(dif_com_sinal, t),
            c: emprestimo,
        },
    )
}

/// NEG: `0 - dst`.
pub fn neg(t: Tamanho, dst: u32, ccr: Ccr) -> Resultado {
    sub(t, dst, 0, ccr)
}

/// NEGX: `0 - dst - X`.
pub fn negx(t: Tamanho, dst: u32, ccr: Ccr) -> Resultado {
    subx(t, dst, 0, ccr)
}

/// CMP, CMPI, CMPM: `dst - src` sem guardar o resultado. X não muda.
pub fn cmp(t: Tamanho, src: u32, dst: u32, ccr: Ccr) -> Ccr {
    Ccr {
        x: ccr.x,
        ..sub(t, src, dst, ccr).ccr
    }
}

/// CMPA: compara `An` (long) com a fonte de word estendida com sinal (ou long).
pub fn cmpa(t_fonte: Tamanho, src: u32, an: u32, ccr: Ccr) -> Ccr {
    cmp(Tamanho::Long, estender_fonte_de_a(t_fonte, src), an, ccr)
}

/// A fonte de ADDA, SUBA, CMPA e MOVEA: word é estendida com sinal para 32 bits.
pub fn estender_fonte_de_a(t: Tamanho, src: u32) -> u32 {
    match t {
        Tamanho::Word => u32_de(u64::from_ne_bytes(
            com_sinal(u64::from(src), Tamanho::Word).to_ne_bytes(),
        )),
        _ => src,
    }
}

/// ADDA: soma em registrador de endereço, sem mexer nas flags.
pub fn adda(t_fonte: Tamanho, src: u32, an: u32) -> u32 {
    estender_fonte_de_a(t_fonte, src).wrapping_add(an)
}

/// SUBA: subtrai em registrador de endereço, sem mexer nas flags.
pub fn suba(t_fonte: Tamanho, src: u32, an: u32) -> u32 {
    an.wrapping_sub(estender_fonte_de_a(t_fonte, src))
}

/// CLR: zera. X não muda.
pub fn clr(ccr: Ccr) -> Resultado {
    Resultado {
        valor: 0,
        ccr: Ccr {
            x: ccr.x,
            n: false,
            z: true,
            v: false,
            c: false,
        },
    }
}

/// TST: N e Z do valor, V e C zerados. X não muda.
pub fn tst(t: Tamanho, v: u32, ccr: Ccr) -> Ccr {
    let (n, z) = nz(u64::from(v), t);
    Ccr {
        x: ccr.x,
        n,
        z,
        v: false,
        c: false,
    }
}

fn logica(t: Tamanho, r: u64, ccr: Ccr) -> Resultado {
    let (n, z) = nz(r, t);
    montar(
        r,
        t,
        Ccr {
            x: ccr.x,
            n,
            z,
            v: false,
            c: false,
        },
    )
}

/// AND, ANDI.
pub fn and(t: Tamanho, src: u32, dst: u32, ccr: Ccr) -> Resultado {
    logica(t, u64::from(src & dst), ccr)
}

/// OR, ORI.
pub fn or(t: Tamanho, src: u32, dst: u32, ccr: Ccr) -> Resultado {
    logica(t, u64::from(src | dst), ccr)
}

/// EOR, EORI.
pub fn eor(t: Tamanho, src: u32, dst: u32, ccr: Ccr) -> Resultado {
    logica(t, u64::from(src ^ dst), ccr)
}

/// NOT.
pub fn not(t: Tamanho, dst: u32, ccr: Ccr) -> Resultado {
    logica(t, u64::from(!dst), ccr)
}

/// MOVE e MOVEQ para registrador de dados ou memória: N e Z do valor, V e C zerados.
pub fn mover(t: Tamanho, v: u32, ccr: Ccr) -> Resultado {
    logica(t, u64::from(v), ccr)
}

/// EXT: byte para word (`t_destino` = Word) ou word para long (`t_destino` = Long).
pub fn ext(t_destino: Tamanho, dst: u32, ccr: Ccr) -> Resultado {
    let (origem, mantem_alto) = match t_destino {
        Tamanho::Word => (Tamanho::Byte, dst & 0xFFFF_0000),
        _ => (Tamanho::Word, 0),
    };
    let estendido =
        u64::from_ne_bytes(com_sinal(u64::from(dst), origem).to_ne_bytes()) & mascara(t_destino);
    let r = logica(t_destino, estendido, ccr);
    Resultado {
        valor: r.valor | mantem_alto,
        ..r
    }
}

/// SWAP: troca as metades do registrador.
pub fn swap(dst: u32, ccr: Ccr) -> Resultado {
    logica(Tamanho::Long, u64::from(dst.rotate_left(16)), ccr)
}

/// MULU: word sem sinal vezes word sem sinal, resultado long.
pub fn mulu(src: u32, dst: u32, ccr: Ccr) -> Resultado {
    let r = u64::from(src & 0xFFFF).wrapping_mul(u64::from(dst & 0xFFFF));
    logica(Tamanho::Long, r, ccr)
}

/// MULS: word com sinal vezes word com sinal, resultado long.
pub fn muls(src: u32, dst: u32, ccr: Ccr) -> Resultado {
    let r = com_sinal(u64::from(src), Tamanho::Word)
        .wrapping_mul(com_sinal(u64::from(dst), Tamanho::Word));
    logica(
        Tamanho::Long,
        u64::from_ne_bytes(r.to_ne_bytes()) & 0xFFFF_FFFF,
        ccr,
    )
}

/// DIVU: dividendo long, divisor word, ambos sem sinal. Quociente nos 16 bits baixos e resto nos
/// altos. Com estouro, o destino não muda e N e V ficam ligados.
pub fn divu(divisor: u32, dividendo: u32, ccr: Ccr) -> Result<Resultado, Excecao> {
    let d = u64::from(divisor & 0xFFFF);
    if d == 0 {
        return Err(Excecao::DivisaoPorZero);
    }
    let dividendo = u64::from(dividendo);
    let q = dividendo.checked_div(d).unwrap_or(0);
    let resto = dividendo.checked_rem(d).unwrap_or(0);
    if q > 0xFFFF {
        return Ok(Resultado {
            valor: u32_de(dividendo),
            ccr: Ccr {
                x: ccr.x,
                n: true,
                z: false,
                v: true,
                c: false,
            },
        });
    }
    Ok(Resultado {
        valor: u32_de((resto << 16) | q),
        ccr: Ccr {
            x: ccr.x,
            n: q & 0x8000 != 0,
            z: q == 0,
            v: false,
            c: false,
        },
    })
}

/// DIVS: dividendo long com sinal, divisor word com sinal. O resto tem o sinal do dividendo.
pub fn divs(divisor: u32, dividendo: u32, ccr: Ccr) -> Result<Resultado, Excecao> {
    let d = com_sinal(u64::from(divisor), Tamanho::Word);
    if d == 0 {
        return Err(Excecao::DivisaoPorZero);
    }
    let n = com_sinal(u64::from(dividendo), Tamanho::Long);
    let q = n.checked_div(d).unwrap_or(0);
    let resto = n.checked_rem(d).unwrap_or(0);
    if !cabe(q, Tamanho::Word) {
        return Ok(Resultado {
            valor: dividendo,
            ccr: Ccr {
                x: ccr.x,
                n: true,
                z: false,
                v: true,
                c: false,
            },
        });
    }
    let qb = u64::from_ne_bytes(q.to_ne_bytes()) & 0xFFFF;
    let rb = u64::from_ne_bytes(resto.to_ne_bytes()) & 0xFFFF;
    Ok(Resultado {
        valor: u32_de((rb << 16) | qb),
        ccr: Ccr {
            x: ccr.x,
            n: q < 0,
            z: q == 0,
            v: false,
            c: false,
        },
    })
}

/// Operação de bit com número de bit (BTST, BSET, BCLR, BCHG).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpBit {
    Testar,
    Ligar,
    Desligar,
    Inverter,
}

/// Operações de bit. `t` é Long para registrador de dados (bit módulo 32) e Byte para memória
/// (bit módulo 8). Só Z muda: fica ligado se o bit testado era zero.
pub fn operar_bit(op: OpBit, t: Tamanho, bit: u32, dst: u32, ccr: Ccr) -> Resultado {
    let n = bit & bits(t).wrapping_sub(1);
    let m = 1u32.wrapping_shl(n);
    let era_zero = dst & m == 0;
    let valor = match op {
        OpBit::Testar => dst,
        OpBit::Ligar => dst | m,
        OpBit::Desligar => dst & !m,
        OpBit::Inverter => dst ^ m,
    };
    Resultado {
        valor,
        ccr: Ccr { z: era_zero, ..ccr },
    }
}

/// As 16 condições de Bcc, Scc e DBcc.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Condicao {
    Verdadeira,
    Falsa,
    Alta,
    BaixaOuIgual,
    CarryLimpo,
    CarryLigado,
    Diferente,
    Igual,
    OverflowLimpo,
    OverflowLigado,
    Positivo,
    Negativo,
    MaiorOuIgual,
    Menor,
    Maior,
    MenorOuIgual,
}

impl Condicao {
    /// A condição de 4 bits do opcode.
    pub const fn do_codigo(c: u8) -> Self {
        match c & 15 {
            0 => Self::Verdadeira,
            1 => Self::Falsa,
            2 => Self::Alta,
            3 => Self::BaixaOuIgual,
            4 => Self::CarryLimpo,
            5 => Self::CarryLigado,
            6 => Self::Diferente,
            7 => Self::Igual,
            8 => Self::OverflowLimpo,
            9 => Self::OverflowLigado,
            10 => Self::Positivo,
            11 => Self::Negativo,
            12 => Self::MaiorOuIgual,
            13 => Self::Menor,
            14 => Self::Maior,
            _ => Self::MenorOuIgual,
        }
    }

    pub const fn avaliar(self, f: Ccr) -> bool {
        match self {
            Self::Verdadeira => true,
            Self::Falsa => false,
            Self::Alta => !f.c && !f.z,
            Self::BaixaOuIgual => f.c || f.z,
            Self::CarryLimpo => !f.c,
            Self::CarryLigado => f.c,
            Self::Diferente => !f.z,
            Self::Igual => f.z,
            Self::OverflowLimpo => !f.v,
            Self::OverflowLigado => f.v,
            Self::Positivo => !f.n,
            Self::Negativo => f.n,
            Self::MaiorOuIgual => f.n == f.v,
            Self::Menor => f.n != f.v,
            Self::Maior => !f.z && f.n == f.v,
            Self::MenorOuIgual => f.z || f.n != f.v,
        }
    }
}

/// Scc: 0xFF se a condição vale, senão 0x00.
pub const fn scc(cond: Condicao, ccr: Ccr) -> u8 {
    if cond.avaliar(ccr) { 0xFF } else { 0x00 }
}

/// DBcc: se a condição vale, o laço termina e o contador não muda. Senão o contador (word)
/// diminui 1 e o desvio é tomado enquanto o novo valor não for -1. Devolve (contador, tomado).
pub fn dbcc(cond: Condicao, ccr: Ccr, contador: u16) -> (u16, bool) {
    if cond.avaliar(ccr) {
        (contador, false)
    } else {
        let novo = contador.wrapping_sub(1);
        (novo, novo != 0xFFFF)
    }
}
