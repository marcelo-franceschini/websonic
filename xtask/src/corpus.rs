//! `cargo xtask corpus-trace`: gera os traces do corpus e prova o determinismo (sub-etapa 2.4).
//!
//! Roda no computador do Marcelo (usa a ROM). A saída no terminal tem só nomes, hashes,
//! contagens e vereditos. Traces e estados ficam em `--saida`, fora do repositório.
//!
//! Casos: `corpus/demos.txt` (linhas `nome<TAB>quadros`, entradas vazias) e cada
//! `corpus/<nome>.s1in` com seu `.meta`.
//!
//! Provas por caso:
//! 1. duas execuções geram traces idênticos byte a byte;
//! 2. o estado salvo no quadro K, recarregado, gera os mesmos quadros do trace completo.
//!
//! Uso: `cargo xtask corpus-trace --rom ROM --oraculo BIN --rotulos refs/rotulos.txt
//! --saida DIR [--corpus corpus] [--intervalo 600] [--caso NOME]`
use sonic1_corpus::{ler_s1in, meta, sha1_das_entradas};
use sonic1_fidelidade::hash::em_hex;
use sonic1_fidelidade::{Leitor, comparar_quadros};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Um caso do corpus.
#[derive(Debug, PartialEq, Eq)]
pub struct Caso {
    pub nome: String,
    pub quadros: u32,
    /// Arquivo `.s1in`, ou `None` para demo (nenhum botão).
    pub entradas: Option<PathBuf>,
}

/// Lê `demos.txt`: `nome<TAB>quadros`, `#` comenta.
pub fn ler_demos(texto: &str) -> Result<Vec<Caso>, String> {
    let mut casos = Vec::new();
    for (i, linha) in texto.lines().enumerate() {
        if linha.trim().is_empty() || linha.starts_with('#') {
            continue;
        }
        let (nome, q) = linha.split_once('\t').ok_or(format!(
            "demos.txt linha {}: falta TAB",
            i.saturating_add(1)
        ))?;
        let quadros: u32 = q
            .trim()
            .parse()
            .map_err(|_| format!("demos.txt linha {}: quadros inválido", i.saturating_add(1)))?;
        if nome.is_empty()
            || !nome
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            return Err(format!(
                "demos.txt linha {}: nome inválido",
                i.saturating_add(1)
            ));
        }
        casos.push(Caso {
            nome: nome.to_string(),
            quadros,
            entradas: None,
        });
    }
    Ok(casos)
}

/// Compara a cauda do trace completo (a partir do quadro `desde`) com o trace retomado.
/// Devolve o número do primeiro quadro que difere, ou `None` se tudo é igual.
pub fn comparar_cauda(completo: &[u8], retomado: &[u8], desde: u32) -> Result<Option<u32>, String> {
    let mut a = Leitor::abrir(completo).map_err(|e| e.to_string())?;
    let mut b = Leitor::abrir(retomado).map_err(|e| e.to_string())?;
    loop {
        let qa = a.proximo().map_err(|e| e.to_string())?;
        let Some(qa) = qa else {
            return match b.proximo().map_err(|e| e.to_string())? {
                None => Ok(None),
                Some(extra) => Ok(Some(extra.numero)),
            };
        };
        if qa.numero < desde {
            continue;
        }
        match b.proximo().map_err(|e| e.to_string())? {
            None => return Ok(Some(qa.numero)),
            Some(qb) => {
                if comparar_quadros(&qa, &qb).is_some() {
                    return Ok(Some(qa.numero));
                }
            }
        }
    }
}

fn opcao<'a>(args: &'a [String], nome: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == nome)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

/// Roda o oráculo e devolve a saída padrão.
fn oraculo(bin: &str, args: &[&str]) -> Result<String, String> {
    let s = Command::new(bin)
        .args(args)
        .output()
        .map_err(|e| format!("oráculo: {e}"))?;
    if !s.status.success() {
        return Err(format!(
            "oráculo falhou: {}",
            String::from_utf8_lossy(&s.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&s.stdout).into_owned())
}

fn campo(saida: &str, chave: &str) -> Result<String, String> {
    saida
        .lines()
        .find_map(|l| l.strip_prefix(chave).and_then(|r| r.strip_prefix('=')))
        .map(str::to_string)
        .ok_or(format!("saída do oráculo sem {chave}"))
}

/// Descobre os casos do corpus.
fn descobrir(corpus: &Path, so: Option<&str>) -> Result<Vec<Caso>, String> {
    let mut casos = Vec::new();
    if let Ok(t) = fs::read_to_string(corpus.join("demos.txt")) {
        casos.extend(ler_demos(&t)?);
    }
    let mut nomes: Vec<PathBuf> = fs::read_dir(corpus)
        .map_err(|e| format!("corpus: {e}"))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "s1in"))
        .collect();
    nomes.sort();
    for p in nomes {
        let texto = fs::read_to_string(&p).map_err(|e| e.to_string())?;
        let q = ler_s1in(&texto).map_err(|e| format!("{}: {e:?}", p.display()))?;
        let nome = p
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        // O .meta tem de bater com as entradas.
        let m = fs::read_to_string(p.with_extension("meta"))
            .map_err(|e| format!("{nome}.meta: {e}"))?;
        let campos = meta::ler(&m).map_err(|e| format!("{nome}.meta: {e:?}"))?;
        if meta::valor(&campos, "entradas_sha1") != Some(em_hex(&sha1_das_entradas(&q)).as_str()) {
            return Err(format!("{nome}: entradas_sha1 do .meta não bate"));
        }
        if meta::valor(&campos, "quadros") != Some(q.len().to_string().as_str()) {
            return Err(format!("{nome}: quadros do .meta não bate"));
        }
        let quadros = u32::try_from(q.len()).map_err(|_| "caso grande")?;
        casos.push(Caso {
            nome,
            quadros,
            entradas: Some(p),
        });
    }
    if let Some(nome) = so {
        casos.retain(|c| c.nome == nome);
        if casos.is_empty() {
            return Err(format!("caso {nome} não existe"));
        }
    }
    Ok(casos)
}

/// `cargo xtask corpus-meta --entradas corpus/X.s1in --fonte TEXTO --licenca TEXTO --autor TEXTO`:
/// valida as entradas e escreve o `.meta` ao lado, com os hashes calculados.
pub fn escrever_meta(args: &[String]) -> Result<(), String> {
    let arq = opcao(args, "--entradas").ok_or("falta --entradas")?;
    let q = ler_s1in(&fs::read_to_string(arq).map_err(|e| e.to_string())?)
        .map_err(|e| format!("{arq}: {e:?}"))?;
    let rom =
        fs::read_to_string(super::raiz().join("refs/rom-hashes.txt")).map_err(|e| e.to_string())?;
    let rom_sha1 = rom
        .lines()
        .find_map(|l| l.strip_prefix("sha1:"))
        .map(str::trim)
        .ok_or("refs/rom-hashes.txt sem sha1")?;
    let texto = format!(
        "fonte: {}\nlicenca: {}\nrom_sha1: {}\nquadros: {}\nentradas_sha1: {}\nautor: {}\n",
        opcao(args, "--fonte").ok_or("falta --fonte")?,
        opcao(args, "--licenca").ok_or("falta --licenca")?,
        rom_sha1,
        q.len(),
        em_hex(&sha1_das_entradas(&q)),
        opcao(args, "--autor").ok_or("falta --autor")?,
    );
    meta::ler(&texto).map_err(|e| format!("meta invalido: {e:?}"))?;
    fs::write(Path::new(arq).with_extension("meta"), texto).map_err(|e| e.to_string())?;
    println!("meta: {} quadros", q.len());
    Ok(())
}

pub fn executar(args: &[String]) -> Result<(), String> {
    let rom = opcao(args, "--rom").ok_or("falta --rom")?;
    let bin = opcao(args, "--oraculo").ok_or("falta --oraculo")?;
    let rotulos = opcao(args, "--rotulos").ok_or("falta --rotulos")?;
    let saida = Path::new(opcao(args, "--saida").ok_or("falta --saida")?);
    let corpus = Path::new(opcao(args, "--corpus").unwrap_or("corpus"));
    let k: u32 = opcao(args, "--intervalo")
        .unwrap_or("600")
        .parse()
        .map_err(|_| "--intervalo inválido")?;
    // Traces nunca podem cair dentro do repositório.
    fs::create_dir_all(saida).map_err(|e| e.to_string())?;
    let real = fs::canonicalize(saida).map_err(|e| e.to_string())?;
    if real.starts_with(fs::canonicalize(super::raiz()).map_err(|e| e.to_string())?) {
        return Err("a saída está dentro do repositório".into());
    }
    let casos = descobrir(corpus, opcao(args, "--caso"))?;
    let mut falhas = 0u32;
    for caso in casos {
        match um_caso(bin, rom, rotulos, &real, &caso, k) {
            Ok(()) => {}
            Err(e) => {
                println!("{}\tFALHOU\t{e}", caso.nome);
                falhas = falhas.saturating_add(1);
            }
        }
    }
    if falhas > 0 {
        return Err(format!("{falhas} caso(s) falharam"));
    }
    println!("determinismo: PASSOU");
    Ok(())
}

fn um_caso(
    bin: &str,
    rom: &str,
    rotulos: &str,
    dir: &Path,
    caso: &Caso,
    k: u32,
) -> Result<(), String> {
    let s = |p: PathBuf| p.to_string_lossy().into_owned();
    let (a, b, c) = (
        s(dir.join(format!("{}.a.s1t", caso.nome))),
        s(dir.join(format!("{}.b.s1t", caso.nome))),
        s(dir.join(format!("{}.c.s1t", caso.nome))),
    );
    let q = caso.quadros.to_string();
    let k_txt = k.to_string();
    let mut base = vec!["rodar", "--rom", rom, "--quadros", &q];
    let ent;
    if let Some(e) = &caso.entradas {
        ent = s(e.clone());
        base.extend(["--entradas", &ent]);
    }
    let mut ra = base.clone();
    ra.extend([
        "--saida",
        &a,
        "--estado-a-cada",
        &k_txt,
        "--rotulos",
        rotulos,
    ]);
    // Video (P6 e P7): imagem por quadro (.s1i) e palavras de DMA (.s1v), para a Etapa 4.
    let prefixo_video = s(dir.join(format!("{}.a", caso.nome)));
    ra.extend(["--video", &prefixo_video]);
    let saida_a = oraculo(bin, &ra)?;
    let mut rb = base.clone();
    rb.extend(["--saida", &b]);
    let saida_b = oraculo(bin, &rb)?;
    let hash = campo(&saida_a, "hash_do_trace")?;
    if hash != campo(&saida_b, "hash_do_trace")?
        || fs::read(&a).map_err(|e| e.to_string())? != fs::read(&b).map_err(|e| e.to_string())?
    {
        return Err("duas execuções diferem".into());
    }
    // Retomada: o último estado salvo antes do fim.
    let ultimo = (1..caso.quadros).rev().find(|n| n % k == 0);
    if let Some(n) = ultimo {
        let estado = s(dir.join(format!("{}.a.{n}.s1e", caso.nome)));
        let mut rc = base.clone();
        rc.extend(["--saida", &c, "--carregar-estado", &estado]);
        oraculo(bin, &rc)?;
        let completo = fs::read(&a).map_err(|e| e.to_string())?;
        let retomado = fs::read(&c).map_err(|e| e.to_string())?;
        if let Some(q) = comparar_cauda(&completo, &retomado, n)? {
            return Err(format!("retomada do quadro {n} difere no quadro {q}"));
        }
    }
    println!("{}\t{hash}\t{}", caso.nome, caso.quadros);
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    use sonic1_fidelidade::{Cabecalho, Categoria, Escritor, Quadro};

    fn trace(n: u32, muda_em: Option<u32>) -> Vec<u8> {
        let cab = Cabecalho {
            oraculo: "t".into(),
            rom_sha1: [1; 20],
            config_sha1: [2; 20],
            entradas_sha1: [3; 20],
            modo: 0,
        };
        let mut buf = Vec::new();
        let mut e = Escritor::novo(&mut buf, &cab).unwrap();
        for i in 0..n {
            let mut h = [[0u8; 20]; Categoria::TOTAL];
            h[0] = [u8::try_from(i % 200).unwrap(); 20];
            if Some(i) == muda_em {
                h[0] = [255; 20];
            }
            let q = Quadro {
                numero: i,
                mclk_fim: u64::from(i) * 100,
                eventos: vec![],
                hashes: h,
            };
            e.escrever(&q).unwrap();
        }
        e.finalizar(&cab).unwrap();
        buf
    }

    /// Retomado = cauda do completo: o trace parcial tem o mesmo cabeçalho e os quadros de K em diante.
    fn cauda(completo: &[u8], desde: u32) -> Vec<u8> {
        let mut l = Leitor::abrir(completo).unwrap();
        let cab = l.cabecalho.clone();
        let mut buf = Vec::new();
        let mut e = Escritor::novo(&mut buf, &cab).unwrap();
        while let Some(q) = l.proximo().unwrap() {
            if q.numero >= desde {
                e.escrever(&q).unwrap();
            }
        }
        e.finalizar(&cab).unwrap();
        buf
    }

    #[test]
    fn cauda_igual() {
        let t = trace(30, None);
        assert_eq!(comparar_cauda(&t, &cauda(&t, 10), 10), Ok(None));
    }

    #[test]
    fn cauda_com_quadro_plantado() {
        let t = trace(30, None);
        let ruim = cauda(&trace(30, Some(17)), 10);
        assert_eq!(comparar_cauda(&t, &ruim, 10), Ok(Some(17)));
    }

    #[test]
    fn cauda_curta_ou_longa() {
        let t = trace(30, None);
        assert_eq!(
            comparar_cauda(&t, &cauda(&trace(25, None), 10), 10),
            Ok(Some(25))
        );
        assert_eq!(
            comparar_cauda(&trace(25, None), &cauda(&t, 10), 10),
            Ok(Some(25))
        );
    }

    #[test]
    fn demos_validas_e_invalidas() {
        let c = ler_demos("# x\ndemo1\t600\ndemo-2\t1200\n").unwrap();
        assert_eq!(c.len(), 2);
        assert_eq!(c[1].quadros, 1200);
        assert!(ler_demos("sem tab 5\n").is_err());
        assert!(ler_demos("a\tx\n").is_err());
        assert!(ler_demos("a b\t5\n").is_err());
    }
}
