//! Formatos dos arquivos de video da Etapa 4. Nenhum deles entra no git: sao derivados da ROM.
//!
//! - `.s1i`: por quadro, o tamanho da imagem e o SHA-1 dos pontos (cada ponto em 2 bytes,
//!   little-endian, cor de 9 bits). Um registro por quadro fechado.
//! - `.s1v`: as palavras que o DMA de memoria para VDP leu em cada quadro, na ordem de leitura.
//! - `.s1p`: um quadro inteiro (todos os pontos), regenerado sob demanda para achar o primeiro
//!   ponto diferente.
//!
//! Tudo little-endian, tamanho fixo por campo, sem relogio nem ordem de hash.
#![forbid(unsafe_code)]

use std::io::{self, Read, Write};

pub const MAGIA_IMAGEM: [u8; 8] = *b"S1IMAGEM";
pub const MAGIA_VIDEO: [u8; 8] = *b"S1VIDEO\0";
pub const MAGIA_PIXELS: [u8; 8] = *b"S1PIXEL\0";

/// Limite de pontos de um quadro aceito na leitura (protecao contra arquivo corrompido).
pub const MAX_PONTOS: usize = 1024 * 1024;
/// Limite de palavras de DMA por quadro na leitura.
pub const MAX_PALAVRAS: usize = 16 * 1024 * 1024;

/// Registro do `.s1i`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegistroImagem {
    pub quadro: u32,
    pub largura: u16,
    pub altura: u16,
    pub sha1: [u8; 20],
}

/// Palavras de DMA de um quadro (`.s1v`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistroDma {
    pub quadro: u32,
    pub palavras: Vec<u16>,
}

/// Quadro inteiro (`.s1p`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuadroInteiro {
    pub quadro: u32,
    pub largura: u16,
    pub altura: u16,
    pub pontos: Vec<u16>,
}

fn invalido(msg: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg)
}

fn ler_n<R: Read, const N: usize>(r: &mut R) -> io::Result<[u8; N]> {
    let mut b = [0u8; N];
    r.read_exact(&mut b)?;
    Ok(b)
}

/// Le o primeiro byte; `None` se o arquivo acabou limpo.
fn ler_ou_fim<R: Read, const N: usize>(r: &mut R) -> io::Result<Option<[u8; N]>> {
    let mut b = [0u8; N];
    let mut lidos = 0usize;
    while lidos < N {
        let n = r.read(&mut b[lidos..])?;
        if n == 0 {
            return if lidos == 0 {
                Ok(None)
            } else {
                Err(invalido("registro truncado"))
            };
        }
        lidos = lidos.checked_add(n).ok_or_else(|| invalido("contador"))?;
    }
    Ok(Some(b))
}

fn conferir_magia<R: Read>(r: &mut R, esperada: &[u8; 8]) -> io::Result<()> {
    if &ler_n::<_, 8>(r)? != esperada {
        return Err(invalido("magia errada"));
    }
    Ok(())
}

// ---- .s1i ----

pub fn escrever_cabecalho_imagem<W: Write>(w: &mut W) -> io::Result<()> {
    w.write_all(&MAGIA_IMAGEM)
}

pub fn escrever_imagem<W: Write>(w: &mut W, r: &RegistroImagem) -> io::Result<()> {
    w.write_all(&r.quadro.to_le_bytes())?;
    w.write_all(&r.largura.to_le_bytes())?;
    w.write_all(&r.altura.to_le_bytes())?;
    w.write_all(&r.sha1)
}

/// Le todo o `.s1i`.
pub fn ler_imagens<R: Read>(mut r: R) -> io::Result<Vec<RegistroImagem>> {
    conferir_magia(&mut r, &MAGIA_IMAGEM)?;
    let mut saida = Vec::new();
    while let Some(b) = ler_ou_fim::<_, 28>(&mut r)? {
        saida.push(RegistroImagem {
            quadro: u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            largura: u16::from_le_bytes([b[4], b[5]]),
            altura: u16::from_le_bytes([b[6], b[7]]),
            sha1: b[8..28].try_into().map_err(|_| invalido("sha1"))?,
        });
    }
    Ok(saida)
}

// ---- .s1v ----

pub fn escrever_cabecalho_video<W: Write>(w: &mut W) -> io::Result<()> {
    w.write_all(&MAGIA_VIDEO)
}

/// Escreve o registro de um quadro. Quadro sem palavras nao precisa ser escrito.
pub fn escrever_dma<W: Write>(w: &mut W, r: &RegistroDma) -> io::Result<()> {
    let n = u32::try_from(r.palavras.len()).map_err(|_| invalido("palavras demais"))?;
    w.write_all(&r.quadro.to_le_bytes())?;
    w.write_all(&n.to_le_bytes())?;
    for p in &r.palavras {
        w.write_all(&p.to_le_bytes())?;
    }
    Ok(())
}

/// Le todo o `.s1v`.
pub fn ler_dma<R: Read>(mut r: R) -> io::Result<Vec<RegistroDma>> {
    conferir_magia(&mut r, &MAGIA_VIDEO)?;
    let mut saida = Vec::new();
    while let Some(c) = ler_ou_fim::<_, 8>(&mut r)? {
        let quadro = u32::from_le_bytes([c[0], c[1], c[2], c[3]]);
        let n = usize::try_from(u32::from_le_bytes([c[4], c[5], c[6], c[7]]))
            .map_err(|_| invalido("tamanho"))?;
        if n > MAX_PALAVRAS {
            return Err(invalido("quadro com palavras demais"));
        }
        let mut palavras = Vec::with_capacity(n);
        for _ in 0..n {
            palavras.push(u16::from_le_bytes(ler_n(&mut r)?));
        }
        saida.push(RegistroDma { quadro, palavras });
    }
    Ok(saida)
}

// ---- .s1p ----

pub fn escrever_quadro_inteiro<W: Write>(w: &mut W, q: &QuadroInteiro) -> io::Result<()> {
    let esperado = usize::from(q.largura)
        .checked_mul(usize::from(q.altura))
        .ok_or_else(|| invalido("tamanho"))?;
    if q.pontos.len() != esperado {
        return Err(invalido("pontos nao batem com largura e altura"));
    }
    w.write_all(&MAGIA_PIXELS)?;
    w.write_all(&q.quadro.to_le_bytes())?;
    w.write_all(&q.largura.to_le_bytes())?;
    w.write_all(&q.altura.to_le_bytes())?;
    for p in &q.pontos {
        w.write_all(&p.to_le_bytes())?;
    }
    Ok(())
}

pub fn ler_quadro_inteiro<R: Read>(mut r: R) -> io::Result<QuadroInteiro> {
    conferir_magia(&mut r, &MAGIA_PIXELS)?;
    let quadro = u32::from_le_bytes(ler_n(&mut r)?);
    let largura = u16::from_le_bytes(ler_n(&mut r)?);
    let altura = u16::from_le_bytes(ler_n(&mut r)?);
    let n = usize::from(largura)
        .checked_mul(usize::from(altura))
        .ok_or_else(|| invalido("tamanho"))?;
    if n > MAX_PONTOS {
        return Err(invalido("imagem grande demais"));
    }
    let mut pontos = Vec::with_capacity(n);
    for _ in 0..n {
        pontos.push(u16::from_le_bytes(ler_n(&mut r)?));
    }
    Ok(QuadroInteiro {
        quadro,
        largura,
        altura,
        pontos,
    })
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn imagem_ida_e_volta() {
        let regs = [
            RegistroImagem {
                quadro: 0,
                largura: 320,
                altura: 224,
                sha1: [1; 20],
            },
            RegistroImagem {
                quadro: 1,
                largura: 256,
                altura: 224,
                sha1: [2; 20],
            },
        ];
        let mut buf = Vec::new();
        escrever_cabecalho_imagem(&mut buf).unwrap();
        for r in &regs {
            escrever_imagem(&mut buf, r).unwrap();
        }
        assert_eq!(ler_imagens(&buf[..]).unwrap(), regs);
    }

    #[test]
    fn dma_ida_e_volta_e_truncado_e_recusado() {
        let regs = [
            RegistroDma {
                quadro: 3,
                palavras: vec![0x1234, 0xffff],
            },
            RegistroDma {
                quadro: 4,
                palavras: vec![],
            },
        ];
        let mut buf = Vec::new();
        escrever_cabecalho_video(&mut buf).unwrap();
        for r in &regs {
            escrever_dma(&mut buf, r).unwrap();
        }
        assert_eq!(ler_dma(&buf[..]).unwrap(), regs);
        buf.pop();
        assert!(ler_dma(&buf[..]).is_err());
    }

    #[test]
    fn quadro_inteiro_ida_e_volta_e_tamanho_errado() {
        let q = QuadroInteiro {
            quadro: 9,
            largura: 3,
            altura: 2,
            pontos: vec![1, 2, 3, 4, 5, 511],
        };
        let mut buf = Vec::new();
        escrever_quadro_inteiro(&mut buf, &q).unwrap();
        assert_eq!(ler_quadro_inteiro(&buf[..]).unwrap(), q);
        let ruim = QuadroInteiro {
            pontos: vec![1],
            ..q
        };
        assert!(escrever_quadro_inteiro(&mut Vec::new(), &ruim).is_err());
    }

    #[test]
    fn magia_errada_e_recusada() {
        assert!(ler_imagens(&b"XXXXXXXX"[..]).is_err());
        assert!(ler_dma(&MAGIA_IMAGEM[..]).is_err());
        assert!(ler_quadro_inteiro(&b"curto"[..]).is_err());
    }

    #[test]
    fn imagem_gigante_e_recusada() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&MAGIA_PIXELS);
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&u16::MAX.to_le_bytes());
        buf.extend_from_slice(&u16::MAX.to_le_bytes());
        assert!(ler_quadro_inteiro(&buf[..]).is_err());
    }
}
