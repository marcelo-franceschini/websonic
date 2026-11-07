//! `cargo xtask tas <converter|conferir>`: TAS do TASVideos como entradas do corpus (sub-etapa 2.10).
//!
//! - `converter`: lê um GMV (Gens Movie) e escreve `corpus/<nome>.s1in` e o `.meta`. O controle
//!   de 6 botões é aceito: X, Y, Z, Mode e a porta 2 são descartados, e o `.meta` guarda em
//!   quantos quadros havia botão descartado. Estado salvo e 50 Hz são recusados.
//! - `conferir`: roda o oráculo com a ROM, com as entradas do TAS e mais 3600 quadros sem botão,
//!   e olha o modo de jogo a cada 60 quadros. O TAS é aceito se o jogo chegar a `id_Ending` ou
//!   `id_Credits`. Se não chegar, o TAS perdeu a sincronia com a REV01 e é descartado (o requisito
//!   aceita o descarte): com `--aplicar`, o caso sai do corpus e ganha uma linha em
//!   `corpus/descartados.txt`.
//!
//! A saída tem só números e nomes. O modo de jogo (`v_gamemode`) vem do mapa de rótulos.
//! O TAS é só botões, sem dado da ROM.
//!
//! Uso:
//! `cargo xtask tas converter --gmv ARQ.gmv --saida corpus/NOME.s1in --fonte URL --autor TEXTO --licenca TEXTO`
//! `cargo xtask tas conferir --nome NOME --rom ROM --oraculo BIN [--rotulos refs/rotulos.txt] [--corpus corpus] [--aplicar]`
use sonic1_corpus::{escrever_s1in, gmv, ler_s1in, meta};
use sonic1_fidelidade::hash::{em_hex, sha1};
use sonic1_fidelidade::rotulos::Rotulos;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

/// Quadros sem botão depois do fim das entradas, para a abertura e o final terminarem.
const QUADROS_EXTRA: usize = 3600;
/// Intervalo, em quadros, entre as leituras do modo de jogo.
const PERFIL_A_CADA: u32 = 60;
/// O jogo liga o bit 7 do modo durante a carga da fase (pré-sequência da fase). Ele é ignorado.
const BIT_CARGA: u8 = 0x80;

fn opcao<'a>(args: &'a [String], nome: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == nome)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

pub fn executar(args: &[String]) -> Result<(), String> {
    let resto = args.get(1..).unwrap_or_default();
    match args.first().map(String::as_str) {
        Some("converter") => converter(resto),
        Some("conferir") => conferir(resto),
        _ => Err("uso: cargo xtask tas <converter|conferir>".into()),
    }
}

/// Uma leitura do modo de jogo.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Amostra {
    pub quadro: u32,
    /// Modo de jogo sem o bit de carga.
    pub modo: u8,
}

/// Lê as linhas `perfil quadro=N modo=M zona=Z ato=A demo=D` da saída do oráculo.
/// As outras linhas e as malformadas são ignoradas.
pub fn ler_perfil(saida: &str) -> Vec<Amostra> {
    let campo = |l: &str, chave: &str| -> Option<u32> {
        l.split_whitespace()
            .find_map(|p| p.strip_prefix(chave)?.strip_prefix('='))?
            .parse()
            .ok()
    };
    saida
        .lines()
        .filter(|l| l.starts_with("perfil "))
        .filter_map(|l| {
            let modo = u8::try_from(campo(l, "modo")?).ok()?;
            Some(Amostra {
                quadro: campo(l, "quadro")?,
                modo: modo & !BIT_CARGA,
            })
        })
        .collect()
}

/// Resultado da conferência.
#[derive(Debug, PartialEq, Eq)]
pub enum Veredito {
    /// O jogo chegou ao modo `modo` no quadro `quadro` (primeira leitura).
    Aceito { quadro: u32, modo: u8 },
    /// Nunca chegou. `ultimo` é a última leitura (ou `None` se não houve nenhuma).
    Descartado { ultimo: Option<Amostra> },
}

/// Aceito se alguma leitura tem um dos modos de `alvos`.
pub fn julgar(amostras: &[Amostra], alvos: &[u8]) -> Veredito {
    match amostras.iter().find(|a| alvos.contains(&a.modo)) {
        Some(a) => Veredito::Aceito {
            quadro: a.quadro,
            modo: a.modo,
        },
        None => Veredito::Descartado {
            ultimo: amostras.last().copied(),
        },
    }
}

/// Resumo dos modos: `quadro=modo` a cada mudança de modo. Só números.
pub fn resumo_dos_modos(amostras: &[Amostra]) -> String {
    let mut partes = Vec::new();
    let mut anterior = None;
    for a in amostras {
        if anterior != Some(a.modo) {
            partes.push(format!("{}={}", a.quadro, a.modo));
            anterior = Some(a.modo);
        }
    }
    partes.join(" ")
}

/// Escreve a linha do TAS descartado em `descartados.txt` e apaga o `.s1in` e o `.meta`.
pub fn descartar(corpus: &Path, nome: &str, fonte: &str, motivo: &str) -> Result<(), String> {
    // Confere antes de escrever: nada de linha registrada sem os arquivos para remover.
    for ext in ["s1in", "meta"] {
        let arq = corpus.join(format!("{nome}.{ext}"));
        if !arq.is_file() {
            return Err(format!("{}: nao existe", arq.display()));
        }
    }
    let lista = corpus.join("descartados.txt");
    let novo = !lista.exists();
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&lista)
        .map_err(|e| format!("{}: {e}", lista.display()))?;
    if novo {
        writeln!(f, "# nome<TAB>fonte<TAB>motivo. Entradas conferidas contra a REV01 que perderam a sincronia.")
            .map_err(|e| e.to_string())?;
    }
    writeln!(f, "{nome}\t{fonte}\t{motivo}").map_err(|e| e.to_string())?;
    for ext in ["s1in", "meta"] {
        fs::remove_file(corpus.join(format!("{nome}.{ext}")))
            .map_err(|e| format!("{nome}.{ext}: {e}"))?;
    }
    Ok(())
}

fn converter(args: &[String]) -> Result<(), String> {
    let gmv_arq = opcao(args, "--gmv").ok_or("falta --gmv")?;
    let saida = opcao(args, "--saida").ok_or("falta --saida")?;
    let dados = fs::read(gmv_arq).map_err(|e| format!("{gmv_arq}: {e}"))?;
    let c = gmv::converter(&dados).map_err(|e| format!("{gmv_arq}: {e:?}"))?;
    fs::write(saida, escrever_s1in(&c.mascaras)).map_err(|e| e.to_string())?;
    // O .meta usa o mesmo código do `corpus-meta`; depois entram os campos do TAS.
    super::corpus::escrever_meta(&[
        "--entradas".into(),
        saida.into(),
        "--fonte".into(),
        opcao(args, "--fonte").ok_or("falta --fonte")?.into(),
        "--licenca".into(),
        opcao(args, "--licenca").ok_or("falta --licenca")?.into(),
        "--autor".into(),
        opcao(args, "--autor").ok_or("falta --autor")?.into(),
    ])?;
    let arq_meta = Path::new(saida).with_extension("meta");
    let mut texto = fs::read_to_string(&arq_meta).map_err(|e| e.to_string())?;
    texto.push_str(&format!(
        "origem_sha1: {}\ndescartados_xyz_mode_quadros: {}\ndescartados_porta2_quadros: {}\n",
        em_hex(&sha1(&dados)),
        c.quadros_xyz_mode,
        c.quadros_porta2
    ));
    meta::ler(&texto).map_err(|e| format!("meta inválido: {e:?}"))?;
    fs::write(&arq_meta, texto).map_err(|e| e.to_string())?;
    println!(
        "tas: {} quadros, {} com X/Y/Z/Mode descartados, {} com a porta 2 descartada",
        c.mascaras.len(),
        c.quadros_xyz_mode,
        c.quadros_porta2
    );
    Ok(())
}

/// Os modos que provam que o TAS terminou o jogo, lidos do mapa de rótulos.
fn modos_de_chegada(rotulos: &Path) -> Result<Vec<u8>, String> {
    let r = Rotulos::analisar(&fs::read_to_string(rotulos).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    ["id_Ending", "id_Credits"]
        .iter()
        .map(|n| {
            let v = r.resolver(n).ok_or(format!("{n} ausente do mapa"))?;
            u8::try_from(v).map_err(|_| format!("{n} não cabe em um byte"))
        })
        .collect()
}

/// Roda a conferência com um oráculo qualquer (o teste usa um de mentira).
fn conferir_com(
    oraculo: &dyn Fn(&[&str]) -> Result<String, String>,
    corpus: &Path,
    nome: &str,
    rom: &str,
    rotulos: &Path,
    tmp: &Path,
) -> Result<(Veredito, Vec<Amostra>), String> {
    let entradas = ler_s1in(
        &fs::read_to_string(corpus.join(format!("{nome}.s1in"))).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("{nome}.s1in: {e:?}"))?;
    let total = entradas.len() + QUADROS_EXTRA;
    // As entradas do TAS e depois os quadros sem botão.
    let mut estendidas = entradas;
    estendidas.resize(total, 0);
    let arq = tmp.join(format!("{nome}.conferir.s1in"));
    fs::write(&arq, escrever_s1in(&estendidas)).map_err(|e| e.to_string())?;
    let q = total.to_string();
    let p = PERFIL_A_CADA.to_string();
    let r = rotulos.to_string_lossy();
    let a = arq.to_string_lossy();
    let saida = oraculo(&[
        "rodar",
        "--rom",
        rom,
        "--quadros",
        &q,
        "--saida",
        "/dev/null",
        "--entradas",
        &a,
        "--rotulos",
        &r,
        "--perfil-a-cada",
        &p,
    ]);
    let _ = fs::remove_file(&arq);
    let amostras = ler_perfil(&saida?);
    Ok((julgar(&amostras, &modos_de_chegada(rotulos)?), amostras))
}

fn conferir(args: &[String]) -> Result<(), String> {
    let nome = opcao(args, "--nome").ok_or("falta --nome")?;
    let rom = opcao(args, "--rom").ok_or("falta --rom")?;
    let bin = opcao(args, "--oraculo")
        .ok_or("falta --oraculo")?
        .to_string();
    let rotulos = Path::new(opcao(args, "--rotulos").unwrap_or("refs/rotulos.txt"));
    let corpus = Path::new(opcao(args, "--corpus").unwrap_or("corpus"));
    let oraculo = move |a: &[&str]| -> Result<String, String> {
        let s = Command::new(&bin)
            .args(a)
            .output()
            .map_err(|e| format!("oráculo: {e}"))?;
        if !s.status.success() {
            return Err(format!(
                "oráculo falhou: {}",
                String::from_utf8_lossy(&s.stderr).trim()
            ));
        }
        Ok(String::from_utf8_lossy(&s.stdout).into_owned())
    };
    let (veredito, amostras) =
        conferir_com(&oraculo, corpus, nome, rom, rotulos, &std::env::temp_dir())?;
    println!(
        "{nome}: modos (quadro=modo) {}",
        resumo_dos_modos(&amostras)
    );
    match veredito {
        Veredito::Aceito { quadro, modo } => {
            println!("{nome}: ACEITO (modo {modo} lido no quadro {quadro})");
            Ok(())
        }
        Veredito::Descartado { ultimo } => {
            let motivo = match ultimo {
                Some(a) => format!(
                    "nao chegou a id_Ending nem a id_Credits; ultimo modo lido {} no quadro {}",
                    a.modo, a.quadro
                ),
                None => "o oraculo nao devolveu nenhuma leitura de modo".to_string(),
            };
            println!("{nome}: DESCARTADO ({motivo})");
            if args.iter().any(|a| a == "--aplicar") {
                let m = fs::read_to_string(corpus.join(format!("{nome}.meta")))
                    .map_err(|e| format!("{nome}.meta: {e}"))?;
                let campos = meta::ler(&m).map_err(|e| format!("{nome}.meta: {e:?}"))?;
                let fonte = meta::valor(&campos, "fonte").unwrap_or("?");
                descartar(corpus, nome, fonte, &motivo)?;
                println!("{nome}: removido do corpus e registrado em descartados.txt");
            }
            // Descartar não é erro da ferramenta: o script segue para o próximo TAS.
            Ok(())
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use std::path::PathBuf;

    fn temp(nome: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("websonic-tas-{nome}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    /// Saída de perfil inventada: o jogo passa por 0, 4, 12 (fase, com o bit de carga) e 24.
    const CHEGA: &str = "hash_do_trace=x\nperfil quadro=60 modo=0 zona=0 ato=0 demo=0\nperfil quadro=120 modo=4 zona=0 ato=0 demo=0\nperfil quadro=180 modo=140 zona=0 ato=0 demo=0\nperfil quadro=240 modo=12 zona=0 ato=0 demo=0\nperfil quadro=300 modo=24 zona=6 ato=0 demo=0\nlixo\nperfil quadro=360 modo=28 zona=6 ato=0 demo=0\n";
    const NAO_CHEGA: &str = "perfil quadro=60 modo=0 zona=0 ato=0 demo=0\nperfil quadro=120 modo=12 zona=1 ato=0 demo=0\nperfil quadro=180 modo=12 zona=1 ato=1 demo=0\n";

    #[test]
    fn perfil_ignora_bit_de_carga_e_linhas_ruins() {
        let a = ler_perfil(CHEGA);
        assert_eq!(a.len(), 6);
        // modo 140 = 0x8c: com o bit de carga, vale 12.
        assert_eq!(
            a[2],
            Amostra {
                quadro: 180,
                modo: 12
            }
        );
        assert!(ler_perfil("perfil quadro=1 modo=999 zona=0\nperfil modo=1\n").is_empty());
    }

    #[test]
    fn aceito_quando_chega_ao_final() {
        // id_Ending = 0x18 = 24 e id_Credits = 0x1c = 28.
        assert_eq!(
            julgar(&ler_perfil(CHEGA), &[0x18, 0x1c]),
            Veredito::Aceito {
                quadro: 300,
                modo: 24
            }
        );
        assert_eq!(
            julgar(
                &ler_perfil("perfil quadro=60 modo=156 zona=0 ato=0 demo=0\n"),
                &[0x18, 0x1c]
            ),
            Veredito::Aceito {
                quadro: 60,
                modo: 28
            }
        );
    }

    #[test]
    fn descartado_quando_nao_chega() {
        assert_eq!(
            julgar(&ler_perfil(NAO_CHEGA), &[0x18, 0x1c]),
            Veredito::Descartado {
                ultimo: Some(Amostra {
                    quadro: 180,
                    modo: 12
                })
            }
        );
        assert_eq!(julgar(&[], &[0x18]), Veredito::Descartado { ultimo: None });
    }

    #[test]
    fn resumo_so_nas_mudancas() {
        assert_eq!(resumo_dos_modos(&ler_perfil(NAO_CHEGA)), "60=0 120=12");
    }

    /// Conferência de ponta a ponta com um oráculo de mentira: um caso aceito e um descartado.
    #[test]
    fn conferencia_com_aceito_e_descartado() {
        let d = temp("conf");
        let corpus = d.join("corpus");
        fs::create_dir_all(&corpus).unwrap();
        for n in ["bom", "ruim"] {
            fs::write(
                corpus.join(format!("{n}.s1in")),
                escrever_s1in(&[0x80, 0x00, 0x08]),
            )
            .unwrap();
            fs::write(corpus.join(format!("{n}.meta")), "fonte: https://exemplo\n").unwrap();
        }
        let rotulos = d.join("rotulos.txt");
        fs::write(&rotulos, "id_Ending\trotina\t18\nid_Credits\trotina\t1c\n").unwrap();
        // O oráculo de mentira confere o que recebe: entradas com 3 + 3600 linhas e perfil a cada 60.
        let falso = |saida: &'static str| {
            move |a: &[&str]| -> Result<String, String> {
                let pos = |n: &str| a.iter().position(|x| *x == n).unwrap();
                assert_eq!(a[pos("--quadros") + 1], "3603");
                assert_eq!(a[pos("--perfil-a-cada") + 1], "60");
                assert_eq!(a[pos("--saida") + 1], "/dev/null");
                let entradas = fs::read_to_string(a[pos("--entradas") + 1]).unwrap();
                assert_eq!(entradas.lines().count(), 3603);
                assert!(entradas.starts_with("80\n00\n08\n00\n"));
                Ok(saida.to_string())
            }
        };
        let (v, _) = conferir_com(&falso(CHEGA), &corpus, "bom", "rom", &rotulos, &d).unwrap();
        assert_eq!(
            v,
            Veredito::Aceito {
                quadro: 300,
                modo: 24
            }
        );
        let (v, _) = conferir_com(&falso(NAO_CHEGA), &corpus, "ruim", "rom", &rotulos, &d).unwrap();
        assert!(matches!(v, Veredito::Descartado { .. }));
        // O arquivo temporário das entradas estendidas não sobra.
        assert!(!d.join("ruim.conferir.s1in").exists());
        // Aplicar o descarte: sai do corpus e ganha uma linha.
        descartar(&corpus, "ruim", "https://exemplo", "motivo x").unwrap();
        assert!(!corpus.join("ruim.s1in").exists());
        assert!(corpus.join("bom.s1in").exists());
        let lista = fs::read_to_string(corpus.join("descartados.txt")).unwrap();
        assert!(
            lista
                .lines()
                .any(|l| l == "ruim\thttps://exemplo\tmotivo x")
        );
        // Sem os arquivos, o descarte falha e não registra nada.
        descartar(&corpus, "ruim", "outra", "y").unwrap_err();
        assert_eq!(
            fs::read_to_string(corpus.join("descartados.txt")).unwrap(),
            lista
        );
    }

    #[test]
    fn oraculo_com_erro_vira_erro() {
        let d = temp("erro");
        fs::write(d.join("a.s1in"), "00\n").unwrap();
        let rotulos = d.join("r.txt");
        fs::write(&rotulos, "id_Ending\trotina\t18\nid_Credits\trotina\t1c\n").unwrap();
        let ruim = |_: &[&str]| -> Result<String, String> { Err("falhou".into()) };
        assert!(conferir_com(&ruim, &d, "a", "rom", &rotulos, &d).is_err());
    }

    #[test]
    fn rotulos_sem_os_modos_falham() {
        let d = temp("rot");
        let r = d.join("r.txt");
        fs::write(&r, "id_Ending\trotina\t18\n").unwrap();
        assert!(modos_de_chegada(&r).is_err());
    }
}
