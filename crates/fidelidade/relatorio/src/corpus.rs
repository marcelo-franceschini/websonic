//! `cargo fidelidade corpus-trace`: gera os traces do corpus e prova o determinismo
//! (sub-etapa 2.4, movido do xtask e sem cópias na 2.10).
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
//! O requisito pede "hashes por quadro e estados completos a intervalos", não o trace inteiro
//! (um TAS daria de 14 a 18 GB por trace). Por isso, depois das provas, a segunda execução e a
//! retomada são apagadas. Por caso ficam:
//! - o hash do trace (impresso e conferido pelo manifesto);
//! - `<nome>.quadros.tsv`: uma linha `quadro<TAB>hash` por quadro;
//! - os estados `<nome>.a.<quadro>.s1e`, a cada `--intervalo` quadros;
//! - a saída de vídeo (`<nome>.a.s1i` e `<nome>.a.s1v`), que o `video-comparar` usa.
//!
//! O trace completo (`<nome>.a.s1t`) só fica com `--manter-trace`, para os casos que o
//! `video-comparar` lê. Qualquer regeneração é possível sob demanda, porque tudo é determinístico.
//!
//! Uso: `cargo fidelidade corpus-trace --rom ROM --oraculo BIN --rotulos refs/rotulos.txt
//! --saida DIR [--corpus corpus] [--intervalo 600] [--caso NOME] [--manter-trace]`
use crate::opcao;
use sonic1_corpus::{ler_s1in, meta, sha1_das_entradas};
use sonic1_fidelidade::hash::{em_hex, sha1_de_partes};
use sonic1_fidelidade::{Leitor, Quadro, comparar_quadros};
use std::fs;
use std::io::{BufReader, BufWriter, Read, Write};
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
/// Lê em sequência (sem carregar os traces na memória): os traces passam de 10 GB.
pub fn comparar_cauda<A: Read, B: Read>(
    completo: A,
    retomado: B,
    desde: u32,
) -> Result<Option<u32>, String> {
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

/// Os dois arquivos têm exatamente os mesmos bytes? Lê em blocos de 1 MiB.
pub fn arquivos_iguais(a: &Path, b: &Path) -> Result<bool, String> {
    let (mut fa, mut fb) = (
        fs::File::open(a).map_err(|e| format!("{}: {e}", a.display()))?,
        fs::File::open(b).map_err(|e| format!("{}: {e}", b.display()))?,
    );
    if fa.metadata().map_err(|e| e.to_string())?.len()
        != fb.metadata().map_err(|e| e.to_string())?.len()
    {
        return Ok(false);
    }
    let (mut ba, mut bb) = (vec![0u8; 1 << 20], vec![0u8; 1 << 20]);
    loop {
        let n = ler_cheio(&mut fa, &mut ba)?;
        let m = ler_cheio(&mut fb, &mut bb)?;
        if n != m || ba[..n] != bb[..m] {
            return Ok(false);
        }
        if n == 0 {
            return Ok(true);
        }
    }
}

/// Lê até encher o bloco (ou até o fim do arquivo). Devolve quantos bytes leu.
fn ler_cheio(f: &mut fs::File, bloco: &mut [u8]) -> Result<usize, String> {
    let mut total = 0;
    while total < bloco.len() {
        let n = f.read(&mut bloco[total..]).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        total += n;
    }
    Ok(total)
}

/// Hash de um quadro: o mesmo cálculo que o formato do trace usa para somar o quadro ao hash
/// do trace inteiro (número, relógio do fim do quadro e o hash de cada categoria).
pub fn hash_do_quadro(q: &Quadro) -> [u8; 20] {
    let numero = q.numero.to_le_bytes();
    let fim = q.mclk_fim.to_le_bytes();
    let mut partes: Vec<&[u8]> = vec![&numero, &fim];
    for h in &q.hashes {
        partes.push(h);
    }
    sha1_de_partes(&partes)
}

/// Lê o trace e escreve `quadro<TAB>hash` por quadro. Devolve o número de quadros e o hash do
/// trace inteiro, calculado na leitura (para conferir com o hash que o oráculo imprimiu).
pub fn escrever_quadros<R: Read, W: Write>(
    trace: R,
    mut saida: W,
) -> Result<(u32, [u8; 20]), String> {
    let mut leitor = Leitor::abrir(trace).map_err(|e| e.to_string())?;
    let mut n = 0u32;
    while let Some(q) = leitor.proximo().map_err(|e| e.to_string())? {
        writeln!(saida, "{}\t{}", q.numero, em_hex(&hash_do_quadro(&q)))
            .map_err(|e| e.to_string())?;
        n = n.saturating_add(1);
    }
    saida.flush().map_err(|e| e.to_string())?;
    Ok((n, leitor.hash_lido()))
}

/// Roda o oráculo e devolve a saída padrão. Abstraído para o teste usar um oráculo de mentira.
type Oraculo<'a> = &'a dyn Fn(&[&str]) -> Result<String, String>;

fn oraculo_real(bin: &str) -> impl Fn(&[&str]) -> Result<String, String> {
    let bin = bin.to_string();
    move |args| {
        let s = Command::new(&bin)
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
    if k == 0 {
        return Err("--intervalo deve ser maior que zero".into());
    }
    let manter = args.iter().any(|a| a == "--manter-trace");
    // Traces nunca podem cair dentro do repositório.
    fs::create_dir_all(saida).map_err(|e| e.to_string())?;
    let real = fs::canonicalize(saida).map_err(|e| e.to_string())?;
    if real.starts_with(fs::canonicalize(crate::raiz()).map_err(|e| e.to_string())?) {
        return Err("a saída está dentro do repositório".into());
    }
    let casos = descobrir(corpus, opcao(args, "--caso"))?;
    let oraculo = oraculo_real(bin);
    let mut falhas = 0u32;
    for caso in casos {
        match um_caso(&oraculo, rom, rotulos, &real, &caso, k, manter) {
            Ok(hash) => println!("{}\t{hash}\t{}", caso.nome, caso.quadros),
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

/// Apaga um arquivo. Não existir não é erro.
fn remover(p: &Path) {
    let _ = fs::remove_file(p);
}

/// Roda um caso e devolve o hash do trace. Deixa na pasta só o que a documentação do módulo diz.
fn um_caso(
    oraculo: Oraculo,
    rom: &str,
    rotulos: &str,
    dir: &Path,
    caso: &Caso,
    k: u32,
    manter_trace: bool,
) -> Result<String, String> {
    let (a, b, c) = (
        dir.join(format!("{}.a.s1t", caso.nome)),
        dir.join(format!("{}.b.s1t", caso.nome)),
        dir.join(format!("{}.c.s1t", caso.nome)),
    );
    // A segunda execução e a retomada são sempre apagadas, com sucesso ou não.
    let r = provar(oraculo, rom, rotulos, dir, caso, k, [&a, &b, &c]);
    remover(&b);
    remover(&c);
    let hash = match r {
        Ok(h) => h,
        Err(e) => {
            remover(&a);
            return Err(e);
        }
    };
    // Hashes por quadro, lidos do trace completo; o hash do trace tem de fechar com o do oráculo.
    let tsv = dir.join(format!("{}.quadros.tsv", caso.nome));
    let entrada = BufReader::new(fs::File::open(&a).map_err(|e| e.to_string())?);
    let saida = BufWriter::new(fs::File::create(&tsv).map_err(|e| e.to_string())?);
    let (n, hash_lido) = escrever_quadros(entrada, saida)?;
    if n != caso.quadros {
        return Err(format!(
            "o trace tem {n} quadros, esperava {}",
            caso.quadros
        ));
    }
    if em_hex(&hash_lido) != hash {
        return Err("o hash lido do trace difere do hash do oráculo".into());
    }
    if !manter_trace {
        remover(&a);
    }
    Ok(hash)
}

/// As duas provas: duas execuções iguais e retomada igual. `[a, b, c]` são os três traces.
fn provar(
    oraculo: Oraculo,
    rom: &str,
    rotulos: &str,
    dir: &Path,
    caso: &Caso,
    k: u32,
    [a, b, c]: [&Path; 3],
) -> Result<String, String> {
    let s = |p: &Path| p.to_string_lossy().into_owned();
    let (sa, sb, sc) = (s(a), s(b), s(c));
    let q = caso.quadros.to_string();
    let k_txt = k.to_string();
    let mut base = vec!["rodar", "--rom", rom, "--quadros", &q];
    let ent;
    if let Some(e) = &caso.entradas {
        ent = s(e);
        base.extend(["--entradas", &ent]);
    }
    let mut ra = base.clone();
    ra.extend([
        "--saida",
        &sa,
        "--estado-a-cada",
        &k_txt,
        "--rotulos",
        rotulos,
    ]);
    // Video (P6 e P7): imagem por quadro (.s1i) e palavras de DMA (.s1v), para a Etapa 4.
    let prefixo_video = s(&dir.join(format!("{}.a", caso.nome)));
    ra.extend(["--video", &prefixo_video]);
    let saida_a = oraculo(&ra)?;
    let mut rb = base.clone();
    rb.extend(["--saida", &sb]);
    let saida_b = oraculo(&rb)?;
    let hash = campo(&saida_a, "hash_do_trace")?;
    if hash != campo(&saida_b, "hash_do_trace")? || !arquivos_iguais(a, b)? {
        return Err("duas execuções diferem".into());
    }
    remover(b);
    // Retomada: o último estado salvo antes do fim.
    let ultimo = (1..caso.quadros).rev().find(|n| n % k == 0);
    if let Some(n) = ultimo {
        let estado = s(&dir.join(format!("{}.a.{n}.s1e", caso.nome)));
        let mut rc = base.clone();
        rc.extend(["--saida", &sc, "--carregar-estado", &estado]);
        oraculo(&rc)?;
        let completo = BufReader::new(fs::File::open(a).map_err(|e| e.to_string())?);
        let retomado = BufReader::new(fs::File::open(c).map_err(|e| e.to_string())?);
        if let Some(q) = comparar_cauda(completo, retomado, n)? {
            return Err(format!("retomada do quadro {n} difere no quadro {q}"));
        }
    }
    Ok(hash)
}

#[cfg(test)]
mod testes {
    use super::*;
    use sonic1_fidelidade::{Cabecalho, Categoria, Escritor};
    use std::cell::RefCell;
    use std::io::BufRead;

    fn cabecalho() -> Cabecalho {
        Cabecalho {
            oraculo: "t".into(),
            rom_sha1: [1; 20],
            config_sha1: [2; 20],
            entradas_sha1: [3; 20],
            modo: 0,
        }
    }

    /// Quadro `i` do trace sintético; `muda_em` troca o hash de um quadro.
    fn quadro(i: u32, muda_em: Option<u32>) -> Quadro {
        let mut h = [[0u8; 20]; Categoria::TOTAL];
        h[0] = [u8::try_from(i % 200).unwrap(); 20];
        if Some(i) == muda_em {
            h[0] = [255; 20];
        }
        Quadro {
            numero: i,
            mclk_fim: u64::from(i) * 100,
            eventos: vec![],
            hashes: h,
        }
    }

    /// Trace dos quadros `de..ate`. Devolve os bytes e o hash do trace.
    fn trace_parcial(de: u32, ate: u32, muda_em: Option<u32>) -> (Vec<u8>, [u8; 20]) {
        let cab = cabecalho();
        let mut buf = Vec::new();
        let mut e = Escritor::novo(&mut buf, &cab).unwrap();
        for i in de..ate {
            e.escrever(&quadro(i, muda_em)).unwrap();
        }
        let hash = e.finalizar(&cab).unwrap();
        (buf, hash)
    }

    fn trace(n: u32, muda_em: Option<u32>) -> Vec<u8> {
        trace_parcial(0, n, muda_em).0
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

    fn temp(nome: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("websonic-{nome}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn cauda_igual() {
        let t = trace(30, None);
        assert_eq!(comparar_cauda(&t[..], &cauda(&t, 10)[..], 10), Ok(None));
    }

    #[test]
    fn cauda_com_quadro_plantado() {
        let t = trace(30, None);
        let ruim = cauda(&trace(30, Some(17)), 10);
        assert_eq!(comparar_cauda(&t[..], &ruim[..], 10), Ok(Some(17)));
    }

    #[test]
    fn cauda_curta_ou_longa() {
        let t = trace(30, None);
        assert_eq!(
            comparar_cauda(&t[..], &cauda(&trace(25, None), 10)[..], 10),
            Ok(Some(25))
        );
        assert_eq!(
            comparar_cauda(&trace(25, None)[..], &cauda(&t, 10)[..], 10),
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

    #[test]
    fn arquivos_iguais_e_diferentes() {
        let d = temp("cmp");
        let grande: Vec<u8> = (0..3_000_000u32).map(|i| (i % 251) as u8).collect();
        fs::write(d.join("a"), &grande).unwrap();
        fs::write(d.join("b"), &grande).unwrap();
        assert_eq!(arquivos_iguais(&d.join("a"), &d.join("b")), Ok(true));
        // Um byte diferente no meio, depois no último bloco, depois tamanho diferente.
        let mut g = grande.clone();
        g[1_500_000] ^= 1;
        fs::write(d.join("b"), &g).unwrap();
        assert_eq!(arquivos_iguais(&d.join("a"), &d.join("b")), Ok(false));
        let mut g = grande.clone();
        *g.last_mut().unwrap() ^= 1;
        fs::write(d.join("b"), &g).unwrap();
        assert_eq!(arquivos_iguais(&d.join("a"), &d.join("b")), Ok(false));
        fs::write(d.join("b"), &grande[..grande.len() - 1]).unwrap();
        assert_eq!(arquivos_iguais(&d.join("a"), &d.join("b")), Ok(false));
        assert!(arquivos_iguais(&d.join("a"), &d.join("nao-existe")).is_err());
    }

    /// Valor de uma opção `--nome VALOR` na linha de comando do oráculo de mentira.
    fn arg<'a>(args: &[&'a str], nome: &str) -> Option<&'a str> {
        args.iter()
            .position(|a| *a == nome)
            .and_then(|i| args.get(i + 1))
            .copied()
    }

    /// Oráculo de mentira: escreve o trace sintético com o mesmo contrato do real (`--saida`,
    /// `--estado-a-cada`, `--carregar-estado`, `hash_do_trace=` na saída). `defeito_retomada`
    /// troca o hash de um quadro só na retomada; `defeito_b` faz a segunda execução diferir.
    fn oraculo_falso(
        defeito_retomada: Option<u32>,
        defeito_b: bool,
        chamadas: &RefCell<Vec<String>>,
    ) -> impl Fn(&[&str]) -> Result<String, String> + '_ {
        move |args: &[&str]| {
            let saida = arg(args, "--saida").ok_or("sem --saida")?;
            let quadros: u32 = arg(args, "--quadros")
                .ok_or("sem --quadros")?
                .parse()
                .unwrap();
            chamadas.borrow_mut().push(saida.to_string());
            if let Some(estado) = arg(args, "--carregar-estado") {
                // `<nome>.a.<n>.s1e`: o estado tem de existir, como no oráculo real.
                assert!(Path::new(estado).is_file(), "estado ausente: {estado}");
                let n: u32 = estado
                    .rsplit('.')
                    .nth(1)
                    .and_then(|x| x.parse().ok())
                    .ok_or("nome de estado inválido")?;
                let (bytes, h) = trace_parcial(n, quadros, defeito_retomada);
                fs::write(saida, bytes).map_err(|e| e.to_string())?;
                return Ok(format!("hash_do_trace={}\n", em_hex(&h)));
            }
            let muda = (defeito_b && saida.ends_with(".b.s1t")).then_some(3);
            let (bytes, h) = trace_parcial(0, quadros, muda);
            fs::write(saida, bytes).map_err(|e| e.to_string())?;
            if let Some(k) = arg(args, "--estado-a-cada") {
                let k: u32 = k.parse().unwrap();
                let base = Path::new(saida).with_extension("");
                for n in (k..quadros).step_by(k as usize) {
                    fs::write(format!("{}.{n}.s1e", base.display()), b"estado").unwrap();
                }
            }
            Ok(format!("hash_do_trace={}\n", em_hex(&h)))
        }
    }

    fn nomes_na_pasta(d: &Path) -> Vec<String> {
        let mut v: Vec<String> = fs::read_dir(d)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        v.sort();
        v
    }

    fn caso_demo(quadros: u32) -> Caso {
        Caso {
            nome: "demo".into(),
            quadros,
            entradas: None,
        }
    }

    /// F2: não sobra arquivo da segunda execução nem da retomada; `quadros.tsv` tem uma linha por
    /// quadro; o hash é o do trace completo.
    #[test]
    fn caso_sem_copias_com_hashes_por_quadro() {
        let d = temp("trace-ok");
        let chamadas = RefCell::new(Vec::new());
        let f = oraculo_falso(None, false, &chamadas);
        let hash = um_caso(&f, "rom", "rotulos", &d, &caso_demo(25), 10, false).unwrap();
        // Três execuções: a, b (segunda) e c (retomada).
        assert_eq!(chamadas.borrow().len(), 3);
        assert_eq!(hash, em_hex(&trace_parcial(0, 25, None).1));
        // Ficam só os estados e o quadros.tsv. Nenhum .s1t.
        assert_eq!(
            nomes_na_pasta(&d),
            vec!["demo.a.10.s1e", "demo.a.20.s1e", "demo.quadros.tsv"]
        );
        let linhas = ler_quadros_tsv(&d.join("demo.quadros.tsv")).unwrap();
        assert_eq!(linhas.len(), 25);
        assert!(linhas.iter().enumerate().all(|(i, l)| l.0 as usize == i));
        // O hash por quadro é o do formato: o hash do trace fecha quando se soma os mesmos quadros.
        // (o escritor preenche o hash dos eventos, então o esperado também precisa fechar o quadro)
        let mut esperado = quadro(7, None);
        esperado.fechar();
        assert_eq!(linhas[7].1, em_hex(&hash_do_quadro(&esperado)));
        assert_ne!(linhas[7].1, linhas[8].1);
    }

    #[test]
    fn manter_trace_deixa_so_o_trace_completo() {
        let d = temp("trace-manter");
        let chamadas = RefCell::new(Vec::new());
        let f = oraculo_falso(None, false, &chamadas);
        um_caso(&f, "rom", "rotulos", &d, &caso_demo(25), 10, true).unwrap();
        assert_eq!(
            nomes_na_pasta(&d),
            vec![
                "demo.a.10.s1e",
                "demo.a.20.s1e",
                "demo.a.s1t",
                "demo.quadros.tsv"
            ]
        );
        assert_eq!(
            Leitor::abrir(BufReader::new(
                fs::File::open(d.join("demo.a.s1t")).unwrap()
            ))
            .unwrap()
            .cabecalho,
            cabecalho()
        );
    }

    #[test]
    fn falha_na_segunda_execucao_nao_deixa_copias() {
        let d = temp("trace-b");
        let chamadas = RefCell::new(Vec::new());
        let f = oraculo_falso(None, true, &chamadas);
        let e = um_caso(&f, "rom", "rotulos", &d, &caso_demo(25), 10, true).unwrap_err();
        assert!(e.contains("duas execuções diferem"), "{e}");
        assert!(!nomes_na_pasta(&d).iter().any(|n| n.ends_with(".s1t")));
    }

    #[test]
    fn falha_na_retomada_aponta_o_quadro_e_nao_deixa_copias() {
        let d = temp("trace-c");
        let chamadas = RefCell::new(Vec::new());
        let f = oraculo_falso(Some(23), false, &chamadas);
        let e = um_caso(&f, "rom", "rotulos", &d, &caso_demo(25), 10, true).unwrap_err();
        assert!(
            e.contains("retomada do quadro 20 difere no quadro 23"),
            "{e}"
        );
        assert!(!nomes_na_pasta(&d).iter().any(|n| n.ends_with(".s1t")));
        assert!(!nomes_na_pasta(&d).contains(&"demo.quadros.tsv".to_string()));
    }

    /// Trace com menos quadros do que o caso declara: falha, não passa em silêncio.
    #[test]
    fn quadros_a_menos_falha() {
        let d = temp("trace-menos");
        let chamadas = RefCell::new(Vec::new());
        let f = oraculo_falso(None, false, &chamadas);
        // O oráculo de mentira escreve 25 quadros, mas o caso diz 26.
        let f25 = |args: &[&str]| {
            let mut v: Vec<&str> = args.to_vec();
            if let Some(i) = v.iter().position(|a| *a == "--quadros") {
                v[i + 1] = "25";
            }
            f(&v)
        };
        let e = um_caso(&f25, "rom", "rotulos", &d, &caso_demo(26), 10, false).unwrap_err();
        assert!(e.contains("o trace tem 25 quadros, esperava 26"), "{e}");
    }

    /// Lê as linhas de um `.quadros.tsv`.
    fn ler_quadros_tsv(p: &Path) -> Result<Vec<(u32, String)>, String> {
        let f = BufReader::new(fs::File::open(p).map_err(|e| e.to_string())?);
        f.lines()
            .map(|l| {
                let l = l.map_err(|e| e.to_string())?;
                let (n, h) = l.split_once('\t').ok_or("linha sem TAB")?;
                Ok((n.parse().map_err(|_| "quadro inválido")?, h.to_string()))
            })
            .collect()
    }
}
