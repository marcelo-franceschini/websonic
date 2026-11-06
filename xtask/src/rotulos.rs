//! `cargo xtask rotulos`: gera `refs/rotulos.txt` a partir da tabela de simbolos do assembler.
//!
//! Roda no computador do Marcelo, nunca pela IA (a listagem do assembler mistura codigo e dados
//! da ROM em texto). O comando le a listagem, ignora tudo ate o titulo "Symbol Table" e extrai
//! so pares nome e valor. O arquivo de saida passa por `Rotulos::analisar`, que rejeita
//! qualquer linha que nao seja metadado.
//!
//! O formato esperado da tabela (assembler AS, secao "Symbol Table"): celulas `NOME : VALOR T`
//! separadas por `|`, com `*` antes do nome se o simbolo nao foi usado. O valor e decimal, ou
//! hexadecimal com prefixo `$` ou `0x`. A base dos numeros sem prefixo vem de `--base`.
//! Este formato ainda nao foi conferido contra uma listagem real: a primeira execucao do
//! Marcelo prova ou corrige.
//!
//! Uso:
//! `cargo xtask rotulos --entrada LISTAGEM --fontes DIR_ASM --saida refs/rotulos.txt [--base 10|16]`
use sonic1_fidelidade::rotulos::{Rotulos, TipoRotulo};
use std::collections::BTreeSet;
use std::path::Path;

/// Acima disto o valor e endereco da RAM do 68000.
const INICIO_RAM: u64 = 0xFF_0000;
/// Maior valor aceito (24 bits de endereco, ou constante de 32 bits ate o limite do formato).
const MAIOR_VALOR: u64 = 0xFFFF_FFFF;

fn opcao<'a>(args: &'a [String], nome: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == nome)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

pub fn executar(args: &[String]) -> Result<(), String> {
    let entrada = opcao(args, "--entrada").ok_or("falta --entrada")?;
    let fontes = opcao(args, "--fontes").ok_or("falta --fontes")?;
    let saida = opcao(args, "--saida").ok_or("falta --saida")?;
    let base: u32 = match opcao(args, "--base") {
        None | Some("16") => 16,
        Some("10") => 10,
        Some(_) => return Err("--base deve ser 10 ou 16".into()),
    };
    let listagem = std::fs::read(entrada).map_err(|e| format!("nao li a listagem: {e}"))?;
    // A listagem pode ter bytes que nao sao UTF-8; so o texto da tabela de simbolos importa.
    let listagem = String::from_utf8_lossy(&listagem);
    let simbolos = extrair_simbolos(&listagem, base)?;
    verificar_sanidade(&simbolos)?;
    let rotinas = nomes_de_rotinas(Path::new(fontes))?;
    let texto = montar_arquivo(&simbolos, &rotinas);
    // Garantia final: so metadado sai.
    let lidos = Rotulos::analisar(&texto).map_err(|e| format!("saida invalida: {e}"))?;
    if lidos.len() != simbolos.len() {
        return Err("a saida perdeu simbolos".into());
    }
    std::fs::write(saida, &texto).map_err(|e| e.to_string())?;
    let rotinas_n = texto.lines().filter(|l| l.contains("\trotina\t")).count();
    let variaveis_n = texto.lines().filter(|l| l.contains("\tvariavel\t")).count();
    println!(
        "simbolos={} rotinas={rotinas_n} variaveis={variaveis_n} bytes={}",
        simbolos.len(),
        texto.len()
    );
    Ok(())
}

/// Um mapa certo tem centenas de variaveis de RAM. O mapa lido na base errada tinha 2.
const MINIMO_DE_VARIAVEIS: usize = 50;

/// Recusa um mapa que parece lido na base errada.
pub fn verificar_sanidade(simbolos: &[(String, u64)]) -> Result<(), String> {
    let variaveis = simbolos.iter().filter(|(_, v)| *v >= INICIO_RAM).count();
    if variaveis < MINIMO_DE_VARIAVEIS {
        return Err(format!(
            "so {variaveis} simbolos de RAM (minimo {MINIMO_DE_VARIAVEIS}): base errada? tente --base 16"
        ));
    }
    Ok(())
}

/// Pares (nome, valor) da tabela de simbolos, ordenados por nome.
pub fn extrair_simbolos(listagem: &str, base: u32) -> Result<Vec<(String, u64)>, String> {
    let mut dentro = false;
    let mut achados: Vec<(String, u64)> = Vec::new();
    for linha in listagem.lines() {
        if !dentro {
            dentro = linha.contains("Symbol Table");
            continue;
        }
        for celula in linha.split('|') {
            let celula = celula.trim().trim_start_matches('*').trim();
            let Some((nome, resto)) = celula.split_once(" : ") else {
                continue;
            };
            let nome = nome.trim();
            let Some(valor) = resto.split_whitespace().next() else {
                continue;
            };
            if !nome_ok(nome) {
                continue;
            }
            if let Some(v) = valor_numerico(valor, base)
                && v <= MAIOR_VALOR
            {
                achados.push((nome.to_owned(), v));
            }
        }
    }
    if !dentro {
        return Err("a listagem nao tem a secao 'Symbol Table'".into());
    }
    if achados.is_empty() {
        return Err("a secao 'Symbol Table' nao trouxe nenhum simbolo no formato esperado".into());
    }
    achados.sort();
    achados.dedup_by(|a, b| a.0 == b.0);
    Ok(achados)
}

fn nome_ok(nome: &str) -> bool {
    !nome.is_empty()
        && nome
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '@'))
}

fn valor_numerico(texto: &str, base: u32) -> Option<u64> {
    if let Some(hex) = texto.strip_prefix('$').or_else(|| texto.strip_prefix("0x")) {
        u64::from_str_radix(hex, 16).ok()
    } else {
        u64::from_str_radix(texto, base).ok()
    }
}

/// Nomes definidos como rotulo (`nome:` no inicio da linha) nos `.asm`. So os nomes.
pub fn nomes_de_rotinas(pasta: &Path) -> Result<BTreeSet<String>, String> {
    let mut nomes = BTreeSet::new();
    let mut pilha = vec![pasta.to_path_buf()];
    while let Some(dir) = pilha.pop() {
        let mut entradas: Vec<_> = std::fs::read_dir(&dir)
            .map_err(|e| format!("{}: {e}", dir.display()))?
            .flatten()
            .collect();
        entradas.sort_by_key(|e| e.path());
        for entrada in entradas {
            let caminho = entrada.path();
            if caminho.is_dir() {
                pilha.push(caminho);
            } else if caminho.extension().is_some_and(|e| e == "asm") {
                let bytes = std::fs::read(&caminho).map_err(|e| e.to_string())?;
                for linha in String::from_utf8_lossy(&bytes).lines() {
                    if let Some(nome) = rotulo_no_inicio(linha) {
                        nomes.insert(nome.to_owned());
                    }
                }
            }
        }
    }
    Ok(nomes)
}

/// Se a linha comeca com `nome:`, devolve o nome.
fn rotulo_no_inicio(linha: &str) -> Option<&str> {
    let fim = linha.find(':')?;
    let nome = &linha[..fim];
    (nome
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && nome_ok(nome))
    .then_some(nome)
}

/// Monta o texto de `refs/rotulos.txt`.
pub fn montar_arquivo(simbolos: &[(String, u64)], rotinas: &BTreeSet<String>) -> String {
    let mut saida = String::from(
        "# Mapa de rotulos do s1disasm fixado (refs/disasm-commit.txt), Revision = 1.\n\
         # Gerado por: cargo xtask rotulos. Campos separados por TAB: nome, tipo, valor em hexadecimal.\n",
    );
    for (nome, valor) in simbolos {
        let tipo = if *valor >= INICIO_RAM {
            TipoRotulo::Variavel
        } else if rotinas.contains(nome) {
            TipoRotulo::Rotina
        } else {
            TipoRotulo::Constante
        };
        saida.push_str(&Rotulos::formatar(
            nome,
            tipo,
            u32::try_from(*valor).unwrap_or(u32::MAX),
        ));
        saida.push('\n');
    }
    saida
}

#[cfg(test)]
mod testes {
    use super::*;

    // Listagem sintetica no formato presumido. As linhas antes do titulo imitam codigo com bytes
    // e nao podem entrar na saida.
    const LISTAGEM: &str = "\
   1/       0 :                     nop\n\
   2/       2 : 4E71                nop\n\
\n\
  Symbol Table (* = unused):\n\
  --------------------------\n\
\n\
*ARCHITECTURE :  \"x86_64\" - |\n\
  EntryPoint :                   200 C |  VBlank :                  654 C |\n\
 v_framecount :              FFFFFE04 - |  f_pause :               $FFFFFE02 - |\n\
 Revision :                       1 - |\n";

    #[test]
    fn extrai_so_a_tabela() {
        let s = extrair_simbolos(LISTAGEM, 16).unwrap();
        let nomes: Vec<&str> = s.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(
            nomes,
            [
                "EntryPoint",
                "Revision",
                "VBlank",
                "f_pause",
                "v_framecount"
            ]
        );
        let valor = |n: &str| s.iter().find(|(x, _)| x == n).unwrap().1;
        assert_eq!(valor("EntryPoint"), 0x200);
        assert_eq!(valor("v_framecount"), 0xFFFF_FE04);
        assert_eq!(valor("f_pause"), 0xFFFF_FE02);
    }

    #[test]
    fn nada_antes_do_titulo_vira_simbolo() {
        // Sem o titulo, nao ha tabela: erro, sem tentar adivinhar.
        assert!(extrair_simbolos("   2/       2 : 4E71                nop\n", 16).is_err());
    }

    #[test]
    fn monta_arquivo_valido_e_classifica() {
        let s = extrair_simbolos(LISTAGEM, 16).unwrap();
        let rotinas: BTreeSet<String> = ["EntryPoint", "VBlank"]
            .iter()
            .map(|s| (*s).to_owned())
            .collect();
        let texto = montar_arquivo(&s, &rotinas);
        let r = Rotulos::analisar(&texto).unwrap();
        assert_eq!(r.tipo("EntryPoint"), Some(TipoRotulo::Rotina));
        assert_eq!(r.tipo("Revision"), Some(TipoRotulo::Constante));
        assert_eq!(r.tipo("v_framecount"), Some(TipoRotulo::Variavel));
        assert_eq!(r.resolver("VBlank"), Some(0x654));
    }

    #[test]
    fn rotulo_so_no_inicio_da_linha() {
        assert_eq!(rotulo_no_inicio("VBlank:"), Some("VBlank"));
        assert_eq!(
            rotulo_no_inicio("loc_1234:\t; comentario"),
            Some("loc_1234")
        );
        assert_eq!(rotulo_no_inicio("\tmove.w\td0,d1"), None);
        assert_eq!(rotulo_no_inicio("; so comentario: aqui"), None);
    }
}

#[cfg(test)]
mod testes_base {
    use super::*;

    /// Prova do defeito da Etapa 1: valores em hexadecimal sem prefixo lidos como decimal.
    #[test]
    fn base_10_descarta_valor_com_letra_e_base_16_le_certo() {
        let tabela = "Symbol Table\n GameProgram :  12C C |  v_x :  FFFFF600 - |\n";
        // Na base 10 nenhum dos dois valores le: o erro e o sintoma que a Etapa 1 nao viu.
        assert!(extrair_simbolos(tabela, 10).is_err());
        let em_16 = extrair_simbolos(tabela, 16).unwrap();
        let valor = |n: &str| em_16.iter().find(|(x, _)| x == n).unwrap().1;
        assert_eq!(valor("GameProgram"), 0x12C);
        assert_eq!(valor("v_x"), 0xFFFF_F600);
    }

    #[test]
    fn mapa_com_poucas_variaveis_e_recusado() {
        let poucos: Vec<(String, u64)> = vec![("a".into(), 0xFF_0000), ("b".into(), 0x200)];
        assert!(verificar_sanidade(&poucos).is_err());
        let muitos: Vec<(String, u64)> = (0..60u64)
            .map(|i| (format!("v_{i}"), 0xFFFF_0000 + i))
            .collect();
        assert!(verificar_sanidade(&muitos).is_ok());
    }
}
