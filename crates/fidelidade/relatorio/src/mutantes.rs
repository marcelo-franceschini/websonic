//! `cargo fidelidade mutantes`: bugs injetados (sub-etapa 5.7).
//!
//! Cada decisão do núcleo que espelha o original (`decisao!("Rotina:nome", ...)`) vira um
//! mutante: com a feature `mutacao`, `WEBSONIC_MUTANTE=Rotina:nome` faz essa decisão devolver o
//! contrário. A suíte roda uma vez por mutante.
//!
//! - `detectado`: algum teste falhou;
//! - `tempo`: a suíte passou do limite por mutante (conta como detectado, mas fica listado à
//!   parte, porque pode ser laço infinito);
//! - `sobrevivente`: nada falhou. Reprova o comando;
//! - `equivalente`: sobrevivente com justificativa em `cobertura/equivalentes.txt`, que só o
//!   Marcelo edita (a IA não aprova equivalência);
//! - `nao_rodado`: acabou o orçamento de tempo da etapa. A rodada completa, sem orçamento, é da
//!   auditoria (Marcelo): `--orcamento 0`.
//!
//! Uso: `cargo fidelidade mutantes [--orcamento SEGUNDOS] [--limite SEGUNDOS]
//!       [--pacotes a,b,c] [--saida ARQ]` (padrão: 600 s no total e 60 s por mutante).
use crate::{opcao, raiz};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Classe {
    Detectado,
    Tempo,
    Sobrevivente,
    Equivalente,
    NaoRodado,
}

impl Classe {
    pub fn nome(self) -> &'static str {
        match self {
            Self::Detectado => "detectado",
            Self::Tempo => "tempo",
            Self::Sobrevivente => "sobrevivente",
            Self::Equivalente => "equivalente",
            Self::NaoRodado => "nao_rodado",
        }
    }
}

/// O que aconteceu numa execução da suíte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Execucao {
    Passou,
    Falhou,
    Estourou,
}

pub fn classificar(e: Execucao, id: &str, equivalentes: &BTreeSet<String>) -> Classe {
    match e {
        Execucao::Falhou => Classe::Detectado,
        Execucao::Estourou => Classe::Tempo,
        Execucao::Passou if equivalentes.contains(id) => Classe::Equivalente,
        Execucao::Passou => Classe::Sobrevivente,
    }
}

/// Lê `id<TAB>justificativa`. Justificativa vazia não vale.
pub fn ler_equivalentes(texto: &str) -> Result<BTreeSet<String>, String> {
    let mut s = BTreeSet::new();
    for (i, l) in texto.lines().enumerate() {
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        let (id, just) = l
            .split_once('\t')
            .ok_or(format!("equivalentes, linha {}: falta TAB", i + 1))?;
        if just.trim().is_empty() {
            return Err(format!(
                "equivalentes, linha {}: justificativa vazia",
                i + 1
            ));
        }
        s.insert(id.to_string());
    }
    Ok(s)
}

/// Roda a suíte com o mutante `id`. `limite` corta a execução.
pub fn rodar_suite(id: &str, pacotes: &[String], limite: Duration) -> Result<Execucao, String> {
    let mut cmd = if let Ok(c) = std::env::var("WEBSONIC_COMANDO_DE_TESTE") {
        // Só para os testes deste pacote: um comando de shell no lugar do cargo.
        let mut x = Command::new("sh");
        x.args(["-c", &c]);
        x
    } else {
        let mut x = Command::new("cargo");
        x.args(["test", "--quiet"]);
        for p in pacotes {
            x.args(["-p", p]);
        }
        x
    };
    cmd.current_dir(raiz())
        .env("WEBSONIC_MUTANTE", id)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut filho = cmd.spawn().map_err(|e| e.to_string())?;
    let inicio = Instant::now();
    loop {
        if let Some(st) = filho.try_wait().map_err(|e| e.to_string())? {
            return Ok(if st.success() {
                Execucao::Passou
            } else {
                Execucao::Falhou
            });
        }
        if limite > Duration::ZERO && inicio.elapsed() > limite {
            let _ = filho.kill();
            let _ = filho.wait();
            return Ok(Execucao::Estourou);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Função que roda a suíte com um mutante.
pub type RodarSuite = dyn Fn(&str, &[String], Duration) -> Result<Execucao, String>;

/// Resumo e problemas (sobreviventes reprovam).
pub struct Resumo {
    pub texto: String,
    pub sobreviventes: Vec<String>,
}

pub fn executar_todos(
    ids: &[String],
    pacotes: &[String],
    equivalentes: &BTreeSet<String>,
    orcamento: Duration,
    limite: Duration,
    rodar: &RodarSuite,
) -> Result<Resumo, String> {
    let inicio = Instant::now();
    let mut classes: BTreeMap<String, Classe> = BTreeMap::new();
    for id in ids {
        if orcamento > Duration::ZERO && inicio.elapsed() >= orcamento {
            classes.insert(id.clone(), Classe::NaoRodado);
            continue;
        }
        let e = rodar(id, pacotes, limite)?;
        classes.insert(id.clone(), classificar(e, id, equivalentes));
    }
    let contar = |c: Classe| classes.values().filter(|x| **x == c).count();
    let mut texto = String::from("id\tclasse\n");
    for (id, c) in &classes {
        texto.push_str(&format!("{id}\t{}\n", c.nome()));
    }
    texto.push_str(&format!(
        "\n# mutantes: {}; detectados: {}; tempo: {}; sobreviventes: {}; equivalentes: {}; nao rodados: {}\n",
        classes.len(),
        contar(Classe::Detectado),
        contar(Classe::Tempo),
        contar(Classe::Sobrevivente),
        contar(Classe::Equivalente),
        contar(Classe::NaoRodado)
    ));
    let sobreviventes = classes
        .iter()
        .filter(|(_, c)| **c == Classe::Sobrevivente)
        .map(|(i, _)| i.clone())
        .collect();
    Ok(Resumo {
        texto,
        sobreviventes,
    })
}

pub fn executar(args: &[String]) -> Result<(), String> {
    let segundos = |n: &str, padrao: u64| -> Result<Duration, String> {
        Ok(Duration::from_secs(match opcao(args, n) {
            Some(v) => v.parse().map_err(|_| format!("{n} invalido"))?,
            None => padrao,
        }))
    };
    let orcamento = segundos("--orcamento", 600)?;
    let limite = segundos("--limite", 60)?;
    let pacotes: Vec<String> = opcao(args, "--pacotes")
        .unwrap_or("sonic1-core,sonic1-teste-nucleo,sonic1-teste-descompressao")
        .split(',')
        .map(str::to_string)
        .collect();
    let equivalentes = match fs::read_to_string(raiz().join("cobertura/equivalentes.txt")) {
        Ok(t) => ler_equivalentes(&t)?,
        Err(_) => BTreeSet::new(),
    };
    let ids = crate::cobertura::ids_do_nucleo(&raiz());
    let r = executar_todos(
        &ids,
        &pacotes,
        &equivalentes,
        orcamento,
        limite,
        &rodar_suite,
    )?;
    print!("{}", r.texto);
    if let Some(s) = opcao(args, "--saida") {
        fs::write(s, &r.texto).map_err(|e| e.to_string())?;
    }
    if r.sobreviventes.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "mutantes sobreviventes: {}",
            r.sobreviventes.join(" ")
        ))
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn classificacao() {
        let eq: BTreeSet<String> = ["B".to_string()].into();
        assert_eq!(classificar(Execucao::Falhou, "A", &eq), Classe::Detectado);
        assert_eq!(
            classificar(Execucao::Passou, "A", &eq),
            Classe::Sobrevivente
        );
        assert_eq!(classificar(Execucao::Passou, "B", &eq), Classe::Equivalente);
        assert_eq!(classificar(Execucao::Estourou, "A", &eq), Classe::Tempo);
    }

    #[test]
    fn equivalente_sem_justificativa_e_recusado() {
        assert!(ler_equivalentes("A:x\tnunca muda o resultado, por que...\n").is_ok());
        assert!(ler_equivalentes("A:x\t \n").is_err());
        assert!(ler_equivalentes("A:x").is_err());
    }

    #[test]
    fn orcamento_deixa_o_resto_como_nao_rodado() {
        let ids: Vec<String> = ["A".into(), "B".into(), "C".into()].into();
        let rodar = |_: &str, _: &[String], _: Duration| -> Result<Execucao, String> {
            std::thread::sleep(Duration::from_millis(30));
            Ok(Execucao::Falhou)
        };
        let r = executar_todos(
            &ids,
            &[],
            &BTreeSet::new(),
            Duration::from_millis(20),
            Duration::from_secs(1),
            &rodar,
        )
        .unwrap();
        assert!(r.texto.contains("A\tdetectado"));
        assert!(r.texto.contains("B\tnao_rodado"));
        assert!(r.texto.contains("nao rodados: 2"));
        // Orcamento zero: sem limite, roda todos.
        let r = executar_todos(
            &ids,
            &[],
            &BTreeSet::new(),
            Duration::ZERO,
            Duration::from_secs(1),
            &rodar,
        )
        .unwrap();
        assert!(r.texto.contains("nao rodados: 0"));
    }

    #[test]
    fn suite_de_teste_distingue_passou_falhou_e_estourou() {
        // Um comando de shell no lugar do cargo: falha so para o mutante "ruim", dorme para "lento".
        // SAFETY: este e o unico teste do pacote que mexe nesta variavel, e os outros nao a leem.
        unsafe {
            std::env::set_var(
                "WEBSONIC_COMANDO_DE_TESTE",
                "if [ \"$WEBSONIC_MUTANTE\" = ruim ]; then exit 1; elif [ \"$WEBSONIC_MUTANTE\" = lento ]; then sleep 5; fi",
            );
        }
        let l = Duration::from_millis(500);
        assert_eq!(rodar_suite("bom", &[], l).unwrap(), Execucao::Passou);
        assert_eq!(rodar_suite("ruim", &[], l).unwrap(), Execucao::Falhou);
        assert_eq!(rodar_suite("lento", &[], l).unwrap(), Execucao::Estourou);
        unsafe { std::env::remove_var("WEBSONIC_COMANDO_DE_TESTE") };
    }
}
