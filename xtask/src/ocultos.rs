//! `cargo xtask ocultos <cifrar|auditar>` (sub-etapa 2.8).
//!
//! Testes ocultos: entradas gravadas pelo Marcelo, cifradas com `age` (chave pública dele).
//! Ficam fora do repositório, em `$SONIC1_EXT_DIR/ocultos/<zona>/<caso>.s1in.age`.
//! A identidade (chave privada) fica só com o Marcelo; a IA nunca a vê.
//!
//! `auditar` decifra cada caso para um arquivo em tmpfs (`/dev/shm`), roda o sistema sob teste
//! (um comando que recebe o caminho do `.s1in` e termina com 0 se o caso passou) e apaga o
//! arquivo. A saída tem SÓ `zona: passou|falhou (N casos)`. Nenhum nome de caso, quadro ou hash.
//! Erros viram um código fixo (`erro: E<n>`).
//!
//! `decifrar` não existe de propósito: não há comando que deixe o texto claro em disco.
//!
//! Uso:
//! `cargo xtask ocultos cifrar --publica refs/ocultos.pub --entrada ARQ.s1in --pasta DIR --zona ghz --caso c1`
//! `cargo xtask ocultos auditar --pasta DIR --identidade ARQ --sut COMANDO`
use age::x25519::{Identity, Recipient};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Zonas aceitas (nome de pasta).
pub const ZONAS: [&str; 9] = [
    "ghz", "mz", "syz", "lz", "slz", "sbz", "especial", "final", "telas",
];

/// Código fixo de erro da auditoria. Não revela nada sobre o caso.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CodigoErro {
    /// Pasta ou identidade ilegível.
    Entrada = 1,
    /// Não foi possível decifrar um arquivo (chave errada ou arquivo corrompido).
    Decifrar = 2,
    /// Falha ao usar o tmpfs.
    Tmpfs = 3,
}

pub fn cifrar_bytes(publica: &str, dados: &[u8]) -> Result<Vec<u8>, String> {
    let r: Recipient = publica
        .trim()
        .parse()
        .map_err(|_| "chave pública inválida".to_string())?;
    age::encrypt(&r, dados).map_err(|e| e.to_string())
}

pub fn decifrar_bytes(identidade: &str, cifrado: &[u8]) -> Result<Vec<u8>, String> {
    let linha = identidade
        .lines()
        .find(|l| l.starts_with("AGE-SECRET-KEY-"))
        .ok_or("identidade inválida")?;
    let i: Identity = linha
        .trim()
        .parse()
        .map_err(|_| "identidade inválida".to_string())?;
    age::decrypt(&i, cifrado).map_err(|e| e.to_string())
}

/// Roda a auditoria. `sut` recebe o caminho do `.s1in` claro (em `tmp`) e diz se passou.
/// Devolve as linhas de saída, uma por zona que tem caso, em ordem alfabética.
pub fn auditar(
    pasta: &Path,
    identidade: &str,
    tmp: &Path,
    sut: &dyn Fn(&Path) -> bool,
) -> Result<Vec<String>, CodigoErro> {
    let mut zonas: Vec<String> = fs::read_dir(pasta)
        .map_err(|_| CodigoErro::Entrada)?
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    zonas.sort();
    let mut saida = Vec::new();
    for zona in zonas {
        let mut arquivos: Vec<PathBuf> = fs::read_dir(pasta.join(&zona))
            .map_err(|_| CodigoErro::Entrada)?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "age"))
            .collect();
        arquivos.sort();
        let total = arquivos.len();
        let mut passou = true;
        for (n, arq) in arquivos.iter().enumerate() {
            let cifrado = fs::read(arq).map_err(|_| CodigoErro::Entrada)?;
            let claro = decifrar_bytes(identidade, &cifrado).map_err(|_| CodigoErro::Decifrar)?;
            // Nome neutro: o do caso não vai para o disco.
            let alvo = tmp.join(format!("c{n}.s1in"));
            fs::write(&alvo, &claro).map_err(|_| CodigoErro::Tmpfs)?;
            let ok = sut(&alvo);
            fs::remove_file(&alvo).map_err(|_| CodigoErro::Tmpfs)?;
            passou &= ok;
        }
        saida.push(format!(
            "{zona}: {} ({total} casos)",
            if passou { "passou" } else { "falhou" }
        ));
    }
    Ok(saida)
}

fn opcao<'a>(args: &'a [String], nome: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == nome)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

pub fn executar(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("cifrar") => cifrar(&args[1..]),
        Some("auditar") => auditar_cli(&args[1..]),
        _ => Err("uso: cargo xtask ocultos <cifrar|auditar>".into()),
    }
}

fn cifrar(args: &[String]) -> Result<(), String> {
    let publica = fs::read_to_string(opcao(args, "--publica").ok_or("falta --publica")?)
        .map_err(|e| e.to_string())?;
    let entrada =
        fs::read(opcao(args, "--entrada").ok_or("falta --entrada")?).map_err(|e| e.to_string())?;
    // Valida o conteúdo antes de cifrar: um caso malformado só apareceria na auditoria.
    sonic1_corpus::ler_s1in(&String::from_utf8_lossy(&entrada))
        .map_err(|e| format!("entrada inválida: {e:?}"))?;
    let zona = opcao(args, "--zona").ok_or("falta --zona")?;
    if !ZONAS.contains(&zona) {
        return Err(format!("zona deve ser uma de {ZONAS:?}"));
    }
    let caso = opcao(args, "--caso").ok_or("falta --caso")?;
    if caso.is_empty()
        || !caso
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err("nome de caso inválido".into());
    }
    let pasta = Path::new(opcao(args, "--pasta").ok_or("falta --pasta")?).join(zona);
    fs::create_dir_all(&pasta).map_err(|e| e.to_string())?;
    let cifrado = cifrar_bytes(&publica, &entrada)?;
    fs::write(pasta.join(format!("{caso}.s1in.age")), cifrado).map_err(|e| e.to_string())?;
    println!("cifrado: {zona}");
    Ok(())
}

fn auditar_cli(args: &[String]) -> Result<(), String> {
    let pasta = Path::new(opcao(args, "--pasta").ok_or("falta --pasta")?);
    let identidade = fs::read_to_string(opcao(args, "--identidade").ok_or("falta --identidade")?)
        .map_err(|_| "erro: E1".to_string())?;
    let sut = opcao(args, "--sut").ok_or("falta --sut")?.to_string();
    let tmp = PathBuf::from(format!("/dev/shm/websonic-ocultos-{}", std::process::id()));
    fs::create_dir_all(&tmp).map_err(|_| "erro: E3".to_string())?;
    let roda = |arquivo: &Path| {
        Command::new(&sut)
            .arg(arquivo)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    };
    let r = auditar(pasta, &identidade, &tmp, &roda);
    let _ = fs::remove_dir_all(&tmp);
    match r {
        Ok(linhas) => {
            for l in &linhas {
                println!("{l}");
            }
            Ok(())
        }
        Err(c) => Err(format!("erro: E{}", c as u8)),
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use age::secrecy::ExposeSecret;

    fn par() -> (String, String) {
        let i = Identity::generate();
        (
            i.to_public().to_string(),
            i.to_string().expose_secret().to_string(),
        )
    }

    fn temp(nome: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("websonic-{nome}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn monta(pasta: &Path, publica: &str, casos: &[(&str, &str, &str)]) {
        for (zona, caso, texto) in casos {
            fs::create_dir_all(pasta.join(zona)).unwrap();
            fs::write(
                pasta.join(zona).join(format!("{caso}.s1in.age")),
                cifrar_bytes(publica, texto.as_bytes()).unwrap(),
            )
            .unwrap();
        }
    }

    #[test]
    fn cifra_e_decifra_voltam_ao_mesmo_conteudo() {
        let (p, i) = par();
        let texto = b"00\n80\n";
        let c = cifrar_bytes(&p, texto).unwrap();
        assert_ne!(c, texto);
        assert_eq!(decifrar_bytes(&i, &c).unwrap(), texto);
        assert_eq!(
            sonic1_fidelidade::hash::sha1(&decifrar_bytes(&i, &c).unwrap()),
            sonic1_fidelidade::hash::sha1(texto)
        );
    }

    #[test]
    fn chave_errada_nao_decifra() {
        let (p, _) = par();
        let (_, outra) = par();
        let c = cifrar_bytes(&p, b"00\n").unwrap();
        assert!(decifrar_bytes(&outra, &c).is_err());
    }

    #[test]
    fn saida_exata_e_nada_claro_sobra() {
        let (p, i) = par();
        let pasta = temp("oc-pasta");
        let tmp = temp("oc-tmp");
        // "falha" no texto do caso faz o sistema falso reprovar a zona mz.
        monta(
            &pasta,
            &p,
            &[
                ("ghz", "a", "00\n"),
                ("ghz", "b", "80\n"),
                ("mz", "c", "00\n"),
                ("mz", "segredo", "ff\n"),
            ],
        );
        let sut = |arq: &Path| fs::read_to_string(arq).unwrap() != "ff\n";
        let saida = auditar(&pasta, &i, &tmp, &sut).unwrap();
        assert_eq!(
            saida,
            vec![
                "ghz: passou (2 casos)".to_string(),
                "mz: falhou (2 casos)".to_string()
            ]
        );
        // (d) nenhum arquivo claro sobra.
        assert_eq!(fs::read_dir(&tmp).unwrap().count(), 0);
        // Nenhum nome de caso aparece na saída.
        assert!(!saida.join("\n").contains("segredo"));
    }

    #[test]
    fn erros_viram_codigo_fixo() {
        let (p, _) = par();
        let (_, outra) = par();
        let pasta = temp("oc-erro");
        let tmp = temp("oc-erro-tmp");
        monta(&pasta, &p, &[("ghz", "a", "00\n")]);
        assert_eq!(
            auditar(&pasta, &outra, &tmp, &|_| true),
            Err(CodigoErro::Decifrar)
        );
        assert_eq!(
            auditar(&pasta.join("nao-existe"), &outra, &tmp, &|_| true),
            Err(CodigoErro::Entrada)
        );
    }
}
