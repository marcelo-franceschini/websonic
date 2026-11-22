//! Fachada organizada por dominio para os modulos do jogo em `sonic1-core`.
//!
//! Agrupa os 89 arquivos de [`crate::abertura`] por responsabilidade (`sistema`, `telas`,
//! `sonic`, `chefes`, `badniks`, `zonas`), preservando `crate::abertura` intacto para
//! compatibilidade com os comparadores congelados.

pub use crate::abertura::{
    Abertura, ErroAbertura, Espera, EventoQuadro, MARCAS, Marca, MomentoSaida,
};

/// Infraestrutura central do jogo: RAM, VBlank, desenho, paleta, PLC e despacho de objetos.
pub mod sistema {
    pub use crate::abertura::{
        boot, cenario, desenho, hud, objetos, objpos, paleta, plc, ram, vblank,
    };
}

/// Maquinas de estado de telas e modos de jogo.
pub mod telas {
    pub use crate::abertura::{
        continue_screen, ending, fase, game_over, got_through, sega, special_result, special_stage,
        titulo,
    };
}

/// Fisica, agua, animacao e perda de aneis do Sonic.
pub mod sonic {
    pub use crate::abertura::{afogamento, animacao, fisica, ring_loss, sonic_agua, splash};
}

/// Chefes de todas as zonas (GHZ, MZ, LZ, SBZ, FZ; SYZ e SLZ ficam em `sistema::objetos`).
pub mod chefes {
    pub use crate::abertura::{
        boss_fz as fz, boss_ghz as ghz, boss_lz as lz, boss_mz as mz, boss_sbz as sbz,
    };
}

/// Inimigos (badniks) e mecanismos comuns.
pub mod badniks {
    pub use crate::abertura::{
        badniks as comuns, basaran, buzz, caterkiller, chopper, crabmeat, motobug,
    };
}

/// Objetos e mecanicas especificas de cada zona.
pub mod zonas {
    pub use crate::abertura::{
        barreira_invisivel, botao, capsula, collapse_ledge, edge_walls, ghz_extras, labirinto,
        labirinto_cenario, lamppost, marble_bloco, marble_fogo, marble_plataformas, marble_stomper,
        parede_quebravel, plataforma_basica, plataforma_oscilante, ponte, sbz_objetos, signpost,
        spikes, tijolo_marble, vidro_marble,
    };
}
