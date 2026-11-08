//! Harness de teste por rotina (sub-etapa 5.5): metadado por rotina e comparação.
//!
//! O metadado diz, só com nomes de registrador, o que o chamador do original usa depois da
//! rotina. O núcleo não tem registradores como estado: o adaptador de cada rotina devolve só os
//! valores que correspondem a registradores de `usados_depois`. Os outros ficam em `ignorados`
//! e a comparação lista os que o original mexeu, para quem revisa ver se algum chamador lê
//! (caso do requisito "rotinas usam valores que sobraram em registradores, inclusive bugs").
//!
//! Formato do arquivo de metadado (`rotinas/NOME.txt`), uma linha por campo:
//! ```text
//! rotulo: KosDec
//! entrada: a0 = origem
//! usados_depois: a1
//! ignorados: d0 d1 d2 d3 d4 d5 d6 d7 a0 a2 a3 a4 a5 a6 ccr
//! ```
//! Todo registrador (d0 a d7, a0 a a6, ccr) tem de estar em `usados_depois` ou em `ignorados`.

use std::collections::{BTreeMap, BTreeSet};

/// Registrador do 68000 como NOME, só para o metadado e a comparação. O núcleo não os guarda.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Reg {
    D(u8),
    A(u8),
    Ccr,
}

impl Reg {
    pub fn de_texto(t: &str) -> Option<Self> {
        let t = t.trim().to_ascii_lowercase();
        if t == "ccr" {
            return Some(Self::Ccr);
        }
        let n: u8 = t.get(1..)?.parse().ok()?;
        match t.chars().next()? {
            'd' if n < 8 => Some(Self::D(n)),
            'a' if n < 7 => Some(Self::A(n)),
            _ => None,
        }
    }

    pub fn nome(self) -> String {
        match self {
            Self::D(n) => format!("d{n}"),
            Self::A(n) => format!("a{n}"),
            Self::Ccr => "ccr".to_string(),
        }
    }

    /// Todos os registradores que o metadado precisa classificar.
    pub fn todos() -> Vec<Reg> {
        let mut v: Vec<Reg> = (0..8).map(Reg::D).collect();
        v.extend((0..7).map(Reg::A));
        v.push(Reg::Ccr);
        v
    }
}

/// Metadado de uma rotina.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Metadado {
    pub rotulo: String,
    pub entradas: Vec<(Reg, String)>,
    pub usados_depois: BTreeSet<Reg>,
    pub ignorados: BTreeSet<Reg>,
}

impl Metadado {
    pub fn analisar(texto: &str) -> Result<Self, String> {
        let (mut rotulo, mut entradas) = (None, Vec::new());
        let (mut usados, mut ignorados) = (BTreeSet::new(), BTreeSet::new());
        for (i, l) in texto.lines().enumerate() {
            let l = l.trim();
            if l.is_empty() || l.starts_with('#') {
                continue;
            }
            let (campo, valor) = l
                .split_once(':')
                .ok_or(format!("linha {}: falta ':'", i + 1))?;
            let valor = valor.trim();
            match campo.trim() {
                "rotulo" => rotulo = Some(valor.to_string()),
                "entrada" => {
                    let (r, nome) = valor
                        .split_once('=')
                        .ok_or(format!("linha {}: entrada sem '='", i + 1))?;
                    let reg =
                        Reg::de_texto(r).ok_or(format!("linha {}: registrador {r:?}", i + 1))?;
                    entradas.push((reg, nome.trim().to_string()));
                }
                "usados_depois" | "ignorados" => {
                    let dest = if campo.trim() == "usados_depois" {
                        &mut usados
                    } else {
                        &mut ignorados
                    };
                    for r in valor.split_whitespace() {
                        let reg = Reg::de_texto(r)
                            .ok_or(format!("linha {}: registrador {r:?}", i + 1))?;
                        dest.insert(reg);
                    }
                }
                outro => return Err(format!("linha {}: campo desconhecido {outro:?}", i + 1)),
            }
        }
        let rotulo = rotulo.ok_or("falta o campo rotulo")?;
        if let Some(r) = usados.intersection(&ignorados).next() {
            return Err(format!("{} esta em usados_depois e em ignorados", r.nome()));
        }
        let faltam: Vec<String> = Reg::todos()
            .into_iter()
            .filter(|r| !usados.contains(r) && !ignorados.contains(r))
            .map(Reg::nome)
            .collect();
        if !faltam.is_empty() {
            return Err(format!(
                "registradores sem classificacao: {}",
                faltam.join(" ")
            ));
        }
        Ok(Self {
            rotulo,
            entradas,
            usados_depois: usados,
            ignorados,
        })
    }
}

/// Estado depois da rotina, de um dos lados.
#[derive(Clone, Debug, Default)]
pub struct EstadoFinal {
    /// O oráculo dá todos os registradores; o núcleo, só os que o adaptador devolve.
    pub regs: BTreeMap<Reg, u32>,
    /// A RAM do 68000 inteira (64 KiB).
    pub ram: Vec<u8>,
}

/// Resultado da comparação de um caso.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RelatorioCaso {
    /// Offsets da RAM que diferem fora das exclusões.
    pub ram_diferente: Vec<usize>,
    /// Quantos bytes diferem dentro de áreas excluídas (só informação).
    pub excluidos_diferentes: usize,
    /// Registradores usados depois que diferem: (registrador, oráculo, núcleo).
    pub usados_diferentes: Vec<(Reg, u32, u32)>,
    /// Registradores usados depois que o núcleo não devolveu (erro do adaptador).
    pub usados_sem_valor: Vec<Reg>,
    /// Registradores ignorados que o original mexeu (valor de entrada diferente do de saída).
    pub ignorados_mexidos: Vec<Reg>,
}

impl RelatorioCaso {
    pub fn passou(&self) -> bool {
        self.ram_diferente.is_empty()
            && self.usados_diferentes.is_empty()
            && self.usados_sem_valor.is_empty()
    }
}

/// Compara os dois lados. `entrada_regs` são os registradores de partida do oráculo, usados só
/// para listar os ignorados que o original mexeu. `exclusoes` são faixas de offset da RAM
/// `[inicio, fim)` aprovadas pelo Marcelo.
pub fn comparar(
    meta: &Metadado,
    entrada_regs: &BTreeMap<Reg, u32>,
    oraculo: &EstadoFinal,
    nucleo: &EstadoFinal,
    exclusoes: &[(usize, usize)],
) -> RelatorioCaso {
    let mut r = RelatorioCaso::default();
    let n = oraculo.ram.len().min(nucleo.ram.len());
    for i in 0..n {
        if oraculo.ram[i] != nucleo.ram[i] {
            if exclusoes.iter().any(|(a, b)| i >= *a && i < *b) {
                r.excluidos_diferentes += 1;
            } else {
                r.ram_diferente.push(i);
            }
        }
    }
    if oraculo.ram.len() != nucleo.ram.len() {
        r.ram_diferente.push(n);
    }
    for reg in &meta.usados_depois {
        let (Some(o), n) = (oraculo.regs.get(reg), nucleo.regs.get(reg)) else {
            continue;
        };
        match n {
            None => r.usados_sem_valor.push(*reg),
            Some(n) if n != o => r.usados_diferentes.push((*reg, *o, *n)),
            Some(_) => {}
        }
    }
    for reg in &meta.ignorados {
        if oraculo.regs.get(reg) != entrada_regs.get(reg) {
            r.ignorados_mexidos.push(*reg);
        }
    }
    r
}

#[cfg(test)]
mod testes {
    use super::*;

    const META_OK: &str = "rotulo: Exemplo\nentrada: a0 = origem\nusados_depois: a1\nignorados: d0 d1 d2 d3 d4 d5 d6 d7 a0 a2 a3 a4 a5 a6 ccr\n";

    #[test]
    fn metadado_valido_e_recusas() {
        let m = Metadado::analisar(META_OK).unwrap();
        assert_eq!(m.rotulo, "Exemplo");
        assert!(m.usados_depois.contains(&Reg::A(1)));
        // Falta classificar o ccr.
        let sem_ccr = META_OK.replace(" ccr", "");
        assert!(Metadado::analisar(&sem_ccr).unwrap_err().contains("ccr"));
        // Mesmo registrador nas duas listas.
        let duplo = META_OK.replace("usados_depois: a1", "usados_depois: a1 d0");
        assert!(Metadado::analisar(&duplo).unwrap_err().contains("d0"));
        assert!(Metadado::analisar("rotulo: X\nfoo: bar").is_err());
        assert!(Metadado::analisar("usados_depois: a9").is_err());
    }

    #[test]
    fn os_metadados_do_piloto_sao_validos() {
        // include_str!, pois o teste também roda em WebAssembly, sem acesso ao disco.
        let metadados = [
            ("KosDec", include_str!("../rotinas/KosDec.txt")),
            ("EniDec", include_str!("../rotinas/EniDec.txt")),
            ("NemDecToRAM", include_str!("../rotinas/NemDecToRAM.txt")),
        ];
        for (nome, t) in metadados {
            assert_eq!(Metadado::analisar(t).unwrap().rotulo, nome);
        }
    }

    fn estado(a1: u32, byte: u8) -> EstadoFinal {
        let mut ram = vec![0u8; 16];
        ram[3] = byte;
        EstadoFinal {
            regs: BTreeMap::from([(Reg::A(1), a1)]),
            ram,
        }
    }

    #[test]
    fn acusa_ram_registrador_usado_e_lista_ignorado_mexido() {
        let m = Metadado::analisar(META_OK).unwrap();
        let entrada = BTreeMap::from([(Reg::A(1), 0x10), (Reg::D(0), 5)]);
        let mut o = estado(0x20, 7);
        o.regs.insert(Reg::D(0), 99); // o original mexeu em d0 (ignorado)
        // Tudo igual.
        let r = comparar(&m, &entrada, &o, &estado(0x20, 7), &[]);
        assert!(r.passou());
        assert_eq!(r.ignorados_mexidos, vec![Reg::D(0)]);
        // Byte de RAM errado.
        let r = comparar(&m, &entrada, &o, &estado(0x20, 8), &[]);
        assert_eq!(r.ram_diferente, vec![3]);
        assert!(!r.passou());
        // A mesma diferenca numa area excluida nao reprova, mas e contada.
        let r = comparar(&m, &entrada, &o, &estado(0x20, 8), &[(0, 8)]);
        assert!(r.passou());
        assert_eq!(r.excluidos_diferentes, 1);
        // Registrador usado errado.
        let r = comparar(&m, &entrada, &o, &estado(0x21, 7), &[]);
        assert_eq!(r.usados_diferentes, vec![(Reg::A(1), 0x20, 0x21)]);
        // O adaptador esqueceu de devolver o registrador usado.
        let sem = EstadoFinal {
            regs: BTreeMap::new(),
            ram: vec![0u8; 16],
        };
        let mut o2 = estado(0x20, 0);
        o2.ram[3] = 0;
        let r = comparar(&m, &entrada, &o2, &sem, &[]);
        assert_eq!(r.usados_sem_valor, vec![Reg::A(1)]);
    }
}
