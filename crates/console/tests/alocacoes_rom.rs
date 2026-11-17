//! Alocações por quadro na gameplay real: a média nos quadros 1000 a 1599 da partida-ghz1 não
//! passa de 2. O alocador conta só a thread do teste (o executor de testes usa outras threads).
use sonic1_console::Console;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

const CORPUS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus");

thread_local! {
    static CONTANDO: Cell<bool> = const { Cell::new(false) };
    static ALOCACOES: Cell<u64> = const { Cell::new(0) };
}

/// Alocador que delega ao do sistema e conta alocações e realocações da thread marcada.
struct Contador;

fn contar() {
    // `try_with` evita pânico se a thread já estiver encerrando.
    let _ = CONTANDO.try_with(|c| {
        if c.get() {
            let _ = ALOCACOES.try_with(|a| a.set(a.get().wrapping_add(1)));
        }
    });
}

// SAFETY: só delega ao `System` e incrementa um contador local da thread, sem alocar.
unsafe impl GlobalAlloc for Contador {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        contar();
        unsafe { System.alloc(l) }
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        contar();
        unsafe { System.alloc_zeroed(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        contar();
        unsafe { System.realloc(p, l, n) }
    }
}

#[global_allocator]
static ALOCADOR: Contador = Contador;

#[test]
#[ignore = "requer SONIC1_ROM"]
fn gameplay_quase_sem_alocacao_por_quadro() {
    let rom = std::fs::read(std::env::var_os("SONIC1_ROM").expect("SONIC1_ROM")).expect("ROM");
    let entradas: Vec<u8> = std::fs::read_to_string(format!("{CORPUS}/partida-ghz1.s1in"))
        .expect("entrada")
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .map(|s| u8::from_str_radix(s, 16).expect("hex"))
        .collect();
    let mut c = Console::com_taxa_audio(rom, 48_000.0);
    let quadro = |c: &mut Console, b: u8| {
        c.definir_ocupacao_audio(2400);
        c.definir_botoes(b);
        c.simular_quadro();
    };
    for &b in &entradas[..1000] {
        quadro(&mut c, b);
    }
    CONTANDO.with(|x| x.set(true));
    for &b in &entradas[1000..1600] {
        quadro(&mut c, b);
    }
    CONTANDO.with(|x| x.set(false));
    let media = ALOCACOES.with(Cell::get) as f64 / 600.0;
    println!("alocacoes por quadro (quadros 1000 a 1599): {media:.1}");
    assert!(media <= 2.0, "média de {media:.1} alocações por quadro (limite 2)");
}
