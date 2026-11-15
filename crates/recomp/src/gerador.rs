//! Recompilação antecipada de blocos 68000 para Rust.
//!
//! A descoberta e a decodificação são feitas nesta ferramenta. O código produzido contém
//! apenas operações Rust e endereços de continuação; ele não lê instruções durante a execução.
//! Este primeiro subconjunto é deliberadamente restrito e rejeita instruções desconhecidas.

use crate::runtime;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Erro {
    EntradaInvalida(u32),
    InstrucaoIncompleta(u32),
    InstrucaoNaoSuportada(u32),
    AlvoForaDaRegiao(u32),
    Sobreposicao(u32),
    MuitasInstrucoes,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Instrucao {
    Moveq {
        reg: u8,
        valor: i8,
    },
    Nop,
    Rts,
    Rte,
    Move {
        tamanho: runtime::Tamanho,
        origem: runtime::Operando,
        destino: runtime::Operando,
        proximo: u32,
    },
    Chamada {
        alvo: u32,
        proximo: u32,
    },
    ChamadaIndireta {
        origem: runtime::Operando,
        proximo: u32,
    },
    Salto {
        alvo: u32,
    },
    SaltoIndireto {
        origem: runtime::Operando,
    },
    AritmeticaRapidaOp {
        subtrair: bool,
        tamanho: runtime::Tamanho,
        destino: runtime::Operando,
        valor: u8,
        proximo: u32,
    },
    Teste {
        tamanho: runtime::Tamanho,
        origem: runtime::Operando,
        proximo: u32,
    },
    Clr {
        tamanho: runtime::Tamanho,
        destino: runtime::Operando,
        proximo: u32,
    },
    Neg {
        tamanho: runtime::Tamanho,
        destino: runtime::Operando,
        proximo: u32,
    },
    Ext {
        tamanho: runtime::Tamanho,
        reg: u8,
        proximo: u32,
    },
    Lea {
        reg: u8,
        origem: runtime::Operando,
        proximo: u32,
    },
    Pea {
        origem: runtime::Operando,
        proximo: u32,
    },
    Movem {
        tamanho: runtime::Tamanho,
        memoria_para_regs: bool,
        operando: runtime::Operando,
        mascara: u16,
        proximo: u32,
    },
    Imediata {
        operacao: runtime::OperacaoImediata,
        tamanho: runtime::Tamanho,
        valor: u32,
        destino: runtime::Operando,
        proximo: u32,
    },
    Registrador {
        operacao: runtime::OperacaoRegistrador,
        tamanho: runtime::Tamanho,
        origem: runtime::Operando,
        reg: u8,
        proximo: u32,
    },
    OperacaoMemoria {
        operacao: runtime::OperacaoRegistrador,
        tamanho: runtime::Tamanho,
        reg_origem: u8,
        destino: runtime::Operando,
        proximo: u32,
    },
    Adda {
        tamanho: runtime::Tamanho,
        origem: runtime::Operando,
        reg: u8,
        proximo: u32,
    },
    Suba {
        tamanho: runtime::Tamanho,
        origem: runtime::Operando,
        reg: u8,
        proximo: u32,
    },
    Cmpa {
        tamanho: runtime::Tamanho,
        origem: runtime::Operando,
        reg: u8,
        proximo: u32,
    },
    DeslocamentoRegistrador {
        tipo: runtime::TipoDeslocamento,
        tamanho: runtime::Tamanho,
        reg: u8,
        contagem_reg: Option<u8>,
        contagem_imediata: u8,
        proximo: u32,
    },
    DeslocamentoMemoria {
        tipo: runtime::TipoDeslocamento,
        destino: runtime::Operando,
        proximo: u32,
    },
    Usp {
        reg: u8,
        para_usp: bool,
    },
    ParaSr {
        origem: runtime::Operando,
        proximo: u32,
    },
    TesteBit {
        origem: runtime::Operando,
        bit_reg: Option<u8>,
        bit_imediato: u8,
        proximo: u32,
    },
    DecrementoDesvio {
        condicao: u8,
        reg: u8,
        alvo: u32,
        proximo: u32,
    },
    Desvio {
        condicao: u8,
        alvo: u32,
        proximo: u32,
    },
}

fn operando(
    rom: &[u8],
    cursor: &mut u32,
    fim: u32,
    campo: (u16, u16),
    tamanho: runtime::Tamanho,
    destino: bool,
    pc: u32,
) -> Result<runtime::Operando, Erro> {
    use runtime::Operando as O;
    let (modo, reg) = campo;
    let reg = reg as u8;
    let pos_cursor = *cursor;
    let mut prox = || {
        let v = palavra(rom, *cursor, fim)?;
        *cursor += 2;
        Ok::<u16, Erro>(v)
    };
    Ok(match (modo, reg) {
        (0, r) => O::Dados(r),
        (1, r) if tamanho != runtime::Tamanho::Byte => O::Endereco(r),
        (2, r) => O::Indireto(r),
        (3, r) => O::PosIncremento(r),
        (4, r) => O::PreDecremento(r),
        (5, r) => O::Deslocamento(r, prox()? as i16),
        (7, 0) => O::Absoluto(i32::from(prox()? as i16) as u32),
        (7, 1) => O::Absoluto((u32::from(prox()?) << 16) | u32::from(prox()?)),
        (7, 2) if !destino => {
            let d = i32::from(prox()? as i16);
            O::Absoluto(pos_cursor.wrapping_add_signed(d))
        }
        (7, 4) if !destino => {
            let v = prox()?;
            O::Imediato(match tamanho {
                runtime::Tamanho::Byte => u32::from(v & 0xff),
                runtime::Tamanho::Word => u32::from(v),
                runtime::Tamanho::Long => (u32::from(v) << 16) | u32::from(prox()?),
            })
        }
        _ => return Err(Erro::InstrucaoNaoSuportada(pc)),
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Programa {
    instrucoes: BTreeMap<u32, Instrucao>,
    entrada: u32,
    saidas: BTreeSet<u32>,
}

fn palavra(rom: &[u8], pc: u32, fim: u32) -> Result<u16, Erro> {
    if pc & 1 != 0 || pc.checked_add(2).is_none_or(|v| v > fim) {
        return Err(Erro::InstrucaoIncompleta(pc));
    }
    let i = usize::try_from(pc).map_err(|_| Erro::InstrucaoIncompleta(pc))?;
    let bytes = rom.get(i..i + 2).ok_or(Erro::InstrucaoIncompleta(pc))?;
    Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
}

/// Descobre apenas instruções alcançáveis a partir da entrada, dentro de [inicio, fim).
/// Desvios para endereços desconhecidos ou instruções ainda não suportadas falham fechados.
pub fn descobrir(rom: &[u8], entrada: u32, inicio: u32, fim: u32) -> Result<Programa, Erro> {
    descobrir_impl(rom, entrada, inicio, fim, false)
}

/// Descobre um fragmento e registra saídas para endereços conhecidos fora da região.
pub fn descobrir_fragmento(
    rom: &[u8],
    entrada: u32,
    inicio: u32,
    fim: u32,
) -> Result<Programa, Erro> {
    descobrir_impl(rom, entrada, inicio, fim, true)
}

fn descobrir_impl(
    rom: &[u8],
    entrada: u32,
    inicio: u32,
    fim: u32,
    aceitar_saidas: bool,
) -> Result<Programa, Erro> {
    if inicio >= fim || entrada < inicio || entrada >= fim || entrada & 1 != 0 {
        return Err(Erro::EntradaInvalida(entrada));
    }
    let mut pendentes = BTreeSet::from([entrada]);
    let mut instrucoes = BTreeMap::new();
    let mut saidas = BTreeSet::new();
    let mut palavras_ocupadas = BTreeMap::new();
    while let Some(pc) = pendentes.pop_first() {
        if instrucoes.contains_key(&pc) {
            continue;
        }
        if (pc < inicio || pc >= fim) && aceitar_saidas && pc & 1 == 0 && (pc as usize) < rom.len()
        {
            saidas.insert(pc);
            continue;
        }
        if pc < inicio || pc >= fim || pc & 1 != 0 {
            return Err(Erro::AlvoForaDaRegiao(pc));
        }
        if palavras_ocupadas
            .get(&pc)
            .is_some_and(|origem| *origem != pc)
        {
            return Err(Erro::Sobreposicao(pc));
        }
        if instrucoes.len() >= 100_000 {
            return Err(Erro::MuitasInstrucoes);
        }
        let op = palavra(rom, pc, fim)?;
        let mut tamanho = 2u32;
        let instrucao = if op == 0x4e71 {
            Instrucao::Nop
        } else if op == 0x4e75 {
            Instrucao::Rts
        } else if op == 0x4e73 {
            Instrucao::Rte
        } else if op & 0xfff0 == 0x4e60 {
            Instrucao::Usp {
                reg: (op & 7) as u8,
                para_usp: op & 8 == 0,
            }
        } else if op & 0xffc0 == 0x46c0 {
            let mut cursor = pc + 2;
            let origem = operando(
                rom,
                &mut cursor,
                fim,
                ((op >> 3) & 7, op & 7),
                runtime::Tamanho::Word,
                false,
                pc,
            )?;
            if matches!(origem, runtime::Operando::Endereco(_)) {
                return Err(Erro::InstrucaoNaoSuportada(pc));
            }
            tamanho = cursor - pc;
            Instrucao::ParaSr {
                origem,
                proximo: cursor,
            }
        } else if op & 0xf100 == 0x7000 {
            Instrucao::Moveq {
                reg: ((op >> 9) & 7) as u8,
                valor: (op as u8) as i8,
            }
        } else if op & 0xf1c0 == 0x0100 || op & 0xffc0 == 0x0800 {
            let dinamico = op & 0xf1c0 == 0x0100;
            let mut cursor = pc + 2;
            let bit = if dinamico {
                0
            } else {
                let b = palavra(rom, cursor, fim)?;
                cursor += 2;
                b as u8
            };
            let origem = operando(
                rom,
                &mut cursor,
                fim,
                ((op >> 3) & 7, op & 7),
                runtime::Tamanho::Byte,
                false,
                pc,
            )?;
            if matches!(
                origem,
                runtime::Operando::Endereco(_) | runtime::Operando::Imediato(_)
            ) {
                return Err(Erro::InstrucaoNaoSuportada(pc));
            }
            tamanho = cursor - pc;
            Instrucao::TesteBit {
                origem,
                bit_reg: if dinamico {
                    Some(((op >> 9) & 7) as u8)
                } else {
                    None
                },
                bit_imediato: bit,
                proximo: cursor,
            }
        } else if matches!(
            op & 0xff00,
            0x0000 | 0x0200 | 0x0400 | 0x0600 | 0x0a00 | 0x0c00
        ) && (op >> 6) & 3 != 3
        {
            let t = match (op >> 6) & 3 {
                0 => runtime::Tamanho::Byte,
                1 => runtime::Tamanho::Word,
                _ => runtime::Tamanho::Long,
            };
            let operacao = match op & 0xff00 {
                0x0000 => runtime::OperacaoImediata::Ou,
                0x0200 => runtime::OperacaoImediata::E,
                0x0400 => runtime::OperacaoImediata::Sub,
                0x0600 => runtime::OperacaoImediata::Add,
                0x0a00 => runtime::OperacaoImediata::Xor,
                _ => runtime::OperacaoImediata::Cmp,
            };
            let modo = (op >> 3) & 7;
            if modo == 1 || (modo == 7 && op & 7 >= 2) {
                return Err(Erro::InstrucaoNaoSuportada(pc));
            }
            let mut cursor = pc + 2;
            let runtime::Operando::Imediato(valor) =
                operando(rom, &mut cursor, fim, (7, 4), t, false, pc)?
            else {
                unreachable!()
            };
            let destino = operando(rom, &mut cursor, fim, (modo, op & 7), t, true, pc)?;
            tamanho = cursor - pc;
            Instrucao::Imediata {
                operacao,
                tamanho: t,
                valor,
                destino,
                proximo: cursor,
            }
        } else if op & 0xff00 == 0x4a00 && (op >> 6) & 3 != 3 {
            if (op >> 3) & 7 == 7 && op & 7 >= 2 {
                return Err(Erro::InstrucaoNaoSuportada(pc));
            }
            let t = match (op >> 6) & 3 {
                0 => runtime::Tamanho::Byte,
                1 => runtime::Tamanho::Word,
                _ => runtime::Tamanho::Long,
            };
            let mut cursor = pc + 2;
            let origem = operando(rom, &mut cursor, fim, ((op >> 3) & 7, op & 7), t, false, pc)?;
            if matches!(
                origem,
                runtime::Operando::Endereco(_) | runtime::Operando::Imediato(_)
            ) {
                return Err(Erro::InstrucaoNaoSuportada(pc));
            }
            tamanho = cursor - pc;
            Instrucao::Teste {
                tamanho: t,
                origem,
                proximo: cursor,
            }
        } else if op & 0xff00 == 0x4200 && (op >> 6) & 3 != 3 {
            if (op >> 3) & 7 == 1 || ((op >> 3) & 7 == 7 && op & 7 >= 2) {
                return Err(Erro::InstrucaoNaoSuportada(pc));
            }
            let t = match (op >> 6) & 3 {
                0 => runtime::Tamanho::Byte,
                1 => runtime::Tamanho::Word,
                _ => runtime::Tamanho::Long,
            };
            let mut cursor = pc + 2;
            let destino = operando(rom, &mut cursor, fim, ((op >> 3) & 7, op & 7), t, true, pc)?;
            tamanho = cursor - pc;
            Instrucao::Clr {
                tamanho: t,
                destino,
                proximo: cursor,
            }
        } else if op & 0xff00 == 0x4400 && (op >> 6) & 3 != 3 {
            if (op >> 3) & 7 == 1 || ((op >> 3) & 7 == 7 && op & 7 >= 2) {
                return Err(Erro::InstrucaoNaoSuportada(pc));
            }
            let t = match (op >> 6) & 3 {
                0 => runtime::Tamanho::Byte,
                1 => runtime::Tamanho::Word,
                _ => runtime::Tamanho::Long,
            };
            let mut cursor = pc + 2;
            let destino = operando(rom, &mut cursor, fim, ((op >> 3) & 7, op & 7), t, true, pc)?;
            tamanho = cursor - pc;
            Instrucao::Neg {
                tamanho: t,
                destino,
                proximo: cursor,
            }
        } else if op & 0xffc0 == 0x4840 {
            let mut cursor = pc + 2;
            let origem = operando(
                rom,
                &mut cursor,
                fim,
                ((op >> 3) & 7, op & 7),
                runtime::Tamanho::Long,
                false,
                pc,
            )?;
            if !matches!(
                origem,
                runtime::Operando::Indireto(_)
                    | runtime::Operando::Deslocamento(_, _)
                    | runtime::Operando::Absoluto(_)
            ) {
                return Err(Erro::InstrucaoNaoSuportada(pc));
            }
            tamanho = cursor - pc;
            Instrucao::Pea {
                origem,
                proximo: cursor,
            }
        } else if op & 0xffb8 == 0x4880 {
            let t = if op & 0x0040 != 0 {
                runtime::Tamanho::Long
            } else {
                runtime::Tamanho::Word
            };
            Instrucao::Ext {
                tamanho: t,
                reg: (op & 7) as u8,
                proximo: pc + 2,
            }
        } else if op & 0xfb80 == 0x4880 {
            let memoria_para_regs = op & 0x0400 != 0;
            let t = if op & 0x0040 != 0 {
                runtime::Tamanho::Long
            } else {
                runtime::Tamanho::Word
            };
            let mascara = palavra(rom, pc + 2, fim)?;
            let mut cursor = pc + 4;
            let operando = operando(
                rom,
                &mut cursor,
                fim,
                ((op >> 3) & 7, op & 7),
                t,
                !memoria_para_regs,
                pc,
            )?;
            let valido = match operando {
                runtime::Operando::Indireto(_)
                | runtime::Operando::Deslocamento(_, _)
                | runtime::Operando::Absoluto(_) => true,
                runtime::Operando::PosIncremento(_) => memoria_para_regs,
                runtime::Operando::PreDecremento(_) => !memoria_para_regs,
                _ => false,
            };
            if !valido {
                return Err(Erro::InstrucaoNaoSuportada(pc));
            }
            tamanho = cursor - pc;
            Instrucao::Movem {
                tamanho: t,
                memoria_para_regs,
                operando,
                mascara,
                proximo: cursor,
            }
        } else if op & 0xf1c0 == 0x41c0 {
            let mut cursor = pc + 2;
            let origem = operando(
                rom,
                &mut cursor,
                fim,
                ((op >> 3) & 7, op & 7),
                runtime::Tamanho::Long,
                false,
                pc,
            )?;
            if !matches!(
                origem,
                runtime::Operando::Indireto(_)
                    | runtime::Operando::Deslocamento(_, _)
                    | runtime::Operando::Absoluto(_)
            ) {
                return Err(Erro::InstrucaoNaoSuportada(pc));
            }
            tamanho = cursor - pc;
            Instrucao::Lea {
                reg: ((op >> 9) & 7) as u8,
                origem,
                proximo: cursor,
            }
        } else if op & 0xffc0 == 0x4e80 {
            let mut cursor = pc + 2;
            let origem = operando(
                rom,
                &mut cursor,
                fim,
                ((op >> 3) & 7, op & 7),
                runtime::Tamanho::Long,
                false,
                pc,
            )?;
            tamanho = cursor - pc;
            match origem {
                runtime::Operando::Absoluto(alvo) => Instrucao::Chamada {
                    alvo,
                    proximo: cursor,
                },
                runtime::Operando::Indireto(_) | runtime::Operando::Deslocamento(_, _) => {
                    Instrucao::ChamadaIndireta {
                        origem,
                        proximo: cursor,
                    }
                }
                _ => return Err(Erro::InstrucaoNaoSuportada(pc)),
            }
        } else if op & 0xffc0 == 0x4ec0 {
            let mut cursor = pc + 2;
            let origem = operando(
                rom,
                &mut cursor,
                fim,
                ((op >> 3) & 7, op & 7),
                runtime::Tamanho::Long,
                false,
                pc,
            )?;
            tamanho = cursor - pc;
            match origem {
                runtime::Operando::Absoluto(alvo) => Instrucao::Salto { alvo },
                runtime::Operando::Indireto(_) | runtime::Operando::Deslocamento(_, _) => {
                    Instrucao::SaltoIndireto { origem }
                }
                _ => return Err(Erro::InstrucaoNaoSuportada(pc)),
            }
        } else if matches!(op >> 12, 8 | 9 | 11 | 12 | 13) {
            let op_campo = (op >> 6) & 7;
            if op_campo <= 2 {
                let t = match op_campo {
                    0 => runtime::Tamanho::Byte,
                    1 => runtime::Tamanho::Word,
                    _ => runtime::Tamanho::Long,
                };
                let operacao = match op >> 12 {
                    8 => runtime::OperacaoRegistrador::Ou,
                    9 => runtime::OperacaoRegistrador::Sub,
                    11 => runtime::OperacaoRegistrador::Cmp,
                    12 => runtime::OperacaoRegistrador::E,
                    _ => runtime::OperacaoRegistrador::Add,
                };
                let mut cursor = pc + 2;
                let origem =
                    operando(rom, &mut cursor, fim, ((op >> 3) & 7, op & 7), t, false, pc)?;
                tamanho = cursor - pc;
                Instrucao::Registrador {
                    operacao,
                    tamanho: t,
                    origem,
                    reg: ((op >> 9) & 7) as u8,
                    proximo: cursor,
                }
            } else if matches!(op_campo, 3 | 7) && matches!(op >> 12, 9 | 11 | 13) {
                let t = if op_campo == 3 {
                    runtime::Tamanho::Word
                } else {
                    runtime::Tamanho::Long
                };
                let reg = ((op >> 9) & 7) as u8;
                let mut cursor = pc + 2;
                let origem =
                    operando(rom, &mut cursor, fim, ((op >> 3) & 7, op & 7), t, false, pc)?;
                tamanho = cursor - pc;
                match op >> 12 {
                    9 => Instrucao::Suba {
                        tamanho: t,
                        origem,
                        reg,
                        proximo: cursor,
                    },
                    11 => Instrucao::Cmpa {
                        tamanho: t,
                        origem,
                        reg,
                        proximo: cursor,
                    },
                    _ => Instrucao::Adda {
                        tamanho: t,
                        origem,
                        reg,
                        proximo: cursor,
                    },
                }
            } else if matches!(op_campo, 4..=6) {
                let t = match op_campo {
                    4 => runtime::Tamanho::Byte,
                    5 => runtime::Tamanho::Word,
                    _ => runtime::Tamanho::Long,
                };
                let modo = (op >> 3) & 7;
                let valido = if op >> 12 == 11 {
                    modo != 1 && !(modo == 7 && op & 7 >= 2)
                } else {
                    modo >= 2 && !(modo == 7 && op & 7 >= 2)
                };
                if !valido {
                    return Err(Erro::InstrucaoNaoSuportada(pc));
                }
                let operacao = match op >> 12 {
                    8 => runtime::OperacaoRegistrador::Ou,
                    9 => runtime::OperacaoRegistrador::Sub,
                    11 => runtime::OperacaoRegistrador::Xor,
                    12 => runtime::OperacaoRegistrador::E,
                    _ => runtime::OperacaoRegistrador::Add,
                };
                let reg_origem = ((op >> 9) & 7) as u8;
                let mut cursor = pc + 2;
                let destino = operando(rom, &mut cursor, fim, (modo, op & 7), t, true, pc)?;
                tamanho = cursor - pc;
                Instrucao::OperacaoMemoria {
                    operacao,
                    tamanho: t,
                    reg_origem,
                    destino,
                    proximo: cursor,
                }
            } else {
                return Err(Erro::InstrucaoNaoSuportada(pc));
            }
        } else if matches!(op >> 12, 1..=3) {
            let t = match op >> 12 {
                1 => runtime::Tamanho::Byte,
                2 => runtime::Tamanho::Long,
                _ => runtime::Tamanho::Word,
            };
            let mut cursor = pc + 2;
            let origem = operando(rom, &mut cursor, fim, ((op >> 3) & 7, op & 7), t, false, pc)?;
            let destino = operando(
                rom,
                &mut cursor,
                fim,
                ((op >> 6) & 7, (op >> 9) & 7),
                t,
                true,
                pc,
            )?;
            tamanho = cursor - pc;
            Instrucao::Move {
                tamanho: t,
                origem,
                destino,
                proximo: cursor,
            }
        } else if op & 0xf000 == 0xe000 {
            let esquerda = (op & 0x0100) != 0;
            if (op >> 6) & 3 == 3 {
                if op & 0x0800 != 0 {
                    return Err(Erro::InstrucaoNaoSuportada(pc));
                }
                let tipo = match (op >> 9) & 3 {
                    0 => {
                        if esquerda {
                            runtime::TipoDeslocamento::Asl
                        } else {
                            runtime::TipoDeslocamento::Asr
                        }
                    }
                    1 => {
                        if esquerda {
                            runtime::TipoDeslocamento::Lsl
                        } else {
                            runtime::TipoDeslocamento::Lsr
                        }
                    }
                    2 => {
                        if esquerda {
                            runtime::TipoDeslocamento::Roxl
                        } else {
                            runtime::TipoDeslocamento::Roxr
                        }
                    }
                    _ => {
                        if esquerda {
                            runtime::TipoDeslocamento::Rol
                        } else {
                            runtime::TipoDeslocamento::Ror
                        }
                    }
                };
                let modo = (op >> 3) & 7;
                if modo <= 1 || (modo == 7 && op & 7 >= 2) {
                    return Err(Erro::InstrucaoNaoSuportada(pc));
                }
                let mut cursor = pc + 2;
                let destino = operando(
                    rom,
                    &mut cursor,
                    fim,
                    (modo, op & 7),
                    runtime::Tamanho::Word,
                    true,
                    pc,
                )?;
                tamanho = cursor - pc;
                Instrucao::DeslocamentoMemoria {
                    tipo,
                    destino,
                    proximo: cursor,
                }
            } else {
                let t = match (op >> 6) & 3 {
                    0 => runtime::Tamanho::Byte,
                    1 => runtime::Tamanho::Word,
                    _ => runtime::Tamanho::Long,
                };
                let tipo = match (op >> 3) & 3 {
                    0 => {
                        if esquerda {
                            runtime::TipoDeslocamento::Asl
                        } else {
                            runtime::TipoDeslocamento::Asr
                        }
                    }
                    1 => {
                        if esquerda {
                            runtime::TipoDeslocamento::Lsl
                        } else {
                            runtime::TipoDeslocamento::Lsr
                        }
                    }
                    2 => {
                        if esquerda {
                            runtime::TipoDeslocamento::Roxl
                        } else {
                            runtime::TipoDeslocamento::Roxr
                        }
                    }
                    _ => {
                        if esquerda {
                            runtime::TipoDeslocamento::Rol
                        } else {
                            runtime::TipoDeslocamento::Ror
                        }
                    }
                };
                let reg_contagem = (op & 0x0020) != 0;
                let n = ((op >> 9) & 7) as u8;
                let (contagem_reg, contagem_imediata) = if reg_contagem {
                    (Some(n), 0)
                } else {
                    (None, if n == 0 { 8 } else { n })
                };
                Instrucao::DeslocamentoRegistrador {
                    tipo,
                    tamanho: t,
                    reg: (op & 7) as u8,
                    contagem_reg,
                    contagem_imediata,
                    proximo: pc + 2,
                }
            }
        } else if op & 0xf0f8 == 0x50c8 {
            tamanho = 4;
            let desloc = i32::from(palavra(rom, pc + 2, fim)? as i16);
            let alvo = (pc + 2).wrapping_add_signed(desloc);
            Instrucao::DecrementoDesvio {
                condicao: ((op >> 8) & 15) as u8,
                reg: (op & 7) as u8,
                alvo,
                proximo: pc + tamanho,
            }
        } else if op & 0xf000 == 0x5000 && (op >> 6) & 3 != 3 {
            let modo = (op >> 3) & 7;
            if modo == 7 && op & 7 >= 2 {
                return Err(Erro::InstrucaoNaoSuportada(pc));
            }
            let t = match (op >> 6) & 3 {
                0 => runtime::Tamanho::Byte,
                1 => runtime::Tamanho::Word,
                _ => runtime::Tamanho::Long,
            };
            if modo == 1 && t == runtime::Tamanho::Byte {
                return Err(Erro::InstrucaoNaoSuportada(pc));
            }
            let mut cursor = pc + 2;
            let destino = operando(rom, &mut cursor, fim, (modo, op & 7), t, true, pc)?;
            tamanho = cursor - pc;
            let n = ((op >> 9) & 7) as u8;
            Instrucao::AritmeticaRapidaOp {
                subtrair: op & 0x100 != 0,
                tamanho: t,
                destino,
                valor: if n == 0 { 8 } else { n },
                proximo: cursor,
            }
        } else if op & 0xf000 == 0x6000 {
            let condicao = ((op >> 8) & 15) as u8;
            let desloc = if op & 0xff == 0 {
                tamanho = 4;
                i32::from(palavra(rom, pc + 2, fim)? as i16)
            } else {
                i32::from((op as u8) as i8)
            };
            let alvo_i = i64::from(pc) + 2 + i64::from(desloc);
            let alvo = u32::try_from(alvo_i).map_err(|_| Erro::AlvoForaDaRegiao(pc))?;
            if condicao == 1 {
                Instrucao::Chamada {
                    alvo,
                    proximo: pc + tamanho,
                }
            } else {
                Instrucao::Desvio {
                    condicao,
                    alvo,
                    proximo: pc + tamanho,
                }
            }
        } else {
            return Err(Erro::InstrucaoNaoSuportada(pc));
        };
        for endereco in (pc..pc + tamanho).step_by(2) {
            if palavras_ocupadas
                .insert(endereco, pc)
                .is_some_and(|origem| origem != pc)
            {
                return Err(Erro::Sobreposicao(endereco));
            }
        }
        match instrucao {
            Instrucao::Rts | Instrucao::Rte => {}
            Instrucao::Salto { alvo } => {
                pendentes.insert(alvo);
            }
            Instrucao::SaltoIndireto { .. } => {}
            Instrucao::Desvio {
                condicao,
                alvo,
                proximo,
            } => {
                pendentes.insert(alvo);
                if condicao != 0 {
                    pendentes.insert(proximo);
                }
            }
            Instrucao::Chamada { alvo, proximo } => {
                pendentes.insert(alvo);
                pendentes.insert(proximo);
            }
            Instrucao::ChamadaIndireta { proximo, .. } => {
                pendentes.insert(proximo);
            }
            Instrucao::DecrementoDesvio { alvo, proximo, .. } => {
                pendentes.insert(alvo);
                pendentes.insert(proximo);
            }
            _ => {
                pendentes.insert(pc + tamanho);
            }
        }
        instrucoes.insert(pc, instrucao);
    }
    Ok(Programa {
        instrucoes,
        entrada,
        saidas,
    })
}

impl Programa {
    /// Endereços descobertos, em ordem determinística.
    pub fn enderecos(&self) -> impl Iterator<Item = u32> + '_ {
        self.instrucoes.keys().copied()
    }

    pub fn saidas(&self) -> impl Iterator<Item = u32> + '_ {
        self.saidas.iter().copied()
    }

    /// Emite código Rust independente, sem interpretar os bytes originais em execução.
    pub fn gerar_rust(&self) -> String {
        let mut out = String::from(
            "// Gerado por sonic1-recomp. Não editar.\n#[allow(unused_imports)]\nuse sonic1_recomp::runtime::{Estado, Parada, Barramento, ErroExecucao, Operando, Tamanho, OperacaoImediata, OperacaoRegistrador, TipoDeslocamento};\n\n#[allow(unused_variables)]\npub fn passo(e: &mut Estado, mem: &mut impl Barramento) -> Result<Parada, ErroExecucao> {\n    Ok(match e.pc {\n",
        );
        for (&pc, i) in &self.instrucoes {
            let corpo = match i {
                Instrucao::Nop => format!("e.pc = {}; Parada::Continua", pc + 2),
                Instrucao::Rts => "e.retornar(mem)?; Parada::Retorno".to_string(),
                Instrucao::Rte => {
                    "e.retornar_interrupcao(mem)?; Parada::RetornoInterrupcao".to_string()
                }
                Instrucao::Moveq { reg, valor } => format!(
                    "e.moveq({reg}, {valor}); e.pc = {}; Parada::Continua",
                    pc + 2
                ),
                Instrucao::Salto { alvo } => format!("e.pc = {alvo}; Parada::Continua"),
                Instrucao::SaltoIndireto { origem } => format!(
                    "let (alvo, _) = e.endereco(Operando::{origem:?}, Tamanho::Long)?; e.pc = alvo; Parada::Continua"
                ),
                Instrucao::Desvio {
                    condicao,
                    alvo,
                    proximo,
                } => format!(
                    "let tomado = e.condicao({condicao}); e.registrar_ciclos_base(Some(if tomado {{ 10 }} else {{ {} }})); e.pc = if tomado {{ {alvo} }} else {{ {proximo} }}; Parada::Continua",
                    if proximo - pc == 2 { 8 } else { 12 }
                ),
                Instrucao::Move {
                    tamanho,
                    origem,
                    destino,
                    proximo,
                } => format!(
                    "e.mov(mem, Tamanho::{tamanho:?}, Operando::{origem:?}, Operando::{destino:?})?; e.pc = {proximo}; Parada::Continua"
                ),
                Instrucao::Chamada { alvo, proximo } => {
                    format!("e.chamar(mem, {proximo})?; e.pc = {alvo}; Parada::Continua")
                }
                Instrucao::ChamadaIndireta { origem, proximo } => {
                    format!(
                        "e.chamar_indireto(mem, Operando::{origem:?}, {proximo})?; Parada::Continua"
                    )
                }
                Instrucao::AritmeticaRapidaOp {
                    subtrair,
                    tamanho,
                    destino,
                    valor,
                    proximo,
                } => format!(
                    "e.aritmetica_rapida_op(mem, {subtrair}, Tamanho::{tamanho:?}, Operando::{destino:?}, {valor})?; e.pc = {proximo}; Parada::Continua"
                ),
                Instrucao::Teste {
                    tamanho,
                    origem,
                    proximo,
                } => format!(
                    "e.testar(mem, Tamanho::{tamanho:?}, Operando::{origem:?})?; e.pc = {proximo}; Parada::Continua"
                ),
                Instrucao::Clr {
                    tamanho,
                    destino,
                    proximo,
                } => format!(
                    "e.clr(mem, Tamanho::{tamanho:?}, Operando::{destino:?})?; e.pc = {proximo}; Parada::Continua"
                ),
                Instrucao::Neg {
                    tamanho,
                    destino,
                    proximo,
                } => format!(
                    "e.neg(mem, Tamanho::{tamanho:?}, Operando::{destino:?})?; e.pc = {proximo}; Parada::Continua"
                ),
                Instrucao::Ext {
                    tamanho,
                    reg,
                    proximo,
                } => format!(
                    "e.ext(Tamanho::{tamanho:?}, {reg}); e.pc = {proximo}; Parada::Continua"
                ),
                Instrucao::Lea {
                    reg,
                    origem,
                    proximo,
                } => format!(
                    "e.lea({reg}, Operando::{origem:?})?; e.pc = {proximo}; Parada::Continua"
                ),
                Instrucao::Pea { origem, proximo } => {
                    format!("e.pea(mem, Operando::{origem:?})?; e.pc = {proximo}; Parada::Continua")
                }
                Instrucao::DecrementoDesvio {
                    condicao,
                    reg,
                    alvo,
                    proximo,
                } => format!(
                    "let cumprida = e.condicao({condicao}); let tomado = e.decremento_desvio({condicao}, {reg}); e.registrar_ciclos_base(Some(if cumprida {{ 12 }} else if tomado {{ 10 }} else {{ 14 }})); e.pc = if tomado {{ {alvo} }} else {{ {proximo} }}; Parada::Continua"
                ),
                Instrucao::Movem {
                    tamanho,
                    memoria_para_regs,
                    operando,
                    mascara,
                    proximo,
                } => format!(
                    "e.movem(mem, Tamanho::{tamanho:?}, {memoria_para_regs}, Operando::{operando:?}, {mascara})?; e.pc = {proximo}; Parada::Continua"
                ),
                Instrucao::Imediata {
                    operacao,
                    tamanho,
                    valor,
                    destino,
                    proximo,
                } => format!(
                    "e.imediata(mem, OperacaoImediata::{operacao:?}, Tamanho::{tamanho:?}, {valor}, Operando::{destino:?})?; e.pc = {proximo}; Parada::Continua"
                ),
                Instrucao::Registrador {
                    operacao,
                    tamanho,
                    origem,
                    reg,
                    proximo,
                } => format!(
                    "e.registrador(mem, OperacaoRegistrador::{operacao:?}, Tamanho::{tamanho:?}, Operando::{origem:?}, {reg})?; e.pc = {proximo}; Parada::Continua"
                ),
                Instrucao::OperacaoMemoria {
                    operacao,
                    tamanho,
                    reg_origem,
                    destino,
                    proximo,
                } => format!(
                    "e.operacao_para_memoria(mem, OperacaoRegistrador::{operacao:?}, Tamanho::{tamanho:?}, {reg_origem}, Operando::{destino:?})?; e.pc = {proximo}; Parada::Continua"
                ),
                Instrucao::Adda {
                    tamanho,
                    origem,
                    reg,
                    proximo,
                } => format!(
                    "e.adda(mem, Tamanho::{tamanho:?}, Operando::{origem:?}, {reg})?; e.pc = {proximo}; Parada::Continua"
                ),
                Instrucao::Suba {
                    tamanho,
                    origem,
                    reg,
                    proximo,
                } => format!(
                    "e.suba(mem, Tamanho::{tamanho:?}, Operando::{origem:?}, {reg})?; e.pc = {proximo}; Parada::Continua"
                ),
                Instrucao::Cmpa {
                    tamanho,
                    origem,
                    reg,
                    proximo,
                } => format!(
                    "e.cmpa(mem, Tamanho::{tamanho:?}, Operando::{origem:?}, {reg})?; e.pc = {proximo}; Parada::Continua"
                ),
                Instrucao::DeslocamentoRegistrador {
                    tipo,
                    tamanho,
                    reg,
                    contagem_reg,
                    contagem_imediata,
                    proximo,
                } => format!(
                    "e.deslocar_registrador(TipoDeslocamento::{tipo:?}, Tamanho::{tamanho:?}, {reg}, {contagem_reg:?}, {contagem_imediata}); e.pc = {proximo}; Parada::Continua"
                ),
                Instrucao::DeslocamentoMemoria {
                    tipo,
                    destino,
                    proximo,
                } => format!(
                    "e.deslocar_memoria(mem, TipoDeslocamento::{tipo:?}, Operando::{destino:?})?; e.pc = {proximo}; Parada::Continua"
                ),
                Instrucao::Usp { reg, para_usp } => format!(
                    "e.mover_usp({reg}, {para_usp})?; e.pc = {}; Parada::Continua",
                    pc + 2
                ),
                Instrucao::ParaSr { origem, proximo } => format!(
                    "e.mover_para_sr(mem, Operando::{origem:?})?; e.pc = {proximo}; Parada::Continua"
                ),
                Instrucao::TesteBit {
                    origem,
                    bit_reg,
                    bit_imediato,
                    proximo,
                } => format!(
                    "e.testar_bit(mem, Operando::{origem:?}, {bit_reg:?}, {bit_imediato})?; e.pc = {proximo}; Parada::Continua"
                ),
            };
            let custo = match i {
                Instrucao::Nop | Instrucao::Moveq { .. } | Instrucao::Usp { .. } => Some(4),
                Instrucao::Rts => Some(16),
                Instrucao::Rte => Some(20),
                Instrucao::Chamada { proximo, .. } => Some(if proximo - pc == 6 { 20 } else { 18 }),
                Instrucao::AritmeticaRapidaOp {
                    tamanho, destino, ..
                } => match destino {
                    runtime::Operando::Endereco(_) => Some(8),
                    runtime::Operando::Dados(_) => Some(if *tamanho == runtime::Tamanho::Long {
                        8
                    } else {
                        4
                    }),
                    _ => None,
                },
                Instrucao::Ext { .. } => Some(4),
                _ => None,
            };
            let corpo = if matches!(
                i,
                Instrucao::Desvio { .. }
                    | Instrucao::DecrementoDesvio { .. }
                    | Instrucao::Salto { .. }
                    | Instrucao::SaltoIndireto { .. }
            ) {
                corpo
            } else {
                let (operacao, parada) = corpo
                    .rsplit_once("; Parada::")
                    .expect("operação gerada tem parada");
                format!("{operacao}; e.registrar_ciclos_base({custo:?}); Parada::{parada}")
            };
            writeln!(out, "        {pc} => {{ {corpo} }},").expect("String aceita escrita");
        }
        out.push_str("        _ => Parada::EnderecoNaoGerado(e.pc),\n    })\n}\n");
        out
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use runtime::Estado;

    #[test]
    fn moveq_preserva_x_e_atualiza_flags() {
        let mut e = Estado {
            ccr: 0x1f,
            ..Estado::default()
        };
        e.moveq(3, -1);
        assert_eq!(e.d[3], u32::MAX);
        assert_eq!(e.ccr, 0x18);
        e.moveq(3, 0);
        assert_eq!(e.d[3], 0);
        assert_eq!(e.ccr, 0x14);
        e.moveq(3, 42);
        assert_eq!(e.ccr, 0x10);
    }

    #[test]
    fn condicoes_sao_complementares_para_todo_ccr() {
        for ccr in 0..32 {
            let e = Estado {
                ccr,
                ..Estado::default()
            };
            assert!(e.condicao(0));
            for par in (2..=14).step_by(2) {
                assert_ne!(e.condicao(par), e.condicao(par + 1));
            }
        }
    }

    #[test]
    fn descobre_os_dois_caminhos_sem_seguir_dados_apos_rts() {
        let rom = [
            0x70, 0x00, 0x67, 0x04, 0x72, 0x01, 0x4e, 0x75, 0x72, 0xff, 0x4e, 0x75, 0xff, 0xff,
        ];
        let p = descobrir(&rom, 0, 0, rom.len() as u32).unwrap();
        assert_eq!(p.enderecos().collect::<Vec<_>>(), vec![0, 2, 4, 6, 8, 10]);
        assert_eq!(
            p.gerar_rust(),
            descobrir(&rom, 0, 0, rom.len() as u32)
                .unwrap()
                .gerar_rust()
        );
    }

    #[test]
    fn recusas_nao_geram_codigo_parcial() {
        assert_eq!(
            descobrir(&[0xff, 0xff], 0, 0, 2),
            Err(Erro::InstrucaoNaoSuportada(0))
        );
        assert_eq!(
            descobrir(&[0x60, 0x00, 0x00, 0x00], 0, 0, 4),
            Err(Erro::Sobreposicao(2))
        );
        assert_eq!(
            descobrir(&[0x60, 0x04], 0, 0, 2),
            Err(Erro::AlvoForaDaRegiao(6))
        );
        assert_eq!(
            descobrir(&[0x60, 0x00], 0, 0, 2),
            Err(Erro::InstrucaoIncompleta(2))
        );
    }

    #[test]
    fn descobre_aritmetica_e_logica() {
        let bytes = [
            0x42, 0x00, // clr.b d0
            0x42, 0x50, // clr.w (a0)
            0x44, 0x41, // neg.w d1
            0x44, 0x90, // neg.l (a0)
            0x48, 0x82, // ext.w d2
            0x48, 0xc2, // ext.l d2
            0xd2, 0x01, // add.b d1, d1
            0xd3, 0x50, // add.w d1, (a0)
            0xd0, 0xfc, 0x00, 0x10, // adda.w #$10, a0
            0x92, 0x01, // sub.b d1, d1
            0x93, 0x50, // sub.w d1, (a0)
            0x90, 0xfc, 0x00, 0x00, 0x00, 0x20, // suba.l #$20, a0
            0xb2, 0x41, // cmp.w d1, d1
            0xb0, 0xfc, 0x00, 0x00, 0x10, 0x00, // cmpa.l #$1000, a0
            0x4a, 0x00, // tst.b d0
            0x4a, 0x50, // tst.w (a0)
            0xc2, 0x01, // and.b d1, d1
            0xc3, 0x50, // and.w d1, (a0)
            0x82, 0x01, // or.b d1, d1
            0x83, 0x50, // or.w d1, (a0)
            0xb3, 0x50, // eor.w d1, (a0)
            0xb1, 0x02, // eor.b d0, d2
            0x52, 0x50, // addq.w #1, (a0)
            0x53, 0x50, // subq.w #1, (a0)
            0x4e, 0x75, // rts
        ];
        let p = descobrir(&bytes, 0, 0, bytes.len() as u32).unwrap();
        assert_eq!(p.enderecos().count(), 26);
        let codigo = p.gerar_rust();
        assert!(codigo.contains("e.clr"));
        assert!(codigo.contains("e.neg"));
        assert!(codigo.contains("e.ext"));
        assert!(codigo.contains("e.adda"));
        assert!(codigo.contains("e.suba"));
        assert!(codigo.contains("e.cmpa"));
        assert!(codigo.contains("OperacaoRegistrador::Xor"));
        assert!(codigo.contains("e.aritmetica_rapida_op"));
    }

    #[test]
    fn descobre_deslocamentos_e_rotacoes() {
        let bytes = [
            0xe1, 0x48, // lsl.w #8, d0
            0xe2, 0x48, // lsr.w #1, d0
            0xe3, 0x81, // asl.l #1, d1
            0xe4, 0x01, // asr.b #2, d1
            0xe7, 0x58, // rol.w #3, d0
            0xe6, 0x58, // ror.w #3, d0
            0xe3, 0x70, // rol.w d1, d0
            0xe3, 0xd0, // rol.w (a0)
            0xe1, 0xd0, // asl.w (a0)
            0x4e, 0x75, // rts
        ];
        let p = descobrir(&bytes, 0, 0, bytes.len() as u32).unwrap();
        assert_eq!(p.enderecos().count(), 10);
        let codigo = p.gerar_rust();
        assert!(codigo.contains("TipoDeslocamento::Lsl"));
        assert!(codigo.contains("TipoDeslocamento::Lsr"));
        assert!(codigo.contains("TipoDeslocamento::Asl"));
        assert!(codigo.contains("TipoDeslocamento::Asr"));
        assert!(codigo.contains("TipoDeslocamento::Rol"));
        assert!(codigo.contains("TipoDeslocamento::Ror"));
        assert!(codigo.contains("e.deslocar_memoria"));
    }

    #[test]
    fn descobre_pea_lea_e_saltos() {
        let mut rom = vec![0; 0x2000];
        let bytes = [
            0x41, 0xe8, 0x00, 0x04, // lea 4(a0), a0
            0x48, 0x68, 0x00, 0x08, // pea 8(a0)
            0x4e, 0x90, // jsr (a0)
            0x4e, 0xb8, 0x10, 0x00, // jsr ($1000).w
            0x4e, 0xd0, // jmp (a0)
        ];
        rom[..bytes.len()].copy_from_slice(&bytes);
        let p = descobrir_fragmento(&rom, 0, 0, bytes.len() as u32).unwrap();
        assert_eq!(p.enderecos().count(), 5);
        assert!(p.saidas().any(|s| s == 0x1000));
        let codigo = p.gerar_rust();
        assert!(codigo.contains("e.lea"));
        assert!(codigo.contains("e.pea"));
        assert!(codigo.contains("e.chamar_indireto"));
        assert!(codigo.contains("e.chamar(mem, 14)"));
    }
}
