//! Linha de comando do diferencial (roda no host, com a ROM). Só imprime nomes, vereditos,
//! contagens e endereços. Nunca imprime bytes de memória.
//!
//! ```text
//! diferencial rotina KosDec|EniDec|NemDecToRAM --rom ROM --rotulos refs/rotulos.txt
//!     --manifesto descompressao/manifesto.txt --metadados crates/teste-nucleo/rotinas
//!     [--estado ARQ.s1e] [--mapa cobertura/mapa-decisoes.txt] [--proposta-de-mapa ARQ]
//!     [--cobertura-nucleo ARQ] [--executados-saida ARQ]
//! ```

use genesis_core::api::oracle::Regs68;
use sonic1_core::cobertura;
use sonic1_core::memoria::Memoria;
use sonic1_core::rotinas;
use sonic1_diferencial::rotina::{
    Adaptador, Caso, ResultadoCaso, executar_caso, inferir_mapa, ler_mapa, verificar_mapa,
};
use sonic1_fidelidade::rotulos::Rotulos;
use sonic1_teste_nucleo::harness::{Metadado, Reg};
use std::collections::BTreeMap;
use std::process::ExitCode;
use websonic_oracle::{Entradas, Oraculo};

/// Onde a saída da descompressão é escrita (início da RAM).
const DESTINO: u32 = 0xFF_0000;
/// Endereço de retorno falso, dentro da ROM, nunca executado.
const SENTINELA: u32 = 0x1_FFF0;
/// Pilha de supervisor das chamadas.
const PILHA: u32 = 0xFF_FE00;
/// Área abaixo da pilha de entrada que a chamada usa (proposta de exclusão, aguarda aprovação).
const AREA_DA_PILHA: usize = 0x200;

fn opcao<'a>(args: &'a [String], nome: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == nome)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Rotina {
    KosDec,
    EniDec,
    NemDecToRam,
}

impl Rotina {
    fn de_nome(n: &str) -> Option<Self> {
        match n {
            "KosDec" => Some(Self::KosDec),
            "EniDec" => Some(Self::EniDec),
            "NemDecToRAM" => Some(Self::NemDecToRam),
            _ => None,
        }
    }
    fn formato(self) -> &'static str {
        match self {
            Self::KosDec => "kosinski",
            Self::EniDec => "enigma",
            Self::NemDecToRam => "nemesis",
        }
    }
}

struct Piloto(Rotina);

impl Adaptador for Piloto {
    fn executar(
        &self,
        mem: &mut Memoria,
        e: &BTreeMap<Reg, u32>,
    ) -> Result<BTreeMap<Reg, u32>, String> {
        let (origem, destino) = (e[&Reg::A(0)], e[&Reg::A(1)]);
        let fim = match self.0 {
            Rotina::KosDec => rotinas::kos_dec(mem, origem, destino),
            Rotina::EniDec => rotinas::eni_dec(
                mem,
                origem,
                destino,
                u16::try_from(e[&Reg::D(0)] & 0xFFFF).unwrap_or(0),
            ),
            Rotina::NemDecToRam => rotinas::nem_dec_to_ram(mem, origem, destino),
        }
        .map_err(|e| format!("{e:?}"))?;
        // A1 final: informativo (as rotinas do original restauram ou usam A1 de jeitos diferentes).
        Ok(BTreeMap::from([(Reg::A(1), fim)]))
    }
}

struct Item {
    rotulo: String,
    formato: String,
    tile: u16,
}

fn ler_manifesto(texto: &str) -> Result<Vec<Item>, String> {
    let mut v = Vec::new();
    for (i, l) in texto.lines().enumerate() {
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        let c: Vec<&str> = l.split('\t').collect();
        let [rotulo, formato, tiles] = c[..] else {
            return Err(format!("manifesto, linha {}: 3 campos", i + 1));
        };
        if formato == "enigma" {
            for t in tiles.split(',') {
                let tile = u16::from_str_radix(t, 16)
                    .map_err(|_| format!("manifesto, linha {}: tile", i + 1))?;
                v.push(Item {
                    rotulo: rotulo.into(),
                    formato: formato.into(),
                    tile,
                });
            }
        } else {
            v.push(Item {
                rotulo: rotulo.into(),
                formato: formato.into(),
                tile: 0,
            });
        }
    }
    Ok(v)
}

fn rotina(args: &[String]) -> Result<ExitCode, String> {
    let nome = args.first().ok_or("falta o nome da rotina")?;
    let r = Rotina::de_nome(nome).ok_or("rotina desconhecida")?;
    let rom_arq = opcao(args, "--rom").ok_or("falta --rom")?;
    let rom = std::fs::read(rom_arq).map_err(|e| format!("ROM: {e}"))?;
    let rotulos = Rotulos::analisar(
        &std::fs::read_to_string(opcao(args, "--rotulos").ok_or("falta --rotulos")?)
            .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let itens = ler_manifesto(
        &std::fs::read_to_string(opcao(args, "--manifesto").ok_or("falta --manifesto")?)
            .map_err(|e| e.to_string())?,
    )?;
    let pasta = opcao(args, "--metadados").ok_or("falta --metadados")?;
    let meta = Metadado::analisar(
        &std::fs::read_to_string(format!("{pasta}/{nome}.txt")).map_err(|e| e.to_string())?,
    )?;
    let endereco_rotina = rotulos
        .resolver(nome)
        .ok_or("rotina ausente do mapa de rotulos")?;
    let mapa = match opcao(args, "--mapa") {
        Some(a) => Some(ler_mapa(
            &std::fs::read_to_string(a).map_err(|e| e.to_string())?,
        )?),
        None => None,
    };

    let mut o = Oraculo::novo(rom.clone(), Entradas::default())?;
    if let Some(a) = opcao(args, "--estado") {
        o.carregar_estado(&std::fs::read(a).map_err(|e| format!("estado: {e}"))?)?;
    }
    let estado = o.salvar_estado()?;
    let base = o.emulador().oracle_regs68();
    let adaptador = Piloto(r);
    let exclusoes = [(
        usize::try_from(PILHA)
            .unwrap_or(0)
            .wrapping_sub(0xFF_0000)
            .wrapping_sub(AREA_DA_PILHA),
        usize::try_from(PILHA).unwrap_or(0).wrapping_sub(0xFF_0000),
    )];

    let (mut casos, mut falhas, mut ausentes) = (Vec::<ResultadoCaso>::new(), 0usize, 0usize);
    let mut nao_iguais_a1 = 0usize;
    for item in itens.iter().filter(|i| i.formato == r.formato()) {
        let Some(origem) = rotulos.resolver(&item.rotulo) else {
            ausentes += 1;
            println!("{nome} {} tile={:04X} AUSENTE", item.rotulo, item.tile);
            continue;
        };
        let mut regs: Regs68 = base;
        regs.d = [0; 8];
        regs.a = [0; 7];
        regs.d[0] = u32::from(item.tile);
        regs.a[0] = origem;
        regs.a[1] = DESTINO;
        regs.a[4] = DESTINO;
        regs.pc = endereco_rotina;
        regs.sr = 0x2700;
        regs.ssp = PILHA;
        let caso = Caso {
            rom: &rom,
            estado: &estado,
            meta: &meta,
            adaptador: &adaptador,
            regs,
            sentinela: SENTINELA,
            exclusoes: &exclusoes,
        };
        let c = match executar_caso(&mut o, &caso) {
            Ok(c) => c,
            Err(e) => {
                falhas += 1;
                println!("{nome} {} tile={:04X} ERRO {e}", item.rotulo, item.tile);
                continue;
            }
        };
        let ok = c.relatorio.passou();
        if !ok {
            falhas += 1;
        }
        let mexidos: Vec<String> = c
            .relatorio
            .ignorados_mexidos
            .iter()
            .map(|r| r.nome())
            .collect();
        println!(
            "{nome} {} tile={:04X} {} ram_difere={} excluidos_difere={} ignorados_mexidos=[{}]",
            item.rotulo,
            item.tile,
            if ok { "IGUAL" } else { "DIFERE" },
            c.relatorio.ram_diferente.len(),
            c.relatorio.excluidos_diferentes,
            mexidos.join(",")
        );
        if let Some(m) = &mapa {
            let ruins = verificar_mapa(m, &c);
            if !ruins.is_empty() {
                falhas += 1;
                println!("  decisao fora do mapa: {}", ruins.join(" "));
            }
        }
        nao_iguais_a1 += usize::from(
            c.relatorio
                .usados_diferentes
                .iter()
                .any(|d| d.0 == Reg::A(1)),
        );
        casos.push(c);
    }
    println!(
        "{nome}: {} casos, {} falhas, {} ausentes",
        casos.len(),
        falhas,
        ausentes
    );

    let proposta = inferir_mapa(&casos);
    let mut texto = String::new();
    for p in &proposta {
        let c: Vec<String> = p
            .candidatos
            .iter()
            .map(|(a, inv)| format!("{a:X}:{}", if *inv { "inverso" } else { "direto" }))
            .collect();
        println!("decisao {} candidatos=[{}]", p.id, c.join(" "));
        if let [(a, inv)] = p.candidatos[..] {
            texto.push_str(&format!(
                "{}\t{a:X}\t{}\n",
                p.id,
                if inv { "inverso" } else { "direto" }
            ));
        }
    }
    if let Some(a) = opcao(args, "--proposta-de-mapa") {
        std::fs::write(a, texto).map_err(|e| e.to_string())?;
    }
    if let Some(a) = opcao(args, "--cobertura-nucleo") {
        let linhas: String = cobertura::contagens()
            .iter()
            .map(|(k, (v, f))| format!("{k}\t{v}\t{f}\n"))
            .collect();
        std::fs::write(a, linhas).map_err(|e| e.to_string())?;
    }
    if let Some(a) = opcao(args, "--executados-saida") {
        // Desvios condicionais do original executados nos casos: endereco, tomado, nao tomado.
        let mut m: BTreeMap<u32, (u64, u64)> = BTreeMap::new();
        for c in &casos {
            for (end, tomado) in &c.desvios_oraculo {
                let x = m.entry(*end).or_insert((0, 0));
                if *tomado {
                    x.0 += 1;
                } else {
                    x.1 += 1;
                }
            }
        }
        let linhas: String = m
            .iter()
            .map(|(a, (t, n))| format!("{a:X}\t{t}\t{n}\n"))
            .collect();
        std::fs::write(a, linhas).map_err(|e| e.to_string())?;
    }
    println!("a1 do nucleo diferente do a1 do original (informativo): {nao_iguais_a1} casos");
    Ok(if falhas == 0 && ausentes < itens.len() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let r = match args.first().map(String::as_str) {
        Some("rotina") => rotina(&args[1..]),
        _ => Err("uso: diferencial rotina NOME --rom ROM --rotulos ARQ --manifesto ARQ --metadados DIR [...]".into()),
    };
    match r {
        Ok(c) => c,
        Err(e) => {
            eprintln!("erro: {e}");
            ExitCode::from(2)
        }
    }
}
