//! Chips de referencia (Nuked-OPN2 no modo YM2612 e Nuked-PSG) ligados por FFI. GPLv3.
//!
//! So para comparar. O `unsafe` fica restrito a este arquivo, que nunca entra no jogo.

use std::ffi::c_void;

pub mod comparador;
pub use comparador::{ResultadoCaso, Veredito, comparar_fm, comparar_psg};

unsafe extern "C" {
    fn ref_psg_novo() -> *mut c_void;
    fn ref_psg_libera(c: *mut c_void);
    fn ref_psg_escreve(c: *mut c_void, d: u8);
    fn ref_psg_ciclo(c: *mut c_void);
    fn ref_psg_le(c: *mut c_void) -> u16;
    fn ref_opn2_novo() -> *mut c_void;
    fn ref_opn2_libera(c: *mut c_void);
    fn ref_opn2_escreve(c: *mut c_void, porta: u32, d: u8);
    fn ref_opn2_ciclo(c: *mut c_void, lr: *mut i16);
    fn ref_opn2_le(c: *mut c_void, porta: u32) -> u8;
}

/// PSG de referencia (Nuked-PSG).
pub struct RefPsg(*mut c_void);
impl RefPsg {
    pub fn novo() -> Self {
        // SAFETY: devolve um ponteiro valido, liberado no Drop.
        Self(unsafe { ref_psg_novo() })
    }
    pub fn escreve(&mut self, d: u8) {
        // SAFETY: ponteiro valido enquanto `self` vive.
        unsafe { ref_psg_escreve(self.0, d) }
    }
    pub fn ciclo(&mut self) {
        // SAFETY: idem.
        unsafe { ref_psg_ciclo(self.0) }
    }
    pub fn le(&mut self) -> u16 {
        // SAFETY: idem.
        unsafe { ref_psg_le(self.0) }
    }
    /// 16 ciclos e uma leitura.
    pub fn amostra(&mut self) -> u16 {
        for _ in 0..16 {
            self.ciclo();
        }
        self.le()
    }
}
impl Drop for RefPsg {
    fn drop(&mut self) {
        // SAFETY: ponteiro vindo de `ref_psg_novo`, liberado uma vez.
        unsafe { ref_psg_libera(self.0) }
    }
}

/// FM de referencia (Nuked-OPN2, modo YM2612).
pub struct RefOpn2(*mut c_void);
impl RefOpn2 {
    pub fn novo() -> Self {
        // SAFETY: devolve um ponteiro valido, liberado no Drop.
        Self(unsafe { ref_opn2_novo() })
    }
    pub fn escreve(&mut self, porta: u32, d: u8) {
        // SAFETY: ponteiro valido enquanto `self` vive.
        unsafe { ref_opn2_escreve(self.0, porta, d) }
    }
    /// Um ciclo interno (6 do mestre/7... ver sintese.md). Devolve (esquerda, direita).
    pub fn ciclo(&mut self) -> (i16, i16) {
        let mut lr = [0i16; 2];
        // SAFETY: `lr` tem 2 elementos.
        unsafe { ref_opn2_ciclo(self.0, lr.as_mut_ptr()) };
        (lr[0], lr[1])
    }
    pub fn le(&mut self, porta: u32) -> u8 {
        // SAFETY: ponteiro valido.
        unsafe { ref_opn2_le(self.0, porta) }
    }
}
impl Drop for RefOpn2 {
    fn drop(&mut self) {
        // SAFETY: ponteiro vindo de `ref_opn2_novo`, liberado uma vez.
        unsafe { ref_opn2_libera(self.0) }
    }
}

impl RefOpn2 {
    /// Estado interno por camada (diagnostico): 0 fase, 1 nivel do envelope, 2 saida do envelope,
    /// 3 saida do operador, 4 saida do canal, 5 incremento de fase, 6 estado do envelope,
    /// 7 modulacao, 8 diversos (ciclo, canal, mol, mor, timer do envelope, contador do LFO...).
    pub fn estado(&self, camada: i32) -> Vec<i32> {
        let mut v = [0i32; 24];
        // SAFETY: `v` tem 24 posicoes, o maximo que a funcao escreve.
        let n = unsafe { ref_opn2_estado(self.0, camada, v.as_mut_ptr()) };
        v[..usize::try_from(n).unwrap()].to_vec()
    }
}

unsafe extern "C" {
    fn ref_opn2_estado(c: *mut c_void, camada: i32, out: *mut i32) -> i32;
}
