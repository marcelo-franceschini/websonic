//! `cargo xtask ids-som` (Etapa 6): lista os IDs de música, efeito, especial e sinalizador do
//! driver de som, com o nome do arquivo de texto do disassembly. Só ID, tipo e nome.
//!
//! Os IDs saem dos rótulos `ptr_musXX`, `ptr_sndXX` e `ptr_flgXX` de `refs/rotulos.txt`. Os
//! nomes saem dos nomes dos arquivos em `sound/music` e `sound/sfx` (opcional, `--fontes`).
//!
//! Uso: `cargo xtask ids-som [--rotulos refs/rotulos.txt] [--fontes DIR_ASM] [--saida crates/teste-som/ids.txt]`
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Tipo do ID, pela faixa do driver: música 81 a 9F, efeito A0 a CF, especial D0 a DF, sinalizador E0 a EF.
fn tipo_do_id(id: u8) -> &'static str {
    match id {
        0x81..=0x9F => "musica",
        0xA0..=0xCF => "efeito",
        0xD0..=0xDF => "especial",
        0xE0..=0xEF => "sinalizador",
        _ => "fora_da_faixa",
    }
}

/// Lê os IDs dos rótulos. Devolve ID e nome do rótulo, ordenados por ID.
pub fn ids_dos_rotulos(texto: &str) -> Result<Vec<(u8, String)>, String> {
    let mut achados: BTreeMap<u8, String> = BTreeMap::new();
    for linha in texto.lines() {
        let Some(nome) = linha.split('\t').next() else {
            continue;
        };
        for prefixo in ["ptr_mus", "ptr_snd", "ptr_flg"] {
            let Some(hex) = nome.strip_prefix(prefixo) else {
                continue;
            };
            // `ptr_musend` e parecidos marcam o fim da tabela: nao sao IDs.
            if hex.len() != 2 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
                continue;
            }
            let id = u8::from_str_radix(hex, 16).map_err(|e| e.to_string())?;
            if achados.insert(id, nome.to_string()).is_some() {
                return Err(format!("ID {id:#04x} aparece duas vezes"));
            }
        }
    }
    if achados.is_empty() {
        return Err("nenhum rotulo ptr_mus, ptr_snd ou ptr_flg no mapa".into());
    }
    Ok(achados.into_iter().collect())
}

/// Nome a partir de um nome de arquivo como `Mus81 - GHZ.asm` ou `SndA2.asm`.
fn nome_do_arquivo(arquivo: &str) -> Option<(u8, String)> {
    let base = arquivo.strip_suffix(".asm")?;
    let resto = base
        .strip_prefix("Mus")
        .or_else(|| base.strip_prefix("Snd"))?;
    let (hex, nome) = match resto.split_once(" - ") {
        Some((h, n)) => (h, n.to_string()),
        None => (resto, "sem_nome".to_string()),
    };
    if hex.len() != 2 {
        return None;
    }
    Some((u8::from_str_radix(hex, 16).ok()?, nome.replace(' ', "_")))
}

fn nomes(fontes: &str) -> BTreeMap<u8, String> {
    let mut m = BTreeMap::new();
    for sub in ["sound/music", "sound/sfx"] {
        let Ok(dir) = fs::read_dir(Path::new(fontes).join(sub)) else {
            continue;
        };
        for e in dir.filter_map(Result::ok) {
            if let Some((id, nome)) = e.file_name().to_str().and_then(nome_do_arquivo) {
                m.insert(id, nome);
            }
        }
    }
    m
}

/// Linhas do arquivo de saída: `ID<TAB>tipo<TAB>nome`. IDs sem arquivo de texto ficam `sem_arquivo`.
pub fn montar(ids: &[(u8, String)], nomes: &BTreeMap<u8, String>) -> String {
    let mut s = String::from(
        "# IDs de som (Etapa 6). Gerado por cargo xtask ids-som. Campos: ID em hexadecimal, tipo, nome do arquivo de texto do disassembly.\n",
    );
    for (id, _) in ids {
        let nome = nomes.get(id).map_or("sem_arquivo", String::as_str);
        s.push_str(&format!("{id:02X}\t{}\t{nome}\n", tipo_do_id(*id)));
    }
    s
}

pub fn executar(args: &[String]) -> Result<(), String> {
    let opc = |n: &str| {
        args.iter()
            .position(|a| a == n)
            .and_then(|i| args.get(i + 1))
            .map(String::as_str)
    };
    let raiz = super::raiz();
    let rotulos = opc("--rotulos").map_or_else(|| raiz.join("refs/rotulos.txt"), Into::into);
    let saida = opc("--saida").map_or_else(|| raiz.join("crates/teste-som/ids.txt"), Into::into);
    let texto = fs::read_to_string(&rotulos).map_err(|e| format!("nao li os rotulos: {e}"))?;
    let ids = ids_dos_rotulos(&texto)?;
    let nm = opc("--fontes").map(nomes).unwrap_or_default();
    let conteudo = montar(&ids, &nm);
    if let Some(pai) = Path::new(&saida).parent() {
        fs::create_dir_all(pai).map_err(|e| e.to_string())?;
    }
    fs::write(&saida, &conteudo).map_err(|e| e.to_string())?;
    let conta = |t: &str| {
        conteudo
            .lines()
            .filter(|l| l.contains(&format!("\t{t}\t")))
            .count()
    };
    println!(
        "ids-som: {} IDs (musica {}, efeito {}, especial {}, sinalizador {})",
        ids.len(),
        conta("musica"),
        conta("efeito"),
        conta("especial"),
        conta("sinalizador")
    );
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn ids_vem_dos_rotulos_sem_o_fim_da_tabela() {
        let r = "ptr_mus81\trotina\t71a9c\nptr_musend\tconstante\t71ae8\nptr_sndA0\trotina\t78b44\nptr_flgE1\trotina\t71f9c\nptr_sndend\tconstante\t1\nOutro\trotina\t0\n";
        let ids = ids_dos_rotulos(r).unwrap();
        assert_eq!(
            ids.iter().map(|(i, _)| *i).collect::<Vec<_>>(),
            vec![0x81, 0xA0, 0xE1]
        );
        assert!(ids_dos_rotulos("Outro\trotina\t0\n").is_err());
        assert!(ids_dos_rotulos("ptr_mus81\trotina\t1\nptr_snd81\trotina\t2\n").is_err());
    }

    #[test]
    fn nome_do_arquivo_com_e_sem_titulo() {
        assert_eq!(
            nome_do_arquivo("Mus81 - GHZ.asm"),
            Some((0x81, "GHZ".to_string()))
        );
        assert_eq!(
            nome_do_arquivo("SndA2.asm"),
            Some((0xA2, "sem_nome".to_string()))
        );
        assert_eq!(nome_do_arquivo("z80.asm"), None);
    }

    #[test]
    fn saida_tem_tipo_por_faixa() {
        let ids = vec![
            (0x81, "a".to_string()),
            (0xA0, "b".to_string()),
            (0xD0, "c".to_string()),
            (0xE1, "d".to_string()),
        ];
        let mut nm = BTreeMap::new();
        nm.insert(0x81, "GHZ".to_string());
        let s = montar(&ids, &nm);
        assert!(s.contains("81\tmusica\tGHZ\n"));
        assert!(s.contains("A0\tefeito\tsem_arquivo\n"));
        assert!(s.contains("D0\tespecial\t"));
        assert!(s.contains("E1\tsinalizador\t"));
    }
}
