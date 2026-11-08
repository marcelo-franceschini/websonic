//! `cargo fidelidade cobertura`: cobertura das decisões do núcleo contra os desvios do original
//! (sub-etapa 5.6). Mede se o jogo está certo, por isso mora neste pacote congelado.
//!
//! Entradas (todas só com nomes, endereços e contagens):
//! - decisões do núcleo: lidas do código-fonte (`decisao!("Rotina:nome", ...)` em `crates/core`);
//! - `--nucleo ARQ`: `id<TAB>verdadeiras<TAB>falsas`, gravado pelos testes de cobertura
//!   (`WEBSONIC_COBERTURA_ARQ`) ou pelo `diferencial rotina --cobertura-nucleo`;
//! - `--executados ARQ` (pode repetir): `endereco_hex<TAB>tomado<TAB>nao_tomado`, dos desvios condicionais do
//!   original executados nos traces e nos testes por rotina (`cobertura extrair`);
//! - `--mapa ARQ`: `id<TAB>endereco_hex<TAB>direto|inverso`, decisão do núcleo para desvio do
//!   original (`cobertura/mapa-decisoes.txt`).
//!
//! Universo: o requisito mede os desvios EXECUTADOS pelo oráculo. Um desvio que nenhum caso
//! executou não aparece aqui; achá-lo exige decodificar o código sem executar (limite registrado
//! em `docs/conhecimento/nucleo.md` e no backlog).
//!
//! Uso:
//! `cargo fidelidade cobertura extrair --traces DIR --saida ARQ`
//! `cargo fidelidade cobertura relatorio --nucleo ARQ --executados ARQ --mapa ARQ [--saida ARQ]`
use crate::{opcao, raiz};
use sonic1_fidelidade::{Leitor, Tipo};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Contagens por decisão do núcleo: (verdadeiras, falsas).
pub type ContagensNucleo = BTreeMap<String, (u64, u64)>;
/// Desvios do original: endereço para (tomado, não tomado).
pub type Executados = BTreeMap<u32, (u64, u64)>;
/// Mapa: decisão para (endereço, inverso).
pub type Mapa = BTreeMap<String, (u32, bool)>;

/// Ids de `decisao!("...")` num texto de código-fonte.
pub fn ids_do_fonte(fonte: &str) -> Vec<String> {
    fonte
        .split("decisao!(\"")
        .skip(1)
        .filter_map(|p| p.find('"').map(|f| p[..f].to_string()))
        .collect()
}

fn arquivos_rs(dir: &Path, saida: &mut Vec<std::path::PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    let mut entradas: Vec<_> = rd.filter_map(Result::ok).map(|e| e.path()).collect();
    entradas.sort();
    for p in entradas {
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n != "target") {
                arquivos_rs(&p, saida);
            }
        } else if p.extension().is_some_and(|e| e == "rs") {
            saida.push(p);
        }
    }
}

/// Decisões declaradas no núcleo, ordenadas e sem repetição. O módulo `cobertura` define a macro
/// e não conta.
pub fn ids_do_nucleo(raiz: &Path) -> Vec<String> {
    let mut arqs = Vec::new();
    arquivos_rs(&raiz.join("crates/core/src"), &mut arqs);
    let mut ids = Vec::new();
    for a in arqs {
        if a.components().any(|c| c.as_os_str() == "cobertura") {
            continue;
        }
        if let Ok(t) = fs::read_to_string(&a) {
            ids.extend(ids_do_fonte(&t));
        }
    }
    ids.sort();
    ids.dedup();
    ids
}

pub fn ler_contagens(texto: &str) -> Result<ContagensNucleo, String> {
    let mut m = ContagensNucleo::new();
    for (i, l) in texto.lines().enumerate() {
        if l.is_empty() {
            continue;
        }
        let c: Vec<&str> = l.split('\t').collect();
        let [id, v, f] = c[..] else {
            return Err(format!("nucleo, linha {}: 3 campos", i + 1));
        };
        let e = m.entry(id.to_string()).or_insert((0, 0));
        e.0 += v
            .parse::<u64>()
            .map_err(|_| format!("nucleo, linha {}: contagem", i + 1))?;
        e.1 += f
            .parse::<u64>()
            .map_err(|_| format!("nucleo, linha {}: contagem", i + 1))?;
    }
    Ok(m)
}

pub fn ler_executados(texto: &str) -> Result<Executados, String> {
    let mut m = Executados::new();
    for (i, l) in texto.lines().enumerate() {
        if l.is_empty() {
            continue;
        }
        let c: Vec<&str> = l.split('\t').collect();
        let [end, t, n] = c[..] else {
            return Err(format!("executados, linha {}: 3 campos", i + 1));
        };
        let end = u32::from_str_radix(end, 16)
            .map_err(|_| format!("executados, linha {}: endereco", i + 1))?;
        let e = m.entry(end).or_insert((0, 0));
        e.0 += t
            .parse::<u64>()
            .map_err(|_| format!("executados, linha {}: contagem", i + 1))?;
        e.1 += n
            .parse::<u64>()
            .map_err(|_| format!("executados, linha {}: contagem", i + 1))?;
    }
    Ok(m)
}

pub fn ler_mapa(texto: &str) -> Result<Mapa, String> {
    let mut m = Mapa::new();
    for (i, l) in texto.lines().enumerate() {
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        let c: Vec<&str> = l.split('\t').collect();
        let [id, end, pol] = c[..] else {
            return Err(format!("mapa, linha {}: 3 campos", i + 1));
        };
        let end =
            u32::from_str_radix(end, 16).map_err(|_| format!("mapa, linha {}: endereco", i + 1))?;
        let inverso = match pol {
            "direto" => false,
            "inverso" => true,
            _ => return Err(format!("mapa, linha {}: polaridade", i + 1)),
        };
        if m.insert(id.to_string(), (end, inverso)).is_some() {
            return Err(format!("mapa, linha {}: decisao repetida", i + 1));
        }
    }
    Ok(m)
}

/// Resultado do relatório: texto e a lista de problemas que reprovam.
pub struct Relatorio {
    pub texto: String,
    pub problemas: Vec<String>,
}

/// Cruza decisões do núcleo, contagens, mapa e desvios executados no original.
pub fn relatorio(
    ids: &[String],
    nucleo: &ContagensNucleo,
    mapa: &Mapa,
    exec: &Executados,
) -> Relatorio {
    let mut texto = String::from(
        "id\tnucleo_verdadeira\tnucleo_falsa\tendereco\tpolaridade\toriginal_tomado\toriginal_nao_tomado\tstatus\n",
    );
    let mut problemas = Vec::new();
    let (mut nos_dois, mut sem_mapa) = (0usize, 0usize);
    for id in ids {
        let (v, f) = nucleo.get(id).copied().unwrap_or((0, 0));
        let mut status = Vec::new();
        if v == 0 && f == 0 {
            status.push("nunca-executada-no-nucleo");
            problemas.push(format!("{id}: nenhum teste executou esta decisao"));
        } else if v == 0 || f == 0 {
            status.push("so-um-caminho-no-nucleo");
            problemas.push(format!(
                "{id}: so um caminho no nucleo ({v} verdadeiras, {f} falsas)"
            ));
        } else {
            nos_dois += 1;
        }
        let (end, pol, ot, on) = match mapa.get(id) {
            Some((end, inv)) => {
                let (t, n) = exec.get(end).copied().unwrap_or((0, 0));
                if t == 0 && n == 0 {
                    status.push("desvio-nao-executado-no-original");
                    problemas.push(format!(
                        "{id}: o desvio {end:X} do mapa nao foi executado no oraculo"
                    ));
                } else if t == 0 || n == 0 {
                    status.push("so-um-caminho-no-original");
                }
                (
                    format!("{end:X}"),
                    if *inv { "inverso" } else { "direto" },
                    t,
                    n,
                )
            }
            None => {
                sem_mapa += 1;
                status.push("sem-mapa");
                ("-".to_string(), "-", 0, 0)
            }
        };
        if status.is_empty() {
            status.push("ok");
        }
        texto.push_str(&format!(
            "{id}\t{v}\t{f}\t{end}\t{pol}\t{ot}\t{on}\t{}\n",
            status.join(",")
        ));
    }
    for id in mapa.keys() {
        if !ids.contains(id) {
            problemas.push(format!(
                "{id}: o mapa cita uma decisao que nao existe no nucleo"
            ));
        }
    }
    let so_um_original = exec
        .values()
        .filter(|(t, n)| (*t == 0) != (*n == 0))
        .count();
    texto.push_str(&format!(
        "\n# decisoes do nucleo: {}; nos dois caminhos nos testes: {nos_dois}; sem mapa para o original: {sem_mapa}\n# desvios do original executados: {}; executados em um caminho so: {so_um_original}\n# problemas: {}\n",
        ids.len(),
        exec.len(),
        problemas.len()
    ));
    Relatorio { texto, problemas }
}

/// Soma os desvios condicionais (Bcc, DBcc) do 68000 de todos os traces `*.s1t` da pasta.
pub fn extrair_executados(dir: &Path) -> Result<Executados, String> {
    let mut m = Executados::new();
    let mut arqs: Vec<_> = fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".a.s1t"))
        })
        .collect();
    arqs.sort();
    for a in arqs {
        let f = fs::File::open(&a).map_err(|e| e.to_string())?;
        let mut l = Leitor::abrir(std::io::BufReader::new(f)).map_err(|e| e.to_string())?;
        while let Some(q) = l.proximo().map_err(|e| e.to_string())? {
            for e in &q.eventos {
                if e.tipo == Tipo::Desvio68 && matches!(e.c, 2 | 4) {
                    let x = m.entry(e.a).or_insert((0, 0));
                    if e.d != 0 {
                        x.0 += 1;
                    } else {
                        x.1 += 1;
                    }
                }
            }
        }
    }
    Ok(m)
}

fn escrever_executados(m: &Executados) -> String {
    m.iter()
        .map(|(a, (t, n))| format!("{a:X}\t{t}\t{n}\n"))
        .collect()
}

pub fn executar(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("extrair") => {
            let m =
                extrair_executados(Path::new(opcao(args, "--traces").ok_or("falta --traces")?))?;
            fs::write(
                opcao(args, "--saida").ok_or("falta --saida")?,
                escrever_executados(&m),
            )
            .map_err(|e| e.to_string())?;
            println!("desvios condicionais executados: {}", m.len());
            Ok(())
        }
        Some("relatorio") => {
            let ler = |n: &str| {
                fs::read_to_string(opcao(args, n).ok_or(format!("falta {n}"))?)
                    .map_err(|e| e.to_string())
            };
            let nucleo = ler_contagens(&ler("--nucleo")?)?;
            // --executados pode repetir: traces do corpus e testes por rotina somam.
            let mut exec = Executados::new();
            let arquivos: Vec<&str> = args
                .windows(2)
                .filter(|w| w[0] == "--executados")
                .map(|w| w[1].as_str())
                .collect();
            if arquivos.is_empty() {
                return Err("falta --executados".into());
            }
            for a in arquivos {
                for (end, (t, n)) in
                    ler_executados(&fs::read_to_string(a).map_err(|e| format!("{a}: {e}"))?)?
                {
                    let x = exec.entry(end).or_insert((0, 0));
                    x.0 += t;
                    x.1 += n;
                }
            }
            let mapa = ler_mapa(&ler("--mapa")?)?;
            let r = relatorio(&ids_do_nucleo(&raiz()), &nucleo, &mapa, &exec);
            print!("{}", r.texto);
            if let Some(s) = opcao(args, "--saida") {
                fs::write(s, &r.texto).map_err(|e| e.to_string())?;
            }
            for p in &r.problemas {
                println!("problema: {p}");
            }
            if r.problemas.is_empty() {
                Ok(())
            } else {
                Err("cobertura reprovada".into())
            }
        }
        _ => Err("uso: cargo fidelidade cobertura <extrair|relatorio>".into()),
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn ids() -> Vec<String> {
        vec!["A:x".into(), "A:y".into(), "B:z".into()]
    }

    #[test]
    fn le_os_ids_do_fonte() {
        assert_eq!(
            ids_do_fonte("if decisao!(\"A:x\", c) {} decisao!(\"B:z\", d)"),
            vec!["A:x", "B:z"]
        );
        assert!(ids_do_fonte("sem decisoes").is_empty());
    }

    #[test]
    fn cobertura_completa_passa() {
        let n = ler_contagens("A:x\t3\t2\nA:y\t1\t1\nB:z\t5\t5\n").unwrap();
        let m = ler_mapa("A:x\t406\tinverso\nA:y\t410\tdireto\nB:z\t500\tdireto\n").unwrap();
        let e = ler_executados("406\t2\t3\n410\t1\t1\n500\t4\t4\n").unwrap();
        let r = relatorio(&ids(), &n, &m, &e);
        assert!(r.problemas.is_empty(), "{:?}", r.problemas);
        assert!(r.texto.contains("A:x\t3\t2\t406\tinverso\t2\t3\tok"));
    }

    #[test]
    fn acusa_decisao_nunca_executada_e_de_um_caminho_so() {
        let n = ler_contagens("A:x\t3\t0\n").unwrap();
        let r = relatorio(&ids(), &n, &Mapa::new(), &Executados::new());
        assert!(
            r.problemas
                .iter()
                .any(|p| p.starts_with("A:x: so um caminho"))
        );
        assert!(
            r.problemas
                .iter()
                .any(|p| p.starts_with("A:y: nenhum teste"))
        );
        assert!(
            r.problemas
                .iter()
                .any(|p| p.starts_with("B:z: nenhum teste"))
        );
        // Sem mapa nao reprova por si so; aparece como sem-mapa.
        assert!(r.texto.contains("sem-mapa"));
    }

    #[test]
    fn acusa_mapa_para_desvio_que_o_oraculo_nao_executou_e_decisao_inexistente() {
        let n = ler_contagens("A:x\t1\t1\nA:y\t1\t1\nB:z\t1\t1\n").unwrap();
        let m = ler_mapa("A:x\t999\tdireto\nFantasma:q\t10\tdireto\n").unwrap();
        let r = relatorio(&ids(), &n, &m, &Executados::new());
        assert!(
            r.problemas
                .iter()
                .any(|p| p.contains("999") && p.contains("nao foi executado"))
        );
        assert!(r.problemas.iter().any(|p| p.starts_with("Fantasma:q")));
    }

    #[test]
    fn formatos_ruins_sao_recusados() {
        assert!(ler_mapa("A:x\t406\ttorto").is_err());
        assert!(ler_mapa("A:x\t406\tdireto\nA:x\t407\tdireto").is_err());
        assert!(ler_executados("zz\t1\t1").is_err());
        assert!(ler_contagens("A:x\t1").is_err());
    }

    #[test]
    fn ids_do_nucleo_ignoram_o_modulo_da_macro() {
        let d = std::env::temp_dir().join(format!("websonic-cob-{}", std::process::id()));
        fs::create_dir_all(d.join("crates/core/src/cobertura")).unwrap();
        fs::create_dir_all(d.join("crates/core/src/rotinas")).unwrap();
        fs::write(
            d.join("crates/core/src/cobertura/mod.rs"),
            "// exemplo: decisao!(\"Doc:x\", c)",
        )
        .unwrap();
        fs::write(
            d.join("crates/core/src/rotinas/a.rs"),
            "decisao!(\"Rot:um\", a); decisao!(\"Rot:um\", b);",
        )
        .unwrap();
        assert_eq!(ids_do_nucleo(&d), vec!["Rot:um"]);
        let _ = fs::remove_dir_all(&d);
    }
}
