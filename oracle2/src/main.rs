//! Hospedeiro libretro do segundo oraculo (Genesis Plus GX), sem tela e sem som.
//!
//! O nucleo (`genesis_plus_gx_libretro.so`) fica fora do repositorio. Este programa roda a ROM
//! quadro a quadro e imprime, por quadro, o SHA-1 da RAM do 68000 (a unica memoria que o libretro
//! expoe nesse nucleo), nas duas ordens de byte. Nunca imprime bytes.
//!
//! ```text
//! oraculo2 rodar --nucleo ARQ.so --rom ROM --quadros N --saida ARQ.tsv
//! oraculo2 comparar --trace ARQ.s1t --tsv ARQ.tsv [--deslocamento -2..2]
//! ```
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

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let r = match args.first().map(String::as_str) {
        Some("rodar") => rodar(&args[1..]),
        Some("comparar") => comparar(&args[1..]),
        _ => Err("uso: oraculo2 rodar|comparar".into()),
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
    fn troca_de_pares() {
        assert_eq!(trocar_pares(&[1, 2, 3, 4]), vec![2, 1, 4, 3]);
    }
}
