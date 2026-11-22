//! Inventários da Etapa 5, gerados dos arquivos que o oráculo grava (patch 0004):
//!
//! - `inventario-68000`: classe e tamanho das instruções executadas (`oraculo rodar --instrucoes`);
//! - `inventario-barramento`: acessos fora da RAM e exceções (`--barramento`);
//! - `inventario-interrupcoes`: que rotinas o corpus interrompe (`--tempo` e o mapa de rótulos).
//!
//! Saída só com nomes, endereços e contagens.
//!
//! Uso:
//! `cargo xtask inventario-68000 --entrada A.ins [--entrada B.ins ...] [--saida docs/inventario-68000.md]`
//! `cargo xtask inventario-barramento --entrada A.bar ... [--saida docs/inventario-barramento.md]`
//! `cargo xtask inventario-interrupcoes --entrada A.s1tempo ... [--rotulos refs/rotulos.txt]
//!   [--saida docs/inventario-interrupcoes.md]`
use sonic1_core::quadro::tempo::{EventoTempo, TempoGravado};
use std::collections::BTreeMap;
use std::fs;

fn opcoes<'a>(args: &'a [String], nome: &str) -> Vec<&'a str> {
    args.windows(2)
        .filter(|w| w[0] == nome)
        .map(|w| w[1].as_str())
        .collect()
}

fn unica<'a>(args: &'a [String], nome: &str, padrao: &'a str) -> &'a str {
    opcoes(args, nome).first().copied().unwrap_or(padrao)
}

fn so_nomes(texto: &str, onde: &str) -> Result<(), String> {
    // Nomes de classe e de região: letras, dígitos e sublinhado. Recusa qualquer outra coisa.
    if texto.is_empty()
        || !texto
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(format!("{onde}: campo fora do formato de nome: {texto:?}"));
    }
    Ok(())
}

/// Soma linhas `campo1<TAB>campo2<TAB>contagem` de vários arquivos.
pub fn somar_ins(textos: &[String]) -> Result<BTreeMap<(String, String), u64>, String> {
    let mut m = BTreeMap::new();
    for t in textos {
        for (i, l) in t.lines().enumerate() {
            let c: Vec<&str> = l.split('\t').collect();
            let [nome, tamanho, n] = c[..] else {
                return Err(format!("linha {}: esperados 3 campos", i.saturating_add(1)));
            };
            so_nomes(nome, "classe")?;
            so_nomes(tamanho, "tamanho")?;
            let n: u64 = n
                .parse()
                .map_err(|_| format!("linha {}: contagem", i.saturating_add(1)))?;
            let e = m
                .entry((nome.to_string(), tamanho.to_string()))
                .or_insert(0u64);
            *e = e.saturating_add(n);
        }
    }
    Ok(m)
}

pub fn renderizar_68000(m: &BTreeMap<(String, String), u64>, fontes: &[String]) -> String {
    let mut md = String::from("# Inventário de instruções do 68000\n\n");
    md.push_str(
        "Gerado por `cargo xtask inventario-68000`. Só nome de classe, tamanho e contagem.\n\n",
    );
    md.push_str(&format!(
        "Fonte: `oraculo rodar --instrucoes`, casos: {}.\n\n",
        fontes.join(", ")
    ));
    md.push_str("A classe é o nome da variante do decodificador do oráculo (jgenesis). O tamanho é `-` quando a classe não tem.\n\n");
    md.push_str(&format!("{} classes distintas.\n\n", m.len()));
    md.push_str("| classe | tamanho | vezes |\n| --- | --- | --- |\n");
    for ((n, t), c) in m {
        md.push_str(&format!("| {n} | {t} | {c} |\n"));
    }
    md
}

/// Linhas `regiao<TAB>operacao<TAB>bytes<TAB>contagem`.
pub fn somar_bar(textos: &[String]) -> Result<BTreeMap<(String, String, String), u64>, String> {
    let mut m = BTreeMap::new();
    for t in textos {
        for (i, l) in t.lines().enumerate() {
            let c: Vec<&str> = l.split('\t').collect();
            let [r, o, b, n] = c[..] else {
                return Err(format!("linha {}: esperados 4 campos", i.saturating_add(1)));
            };
            so_nomes(r, "regiao")?;
            so_nomes(o, "operacao")?;
            so_nomes(b, "bytes")?;
            let n: u64 = n
                .parse()
                .map_err(|_| format!("linha {}: contagem", i.saturating_add(1)))?;
            let e = m
                .entry((r.to_string(), o.to_string(), b.to_string()))
                .or_insert(0u64);
            *e = e.saturating_add(n);
        }
    }
    Ok(m)
}

pub fn renderizar_barramento(
    m: &BTreeMap<(String, String, String), u64>,
    fontes: &[String],
) -> String {
    let mut md = String::from("# Inventário do barramento do 68000\n\n");
    md.push_str("Gerado por `cargo xtask inventario-barramento`. Só região, operação, tamanho em bytes e contagem.\n\n");
    md.push_str(&format!(
        "Fonte: `oraculo rodar --barramento`, casos: {}.\n\n",
        fontes.join(", ")
    ));
    md.push_str("A região `ram` conta os acessos fora do DMA; `sem_dispositivo` é o acesso a endereço sem nada ligado (leitura devolve barramento aberto, escrita é ignorada). Na linha `excecao`, o campo de bytes é o número do vetor.\n\n");
    md.push_str("| região | operação | bytes | vezes |\n| --- | --- | --- | --- |\n");
    for ((r, o, b), c) in m {
        md.push_str(&format!("| {r} | {o} | {b} | {c} |\n"));
    }
    md
}

/// Rotinas do mapa (tipo `rotina`), ordenadas por endereço: `(endereço, nome)`.
pub fn rotinas_do_mapa(rotulos: &str) -> Vec<(u32, String)> {
    let mut v: Vec<(u32, String)> = rotulos
        .lines()
        .filter_map(|l| {
            let c: Vec<&str> = l.split('\t').collect();
            let [nome, "rotina", valor] = c[..] else {
                return None;
            };
            Some((u32::from_str_radix(valor, 16).ok()?, nome.to_string()))
        })
        .collect();
    v.sort();
    v
}

/// A rotina que contém `pc`: a de maior endereço menor ou igual a ele.
pub fn rotina_de(rotinas: &[(u32, String)], pc: u32) -> Option<(&str, u32)> {
    let i = rotinas.partition_point(|(a, _)| *a <= pc).checked_sub(1)?;
    let (a, n) = rotinas.get(i)?;
    Some((n.as_str(), pc.wrapping_sub(*a)))
}

/// Por rotina interrompida e nível: quantas vezes, menor e maior deslocamento.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Ocorrencia {
    pub vezes: u64,
    pub menor: u32,
    pub maior: u32,
}

pub type Interrupcoes = BTreeMap<(String, u8), Ocorrencia>;

/// Quadros de lag e quadros com VBlank, pela rotina de espera.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct ResumoLag {
    pub quadros: u64,
    pub com_vblank: u64,
    pub lag: u64,
}

pub fn acumular_interrupcoes(
    tempo: &TempoGravado,
    rotinas: &[(u32, String)],
    espera: &str,
    acc: &mut Interrupcoes,
    lag: &mut ResumoLag,
) {
    for q in &tempo.quadros {
        lag.quadros = lag.quadros.saturating_add(1);
        let mut tem_vblank = false;
        let mut fora = false;
        for e in &q.eventos {
            if let EventoTempo::Interrupcao {
                nivel,
                pc_interrompido,
                ..
            } = *e
            {
                let (nome, desloc) =
                    rotina_de(rotinas, pc_interrompido).unwrap_or(("(sem rotina)", 0));
                let o = acc.entry((nome.to_string(), nivel)).or_insert(Ocorrencia {
                    vezes: 0,
                    menor: u32::MAX,
                    maior: 0,
                });
                o.vezes = o.vezes.saturating_add(1);
                o.menor = o.menor.min(desloc);
                o.maior = o.maior.max(desloc);
                if nivel == 6 {
                    tem_vblank = true;
                    fora |= nome != espera;
                }
            }
        }
        if tem_vblank {
            lag.com_vblank = lag.com_vblank.saturating_add(1);
            if fora {
                lag.lag = lag.lag.saturating_add(1);
            }
        }
    }
}

pub fn renderizar_interrupcoes(
    acc: &Interrupcoes,
    lag: &ResumoLag,
    espera: &str,
    fontes: &[String],
) -> String {
    let mut md = String::from("# Inventário de interrupções por rotina\n\n");
    md.push_str("Gerado por `cargo xtask inventario-interrupcoes`. Só nomes de rotina, níveis, contagens e deslocamentos dentro da rotina.\n\n");
    md.push_str(&format!(
        "Fonte: arquivos `.s1tempo` dos casos: {}.\n\n",
        fontes.join(", ")
    ));
    md.push_str(&format!(
        "Quadros: {}. Com VBlank gravado: {}. Quadros de lag (VBlank com o PC fora de `{espera}`): {}.\n\n",
        lag.quadros, lag.com_vblank, lag.lag
    ));
    md.push_str("Cada linha é uma rotina do original que o corpus interrompeu. É a lista de onde os pontos de checagem podem ser necessários: uma rotina que lê ou escreve memória compartilhada (`docs/inventario-memoria.md`) e aparece aqui precisa de ponto.\n\n");
    md.push_str("| rotina | nível | vezes | menor deslocamento | maior deslocamento |\n| --- | --- | --- | --- | --- |\n");
    let mut linhas: Vec<_> = acc.iter().collect();
    linhas.sort_by(|a, b| b.1.vezes.cmp(&a.1.vezes).then(a.0.cmp(b.0)));
    for ((r, n), o) in linhas {
        md.push_str(&format!(
            "| {r} | {n} | {} | {:#x} | {:#x} |\n",
            o.vezes, o.menor, o.maior
        ));
    }
    md
}

fn ler_todos(args: &[String]) -> Result<(Vec<String>, Vec<String>), String> {
    let arquivos = opcoes(args, "--entrada");
    if arquivos.is_empty() {
        return Err("falta --entrada".into());
    }
    let mut textos = Vec::new();
    let mut nomes = Vec::new();
    for a in arquivos {
        textos.push(fs::read_to_string(a).map_err(|e| format!("{a}: {e}"))?);
        nomes.push(
            std::path::Path::new(a)
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string(),
        );
    }
    Ok((textos, nomes))
}

fn gravar(md: &str, saida: &str) -> Result<(), String> {
    super::inventario::filtro(md)?;
    fs::write(super::raiz().join(saida), md).map_err(|e| e.to_string())
}

pub fn instrucoes(args: &[String]) -> Result<(), String> {
    let (textos, nomes) = ler_todos(args)?;
    let m = somar_ins(&textos)?;
    gravar(
        &renderizar_68000(&m, &nomes),
        unica(args, "--saida", "docs/inventarios/inventario-68000.md"),
    )?;
    println!("inventário do 68000: {} classes", m.len());
    Ok(())
}

pub fn barramento(args: &[String]) -> Result<(), String> {
    let (textos, nomes) = ler_todos(args)?;
    let m = somar_bar(&textos)?;
    gravar(
        &renderizar_barramento(&m, &nomes),
        unica(args, "--saida", "docs/inventarios/inventario-barramento.md"),
    )?;
    println!("inventário do barramento: {} linhas", m.len());
    Ok(())
}

pub fn interrupcoes(args: &[String]) -> Result<(), String> {
    let arquivos = opcoes(args, "--entrada");
    if arquivos.is_empty() {
        return Err("falta --entrada".into());
    }
    let mapa = fs::read_to_string(super::raiz().join(unica(args, "--rotulos", "refs/rotulos.txt")))
        .map_err(|e| e.to_string())?;
    let rotinas = rotinas_do_mapa(&mapa);
    let espera = unica(args, "--espera", "WaitForVBlank");
    let mut acc = Interrupcoes::new();
    let mut lag = ResumoLag::default();
    let mut nomes = Vec::new();
    for a in arquivos {
        let bytes = fs::read(a).map_err(|e| format!("{a}: {e}"))?;
        let t = TempoGravado::de_bytes(&bytes).map_err(|e| format!("{a}: {e:?}"))?;
        acumular_interrupcoes(&t, &rotinas, espera, &mut acc, &mut lag);
        nomes.push(
            std::path::Path::new(a)
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string(),
        );
    }
    gravar(
        &renderizar_interrupcoes(&acc, &lag, espera, &nomes),
        unica(args, "--saida", "docs/inventarios/inventario-interrupcoes.md"),
    )?;
    println!(
        "inventário de interrupções: {} rotinas/níveis, {} quadros, {} com lag",
        acc.len(),
        lag.quadros,
        lag.lag
    );
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    use sonic1_core::quadro::tempo::TempoQuadro;

    #[test]
    fn soma_instrucoes_de_varios_casos_e_recusa_lixo() {
        let a = "Add\tByte\t3\nBranch\t-\t10\n".to_string();
        let b = "Add\tByte\t4\n".to_string();
        let m = somar_ins(&[a, b]).unwrap();
        assert_eq!(m[&("Add".to_string(), "Byte".to_string())], 7);
        assert!(somar_ins(&["Add d1,d0\tByte\t3".to_string()]).is_err());
        assert!(somar_ins(&["Add\tByte".to_string()]).is_err());
        let md = renderizar_68000(&m, &["caso".into()]);
        assert!(md.contains("| Add | Byte | 7 |"));
    }

    #[test]
    fn barramento_soma_e_renderiza() {
        let m = somar_bar(&["ram\tleitura\t2\t5\nexcecao\tvetor\t3\t1\n".to_string()]).unwrap();
        let md = renderizar_barramento(&m, &["c".into()]);
        assert!(md.contains("| excecao | vetor | 3 | 1 |"));
        assert!(somar_bar(&["ram\tleitura\t2\tx".to_string()]).is_err());
    }

    #[test]
    fn interrupcoes_por_rotina_e_lag() {
        let rotinas = rotinas_do_mapa(
            "WaitForVBlank\trotina\t29a0\nGameInit\trotina\t364\nLoop\trotina\t3000\nLoop.x\tconstante\t3010\n",
        );
        assert_eq!(rotina_de(&rotinas, 0x29A4), Some(("WaitForVBlank", 4)));
        assert_eq!(rotina_de(&rotinas, 0x3012), Some(("Loop", 0x12)));
        assert_eq!(rotina_de(&rotinas, 0x10), None);
        let int = |pc| EventoTempo::Interrupcao {
            nivel: 6,
            pc_interrompido: pc,
            pc_tratador: 0,
            mclk: 0,
            linha: 0,
            coluna: 0,
        };
        let t = TempoGravado {
            quadros: vec![
                TempoQuadro {
                    numero: 0,
                    mclk_inicio: 0,
                    mclk_fim: 1,
                    eventos: vec![int(0x29A4)],
                },
                TempoQuadro {
                    numero: 1,
                    mclk_inicio: 1,
                    mclk_fim: 2,
                    eventos: vec![int(0x3004)],
                },
                TempoQuadro {
                    numero: 2,
                    mclk_inicio: 2,
                    mclk_fim: 3,
                    eventos: vec![],
                },
            ],
        };
        let (mut acc, mut lag) = (Interrupcoes::new(), ResumoLag::default());
        acumular_interrupcoes(&t, &rotinas, "WaitForVBlank", &mut acc, &mut lag);
        assert_eq!(
            lag,
            ResumoLag {
                quadros: 3,
                com_vblank: 2,
                lag: 1
            }
        );
        assert_eq!(
            acc[&("Loop".to_string(), 6)],
            Ocorrencia {
                vezes: 1,
                menor: 4,
                maior: 4
            }
        );
        let md = renderizar_interrupcoes(&acc, &lag, "WaitForVBlank", &["c".into()]);
        assert!(md.contains("| Loop | 6 | 1 | 0x4 | 0x4 |"));
    }
}
