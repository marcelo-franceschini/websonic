//! Hospedeiro libretro do segundo oraculo (Genesis Plus GX), sem tela e sem som.
//!
//! O nucleo (`genesis_plus_gx_libretro.so`) fica fora do repositorio. Este programa roda a ROM
//! quadro a quadro e imprime, por quadro, o SHA-1 da RAM do 68000 (a unica memoria que o libretro
//! expoe nesse nucleo), nas duas ordens de byte. Nunca imprime bytes.
//!
//! ```text
//! oraculo2 rodar --nucleo ARQ.so --rom ROM --quadros N --saida ARQ.tsv
//! oraculo2 comparar --trace ARQ.s1t --tsv ARQ.tsv [--deslocamento -2..2]
//! oraculo2 diferencas --a RAM_PRINCIPAL --b RAM_GPGX --rotulos refs/rotulos.txt
//! ```
//! `rodar` aceita `--despejar N:ARQ`: grava a RAM do 68000 inteira depois do quadro N (so para o
//! host, nunca entra no git). `diferencas` compara dois despejos e imprime so enderecos, nomes de
//! variaveis e contagens. Nunca imprime valores.
//! Formato do `.tsv`: `quadro<TAB>sha1_normal<TAB>sha1_pares_trocados`.
//! Entradas de controle ainda NAO sao suportadas (so demos, sem botoes): o mapa entre os
//! botoes do libretro e os do Mega Drive precisa ser conferido antes.
#![allow(unsafe_code)]

use sonic1_fidelidade::hash::{em_hex, sha1};
use sonic1_fidelidade::{Categoria, Leitor};
use std::ffi::{CString, c_char, c_int, c_uint, c_void};
use std::fs;
use std::io::{BufReader, Write};
use std::process::ExitCode;

unsafe extern "C" {
    fn dlopen(nome: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(h: *mut c_void, nome: *const c_char) -> *mut c_void;
}
const RTLD_NOW: c_int = 2;

#[repr(C)]
struct InfoJogo {
    path: *const c_char,
    data: *const c_void,
    size: usize,
    meta: *const c_char,
}

type Ambiente = unsafe extern "C" fn(c_uint, *mut c_void) -> bool;
type Video = unsafe extern "C" fn(*const c_void, c_uint, c_uint, usize);
type AudioUm = unsafe extern "C" fn(i16, i16);
type AudioLote = unsafe extern "C" fn(*const i16, usize) -> usize;
type Poll = unsafe extern "C" fn();
type Entrada = unsafe extern "C" fn(c_uint, c_uint, c_uint, c_uint) -> i16;

const ENV_SET_PIXEL_FORMAT: c_uint = 10;
const ENV_SET_VARIABLES: c_uint = 16;
const MEMORIA_RAM_DO_SISTEMA: c_uint = 2;

unsafe extern "C" fn ambiente(cmd: c_uint, _dados: *mut c_void) -> bool {
    // Aceita o formato de pixel e a lista de variaveis; recusa o resto (o nucleo usa o padrao).
    matches!(cmd, ENV_SET_PIXEL_FORMAT | ENV_SET_VARIABLES)
}
unsafe extern "C" fn video(_: *const c_void, _: c_uint, _: c_uint, _: usize) {}
unsafe extern "C" fn audio_um(_: i16, _: i16) {}
unsafe extern "C" fn audio_lote(_: *const i16, quadros: usize) -> usize {
    quadros
}
unsafe extern "C" fn poll() {}
unsafe extern "C" fn entrada(_: c_uint, _: c_uint, _: c_uint, _: c_uint) -> i16 {
    0
}

fn simbolo<T: Copy>(h: *mut c_void, nome: &str) -> Result<T, String> {
    let c = CString::new(nome).map_err(|e| e.to_string())?;
    // SAFETY: `h` veio de dlopen e `nome` e uma string C valida.
    let p = unsafe { dlsym(h, c.as_ptr()) };
    if p.is_null() {
        return Err(format!("simbolo ausente: {nome}"));
    }
    // SAFETY: o tipo T e o ponteiro de funcao do simbolo conhecido do libretro.
    Ok(unsafe { std::mem::transmute_copy::<*mut c_void, T>(&p) })
}

fn opcao<'a>(args: &'a [String], nome: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == nome)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

fn trocar_pares(b: &[u8]) -> Vec<u8> {
    let mut v = b.to_vec();
    for p in v.as_chunks_mut::<2>().0 {
        p.swap(0, 1);
    }
    v
}

fn rodar(args: &[String]) -> Result<(), String> {
    if opcao(args, "--entradas").is_some() {
        return Err("entradas ainda nao suportadas (mapa de botoes nao conferido)".into());
    }
    let nucleo = CString::new(opcao(args, "--nucleo").ok_or("falta --nucleo")?)
        .map_err(|e| e.to_string())?;
    let rom_arq = opcao(args, "--rom").ok_or("falta --rom")?;
    let quadros: u32 = opcao(args, "--quadros")
        .ok_or("falta --quadros")?
        .parse()
        .map_err(|_| "--quadros invalido")?;
    let saida = opcao(args, "--saida").ok_or("falta --saida")?;
    // `--despejar N:ARQ`: grava a RAM crua (como o nucleo a guarda) depois do quadro N.
    let despejar: Option<(u32, &str)> = match opcao(args, "--despejar") {
        Some(v) => {
            let (n, arq) = v.split_once(':').ok_or("--despejar deve ser N:ARQ")?;
            Some((n.parse().map_err(|_| "--despejar: N invalido")?, arq))
        }
        None => None,
    };
    let rom = fs::read(rom_arq).map_err(|e| format!("ROM: {e}"))?;
    let caminho = CString::new(rom_arq).map_err(|e| e.to_string())?;

    // SAFETY: carrega um nucleo libretro escolhido pelo usuario; as chamadas seguem a API publica.
    unsafe {
        let h = dlopen(nucleo.as_ptr(), RTLD_NOW);
        if h.is_null() {
            return Err("dlopen falhou".into());
        }
        let set_env: unsafe extern "C" fn(Ambiente) = simbolo(h, "retro_set_environment")?;
        let set_video: unsafe extern "C" fn(Video) = simbolo(h, "retro_set_video_refresh")?;
        let set_audio: unsafe extern "C" fn(AudioUm) = simbolo(h, "retro_set_audio_sample")?;
        let set_lote: unsafe extern "C" fn(AudioLote) = simbolo(h, "retro_set_audio_sample_batch")?;
        let set_poll: unsafe extern "C" fn(Poll) = simbolo(h, "retro_set_input_poll")?;
        let set_ent: unsafe extern "C" fn(Entrada) = simbolo(h, "retro_set_input_state")?;
        let init: unsafe extern "C" fn() = simbolo(h, "retro_init")?;
        let carregar: unsafe extern "C" fn(*const InfoJogo) -> bool =
            simbolo(h, "retro_load_game")?;
        let executar: unsafe extern "C" fn() = simbolo(h, "retro_run")?;
        let mem_dados: unsafe extern "C" fn(c_uint) -> *mut c_void =
            simbolo(h, "retro_get_memory_data")?;
        let mem_tam: unsafe extern "C" fn(c_uint) -> usize = simbolo(h, "retro_get_memory_size")?;

        set_env(ambiente);
        init();
        set_video(video);
        set_audio(audio_um);
        set_lote(audio_lote);
        set_poll(poll);
        set_ent(entrada);
        let info = InfoJogo {
            path: caminho.as_ptr(),
            data: rom.as_ptr().cast(),
            size: rom.len(),
            meta: std::ptr::null(),
        };
        if !carregar(&info) {
            return Err("retro_load_game recusou a ROM".into());
        }
        let tam = mem_tam(MEMORIA_RAM_DO_SISTEMA);
        let ptr = mem_dados(MEMORIA_RAM_DO_SISTEMA).cast::<u8>();
        if ptr.is_null() || tam != 0x1_0000 {
            return Err(format!("RAM do sistema inesperada (tamanho {tam})"));
        }
        let mut arq = std::io::BufWriter::new(fs::File::create(saida).map_err(|e| e.to_string())?);
        for q in 0..quadros {
            executar();
            let ram = std::slice::from_raw_parts(ptr, tam);
            if let Some((n, arq)) = despejar
                && n == q
            {
                fs::write(arq, ram).map_err(|e| e.to_string())?;
            }
            writeln!(
                arq,
                "{q}\t{}\t{}",
                em_hex(&sha1(ram)),
                em_hex(&sha1(&trocar_pares(ram)))
            )
            .map_err(|e| e.to_string())?;
        }
        arq.flush().map_err(|e| e.to_string())?;
    }
    println!("quadros={quadros}");
    Ok(())
}

/// Hashes de RAM por quadro do trace do oraculo principal.
fn hashes_do_trace(caminho: &str) -> Result<Vec<String>, String> {
    let mut l = Leitor::abrir(BufReader::new(
        fs::File::open(caminho).map_err(|e| e.to_string())?,
    ))
    .map_err(|e| e.to_string())?;
    let mut v = Vec::new();
    while let Some(q) = l.proximo().map_err(|e| e.to_string())? {
        v.push(em_hex(&q.hashes[Categoria::Ram68k as usize]));
    }
    Ok(v)
}

/// Primeiro quadro (a partir de `n_min`) em que as duas listas diferem, com `desloc` aplicado:
/// o quadro `q` do trace e comparado com o quadro `q + desloc` do segundo oraculo.
pub fn primeira_diferenca(a: &[String], b: &[String], desloc: i64) -> (usize, Option<usize>) {
    let mut iguais = 0;
    for (q, ha) in a.iter().enumerate() {
        let Some(j) = i64::try_from(q)
            .ok()
            .and_then(|q| usize::try_from(q + desloc).ok())
        else {
            continue;
        };
        let Some(hb) = b.get(j) else { break };
        if ha != hb {
            return (iguais, Some(q));
        }
        iguais += 1;
    }
    (iguais, None)
}

fn comparar(args: &[String]) -> Result<(), String> {
    let a = hashes_do_trace(opcao(args, "--trace").ok_or("falta --trace")?)?;
    let tsv = fs::read_to_string(opcao(args, "--tsv").ok_or("falta --tsv")?)
        .map_err(|e| e.to_string())?;
    let (mut normal, mut trocado) = (Vec::new(), Vec::new());
    for l in tsv.lines() {
        let c: Vec<&str> = l.split('\t').collect();
        if let [_, n, t] = c[..] {
            normal.push(n.to_string());
            trocado.push(t.to_string());
        }
    }
    println!(
        "quadros do trace={} do segundo oraculo={}",
        a.len(),
        normal.len()
    );
    for (nome, b) in [("normal", &normal), ("pares_trocados", &trocado)] {
        for desloc in -3i64..=3 {
            let (iguais, dif) = primeira_diferenca(&a, b, desloc);
            match dif {
                None => println!("ordem={nome} deslocamento={desloc}: IGUAL em {iguais} quadros"),
                Some(q) => println!(
                    "ordem={nome} deslocamento={desloc}: {iguais} iguais, primeira diferenca no quadro {q}"
                ),
            }
        }
    }
    Ok(())
}

/// Variaveis de RAM do mapa de rotulos: `(offset na RAM, nome)`, ordenadas por offset.
/// So entram valores em `$FF0000..$FFFFFF` (o mapa guarda com extensao de sinal, `FFFFxxxx`).
fn variaveis_de_ram(texto: &str) -> Vec<(usize, String)> {
    let mut v: Vec<(usize, String)> = Vec::new();
    for l in texto.lines() {
        let c: Vec<&str> = l.split('\t').collect();
        let [nome, "variavel", valor] = c[..] else {
            continue;
        };
        let Ok(x) = u32::from_str_radix(valor, 16) else {
            continue;
        };
        if (x >= 0xFFFF_0000) || (0x00FF_0000..=0x00FF_FFFF).contains(&x) {
            v.push(((x & 0xFFFF) as usize, nome.to_string()));
        }
    }
    // Para o mesmo endereco, prefere o nome sem ponto (o campo e mais fino que a estrutura
    // so quando tem ponto, mas o nome simples e o mais legivel), depois o mais curto.
    v.sort_by_key(|(a, n)| (*a, n.contains('.'), n.len(), n.clone()));
    v
}

/// Variavel que contem o offset: a de maior endereco `<= offset`, e entre as de mesmo endereco
/// a preferida (a primeira na ordem de `variaveis_de_ram`).
fn variavel_do_offset(vars: &[(usize, String)], offset: usize) -> Option<&(usize, String)> {
    let ultima = vars.partition_point(|(a, _)| *a <= offset).checked_sub(1)?;
    let endereco = vars.get(ultima)?.0;
    vars.get(vars.partition_point(|(a, _)| *a < endereco))
}

/// Nome da variavel que contem o offset, com o deslocamento dentro dela.
fn nome_do_offset(vars: &[(usize, String)], offset: usize) -> String {
    match variavel_do_offset(vars, offset) {
        Some((a, n)) => format!("{n}+{:#x}", offset - a),
        None => "(antes da primeira variavel)".to_string(),
    }
}

/// Offsets dos bytes que diferem.
fn bytes_diferentes(a: &[u8], b: &[u8]) -> Vec<usize> {
    a.iter()
        .zip(b)
        .enumerate()
        .filter(|(_, (x, y))| x != y)
        .map(|(i, _)| i)
        .collect()
}

/// Junta offsets vizinhos (distancia ate `folga`) em faixas `(inicio, fim)` inclusivas.
fn faixas(offsets: &[usize], folga: usize) -> Vec<(usize, usize)> {
    let mut f: Vec<(usize, usize)> = Vec::new();
    for &o in offsets {
        match f.last_mut() {
            Some((_, fim)) if o <= *fim + folga => *fim = o,
            _ => f.push((o, o)),
        }
    }
    f
}

fn diferencas(args: &[String]) -> Result<(), String> {
    let a = fs::read(opcao(args, "--a").ok_or("falta --a")?).map_err(|e| e.to_string())?;
    let b = fs::read(opcao(args, "--b").ok_or("falta --b")?).map_err(|e| e.to_string())?;
    let rotulos = fs::read_to_string(opcao(args, "--rotulos").ok_or("falta --rotulos")?)
        .map_err(|e| e.to_string())?;
    if a.len() != 0x1_0000 || b.len() != 0x1_0000 {
        return Err("os despejos devem ter 65536 bytes".into());
    }
    let vars = variaveis_de_ram(&rotulos);
    // O nucleo guarda a RAM com os pares trocados; o certo e o que difere em menos bytes.
    let normal = bytes_diferentes(&a, &b);
    let trocado = bytes_diferentes(&a, &trocar_pares(&b));
    println!(
        "bytes diferentes: ordem normal={} pares trocados={}",
        normal.len(),
        trocado.len()
    );
    let (ordem, dif) = if trocado.len() <= normal.len() {
        ("pares_trocados", trocado)
    } else {
        ("normal", normal)
    };
    println!("ordem usada: {ordem}; bytes diferentes: {}", dif.len());
    let fx = faixas(&dif, 1);
    println!("faixas: {}", fx.len());
    for (i, f) in fx.iter().enumerate().take(200) {
        println!(
            "faixa {i}: FF{:04X}..FF{:04X} ({} bytes no maximo) inicio={}",
            f.0,
            f.1,
            f.1 - f.0 + 1,
            nome_do_offset(&vars, f.0)
        );
    }
    // Resumo por variavel: quantos bytes diferem em cada uma.
    let mut por_nome: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for &o in &dif {
        let n = variavel_do_offset(&vars, o).map_or("(sem variavel)", |(_, n)| n.as_str());
        *por_nome.entry(n.to_string()).or_default() += 1;
    }
    println!("por variavel (mais proxima antes do byte):");
    for (n, c) in &por_nome {
        println!("  {n}: {c}");
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let r = match args.first().map(String::as_str) {
        Some("rodar") => rodar(&args[1..]),
        Some("comparar") => comparar(&args[1..]),
        Some("diferencas") => diferencas(&args[1..]),
        _ => Err("uso: oraculo2 rodar|comparar|diferencas".into()),
    };
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("erro: {e}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn v(s: &[&str]) -> Vec<String> {
        s.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn diferenca_e_deslocamento() {
        let a = v(&["a", "b", "c", "d"]);
        assert_eq!(primeira_diferenca(&a, &a, 0), (4, None));
        assert_eq!(
            primeira_diferenca(&a, &v(&["a", "b", "x", "d"]), 0),
            (2, Some(2))
        );
        // b esta um quadro adiantado: b[q+1] == a[q]
        assert_eq!(
            primeira_diferenca(&a, &v(&["z", "a", "b", "c", "d"]), 1),
            (4, None)
        );
    }

    #[test]
    fn nome_e_faixas() {
        let mapa = "b\tvariavel\tfffff010\na\tvariavel\tfffff000\na.x\tvariavel\tfffff000\nr\trotina\t200\n";
        let v = variaveis_de_ram(mapa);
        assert_eq!(v.len(), 3);
        assert_eq!(nome_do_offset(&v, 0xF012), "b+0x2");
        assert_eq!(nome_do_offset(&v, 0xF000), "a+0x0");
        assert_eq!(nome_do_offset(&v, 0x10), "(antes da primeira variavel)");
        assert_eq!(faixas(&[1, 2, 3, 9, 11], 1), vec![(1, 3), (9, 9), (11, 11)]);
        assert_eq!(bytes_diferentes(&[1, 2, 3], &[1, 9, 9]), vec![1, 2]);
    }

    #[test]
    fn troca_de_pares() {
        assert_eq!(trocar_pares(&[1, 2, 3, 4]), vec![2, 1, 4, 3]);
    }
}
