//! Metadados de um caso do corpus (`corpus/<nome>.meta`): texto `chave: valor`, uma por linha.

use crate::Erro;

/// Chaves obrigatorias, na ordem em que sao escritas.
pub const CHAVES: [&str; 6] = [
    "fonte",
    "licenca",
    "rom_sha1",
    "quadros",
    "entradas_sha1",
    "autor",
];

/// Le o `.meta` e valida as chaves obrigatorias e os formatos (SHA-1 com 40 hex, quadros numerico).
pub fn ler(texto: &str) -> Result<Vec<(String, String)>, Erro> {
    let mut campos: Vec<(String, String)> = Vec::new();
    for linha in texto.lines() {
        let linha = linha.trim();
        if linha.is_empty() || linha.starts_with('#') {
            continue;
        }
        let (k, v) = linha
            .split_once(':')
            .ok_or(Erro::Formato("linha sem ':'"))?;
        campos.push((k.trim().to_string(), v.trim().to_string()));
    }
    for chave in CHAVES {
        let v = campos
            .iter()
            .find(|(k, _)| k == chave)
            .map(|(_, v)| v.as_str())
            .ok_or(Erro::Formato("falta chave obrigatoria"))?;
        let hex40 = v.len() == 40 && v.bytes().all(|b| b.is_ascii_hexdigit());
        match chave {
            "rom_sha1" | "entradas_sha1" if !hex40 => return Err(Erro::Formato("sha1 invalido")),
            "quadros" if v.parse::<usize>().is_err() => {
                return Err(Erro::Formato("quadros invalido"));
            }
            _ => {}
        }
    }
    Ok(campos)
}

/// Valor de uma chave.
pub fn valor<'a>(campos: &'a [(String, String)], chave: &str) -> Option<&'a str> {
    campos
        .iter()
        .find(|(k, _)| k == chave)
        .map(|(_, v)| v.as_str())
}

#[cfg(test)]
mod testes {
    use super::*;

    const H: &str = "0123456789abcdef0123456789abcdef01234567";

    fn ok() -> String {
        format!(
            "fonte: sintetica\nlicenca: propria\nrom_sha1: {H}\nquadros: 3\nentradas_sha1: {H}\nautor: teste\n"
        )
    }

    #[test]
    fn valido() {
        let c = ler(&ok()).unwrap();
        assert_eq!(valor(&c, "quadros"), Some("3"));
    }

    #[test]
    fn invalidos() {
        assert!(ler(&ok().replace("quadros: 3\n", "")).is_err());
        assert!(ler(&ok().replace("quadros: 3", "quadros: x")).is_err());
        assert!(ler(&ok().replacen(H, "zz", 1)).is_err());
        assert!(ler("sem dois pontos\n").is_err());
    }
}
