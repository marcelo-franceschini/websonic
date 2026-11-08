//! Experimento do nível 2 (sub-etapa 7.8): prever o custo em ciclos de laços de espera lendo o
//! código do 68000 na ROM do jogador, sem executá-lo.
//!
//! O que existe aqui NÃO é um interpretador: reconhece só três formas de laço (a soma de
//! verificação, a limpeza da RAM e o atraso da voz SEGA), lê as constantes delas nos bytes das
//! instruções e soma os ciclos da tabela de tempo do manual do 68000 (Motorola, M68000 Family
//! Programmer's Reference Manual, tabelas de tempo de instrução). Fica fora do modelo tudo que
//! vem do hardware: espera do Z80 pela ROM, DMA, espera do chip de vídeo e atualização da
//! memória. A medida do erro contra o oráculo diz quanto isso pesa (`docs/conhecimento/abertura.md`).

/// Resultado de uma previsão: ciclos do 68000 e endereço da instrução depois do trecho.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Previsao {
    pub ciclos: u64,
    pub fim: u32,
}

/// Ajuste da atualização da memória (refresh): a medida do oráculo na 7.8 deu o mesmo desvio
/// de 1,5877% para a soma da ROM e para a limpeza da RAM, igual a 128/126, isto é, 2 ciclos
/// perdidos a cada 128. É um ajuste tirado da medida, ainda sem fonte de hardware conferida.
pub fn com_refresh(ciclos: u64) -> u64 {
    ciclos.saturating_mul(128) / 126
}

fn palavra(rom: &[u8], a: u32) -> Option<u16> {
    let i = usize::try_from(a).ok()?;
    let b = rom.get(i..i.checked_add(2)?)?;
    Some(u16::from_be_bytes([*b.first()?, *b.get(1)?]))
}

fn longa(rom: &[u8], a: u32) -> Option<u32> {
    let alta = palavra(rom, a)?;
    let baixa = palavra(rom, a.checked_add(2)?)?;
    Some((u32::from(alta) << 16) | u32::from(baixa))
}

/// Destino de um `dbf` ou `bcc.s`: relativo ao endereço da palavra de deslocamento.
fn alvo16(rom: &[u8], a_disp: u32) -> Option<u32> {
    let d = palavra(rom, a_disp)?;
    let d = i32::from(i16::from_be_bytes(d.to_be_bytes()));
    u32::try_from(i64::from(a_disp).checked_add(i64::from(d))?).ok()
}

fn soma(a: u64, b: u64) -> Option<u64> {
    a.checked_add(b)
}

/// `PlaySegaSound` (a partir da entrada): preâmbulo e laço duplo de atraso. Devolve o custo até
/// a instrução que ajusta a pilha para sair (`addq.w #4,sp`), que é onde o laço acaba.
pub fn laco_da_voz(rom: &[u8], inicio: u32) -> Option<Previsao> {
    let mut pc = inicio;
    let mut total = 0u64;
    for _ in 0..16 {
        match palavra(rom, pc)? {
            // move.b #imm,(abs).l e move.w #imm,(abs).l: 20 ciclos.
            0x13FC => {
                total = soma(total, 20)?;
                pc = pc.checked_add(8)?;
            }
            0x33FC => {
                total = soma(total, 20)?;
                pc = pc.checked_add(10)?;
            }
            // move.w #o,d1 ... laço duplo.
            0x323C => {
                let externo = u64::from(palavra(rom, pc.checked_add(2)?)?);
                let mut p = pc.checked_add(4)?;
                let inicio_interno = p;
                if palavra(rom, p)? != 0x303C {
                    return None;
                }
                let interno = u64::from(palavra(rom, p.checked_add(2)?)?);
                p = p.checked_add(4)?;
                let primeiro_nop = p;
                let mut nops = 0u64;
                while palavra(rom, p)? == 0x4E71 {
                    nops = nops.checked_add(1)?;
                    p = p.checked_add(2)?;
                }
                // dbf d0,primeiro_nop
                if palavra(rom, p)? != 0x51C8 || alvo16(rom, p.checked_add(2)?)? != primeiro_nop {
                    return None;
                }
                p = p.checked_add(4)?;
                // dbf d1,inicio_interno
                if palavra(rom, p)? != 0x51C9 || alvo16(rom, p.checked_add(2)?)? != inicio_interno {
                    return None;
                }
                p = p.checked_add(4)?;
                // Interno: cada volta tem os nops e um dbf (10 se volta, 14 na última).
                let voltas_internas = interno.checked_add(1)?;
                let interno_total = voltas_internas
                    .checked_mul(nops.checked_mul(4)?)?
                    .checked_add(interno.checked_mul(10)?)?
                    .checked_add(14)?;
                // Externo: move.w #i,d0 (8) + interno + dbf d1 (10 se volta, 14 na última).
                let passadas = externo.checked_add(1)?;
                let externo_total = passadas
                    .checked_mul(soma(8, interno_total)?)?
                    .checked_add(externo.checked_mul(10)?)?
                    .checked_add(14)?;
                total = soma(total, soma(8, externo_total)?)?;
                pc = p;
            }
            // addq.w #4,sp: o fim do trecho.
            0x584F => {
                return Some(Previsao {
                    ciclos: total,
                    fim: pc,
                });
            }
            _ => return None,
        }
    }
    None
}

/// `CheckSumCheck`: preâmbulo e laço de soma das palavras da ROM.
/// Soma `add.w (a0)+,d1` (8), `cmp.l a0,d0` (6) e `bcc.s` (10 se volta, 8 na última).
pub fn laco_da_soma(rom: &[u8], inicio: u32) -> Option<Previsao> {
    let mut pc = inicio;
    let mut total = 0u64;
    let mut a0 = 0u32;
    let mut fim_rom = 0u32;
    for _ in 0..16 {
        match palavra(rom, pc)? {
            // movea.l #imm,a0 / movea.l #endereco,a1: 12 ciclos.
            0x207C => {
                a0 = longa(rom, pc.checked_add(2)?)?;
                total = soma(total, 12)?;
                pc = pc.checked_add(6)?;
            }
            0x227C => {
                fim_rom = longa(rom, longa(rom, pc.checked_add(2)?)?)?;
                total = soma(total, 12)?;
                pc = pc.checked_add(6)?;
            }
            // move.l (a1),d0: 12. moveq #0,d1: 4.
            0x2011 => {
                total = soma(total, 12)?;
                pc = pc.checked_add(2)?;
            }
            0x7200 => {
                total = soma(total, 4)?;
                pc = pc.checked_add(2)?;
            }
            // add.w (a0)+,d1 ; cmp.l a0,d0 ; bhs.s volta
            0xD258 => {
                if palavra(rom, pc.checked_add(2)?)? != 0xB088 {
                    return None;
                }
                let b = palavra(rom, pc.checked_add(4)?)?;
                if b >> 8 != 0x64 {
                    return None;
                }
                let disp = i32::from(i8::from_be_bytes([b.to_be_bytes()[1]]));
                let alvo = i64::from(pc).checked_add(6)?.checked_add(i64::from(disp))?;
                if alvo != i64::from(pc) {
                    return None;
                }
                // Passadas: a primeira sempre; continua enquanto fim >= a0 depois de somar 2.
                let mut passadas = 1u64;
                let depois = u64::from(a0).checked_add(2)?;
                if u64::from(fim_rom) >= depois {
                    passadas =
                        passadas.checked_add((u64::from(fim_rom).checked_sub(depois)?) / 2)?;
                    passadas = passadas.checked_add(0)?;
                }
                let laco = passadas.checked_mul(8 + 6 + 10)?.checked_sub(2)?;
                total = soma(total, laco)?;
                return Some(Previsao {
                    ciclos: total,
                    fim: pc.checked_add(6)?,
                });
            }
            _ => return None,
        }
    }
    None
}

/// `GameInit`: limpeza da RAM com `move.l d7,(a6)+` e `dbf d6`.
pub fn laco_da_limpeza(rom: &[u8], inicio: u32) -> Option<Previsao> {
    let mut pc = inicio;
    let mut total = 0u64;
    let mut contagem = 0u64;
    for _ in 0..8 {
        match palavra(rom, pc)? {
            // lea (abs).l,a6: 12. moveq #0,d7: 4. move.w #imm,d6: 8.
            0x4DF9 => {
                total = soma(total, 12)?;
                pc = pc.checked_add(6)?;
            }
            0x7E00 => {
                total = soma(total, 4)?;
                pc = pc.checked_add(2)?;
            }
            0x3C3C => {
                contagem = u64::from(palavra(rom, pc.checked_add(2)?)?);
                total = soma(total, 8)?;
                pc = pc.checked_add(4)?;
            }
            0x2CC7 => {
                let inicio_laco = pc;
                if palavra(rom, pc.checked_add(2)?)? != 0x51CE
                    || alvo16(rom, pc.checked_add(4)?)? != inicio_laco
                {
                    return None;
                }
                // Cada volta: move.l d7,(a6)+ (12) + dbf (10 se volta, 14 na última).
                let voltas = contagem.checked_add(1)?;
                let laco = voltas
                    .checked_mul(12)?
                    .checked_add(contagem.checked_mul(10)?)?
                    .checked_add(14)?;
                total = soma(total, laco)?;
                return Some(Previsao {
                    ciclos: total,
                    fim: pc.checked_add(6)?,
                });
            }
            _ => return None,
        }
    }
    None
}
