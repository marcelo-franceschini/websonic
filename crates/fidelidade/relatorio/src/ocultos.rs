//! `cargo fidelidade ocultos <chaves|cifrar|proteger|conferir|auditar>` (sub-etapas 2.8 e 2.10).
//!
//! Testes ocultos: entradas gravadas pelo Marcelo, cifradas com `age` (chave pública dele).
//! Ficam fora do repositório, em `$SONIC1_EXT_DIR/ocultos/<zona>/<caso>.s1in.age`.
//! A identidade (chave privada) fica só com o Marcelo; a IA nunca a vê.
//!
//! A identidade pode ficar em dois formatos: texto claro (`AGE-SECRET-KEY-...`, arquivo 0600) ou
//! cifrada com senha (scrypt do próprio `age`). O comando `proteger` converte o primeiro no
//! segundo. A senha vem sempre pela entrada padrão (uma linha), nunca por argumento, para não
//! aparecer na lista de processos nem no histórico do terminal.
//!
//! `auditar` decifra cada caso para um arquivo em tmpfs (`/dev/shm`), roda o sistema sob teste
//! (um comando que recebe o caminho do `.s1in` e termina com 0 se o caso passou) e apaga o
//! arquivo. A saída tem SÓ `zona: passou|falhou (N casos)`. Nenhum nome de caso, quadro ou hash.
//! Erros viram um código fixo (`erro: E<n>`).
//!
//! `decifrar` não existe de propósito: não há comando que deixe o texto claro em disco.
//!
//! Uso:
//! `cargo fidelidade ocultos chaves --publica refs/ocultos.pub --identidade ~/.config/websonic/ocultos.key`
//! `cargo fidelidade ocultos proteger --identidade ARQ_CLARO --saida ARQ.age [--publica refs/ocultos.pub] < senha`
//! `cargo fidelidade ocultos cifrar --publica refs/ocultos.pub --entrada ARQ.s1in --pasta DIR --zona ghz --caso c1`
//! `cargo fidelidade ocultos conferir --identidade ARQ [--publica refs/ocultos.pub] [< senha]`
//! `cargo fidelidade ocultos auditar --pasta DIR --identidade ARQ --sut COMANDO [< senha]`
use crate::opcao;
use age::secrecy::SecretString;
use age::x25519::{Identity, Recipient};
use std::fs;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Zonas aceitas (nome de pasta).
pub const ZONAS: [&str; 9] = [
    "ghz", "mz", "syz", "lz", "slz", "sbz", "especial", "final", "telas",
];

/// Custo do scrypt na proteção da identidade: N = 2^18 (cerca de 256 MiB e 1 s). Fixo, e não o
/// que o `age` calcularia pela velocidade da máquina, para a identidade abrir igual em qualquer
/// máquina e em qualquer perfil de compilação.
pub const FATOR_SCRYPT: u8 = 18;
/// Maior custo aceito ao abrir uma identidade (limite do próprio `age` contra arquivo malicioso).
const FATOR_SCRYPT_MAXIMO: u8 = 22;
/// Início de todo arquivo cifrado do `age` (formato binário).
const CABECALHO_AGE: &[u8] = b"age-encryption.org/";

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

/// A identidade está cifrada com senha? (Arquivo binário do `age`, em vez do texto da chave.)
pub fn identidade_protegida(arquivo: &[u8]) -> bool {
    arquivo.starts_with(CABECALHO_AGE)
}

/// Cifra a identidade (texto da chave) com a senha. `fator` é o log2 do custo do scrypt.
pub fn proteger_bytes(
    identidade: &str,
    senha: &SecretString,
    fator: u8,
) -> Result<Vec<u8>, String> {
    if !identidade.lines().any(|l| l.starts_with("AGE-SECRET-KEY-")) {
        return Err("identidade inválida".into());
    }
    let mut r = age::scrypt::Recipient::new(senha.clone());
    r.set_work_factor(fator);
    age::encrypt(&r, identidade.as_bytes()).map_err(|e| e.to_string())
}

/// Abre a identidade cifrada com senha e devolve o texto da chave. Senha errada vira erro.
pub fn abrir_identidade(cifrado: &[u8], senha: &SecretString) -> Result<String, String> {
    let mut i = age::scrypt::Identity::new(senha.clone());
    i.set_max_work_factor(FATOR_SCRYPT_MAXIMO);
    let claro = age::decrypt(&i, cifrado).map_err(|_| "senha errada ou arquivo corrompido")?;
    String::from_utf8(claro).map_err(|_| "identidade inválida".to_string())
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

pub fn executar(args: &[String]) -> Result<(), String> {
    let resto = args.get(1..).unwrap_or_default();
    match args.first().map(String::as_str) {
        Some("cifrar") => cifrar(resto),
        Some("auditar") => auditar_cli(resto),
        Some("chaves") => chaves(resto),
        Some("proteger") => proteger(resto),
        Some("conferir") => conferir_cli(resto),
        _ => Err("uso: cargo fidelidade ocultos <chaves|cifrar|proteger|conferir|auditar>".into()),
    }
}

/// Lê uma linha da entrada padrão, sem o fim de linha. É a senha; nunca é impressa.
fn ler_senha() -> Result<SecretString, String> {
    let mut linha = String::new();
    std::io::stdin()
        .lock()
        .read_line(&mut linha)
        .map_err(|e| e.to_string())?;
    let senha = linha.trim_end_matches(['\n', '\r']);
    if senha.is_empty() {
        return Err("senha vazia na entrada padrão".into());
    }
    Ok(SecretString::from(senha.to_string()))
}

/// Cria um arquivo novo só do dono (0600). Recusa sobrescrever.
fn criar_privado(caminho: &str, conteudo: &[u8]) -> Result<(), String> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    if let Some(pai) = Path::new(caminho).parent() {
        fs::create_dir_all(pai).map_err(|e| e.to_string())?;
    }
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(caminho)
        .map_err(|e| format!("{caminho}: {e}"))?;
    f.write_all(conteudo).map_err(|e| e.to_string())
}

/// Cria o par de chaves `age`. A identidade (privada) vai para um arquivo só do dono (0600) e
/// nunca é impressa. Recusa sobrescrever. Só o Marcelo roda, no computador dele.
fn chaves(args: &[String]) -> Result<(), String> {
    use age::secrecy::ExposeSecret;
    let publica = opcao(args, "--publica").ok_or("falta --publica")?;
    let identidade = opcao(args, "--identidade").ok_or("falta --identidade")?;
    let i = Identity::generate();
    let mut texto = i.to_string().expose_secret().to_string();
    texto.push('\n');
    criar_privado(identidade, texto.as_bytes())?;
    fs::write(publica, format!("{}\n", i.to_public())).map_err(|e| e.to_string())?;
    println!("chaves criadas (identidade em {identidade}, nunca versione esse arquivo)");
    Ok(())
}

/// Cifra a identidade em texto claro com uma senha (lida da entrada padrão) e confere que o
/// arquivo cifrado abre com a mesma senha e dá a mesma chave. Só então diz que deu certo.
/// Não apaga o arquivo claro: quem apaga é o script, depois de ver esta mensagem.
/// Com `--publica`, confere também que a identidade é a do par publicado.
fn proteger(args: &[String]) -> Result<(), String> {
    let origem = opcao(args, "--identidade").ok_or("falta --identidade")?;
    let destino = opcao(args, "--saida").ok_or("falta --saida")?;
    let claro = fs::read_to_string(origem).map_err(|e| format!("{origem}: {e}"))?;
    let linha = claro
        .lines()
        .find(|l| l.starts_with("AGE-SECRET-KEY-"))
        .ok_or("identidade inválida")?;
    if let Some(p) = opcao(args, "--publica") {
        let esperada = fs::read_to_string(p).map_err(|e| format!("{p}: {e}"))?;
        identidade_confere(&claro, Some(&esperada))
            .map_err(|_| "a identidade não é a do par publicado".to_string())?;
    }
    let senha = ler_senha()?;
    let cifrado = proteger_bytes(&claro, &senha, FATOR_SCRYPT)?;
    // Prova de ida e volta antes de qualquer coisa ser apagada.
    let volta = abrir_identidade(&cifrado, &senha)?;
    if volta.lines().find(|l| l.starts_with("AGE-SECRET-KEY-")) != Some(linha) {
        return Err("a identidade cifrada não volta igual".into());
    }
    criar_privado(destino, &cifrado)?;
    println!("identidade protegida com senha em {destino}");
    Ok(())
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

/// Lê o arquivo da identidade (clara ou cifrada com senha) e devolve o texto da chave.
/// A senha, se preciso, vem da entrada padrão. Qualquer falha vira o mesmo código, sem detalhe.
fn ler_identidade(caminho: &str) -> Result<String, String> {
    let arquivo = fs::read(caminho).map_err(|_| "erro: E1".to_string())?;
    if identidade_protegida(&arquivo) {
        let senha = ler_senha().map_err(|_| "erro: E1".to_string())?;
        abrir_identidade(&arquivo, &senha).map_err(|_| "erro: E1".to_string())
    } else {
        String::from_utf8(arquivo).map_err(|_| "erro: E1".to_string())
    }
}

/// A identidade é válida e, se `publica` vier, é a do par publicado?
pub fn identidade_confere(texto: &str, publica: Option<&str>) -> Result<(), ()> {
    let linha = texto
        .lines()
        .find(|l| l.starts_with("AGE-SECRET-KEY-"))
        .ok_or(())?;
    let i: Identity = linha.trim().parse().map_err(|_| ())?;
    match publica {
        Some(p) if p.trim() != i.to_public().to_string() => Err(()),
        _ => Ok(()),
    }
}

/// Confere a identidade (e a senha) sem decifrar nenhum teste. Com `--publica`, confere também
/// que ela é a do par publicado. Serve para achar senha errada antes de uma rodada longa.
fn conferir_cli(args: &[String]) -> Result<(), String> {
    let texto = ler_identidade(opcao(args, "--identidade").ok_or("falta --identidade")?)?;
    let publica = match opcao(args, "--publica") {
        Some(p) => Some(fs::read_to_string(p).map_err(|_| "erro: E1".to_string())?),
        None => None,
    };
    identidade_confere(&texto, publica.as_deref()).map_err(|_| "erro: E1".to_string())?;
    println!("identidade: OK");
    Ok(())
}

fn auditar_cli(args: &[String]) -> Result<(), String> {
    let pasta = Path::new(opcao(args, "--pasta").ok_or("falta --pasta")?);
    let identidade = ler_identidade(opcao(args, "--identidade").ok_or("falta --identidade")?)?;
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

    /// Custo baixo nos testes: o scrypt em perfil de depuração é lento.
    const FATOR_TESTE: u8 = 10;

    fn par() -> (String, String) {
        let i = Identity::generate();
        (
            i.to_public().to_string(),
            i.to_string().expose_secret().to_string(),
        )
    }

    fn senha(s: &str) -> SecretString {
        SecretString::from(s.to_string())
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

    /// F6: a identidade cifrada abre com a senha certa e falha com a errada.
    #[test]
    fn identidade_cifrada_abre_com_a_senha_certa_e_falha_com_a_errada() {
        let (_, i) = par();
        let cifrada = proteger_bytes(&i, &senha("senha certa"), FATOR_TESTE).unwrap();
        assert!(identidade_protegida(&cifrada));
        assert!(!identidade_protegida(i.as_bytes()));
        // O texto da chave não aparece no arquivo cifrado.
        assert!(!String::from_utf8_lossy(&cifrada).contains("AGE-SECRET-KEY-"));
        assert_eq!(
            abrir_identidade(&cifrada, &senha("senha certa")).unwrap(),
            i
        );
        assert!(abrir_identidade(&cifrada, &senha("senha errada")).is_err());
        assert!(abrir_identidade(&cifrada, &senha("")).is_err());
    }

    #[test]
    fn proteger_recusa_o_que_nao_e_identidade() {
        assert!(proteger_bytes("lixo", &senha("x"), FATOR_TESTE).is_err());
    }

    /// F6: a auditoria funciona com a identidade cifrada (e dá o mesmo resultado que com a clara).
    #[test]
    fn auditoria_com_identidade_cifrada() {
        let (p, i) = par();
        let pasta = temp("oc-prot-pasta");
        let tmp = temp("oc-prot-tmp");
        monta(&pasta, &p, &[("ghz", "a", "00\n"), ("mz", "b", "ff\n")]);
        let cifrada = proteger_bytes(&i, &senha("abre"), FATOR_TESTE).unwrap();
        let aberta = abrir_identidade(&cifrada, &senha("abre")).unwrap();
        let sut = |arq: &Path| fs::read_to_string(arq).unwrap() != "ff\n";
        let com_cifrada = auditar(&pasta, &aberta, &tmp, &sut).unwrap();
        let com_clara = auditar(&pasta, &i, &tmp, &sut).unwrap();
        assert_eq!(com_cifrada, com_clara);
        assert_eq!(
            com_cifrada,
            vec![
                "ghz: passou (1 casos)".to_string(),
                "mz: falhou (1 casos)".to_string()
            ]
        );
        // Com a senha errada nada é decifrado: não há identidade para a auditoria.
        assert!(abrir_identidade(&cifrada, &senha("nao abre")).is_err());
    }

    #[test]
    fn conferir_identidade_e_par_publicado() {
        let (p, i) = par();
        let (outra_p, _) = par();
        assert_eq!(identidade_confere(&i, None), Ok(()));
        assert_eq!(identidade_confere(&i, Some(&format!("{p}\n"))), Ok(()));
        assert_eq!(identidade_confere(&i, Some(&outra_p)), Err(()));
        assert_eq!(identidade_confere("lixo", None), Err(()));
    }

    #[test]
    fn arquivo_privado_nasce_0600_e_nao_sobrescreve() {
        use std::os::unix::fs::PermissionsExt;
        let d = temp("oc-priv");
        let alvo = d.join("sub/k");
        let alvo = alvo.to_str().unwrap();
        criar_privado(alvo, b"x").unwrap();
        let modo = fs::metadata(alvo).unwrap().permissions().mode() & 0o777;
        assert_eq!(modo, 0o600);
        assert!(criar_privado(alvo, b"y").is_err());
        assert_eq!(fs::read(alvo).unwrap(), b"x");
    }
}
