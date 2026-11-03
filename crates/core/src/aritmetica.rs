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
}
