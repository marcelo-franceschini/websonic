//! `cargo xtask inventario` (sub-etapa 2.6, parte do hardware).
//!
//! Lê os traces do corpus (`*.a.s1t` em `--traces`) e gera `docs/inventario-hardware.md`:
//! o que o Sonic 1 usa da VDP, do YM2612, do PSG e do barramento do Z80, com contagem e o
//! primeiro caso, quadro e `seq` em que cada item apareceu (a evidência). Item que o corpus
//! não mostra não entra. Só números e nomes: nunca bytes de dados.
//!
//! A porta de dados da VDP (valores de tiles, cores) não é registrada aqui, só contada.
//! Não observável nos eventos atuais: DMA (sem evento próprio, R6) e E/S do 68000 (pedido P2).
//!
//! Uso: `cargo xtask inventario --traces DIR [--saida docs/inventario-hardware.md]`
use sonic1_fidelidade::{Leitor, Tipo};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// Onde um item apareceu pela primeira vez.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Primeiro {
    pub caso: String,
    pub quadro: u32,
    pub seq: u32,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Contagem {
    pub vezes: u64,
    pub primeiro: Option<Primeiro>,
}

impl Contagem {
    fn somar(&mut self, onde: &Primeiro) {
        self.vezes = self.vezes.saturating_add(1);
        if self.primeiro.is_none() {
            self.primeiro = Some(onde.clone());
        }
    }
}

/// Tudo o que o inventário coleta. `BTreeMap` mantém a ordem estável.
#[derive(Default, Debug)]
pub struct Inventario {
    /// Registradores da VDP: (registrador, valor) -> contagem.
    pub vdp_registradores: BTreeMap<(u8, u8), Contagem>,
    /// Comandos de endereço: nome do alvo e direção -> contagem.
    pub vdp_comandos: BTreeMap<String, Contagem>,
    /// DMA disparado: modo (pelo registrador 23) -> contagem.
    pub vdp_dma: BTreeMap<String, Contagem>,
    /// Escritas na porta de dados, por tamanho.
    pub vdp_dados: BTreeMap<u8, Contagem>,
    /// Registradores do YM2612 escritos (porta, registrador).
    pub ym_registradores: BTreeMap<(u8, u8), Contagem>,
    pub ym_dac: Contagem,
    /// PSG: classe do byte (tom, volume, ruído, dado).
    pub psg: BTreeMap<String, Contagem>,
    /// Interrupções do 68000 por nível.
    pub int68: BTreeMap<u32, Contagem>,
    /// BUSREQ e RESET do Z80: (sinal, ativo).
    pub z80: BTreeMap<(u32, u32), Contagem>,
}

/// Estado do decodificador da porta de controle, por caso.
#[derive(Default)]
struct Porta {
    primeira: Option<u16>,
    reg23: u8,
}

fn alvo(cd: u8) -> Option<&'static str> {
    Some(match cd & 0x0f {
        0x0 => "leitura de VRAM",
        0x1 => "escrita em VRAM",
        0x3 => "escrita em CRAM",
        0x4 => "leitura de VSRAM",
        0x5 => "escrita em VSRAM",
        0x8 => "leitura de CRAM",
        _ => return None,
    })
}

fn modo_dma(reg23: u8) -> &'static str {
    match reg23 >> 6 {
        0 | 1 => "68000 para VDP",
        2 => "preenchimento de VRAM",
        _ => "cópia de VRAM",
    }
}

/// Acumula os eventos de um trace no inventário.
pub fn acumular<R: std::io::Read>(
    inv: &mut Inventario,
    caso: &str,
    mut leitor: Leitor<R>,
) -> Result<(), String> {
    let mut porta = Porta::default();
    while let Some(q) = leitor.proximo().map_err(|e| e.to_string())? {
        for e in &q.eventos {
            let onde = Primeiro {
                caso: caso.to_string(),
                quadro: q.numero,
                seq: e.seq,
            };
            match e.tipo {
                Tipo::Vdp if e.a == 1 => porta_controle(inv, &mut porta, e.b as u16, &onde),
                Tipo::Vdp => inv.vdp_dados.entry(e.c as u8).or_default().somar(&onde),
                Tipo::Ym => {
                    // Só escrita de dado traz o registrador (campo c).
                    if e.a % 2 == 1 {
                        let reg = e.c as u8;
                        inv.ym_registradores
                            .entry((u8::try_from(e.a / 2).unwrap_or(0), reg))
                            .or_default()
                            .somar(&onde);
                        if e.d == 1 {
                            inv.ym_dac.somar(&onde);
                        }
                    }
                }
                Tipo::Psg => {
                    let v = e.a as u8;
                    let classe = if v & 0x80 == 0 {
                        "dado de tom".to_string()
                    } else {
                        let canal = (v >> 5) & 3;
                        let tipo = if v & 0x10 != 0 {
                            "volume"
                        } else if canal == 3 {
                            "ruído"
                        } else {
                            "tom"
                        };
                        format!("{tipo}, canal {canal}")
                    };
                    inv.psg.entry(classe).or_default().somar(&onde);
                }
                Tipo::Int68 => inv.int68.entry(e.a).or_default().somar(&onde),
                Tipo::BarramentoZ80 => inv.z80.entry((e.a, e.b)).or_default().somar(&onde),
                _ => {}
            }
        }
    }
    Ok(())
}

/// Decodifica uma palavra escrita na porta de controle (duas palavras formam um comando).
fn porta_controle(inv: &mut Inventario, porta: &mut Porta, w: u16, onde: &Primeiro) {
    if let Some(primeira) = porta.primeira.take() {
        let cd = ((primeira >> 14) & 3) as u8 | (((w >> 2) & 0x3c) as u8);
        if let Some(a) = alvo(cd) {
            inv.vdp_comandos
                .entry(a.to_string())
                .or_default()
                .somar(onde);
        }
        if cd & 0x20 != 0 {
            inv.vdp_dma
                .entry(modo_dma(porta.reg23).to_string())
                .or_default()
                .somar(onde);
        }
        return;
    }
    if w & 0xe000 == 0x8000 {
        let reg = ((w >> 8) & 0x1f) as u8;
        let val = (w & 0xff) as u8;
        if reg == 23 {
            porta.reg23 = val;
        }
        inv.vdp_registradores
            .entry((reg, val))
            .or_default()
            .somar(onde);
    } else {
        porta.primeira = Some(w);
    }
}

fn linha(md: &mut String, nome: &str, c: &Contagem) {
    let p = c.primeiro.as_ref();
    let _ = writeln!(
        md,
        "| {nome} | {} | {} | {} | {} |",
        c.vezes,
        p.map_or("-", |p| p.caso.as_str()),
        p.map_or(0, |p| p.quadro),
        p.map_or(0, |p| p.seq)
    );
}

fn tabela<K: Ord, F: Fn(&K) -> String>(
    md: &mut String,
    titulo: &str,
    mapa: &BTreeMap<K, Contagem>,
    nome: F,
) {
    let _ = writeln!(md, "\n## {titulo}\n");
    if mapa.is_empty() {
        let _ = writeln!(md, "Nenhum item observado no corpus.");
        return;
    }
    let _ = writeln!(
        md,
        "| Item | Vezes | Primeiro caso | Quadro | seq |\n| --- | --- | --- | --- | --- |"
    );
    for (k, c) in mapa {
        linha(md, &nome(k), c);
    }
}

/// Gera o Markdown. Determinístico: mesma entrada, mesmo texto.
pub fn renderizar(inv: &Inventario, casos: &[String]) -> String {
    let mut md = String::from(
        "# Inventário de hardware\n\nGerado por `cargo xtask inventario`. Só nomes, números e contagens. Cada item cita o primeiro caso, quadro e `seq` em que apareceu no corpus.\n",
    );
    let _ = writeln!(
        md,
        "\nCasos lidos ({}): {}\n",
        casos.len(),
        casos.join(", ")
    );
    md.push_str("Fora deste inventário: DMA como evento próprio (o trace não o registra) e E/S do 68000 (pedido P2).\n");
    tabela(
        &mut md,
        "VDP: registradores (registrador, valor)",
        &inv.vdp_registradores,
        |(r, v)| format!("reg {r} = {v:#04x}"),
    );
    tabela(
        &mut md,
        "VDP: comandos de endereço",
        &inv.vdp_comandos,
        |k| k.clone(),
    );
    tabela(&mut md, "VDP: DMA disparado", &inv.vdp_dma, |k| k.clone());
    tabela(
        &mut md,
        "VDP: escritas na porta de dados (bytes por escrita)",
        &inv.vdp_dados,
        |k| format!("{k} byte(s)"),
    );
    tabela(
        &mut md,
        "YM2612: registradores escritos (porta, registrador)",
        &inv.ym_registradores,
        |(p, r)| format!("porta {p}, reg {r:#04x}"),
    );
    let mut dac = BTreeMap::new();
    if inv.ym_dac.vezes > 0 {
        dac.insert(
            "escritas no registrador $2A (DAC)".to_string(),
            Contagem {
                vezes: inv.ym_dac.vezes,
                primeiro: inv.ym_dac.primeiro.clone(),
            },
        );
    }
    tabela(&mut md, "YM2612: uso do DAC", &dac, |k| k.clone());
    tabela(&mut md, "PSG", &inv.psg, |k| k.clone());
    tabela(
        &mut md,
        "Interrupções do 68000 (nível)",
        &inv.int68,
        |n| format!("nível {n}"),
    );
    tabela(
        &mut md,
        "Z80: BUSREQ e RESET (sinal 0=busreq 1=reset, ativo)",
        &inv.z80,
        |(s, a)| format!("sinal {s}, ativo {a}"),
    );
    md
}

pub fn executar(args: &[String]) -> Result<(), String> {
    let opc = |n: &str| {
        args.iter()
            .position(|a| a == n)
            .and_then(|i| args.get(i + 1))
            .map(String::as_str)
    };
    let dir = opc("--traces").ok_or("falta --traces")?;
    let saida = opc("--saida").unwrap_or("docs/inventario-hardware.md");
    let mut arquivos: Vec<_> = fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".a.s1t"))
        })
        .collect();
    arquivos.sort();
    let mut inv = Inventario::default();
    let mut casos = Vec::new();
    for p in &arquivos {
        let nome = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .trim_end_matches(".a.s1t")
            .to_string();
        let f = fs::File::open(p).map_err(|e| e.to_string())?;
        acumular(
            &mut inv,
            &nome,
            Leitor::abrir(std::io::BufReader::new(f)).map_err(|e| e.to_string())?,
        )?;
        casos.push(nome);
    }
    let md = renderizar(&inv, &casos);
    filtro(&md)?;
    fs::write(Path::new(&super::raiz()).join(saida), md).map_err(|e| e.to_string())?;
    println!(
        "inventário: {} casos, {} registradores da VDP distintos",
        casos.len(),
        inv.vdp_registradores.len()
    );
    Ok(())
}

/// Recusa linha com sequência longa de hexadecimais (possível dado da ROM) antes de escrever.
pub fn filtro(md: &str) -> Result<(), String> {
    for l in md.lines() {
        let mut seq = 0usize;
        for c in l.chars() {
            seq = if c.is_ascii_hexdigit() {
                seq.saturating_add(1)
            } else {
                0
            };
            if seq >= 16 {
                return Err(format!("linha suspeita de dado: {l}"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    use sonic1_fidelidade::{Cabecalho, Categoria, Escritor, Evento, Origem, Quadro};

    fn ev(seq: u32, tipo: Tipo, a: u32, b: u32, c: u32, d: u32) -> Evento {
        Evento {
            mclk: 0,
            seq,
            tipo,
            origem: Origem::Cpu68,
            linha: 0,
            coluna: 0,
            a,
            b,
            c,
            d,
        }
    }

    fn trace(eventos: Vec<Evento>) -> Vec<u8> {
        let cab = Cabecalho {
            oraculo: "t".into(),
            rom_sha1: [0; 20],
            config_sha1: [0; 20],
            entradas_sha1: [0; 20],
            modo: 0,
        };
        let mut buf = Vec::new();
        let mut e = Escritor::novo(&mut buf, &cab).unwrap();
        e.escrever(&Quadro {
            numero: 7,
            mclk_fim: 0,
            eventos,
            hashes: [[0; 20]; Categoria::TOTAL],
        })
        .unwrap();
        e.finalizar(&cab).unwrap();
        buf
    }

    #[test]
    fn decodifica_registrador_comando_e_dma() {
        let eventos = vec![
            ev(0, Tipo::Vdp, 1, 0x8174, 2, 0), // reg 1 = 0x74
            ev(1, Tipo::Vdp, 1, 0x9780, 2, 0), // reg 23 = 0x80 (preenchimento)
            ev(2, Tipo::Vdp, 1, 0x4000, 2, 0), // 1a palavra: escrita em VRAM
            ev(3, Tipo::Vdp, 1, 0x0080, 2, 0), // 2a palavra com bit de DMA
            ev(4, Tipo::Vdp, 0, 0x1234, 2, 0),
            ev(5, Tipo::Int68, 6, 0, 0, 0),
        ];
        let mut inv = Inventario::default();
        acumular(&mut inv, "c1", Leitor::abrir(&trace(eventos)[..]).unwrap()).unwrap();
        assert_eq!(inv.vdp_registradores[&(1, 0x74)].vezes, 1);
        assert_eq!(
            inv.vdp_registradores[&(23, 0x80)].primeiro,
            Some(Primeiro {
                caso: "c1".into(),
                quadro: 7,
                seq: 1
            })
        );
        assert_eq!(inv.vdp_comandos["escrita em VRAM"].vezes, 1);
        assert_eq!(inv.vdp_dma["preenchimento de VRAM"].vezes, 1);
        assert_eq!(inv.vdp_dados[&2].vezes, 1);
        assert_eq!(inv.int68[&6].vezes, 1);
    }

    #[test]
    fn ym_psg_e_z80() {
        let eventos = vec![
            ev(0, Tipo::Ym, 0, 0x2a, 0, 0),         // endereço: ignorado
            ev(1, Tipo::Ym, 1, 0x80, 0x2a, 1),      // dado no DAC
            ev(2, Tipo::Psg, 0x9f, 0, 0, 0),        // volume canal 0
            ev(3, Tipo::Psg, 0xe5, 0, 0, 0),        // ruído
            ev(4, Tipo::BarramentoZ80, 0, 1, 0, 0), // busreq ativo
        ];
        let mut inv = Inventario::default();
        acumular(&mut inv, "c", Leitor::abrir(&trace(eventos)[..]).unwrap()).unwrap();
        assert_eq!(inv.ym_registradores.len(), 1);
        assert_eq!(inv.ym_dac.vezes, 1);
        assert!(inv.psg.contains_key("volume, canal 0"));
        assert!(inv.psg.contains_key("ruído, canal 3"));
        assert_eq!(inv.z80[&(0, 1)].vezes, 1);
    }

    #[test]
    fn saida_deterministica_e_item_nao_observado_nao_entra() {
        let mut inv = Inventario::default();
        acumular(
            &mut inv,
            "c",
            Leitor::abrir(&trace(vec![ev(0, Tipo::Int68, 6, 0, 0, 0)])[..]).unwrap(),
        )
        .unwrap();
        let a = renderizar(&inv, &["c".into()]);
        assert_eq!(a, renderizar(&inv, &["c".into()]));
        assert!(a.contains("nível 6"));
        assert!(a.contains("Nenhum item observado"));
        assert!(!a.contains("nível 4"));
        assert!(filtro(&a).is_ok());
    }

    #[test]
    fn filtro_recusa_hexadecimal_longo() {
        assert!(filtro("| x | 0123456789abcdef0 |").is_err());
        assert!(filtro("| comum | 12 |").is_ok());
    }
}
