//! `cargo xtask inventario-memoria` (pedido P2): inventário da memória compartilhada.
//!
//! Entrada: mapas de acessos à RAM do 68000 por contexto, gerados pelo oráculo
//! (`oraculo rodar --acessos ARQ`), e o mapa de rótulos. Saída: `docs/inventario-memoria.md`,
//! só com nomes, endereços, contextos e contagens. Nenhum valor da RAM passa por aqui.
//!
//! Formato do mapa de acessos (uma linha por byte tocado): `ENDERECO<TAB>BITS`, com o endereço em
//! hexadecimal de 6 dígitos e os bits em 4 dígitos hexadecimais. Bit `contexto * 3 + tipo`:
//! contexto 0 laço principal, 1 VBlank, 2 HBlank, 3 outro; tipo 0 leitura do 68000, 1 escrita do
//! 68000, 2 leitura do DMA da VDP.
//!
//! Regra de "compartilhada": a variável é tocada no laço principal e também dentro de uma
//! interrupção, e pelo menos um dos acessos é escrita. Leitura por DMA conta como acesso de
//! interrupção (a tabela de sprites, por exemplo, é escrita no laço e lida pelo DMA no VBlank).
//!
//! Uso: `cargo xtask inventario-memoria --acessos A.acc [--acessos B.acc ...]
//!       [--rotulos refs/rotulos.txt] [--saida docs/inventario-memoria.md]`
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const NOMES_CONTEXTO: [&str; 4] = ["principal", "vblank", "hblank", "outro"];
const LETRAS_TIPO: [char; 3] = ['L', 'E', 'D'];

/// Bits de escrita (tipo 1) nos quatro contextos.
const BITS_ESCRITA: u16 = 0b010_010_010_010;
/// Bits do laço principal.
const BITS_PRINCIPAL: u16 = 0b000_000_000_111;
/// Bits de qualquer interrupção.
const BITS_INTERRUPCAO: u16 = 0b111_111_111_000;

/// Lê um mapa de acessos. Linha fora do formato é erro: a ferramenta não aceita outra coisa.
pub fn ler_acessos(texto: &str) -> Result<BTreeMap<usize, u16>, String> {
    let mut m = BTreeMap::new();
    for (i, l) in texto.lines().enumerate() {
        let c: Vec<&str> = l.split('\t').collect();
        let [end, bits] = c[..] else {
            return Err(format!("linha {}: esperados 2 campos", i + 1));
        };
        if end.len() != 6 || bits.len() != 4 {
            return Err(format!("linha {}: tamanhos inesperados", i + 1));
        }
        let e = u32::from_str_radix(end, 16).map_err(|_| format!("linha {}: endereço", i + 1))?;
        let b = u16::from_str_radix(bits, 16).map_err(|_| format!("linha {}: bits", i + 1))?;
        if !(0xFF_0000..=0xFF_FFFF).contains(&e) || b >= 0x1000 {
            return Err(format!("linha {}: fora da RAM do 68000", i + 1));
        }
        let o = (e & 0xFFFF) as usize;
        *m.entry(o).or_insert(0) |= b;
    }
    Ok(m)
}

/// Variáveis de RAM do mapa de rótulos: `(offset, nome)`, ordenadas. O mapa guarda os endereços
/// com extensão de sinal (`FFFFxxxx`); `$FF0000..$FFFFFF` também vale.
pub fn variaveis_de_ram(rotulos: &str) -> Vec<(usize, String)> {
    let mut v = Vec::new();
    for l in rotulos.lines() {
        let c: Vec<&str> = l.split('\t').collect();
        let [nome, "variavel", valor] = c[..] else {
            continue;
        };
        let Ok(x) = u32::from_str_radix(valor, 16) else {
            continue;
        };
        if x >= 0xFFFF_0000 || (0x00FF_0000..=0x00FF_FFFF).contains(&x) {
            v.push(((x & 0xFFFF) as usize, nome.to_string()));
        }
    }
    // Mesmo endereço: prefere o nome sem ponto, depois o mais curto.
    v.sort_by_key(|(a, n)| (*a, n.contains('.'), n.len(), n.clone()));
    v
}

/// Índice da variável preferida que contém o offset.
fn indice_da_variavel(vars: &[(usize, String)], offset: usize) -> Option<usize> {
    let ultima = vars.partition_point(|(a, _)| *a <= offset).checked_sub(1)?;
    let endereco = vars.get(ultima)?.0;
    Some(vars.partition_point(|(a, _)| *a < endereco))
}

/// Acessos agregados de uma variável.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct PorVariavel {
    pub endereco: usize,
    pub bytes: usize,
    pub bits: u16,
    /// Maior distância entre um byte tocado e o início da variável.
    pub alcance: usize,
}

/// Agrupa os acessos por variável. A chave `None` é "sem variável" (antes da primeira).
pub fn agrupar(
    acessos: &BTreeMap<usize, u16>,
    vars: &[(usize, String)],
) -> BTreeMap<String, PorVariavel> {
    let mut g: BTreeMap<String, PorVariavel> = BTreeMap::new();
    for (&o, &b) in acessos {
        let (nome, inicio) = match indice_da_variavel(vars, o).and_then(|i| vars.get(i)) {
            Some((a, n)) => (n.clone(), *a),
            None => ("(sem variavel)".to_string(), 0),
        };
        let e = g.entry(nome).or_default();
        e.endereco = inicio;
        e.bytes += 1;
        e.bits |= b;
        e.alcance = e.alcance.max(o - inicio);
    }
    g
}

/// A variável é compartilhada pela regra do cabeçalho.
pub fn compartilhada(bits: u16) -> bool {
    bits & BITS_PRINCIPAL != 0 && bits & BITS_INTERRUPCAO != 0 && bits & BITS_ESCRITA != 0
}

/// Texto dos acessos de um contexto, como `LE`, `D` ou `-`.
fn texto_contexto(bits: u16, ctx: usize) -> String {
    let s: String = (0..3)
        .filter(|t| bits & (1 << (ctx * 3 + t)) != 0)
        .map(|t| LETRAS_TIPO[t])
        .collect();
    if s.is_empty() { "-".into() } else { s }
}

/// Gera o documento.
pub fn renderizar(g: &BTreeMap<String, PorVariavel>, fontes: &[String]) -> String {
    let mut md = String::new();
    md.push_str("# Inventário de memória compartilhada\n\n");
    md.push_str("Gerado por `cargo xtask inventario-memoria`. Só nomes, endereços, contextos e contagens.\n\n");
    md.push_str("Fonte: acessos à RAM do 68000 por contexto, medidos no oráculo nos casos: ");
    md.push_str(&fontes.join(", "));
    md.push_str(".\n\nLegenda: L leitura do 68000, E escrita do 68000, D leitura por DMA da VDP. ");
    md.push_str("Compartilhada: tocada no laço principal e numa interrupção, com pelo menos uma escrita.\n\n");
    md.push_str("Pilha e áreas sem rótulo próprio aparecem agrupadas na última variável anterior, com alcance grande.\n\n");
    let (comp, resto): (Vec<_>, Vec<_>) = g.iter().partition(|(_, v)| compartilhada(v.bits));
    for (titulo, itens) in [("Compartilhadas", &comp), ("Não compartilhadas", &resto)] {
        md.push_str(&format!("## {titulo} ({})\n\n", itens.len()));
        md.push_str("| variável | endereço | bytes tocados | alcance | principal | vblank | hblank | outro |\n");
        md.push_str("| --- | --- | --- | --- | --- | --- | --- | --- |\n");
        for (nome, v) in itens {
            md.push_str(&format!(
                "| {nome} | FF{:04X} | {} | {:#x} | {} | {} | {} | {} |\n",
                v.endereco,
                v.bytes,
                v.alcance,
                texto_contexto(v.bits, 0),
                texto_contexto(v.bits, 1),
                texto_contexto(v.bits, 2),
                texto_contexto(v.bits, 3),
            ));
        }
        md.push('\n');
    }
    md.push_str(&format!("Contextos: {}.\n", NOMES_CONTEXTO.join(", ")));
    md
}

pub fn executar(args: &[String]) -> Result<(), String> {
    let opcoes = |n: &str| -> Vec<&str> {
        args.windows(2)
            .filter(|w| w[0] == n)
            .map(|w| w[1].as_str())
            .collect()
    };
    let arquivos = opcoes("--acessos");
    if arquivos.is_empty() {
        return Err("falta --acessos".into());
    }
    let rotulos = opcoes("--rotulos")
        .first()
        .copied()
        .unwrap_or("refs/rotulos.txt");
    let saida = opcoes("--saida")
        .first()
        .copied()
        .unwrap_or("docs/inventarios/inventario-memoria.md");
    let mut todos: BTreeMap<usize, u16> = BTreeMap::new();
    let mut fontes = Vec::new();
    for a in &arquivos {
        let texto = fs::read_to_string(a).map_err(|e| format!("{a}: {e}"))?;
        for (o, b) in ler_acessos(&texto).map_err(|e| format!("{a}: {e}"))? {
            *todos.entry(o).or_insert(0) |= b;
        }
        let nome = Path::new(a)
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        fontes.push(nome.to_string());
    }
    let raiz = super::raiz();
    let mapa = fs::read_to_string(raiz.join(rotulos)).map_err(|e| format!("{rotulos}: {e}"))?;
    let vars = variaveis_de_ram(&mapa);
    if vars.is_empty() {
        return Err("o mapa de rótulos não tem variáveis de RAM (base 16?)".into());
    }
    let g = agrupar(&todos, &vars);
    let md = renderizar(&g, &fontes);
    super::inventario::filtro(&md)?;
    fs::write(raiz.join(saida), md).map_err(|e| e.to_string())?;
    let n = g.values().filter(|v| compartilhada(v.bits)).count();
    println!(
        "inventário de memória: {} bytes tocados, {} variáveis, {n} compartilhadas",
        todos.len(),
        g.len()
    );
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    const MAPA: &str = "v_a\tvariavel\tfffff000\nv_b\tvariavel\tfffff010\nv_b.campo\tvariavel\tfffff010\nr\trotina\t200\n";

    #[test]
    fn le_e_recusa() {
        let m = ler_acessos("FFF000\t0002\nFFF000\t0008\n").unwrap();
        assert_eq!(m[&0xF000], 0x000A);
        for ruim in [
            "FFF000",
            "FFF000\t2",
            "00F000\t0002",
            "FFF000\t1002",
            "FFF000\t0002\tx",
        ] {
            assert!(ler_acessos(ruim).is_err(), "{ruim}");
        }
    }

    #[test]
    fn agrupa_por_variavel_e_classifica() {
        let vars = variaveis_de_ram(MAPA);
        assert_eq!(vars.len(), 3);
        // v_a: escrita no principal (bit 1) e leitura no VBlank (bit 3): compartilhada.
        // v_b: so leitura no principal (bit 0) e escrita no principal (bit 1): nao.
        // v_b+2: leitura por DMA no VBlank (bit 5) e escrita no principal: compartilhada.
        let mut a = BTreeMap::new();
        a.insert(0xF000, 0b0010);
        a.insert(0xF001, 0b1000);
        a.insert(0xF010, 0b0011);
        a.insert(0xF012, 0b10_0010);
        let g = agrupar(&a, &vars);
        assert_eq!(g["v_a"].bytes, 2);
        assert!(compartilhada(g["v_a"].bits));
        assert_eq!(g["v_b"].bytes, 2);
        assert_eq!(g["v_b"].alcance, 2);
        assert!(compartilhada(g["v_b"].bits)); // o DMA do byte +2 entra no mesmo grupo
        let md = renderizar(&g, &["caso".to_string()]);
        assert!(md.contains("## Compartilhadas (2)"));
        assert!(md.contains("| v_a | FFF000 | 2 |"));
        assert!(!md.contains("v_b.campo"));
    }

    #[test]
    fn so_leitura_nao_e_compartilhada() {
        // Leitura no principal e no VBlank, sem escrita: nao e compartilhada.
        assert!(!compartilhada(0b1001));
        // So interrupcao: nao.
        assert!(!compartilhada(0b01_1000));
    }

    #[test]
    fn sem_variavel_antes_da_primeira() {
        let vars = variaveis_de_ram(MAPA);
        let mut a = BTreeMap::new();
        a.insert(0x0010, 0b1);
        let g = agrupar(&a, &vars);
        assert!(g.contains_key("(sem variavel)"));
    }
}
