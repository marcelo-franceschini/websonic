//! Teste por rotina (sub-etapa 5.5): a mesma rotina no oráculo e no núcleo, a partir do mesmo
//! estado, comparando a RAM e os registradores que o chamador usa depois, e a sequência de
//! decisões.

use genesis_core::api::oracle::Regs68;
use sonic1_core::cobertura;
use sonic1_core::memoria::Memoria;
use sonic1_fidelidade::Tipo;
use sonic1_teste_nucleo::harness::{EstadoFinal, Metadado, Reg, RelatorioCaso, comparar};
use std::collections::BTreeMap;
use websonic_oracle::Oraculo;
use websonic_oracle::oraculo::ChamadaRotina;

/// O lado do núcleo de uma rotina: traduz os registradores de entrada em argumentos Rust, chama
/// a função do núcleo e devolve só os valores que correspondem a registradores do metadado.
pub trait Adaptador {
    fn executar(
        &self,
        mem: &mut Memoria,
        entrada: &BTreeMap<Reg, u32>,
    ) -> Result<BTreeMap<Reg, u32>, String>;
}

/// Registradores do oráculo como mapa de nomes.
pub fn mapa_de_regs(r: &Regs68) -> BTreeMap<Reg, u32> {
    let mut m = BTreeMap::new();
    for (i, v) in r.d.iter().enumerate() {
        m.insert(Reg::D(u8::try_from(i).unwrap_or(0)), *v);
    }
    for (i, v) in r.a.iter().enumerate() {
        m.insert(Reg::A(u8::try_from(i).unwrap_or(0)), *v);
    }
    m.insert(Reg::Ccr, u32::from(r.sr & 0x1F));
    m
}

/// Um desvio condicional executado: endereço do desvio e se foi tomado.
pub type Desvio = (u32, bool);

/// Resultado de um caso.
#[derive(Debug)]
pub struct ResultadoCaso {
    pub relatorio: RelatorioCaso,
    /// Desvios condicionais (Bcc e DBcc) do oráculo, na ordem.
    pub desvios_oraculo: Vec<Desvio>,
    /// Decisões do núcleo, na ordem.
    pub decisoes_nucleo: Vec<(&'static str, bool)>,
}

/// Configuração de um caso.
pub struct Caso<'a> {
    pub rom: &'a [u8],
    /// Estado de partida do oráculo (de `salvar_estado`).
    pub estado: &'a [u8],
    pub meta: &'a Metadado,
    pub adaptador: &'a dyn Adaptador,
    /// Registradores de entrada (a rotina é `pc`). A pilha é `ssp`.
    pub regs: Regs68,
    pub sentinela: u32,
    pub exclusoes: &'a [(usize, usize)],
}

/// Roda um caso nos dois lados.
pub fn executar_caso(o: &mut Oraculo, c: &Caso<'_>) -> Result<ResultadoCaso, String> {
    o.carregar_estado(c.estado)?;
    let ram_antes = o.ram68k();
    let entrada = mapa_de_regs(&c.regs);
    let r = o.chamar_rotina(&ChamadaRotina {
        regs: c.regs,
        pokes_ram68: Vec::new(),
        sentinela: c.sentinela,
        mascarar_interrupcoes: true,
        max_passos: 50_000_000,
    })?;
    let desvios_oraculo: Vec<Desvio> = r
        .eventos
        .iter()
        .filter(|e| e.tipo == Tipo::Desvio68 && matches!(e.c, 2 | 4))
        .map(|e| (e.a, e.d != 0))
        .collect();
    let ram_depois = o.ram68k();
    let oraculo = EstadoFinal {
        regs: mapa_de_regs(&r.regs),
        ram: ram_depois,
    };

    let mut mem = Memoria::nova(c.rom.to_vec());
    mem.ram_mut().copy_from_slice(&ram_antes);
    cobertura::zerar();
    cobertura::registrar_sequencia(true);
    let regs_nucleo = c.adaptador.executar(&mut mem, &entrada)?;
    let decisoes_nucleo = cobertura::sequencia();
    cobertura::registrar_sequencia(false);
    let nucleo = EstadoFinal {
        regs: regs_nucleo,
        ram: mem.ram().to_vec(),
    };

    Ok(ResultadoCaso {
        relatorio: comparar(c.meta, &entrada, &oraculo, &nucleo, c.exclusoes),
        desvios_oraculo,
        decisoes_nucleo,
    })
}

/// Sequência de resultados por chave, na ordem em que apareceram.
fn por_chave<K: Ord + Clone>(itens: impl Iterator<Item = (K, bool)>) -> BTreeMap<K, Vec<bool>> {
    let mut m: BTreeMap<K, Vec<bool>> = BTreeMap::new();
    for (k, v) in itens {
        m.entry(k).or_default().push(v);
    }
    m
}

/// Proposta de mapa: decisão do núcleo para endereço de desvio do original.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proposta {
    pub id: String,
    /// Endereços cujas sequências batem em todos os casos, com a polaridade.
    pub candidatos: Vec<(u32, bool)>,
}

/// Infere o mapa decisão para desvio: o desvio cuja sequência de resultados é igual à da
/// decisão em todos os casos. `inverso` quando o desvio é o complemento (o assembly desvia
/// quando a condição do Rust é falsa).
pub fn inferir_mapa(casos: &[ResultadoCaso]) -> Vec<Proposta> {
    let mut ids: Vec<&'static str> = casos
        .iter()
        .flat_map(|c| c.decisoes_nucleo.iter().map(|d| d.0))
        .collect();
    ids.sort_unstable();
    ids.dedup();
    let mut saida = Vec::new();
    for id in ids {
        // Candidatos = interseção, entre os casos, dos (endereço, polaridade) que batem.
        let mut comum: Option<Vec<(u32, bool)>> = None;
        for c in casos {
            let n = por_chave(c.decisoes_nucleo.iter().map(|d| (d.0, d.1)));
            let o = por_chave(c.desvios_oraculo.iter().map(|d| (d.0, d.1)));
            let Some(seq) = n.get(id) else { continue };
            let mut aqui = Vec::new();
            for (end, sq) in &o {
                if sq == seq {
                    aqui.push((*end, false));
                }
                if sq.len() == seq.len() && sq.iter().zip(seq).all(|(a, b)| a != b) {
                    aqui.push((*end, true));
                }
            }
            comum = Some(match comum {
                None => aqui,
                Some(prev) => prev.into_iter().filter(|p| aqui.contains(p)).collect(),
            });
        }
        saida.push(Proposta {
            id: id.to_string(),
            candidatos: comum.unwrap_or_default(),
        });
    }
    saida
}

/// Mapa declarado: `id<TAB>endereco_hex<TAB>direto|inverso`.
pub fn ler_mapa(texto: &str) -> Result<BTreeMap<String, (u32, bool)>, String> {
    let mut m = BTreeMap::new();
    for (i, l) in texto.lines().enumerate() {
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        let c: Vec<&str> = l.split('\t').collect();
        let [id, end, pol] = c[..] else {
            return Err(format!("mapa, linha {}: 3 campos", i + 1));
        };
        let end =
            u32::from_str_radix(end, 16).map_err(|_| format!("mapa, linha {}: endereco", i + 1))?;
        let inverso = match pol {
            "direto" => false,
            "inverso" => true,
            _ => return Err(format!("mapa, linha {}: polaridade", i + 1)),
        };
        m.insert(id.to_string(), (end, inverso));
    }
    Ok(m)
}

/// Confere o mapa declarado contra um caso: para cada decisão mapeada, a sequência do núcleo
/// (com a polaridade) tem de ser igual à do desvio do original. Devolve as decisões que falham.
pub fn verificar_mapa(mapa: &BTreeMap<String, (u32, bool)>, c: &ResultadoCaso) -> Vec<String> {
    let n = por_chave(c.decisoes_nucleo.iter().map(|d| (d.0, d.1)));
    let o = por_chave(c.desvios_oraculo.iter().map(|d| (d.0, d.1)));
    let mut ruins = Vec::new();
    for (id, seq) in &n {
        let Some((end, inverso)) = mapa.get(*id) else {
            continue;
        };
        let esperado: Vec<bool> = o.get(end).cloned().unwrap_or_default();
        let obtido: Vec<bool> = if *inverso {
            seq.iter().map(|b| !b).collect()
        } else {
            seq.clone()
        };
        if esperado != obtido {
            ruins.push((*id).to_string());
        }
    }
    ruins
}
