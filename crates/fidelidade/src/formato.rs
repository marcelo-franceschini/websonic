//! Formato binario do trace: cabecalho e quadros.
//!
//! ```text
//! cabecalho: MAGIA(8) versao(u16) tam_oraculo(u16) oraculo(bytes)
//!            rom_sha1(20) config_sha1(20) entradas_sha1(20) modo(u8)
//! quadro:    'Q' numero(u32) mclk_fim(u64) n_eventos(u32) eventos(n*34) hashes(9*20)
//! ```

use crate::evento::{Evento, TAMANHO_EVENTO};
use crate::hash::{sha1, sha1_de_partes};
use std::fmt;
use std::io::{self, Read, Write};

/// Identifica o arquivo como trace.
pub const MAGIA: [u8; 8] = *b"S1TRACE\0";
/// Versao do formato. Muda se o formato mudar.
pub const VERSAO: u16 = 1;
const MARCA_QUADRO: u8 = b'Q';

/// Categorias com hash por quadro. O comparador aponta a primeira que difere.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Categoria {
    Ram68k = 0,
    RamZ80 = 1,
    Vram = 2,
    Cram = 3,
    Vsram = 4,
    RegsVdp = 5,
    Ym = 6,
    Psg = 7,
    Eventos = 8,
}

impl Categoria {
    pub const TOTAL: usize = 9;

    pub const TODAS: [Categoria; Self::TOTAL] = [
        Self::Ram68k,
        Self::RamZ80,
        Self::Vram,
        Self::Cram,
        Self::Vsram,
        Self::RegsVdp,
        Self::Ym,
        Self::Psg,
        Self::Eventos,
    ];

    pub fn nome(self) -> &'static str {
        match self {
            Self::Ram68k => "ram68k",
            Self::RamZ80 => "ram_z80",
            Self::Vram => "vram",
            Self::Cram => "cram",
            Self::Vsram => "vsram",
            Self::RegsVdp => "regs_vdp",
            Self::Ym => "ym",
            Self::Psg => "psg",
            Self::Eventos => "eventos",
        }
    }
}

/// Erros de leitura e escrita do trace.
#[derive(Debug)]
pub enum Erro {
    Io(io::Error),
    MagiaInvalida,
    VersaoNaoSuportada(u16),
    MarcaInvalida(u8),
    TipoDesconhecido(u8),
    OrigemDesconhecida(u8),
    /// O arquivo acabou no meio de um registro.
    Truncado,
}

impl PartialEq for Erro {
    fn eq(&self, outro: &Self) -> bool {
        match (self, outro) {
            (Self::Io(a), Self::Io(b)) => a.kind() == b.kind(),
            (Self::VersaoNaoSuportada(a), Self::VersaoNaoSuportada(b)) => a == b,
            (Self::MarcaInvalida(a), Self::MarcaInvalida(b)) => a == b,
            (Self::TipoDesconhecido(a), Self::TipoDesconhecido(b)) => a == b,
            (Self::OrigemDesconhecida(a), Self::OrigemDesconhecida(b)) => a == b,
            (Self::MagiaInvalida, Self::MagiaInvalida) | (Self::Truncado, Self::Truncado) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Erro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "erro de E/S: {e}"),
            Self::MagiaInvalida => write!(f, "nao e um trace (magia invalida)"),
            Self::VersaoNaoSuportada(v) => write!(f, "versao {v} do formato nao suportada"),
            Self::MarcaInvalida(m) => write!(f, "marca de quadro invalida: {m}"),
            Self::TipoDesconhecido(t) => write!(f, "tipo de evento desconhecido: {t}"),
            Self::OrigemDesconhecida(o) => write!(f, "origem de evento desconhecida: {o}"),
            Self::Truncado => write!(f, "trace truncado"),
        }
    }
}

impl std::error::Error for Erro {}

impl From<io::Error> for Erro {
    fn from(e: io::Error) -> Self {
        if e.kind() == io::ErrorKind::UnexpectedEof {
            Self::Truncado
        } else {
            Self::Io(e)
        }
    }
}

/// Cabecalho do trace. Tudo aqui precisa ser igual para dois traces serem comparaveis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cabecalho {
    /// Identifica o oraculo: nome, commit e hash dos patches.
    pub oraculo: String,
    pub rom_sha1: [u8; 20],
    /// Hash da configuracao do console (regiao, tempo, controle).
    pub config_sha1: [u8; 20],
    /// Hash da lista de entradas.
    pub entradas_sha1: [u8; 20],
    /// 0 = execucao normal. Outros valores sao reservados.
    pub modo: u8,
}

impl Cabecalho {
    fn para_bytes(&self) -> Vec<u8> {
        let nome = self.oraculo.as_bytes();
        // Nome limitado a 65535 bytes. Mais que isso e erro de uso, entao corta.
        let tam = u16::try_from(nome.len()).unwrap_or(u16::MAX);
        let mut saida = Vec::with_capacity(8 + 2 + 2 + usize::from(tam) + 61);
        saida.extend_from_slice(&MAGIA);
        saida.extend_from_slice(&VERSAO.to_le_bytes());
        saida.extend_from_slice(&tam.to_le_bytes());
        saida.extend_from_slice(&nome[..usize::from(tam)]);
        saida.extend_from_slice(&self.rom_sha1);
        saida.extend_from_slice(&self.config_sha1);
        saida.extend_from_slice(&self.entradas_sha1);
        saida.push(self.modo);
        saida
    }
}

/// Um quadro: eventos mais os hashes das categorias.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quadro {
    pub numero: u32,
    /// Relogio mestre no fim do quadro.
    pub mclk_fim: u64,
    pub eventos: Vec<Evento>,
    /// Hash de cada categoria, na ordem de `Categoria`. O hash de `Eventos` e calculado.
    pub hashes: [[u8; 20]; Categoria::TOTAL],
}

impl Quadro {
    /// Hash dos eventos serializados do quadro.
    pub fn hash_dos_eventos(eventos: &[Evento]) -> [u8; 20] {
        let mut bytes = Vec::with_capacity(eventos.len() * TAMANHO_EVENTO);
        for e in eventos {
            bytes.extend_from_slice(&e.para_bytes());
        }
        sha1(&bytes)
    }

    /// Preenche o hash de `Eventos` a partir dos eventos. Chame antes de escrever.
    pub fn fechar(&mut self) {
        self.hashes[Categoria::Eventos as usize] = Self::hash_dos_eventos(&self.eventos);
    }

    /// Resumo de um quadro: hash dos hashes. Usado no hash do trace inteiro.
    fn resumo(&self) -> [u8; 20] {
        let numero = self.numero.to_le_bytes();
        let fim = self.mclk_fim.to_le_bytes();
        let mut partes: Vec<&[u8]> = vec![&numero, &fim];
        for h in &self.hashes {
            partes.push(h);
        }
        sha1_de_partes(&partes)
    }
}

/// Escreve um trace em qualquer destino.
pub struct Escritor<W: Write> {
    destino: W,
    resumos: Vec<u8>,
}

impl<W: Write> Escritor<W> {
    pub fn novo(mut destino: W, cabecalho: &Cabecalho) -> Result<Self, Erro> {
        destino.write_all(&cabecalho.para_bytes())?;
        Ok(Self {
            destino,
            resumos: Vec::new(),
        })
    }

    /// Escreve um quadro. O hash de eventos e recalculado aqui, para nao haver quadro incoerente.
    pub fn escrever(&mut self, quadro: &Quadro) -> Result<(), Erro> {
        let mut quadro = quadro.clone();
        quadro.fechar();
        let n = u32::try_from(quadro.eventos.len()).map_err(|_| Erro::Truncado)?;
        self.destino.write_all(&[MARCA_QUADRO])?;
        self.destino.write_all(&quadro.numero.to_le_bytes())?;
        self.destino.write_all(&quadro.mclk_fim.to_le_bytes())?;
        self.destino.write_all(&n.to_le_bytes())?;
        for e in &quadro.eventos {
            self.destino.write_all(&e.para_bytes())?;
        }
        for h in &quadro.hashes {
            self.destino.write_all(h)?;
        }
        self.resumos.extend_from_slice(&quadro.resumo());
        Ok(())
    }

    /// Termina e devolve o hash do trace inteiro (cabecalho e todos os quadros).
    pub fn finalizar(mut self, cabecalho: &Cabecalho) -> Result<[u8; 20], Erro> {
        self.destino.flush()?;
        Ok(hash_do_trace(cabecalho, &self.resumos))
    }
}

fn hash_do_trace(cabecalho: &Cabecalho, resumos: &[u8]) -> [u8; 20] {
    sha1_de_partes(&[&cabecalho.para_bytes(), resumos])
}

/// Le um trace em ordem.
pub struct Leitor<R: Read> {
    origem: R,
    pub cabecalho: Cabecalho,
    resumos: Vec<u8>,
}

fn ler_n<R: Read, const N: usize>(origem: &mut R) -> Result<[u8; N], Erro> {
    let mut buf = [0u8; N];
    origem.read_exact(&mut buf)?;
    Ok(buf)
}

impl<R: Read> Leitor<R> {
    pub fn abrir(mut origem: R) -> Result<Self, Erro> {
        let magia: [u8; 8] = ler_n(&mut origem)?;
        if magia != MAGIA {
            return Err(Erro::MagiaInvalida);
        }
        let versao = u16::from_le_bytes(ler_n(&mut origem)?);
        if versao != VERSAO {
            return Err(Erro::VersaoNaoSuportada(versao));
        }
        let tam = usize::from(u16::from_le_bytes(ler_n(&mut origem)?));
        let mut nome = vec![0u8; tam];
        origem.read_exact(&mut nome)?;
        let cabecalho = Cabecalho {
            oraculo: String::from_utf8_lossy(&nome).into_owned(),
            rom_sha1: ler_n(&mut origem)?,
            config_sha1: ler_n(&mut origem)?,
            entradas_sha1: ler_n(&mut origem)?,
            modo: ler_n::<_, 1>(&mut origem)?[0],
        };
        Ok(Self {
            origem,
            cabecalho,
            resumos: Vec::new(),
        })
    }

    /// Le o proximo quadro. `Ok(None)` no fim limpo do arquivo.
    pub fn proximo(&mut self) -> Result<Option<Quadro>, Erro> {
        let mut marca = [0u8; 1];
        // Fim limpo: zero bytes lidos na marca.
        if self.origem.read(&mut marca)? == 0 {
            return Ok(None);
        }
        if marca[0] != MARCA_QUADRO {
            return Err(Erro::MarcaInvalida(marca[0]));
        }
        let numero = u32::from_le_bytes(ler_n(&mut self.origem)?);
        let mclk_fim = u64::from_le_bytes(ler_n(&mut self.origem)?);
        let n = u32::from_le_bytes(ler_n(&mut self.origem)?);
        let mut eventos = Vec::new();
        for _ in 0..n {
            let bytes: [u8; TAMANHO_EVENTO] = ler_n(&mut self.origem)?;
            eventos.push(Evento::de_bytes(&bytes)?);
        }
        let mut hashes = [[0u8; 20]; Categoria::TOTAL];
        for h in &mut hashes {
            *h = ler_n(&mut self.origem)?;
        }
        let quadro = Quadro {
            numero,
            mclk_fim,
            eventos,
            hashes,
        };
        self.resumos.extend_from_slice(&quadro.resumo());
        Ok(Some(quadro))
    }

    /// Hash do trace inteiro, igual ao que o escritor devolveu. Valido depois de ler tudo.
    pub fn hash_lido(&self) -> [u8; 20] {
        hash_do_trace(&self.cabecalho, &self.resumos)
    }
}

#[cfg(test)]
pub(crate) mod testes_comuns {
    use super::*;
    use crate::evento::{Origem, Tipo};

    pub(crate) fn cabecalho() -> Cabecalho {
        Cabecalho {
            oraculo: "teste".into(),
            rom_sha1: [1; 20],
            config_sha1: [2; 20],
            entradas_sha1: [3; 20],
            modo: 0,
        }
    }

    pub(crate) fn quadro(numero: u32, n: u32) -> Quadro {
        let eventos = (0..n)
            .map(|i| Evento {
                mclk: u64::from(numero) * 1000 + u64::from(i),
                seq: i,
                tipo: Tipo::Vdp,
                origem: Origem::Cpu68,
                linha: 10,
                coluna: 20,
                a: 0,
                b: i,
                c: 2,
                d: 0,
            })
            .collect();
        let mut hashes = [[0u8; 20]; Categoria::TOTAL];
        for (i, h) in hashes.iter_mut().enumerate() {
            h[0] = u8::try_from(i).unwrap();
        }
        let mut q = Quadro {
            numero,
            mclk_fim: u64::from(numero) * 1000 + 999,
            eventos,
            hashes,
        };
        q.fechar();
        q
    }

    pub(crate) fn serializar(quadros: &[Quadro]) -> (Vec<u8>, [u8; 20]) {
        let c = cabecalho();
        let mut buf = Vec::new();
        let mut esc = Escritor::novo(&mut buf, &c).unwrap();
        for q in quadros {
            esc.escrever(q).unwrap();
        }
        let h = esc.finalizar(&c).unwrap();
        (buf, h)
    }

    #[test]
    fn ida_e_volta() {
        let quadros = vec![quadro(0, 3), quadro(1, 0), quadro(2, 5)];
        let (buf, hash) = serializar(&quadros);
        let mut leitor = Leitor::abrir(buf.as_slice()).unwrap();
        assert_eq!(leitor.cabecalho, cabecalho());
        let mut lidos = Vec::new();
        while let Some(q) = leitor.proximo().unwrap() {
            lidos.push(q);
        }
        assert_eq!(lidos, quadros);
        assert_eq!(leitor.hash_lido(), hash);
    }

    #[test]
    fn mesma_entrada_mesmos_bytes() {
        let quadros = vec![quadro(0, 4), quadro(1, 2)];
        let (a, ha) = serializar(&quadros);
        let (b, hb) = serializar(&quadros);
        assert_eq!(a, b);
        assert_eq!(ha, hb);
    }

    #[test]
    fn hash_muda_com_um_bit() {
        let (_, h1) = serializar(&[quadro(0, 4)]);
        let mut q = quadro(0, 4);
        q.hashes[Categoria::Vram as usize][5] ^= 1;
        let (_, h2) = serializar(&[q]);
        assert_ne!(h1, h2);
    }

    #[test]
    fn magia_invalida() {
        let r = Leitor::abrir(&b"nao e trace, tamanho suficiente"[..]);
        assert_eq!(r.err(), Some(Erro::MagiaInvalida));
    }

    #[test]
    fn truncado_e_detectado() {
        let (mut buf, _) = serializar(&[quadro(0, 3)]);
        buf.truncate(buf.len() - 7);
        let mut leitor = Leitor::abrir(buf.as_slice()).unwrap();
        assert_eq!(leitor.proximo().err(), Some(Erro::Truncado));
    }

    #[test]
    fn marca_invalida() {
        let (mut buf, _) = serializar(&[quadro(0, 1)]);
        let pos = Cabecalho::para_bytes(&cabecalho()).len();
        buf[pos] = b'X';
        let mut leitor = Leitor::abrir(buf.as_slice()).unwrap();
        assert_eq!(leitor.proximo().err(), Some(Erro::MarcaInvalida(b'X')));
    }
}
