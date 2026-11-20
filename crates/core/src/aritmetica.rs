//! Aritmética inteira com estouro explícito.
//!
//! O 68000 faz a volta (wraparound) em somas e subtrações. O núcleo precisa dizer
//! isso no código: cada operação aqui escolhe o comportamento de propósito.

/// Soma de 16 bits com volta, como um registrador de 16 bits.
pub const fn somar_u16(a: u16, b: u16) -> u16 {
    a.wrapping_add(b)
}

/// Subtração de 16 bits com volta.
pub const fn subtrair_u16(a: u16, b: u16) -> u16 {
    a.wrapping_sub(b)
}

/// Soma de 32 bits com volta.
pub const fn somar_u32(a: u32, b: u32) -> u32 {
    a.wrapping_add(b)
}

/// Soma de 16 bits que também devolve o "vai um" (carry).
pub const fn somar_com_carry_u16(a: u16, b: u16) -> (u16, bool) {
    a.overflowing_add(b)
}

/// Reinterpreta os 16 bits de um `u16` como `i16` em complemento de dois.
pub const fn u16_para_i16(v: u16) -> i16 {
    i16::from_ne_bytes(v.to_ne_bytes())
}

/// Reinterpreta os 16 bits de um `i16` como `u16`.
pub const fn i16_para_u16(v: i16) -> u16 {
    u16::from_ne_bytes(v.to_ne_bytes())
}

/// Reinterpreta os 32 bits de um `u32` como `i32` em complemento de dois.
pub const fn u32_para_i32(v: u32) -> i32 {
    i32::from_ne_bytes(v.to_ne_bytes())
}

/// Reinterpreta os 32 bits de um `i32` como `u32`.
pub const fn i32_para_u32(v: i32) -> u32 {
    u32::from_ne_bytes(v.to_ne_bytes())
}

/// Estende o sinal de uma palavra de 16 bits para 32 bits (como `ext.l`, `movea.w`, `adda.w`).
pub const fn estender_sinal_u16(v: u16) -> u32 {
    i32_para_u32(u16_para_i16(v) as i32)
}

/// Extrai os 16 bits baixos de uma longa de 32 bits.
pub const fn palavra_baixa_u32(v: u32) -> u16 {
    let b = v.to_be_bytes();
    u16::from_be_bytes([b[2], b[3]])
}

/// Extrai os 8 bits baixos de uma longa de 32 bits.
pub const fn byte_baixo_u32(v: u32) -> u8 {
    v.to_be_bytes()[3]
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn soma_sem_estouro() {
        assert_eq!(somar_u16(1, 2), 3);
    }

    #[test]
    fn soma_volta_no_limite() {
        assert_eq!(somar_u16(u16::MAX, 1), 0);
        assert_eq!(somar_u32(u32::MAX, 2), 1);
    }

    #[test]
    fn subtracao_volta_abaixo_de_zero() {
        assert_eq!(subtrair_u16(0, 1), u16::MAX);
    }

    #[test]
    fn carry_so_quando_estoura() {
        assert_eq!(somar_com_carry_u16(10, 20), (30, false));
        assert_eq!(somar_com_carry_u16(u16::MAX, 1), (0, true));
    }

    #[test]
    fn conversoes_de_bits_e_extensao_de_sinal() {
        assert_eq!(u16_para_i16(0xFFFF), -1);
        assert_eq!(i16_para_u16(-1), 0xFFFF);
        assert_eq!(u32_para_i32(0xFFFF_FFFE), -2);
        assert_eq!(i32_para_u32(-2), 0xFFFF_FFFE);
        assert_eq!(estender_sinal_u16(0x8000), 0xFFFF_8000);
        assert_eq!(estender_sinal_u16(0x7FFF), 0x0000_7FFF);
        assert_eq!(palavra_baixa_u32(0x1234_5678), 0x5678);
        assert_eq!(byte_baixo_u32(0x1234_5678), 0x78);
    }
}
