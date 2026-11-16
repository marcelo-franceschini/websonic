//! Motor de objetos mínimo (sub-etapa 7.5): `ExecuteObjects`, `DisplaySprite`, `BuildSprites`
//! e o objeto `CreditsText`. Os objetos do original são código na ROM (tabela `Obj_Index`);
//! aqui cada identificador chama uma função nativa, citando o rótulo que reproduz.

use super::boot::{rom_b, rom_l, rom_w};
use super::ram::*;
use super::{ErroAbertura, plc, rb, rl, rw, wb, wl, ww};
use crate::memoria::Memoria;

#[path = "bloco_labirinto.rs"]
mod bloco_labirinto;
#[path = "bolhas_labirinto.rs"]
mod bolhas_labirinto;
#[path = "boss_syz.rs"]
mod boss_syz;
#[path = "burrobot.rs"]
mod burrobot;
#[path = "corrente_espinhos.rs"]
mod corrente_espinhos;
#[path = "jaws.rs"]
mod jaws;
#[path = "jangada_labirinto.rs"]
mod jangada_labirinto;
#[path = "portas_labirinto.rs"]
mod portas_labirinto;

const SPRITES_MAX: u8 = 80;
const SPRITELAYER_NUM: u32 = 8;
const SPRITELAYER_SIZE: u32 = 0x80;
const SCEN_VALUES: u32 = 0x8EAC;

/// Estende o sinal de uma palavra, como `movea.w` e `adda.w` fazem.
fn ext(a: u16) -> u32 {
    u32::from_be_bytes(i32::from(i16::from_be_bytes(a.to_be_bytes())).to_be_bytes())
}

/// `DeleteObject`: zera as 16 longas do objeto.
pub fn delete_object(m: &mut Memoria, a0: u32) {
    for i in 0..16u32 {
        wl(m, a0.wrapping_add(i.wrapping_mul(4)), 0);
    }
}

/// `DisplaySprite`: põe o objeto na fila de sprites da sua prioridade.
pub fn display_sprite(m: &mut Memoria, a0: u32) {
    let prio = rw(m, a0.wrapping_add(OB_PRIORITY));
    let d0 = u32::from(prio >> 1) & (0x80 * (SPRITELAYER_NUM - 1));
    let camada = V_SPRITEQUEUE.wrapping_add(d0);
    let n = rw(m, camada);
    if u32::from(n) >= SPRITELAYER_SIZE - 2 {
        return;
    }
    let n = n.wrapping_add(2);
    ww(m, camada, n);
    ww(
        m,
        camada.wrapping_add(u32::from(n)),
        u16::try_from(a0 & 0xFFFF).unwrap_or(0),
    );
}

/// `ExecuteObjects`: roda cada objeto da memória de objetos.
pub fn execute_objects(m: &mut Memoria) -> Result<(), ErroAbertura> {
    if rb(m, V_PLAYER.wrapping_add(OB_ID)) == ID_SONIC
        && rb(m, V_PLAYER.wrapping_add(OB_ROUTINE)) >= 6
    {
        // Roda os primeiros 32 objetos (reservados, incluindo o Sonic) normalmente
        let mut a = V_OBJSPACE;
        while a < V_LVLOBJSPACE {
            let id = rb(m, a.wrapping_add(OB_ID));
            if id != 0 {
                rodar_objeto(m, a, id)?;
            }
            a = a.wrapping_add(OBJECT_SIZE);
        }
        // Os demais 96 objetos de fase ficam congelados, apenas sendo exibidos se visíveis na tela
        while a < V_LVLOBJEND {
            let id = rb(m, a.wrapping_add(OB_ID));
            if id != 0 && rb(m, a.wrapping_add(OB_RENDER)) & 0x80 != 0 {
                display_sprite(m, a);
            }
            a = a.wrapping_add(OBJECT_SIZE);
        }
        return Ok(());
    }
    let mut a0 = V_OBJSPACE;
    while a0 < V_OBJSPACE_END {
        let id = rb(m, a0.wrapping_add(OB_ID));
        if id != 0 {
            rodar_objeto(m, a0, id)?;
        }
        a0 = a0.wrapping_add(OBJECT_SIZE);
    }
    Ok(())
}

fn rodar_objeto(m: &mut Memoria, a0: u32, id: u8) -> Result<(), ErroAbertura> {
    match id {
        ID_CREDITS_TEXT => {
            credits_text(m, a0);
            Ok(())
        }
        ID_TITLE_SONIC => titulo_sonic(m, a0),
        ID_PSBTM => psbtm(m, a0),
        ID_TITLE_CARD => cartao_de_titulo(m, a0),
        ID_SONIC => super::fisica::sonic_player(m, a0),
        ID_HUD => hud_objeto(m, a0),
        ID_RINGS => aneis(m, a0),
        ID_MONITOR => monitor(m, a0),
        ID_SPRING => mola(m, a0),
        ID_SCENERY => cenario(m, a0),
        ID_EXPLOSION_ITEM => explosao_item(m, a0),
        ID_ANIMALS => animal(m, a0),
        ID_POINTS => pontos_objeto(m, a0),
        ID_CHOPPER => super::chopper::executar(m, a0),
        ID_WATER_SOUND => water_sound(m, a0),
        ID_BRIDGE => super::ponte::executar(m, a0),
        ID_CRABMEAT => super::crabmeat::executar(m, a0),
        ID_PURPLE_ROCK => purple_rock(m, a0),
        ID_EDGE_WALLS => super::edge_walls::executar(m, a0),
        ID_SPIKES => super::spikes::executar(m, a0),
        ID_COLLAPSE_LEDGE => super::collapse_ledge::executar(m, a0),
        ID_SIGNPOST => super::signpost::executar(m, a0),
        0x79 => super::lamppost::executar(m, a0),
        0x08 => super::splash::executar(m, a0),
        0x0A => super::afogamento::executar(m, a0),
        0x3E => super::capsula::executar(m, a0),
        0x3D => super::boss_ghz::executar(m, a0),
        0x48 => super::boss_ghz::bola(m, a0),
        0x75 => boss_syz::chefe(m, a0),
        0x76 => boss_syz::bloco(m, a0),
        0x3C => super::parede_quebravel::executar(m, a0),
        0x15 => super::plataforma_oscilante::executar(m, a0),
        0x18 => super::plataforma_basica::executar(m, a0),
        0x17 => super::ghz_extras::helix(m, a0),
        0x7D => super::ghz_extras::bonus(m, a0),
        0x4B => super::ghz_extras::anel_gigante(m, a0),
        0x7C => super::ghz_extras::brilho_anel(m, a0),
        0x2F => super::marble_plataformas::plataforma(m, a0),
        0x35 => super::marble_plataformas::fogo(m, a0),
        0x54 => super::marble_plataformas::lava(m, a0),
        0x42 => super::badniks::newtron(m, a0),
        0x1E => super::badniks::ball_hog(m, a0),
        0x20 => super::badniks::cannonball(m, a0),
        0x3F => super::badniks::explosao(m, a0),
        0x39 => super::game_over::executar(m, a0),
        0x80 => super::continue_screen::item(m, a0),
        0x81 => super::continue_screen::sonic(m, a0),
        0x78 => super::caterkiller::executar(m, a0),
        0x13 => super::marble_fogo::gerador(m, a0),
        0x14 => super::marble_fogo::bola(m, a0),
        0x31 => super::marble_stomper::executar(m, a0),
        0x1B => super::labirinto::superficie(m, a0),
        0x61 => bloco_labirinto::executar(m, a0),
        0x2D => burrobot::executar(m, a0),
        0x57 => corrente_espinhos::executar(m, a0),
        0x56 => portas_labirinto::executar(m, a0),
        0x64 => bolhas_labirinto::executar(m, a0),
        0x2C => jaws::executar(m, a0),
        0x52 => jangada_labirinto::executar(m, a0),
        0x33 => super::marble_bloco::executar(m, a0),
        0x32 => super::botao::executar(m, a0),
        0x55 => super::basaran::executar(m, a0),
        0x71 => super::barreira_invisivel::executar(m, a0),
        0x30 => super::vidro_marble::executar(m, a0),
        0x46 => super::tijolo_marble::executar(m, a0),
        ID_GOT_THROUGH_CARD => super::got_through::executar(m, a0),
        0x40 => super::motobug::executar(m, a0),
        0x22 => super::buzz::executar(m, a0),
        0x23 => super::buzz::missil(m, a0),
        outro => Err(ErroAbertura::ObjetoNaoImplementado(outro)),
    }
}

/// Os IDs que `rodar_objeto` conhece.
pub fn objeto_existe(id: u8) -> bool {
    matches!(
        id,
        ID_CREDITS_TEXT
            | ID_TITLE_SONIC
            | ID_PSBTM
            | ID_TITLE_CARD
            | ID_SONIC
            | ID_HUD
            | ID_RINGS
            | ID_MONITOR
            | ID_SPRING
            | ID_SCENERY
            | ID_EXPLOSION_ITEM
            | ID_ANIMALS
            | ID_POINTS
            | ID_CHOPPER
            | ID_WATER_SOUND
            | ID_BRIDGE
            | ID_CRABMEAT
            | ID_PURPLE_ROCK
            | ID_EDGE_WALLS
            | ID_SPIKES
            | ID_COLLAPSE_LEDGE
            | ID_SIGNPOST
            | 0x79
            | 0x08
            | 0x0A
            | 0x15
            | 0x18
            | 0x17
            | 0x7D
            | 0x4B
            | 0x7C
            | 0x2F
            | 0x35
            | 0x54
            | 0x3C
            | 0x3D
            | 0x48
            | 0x3E
            | 0x42
            | 0x1E
            | 0x20
            | 0x3F
            | 0x39
            | 0x80
            | 0x81
            | 0x78
            | 0x13
            | 0x14
            | 0x31
            | 0x1B
            | 0x61
            | 0x2D
            | 0x57
            | 0x56
            | 0x64
            | 0x2C
            | 0x52
            | 0x76
            | 0x75
            | 0x33
            | 0x32
            | 0x55
            | 0x71
            | 0x30
            | 0x46
            | ID_GOT_THROUGH_CARD
            | 0x40
            | 0x22
            | 0x23
    )
}

/// `SolidObject`: rotina padrão de colisão sólida para molas, pedras, blocos, etc.
pub fn solid_object(
    m: &mut Memoria,
    objeto: u32,
    largura: u16,
    altura: u16,
    altura_em_cima: u16,
) -> Result<i8, ErroAbertura> {
    let jogador = V_PLAYER;
    let x_obj = rw(m, objeto.wrapping_add(OB_X));
    let y_obj = rw(m, objeto.wrapping_add(OB_Y));
    let x_sonic = rw(m, jogador.wrapping_add(OB_X));
    let y_sonic = rw(m, jogador.wrapping_add(OB_Y));
    let sonic_status = rb(m, jogador.wrapping_add(OB_STATUS));
    if rb(m, objeto.wrapping_add(OB_ROUTINE_2)) != 0 {
        let pos = x_sonic.wrapping_sub(x_obj).wrapping_add(largura);
        if sonic_status & 2 != 0 || com_sinal(pos) < 0 || pos >= largura.wrapping_mul(2) {
            wb(m, jogador.wrapping_add(OB_STATUS), sonic_status & !8);
            let status = rb(m, objeto.wrapping_add(OB_STATUS));
            wb(m, objeto.wrapping_add(OB_STATUS), status & !8);
            wb(m, objeto.wrapping_add(OB_ROUTINE_2), 0);
        } else if rb(m, F_PLAYERCTRL) & 0x80 == 0
            && rb(m, jogador.wrapping_add(OB_ROUTINE)) < 6
            && rw(m, V_DEBUGUSE) == 0
        {
            let h = u16::from(rb(m, jogador.wrapping_add(OB_HEIGHT)));
            ww(
                m,
                jogador.wrapping_add(OB_Y),
                y_obj.wrapping_sub(altura_em_cima).wrapping_sub(h),
            );
        }
        return Ok(0);
    }
    let x = x_sonic.wrapping_sub(x_obj).wrapping_add(largura);
    let altura_sonic = u16::from(rb(m, jogador.wrapping_add(OB_HEIGHT)));
    let soma_altura = altura.wrapping_add(altura_sonic);
    let y = y_sonic
        .wrapping_sub(y_obj)
        .wrapping_add(4)
        .wrapping_add(soma_altura);
    if rb(m, objeto.wrapping_add(OB_RENDER)) & 0x80 == 0
        || com_sinal(x) < 0
        || x > largura.wrapping_mul(2)
        || com_sinal(y) < 0
        || y >= soma_altura.wrapping_mul(2)
    {
        let status = rb(m, objeto.wrapping_add(OB_STATUS));
        if status & 0x20 != 0 {
            ww(m, jogador.wrapping_add(OB_ANIM), 1);
            wb(m, objeto.wrapping_add(OB_STATUS), status & !0x20);
            wb(m, jogador.wrapping_add(OB_STATUS), sonic_status & !0x20);
        }
        return Ok(0);
    }
    if rb(m, F_PLAYERCTRL) & 0x80 != 0
        || rb(m, jogador.wrapping_add(OB_ROUTINE)) >= 6
        || rw(m, V_DEBUGUSE) != 0
    {
        return Ok(0);
    }
    let (desloc_x, borda_x) = if x <= largura {
        (x, x)
    } else {
        let d = x.wrapping_sub(largura.wrapping_mul(2));
        (d, d.wrapping_neg())
    };
    let (desloc_y, borda_y) = if y <= soma_altura {
        (y, y)
    } else {
        let d = y.wrapping_sub(4).wrapping_sub(soma_altura.wrapping_mul(2));
        (d, d.wrapping_neg())
    };
    if borda_x <= borda_y {
        if borda_y > 4 {
            let vx = com_sinal(rw(m, jogador.wrapping_add(OB_VELX)));
            if (com_sinal(desloc_x) > 0 && vx >= 0) || (com_sinal(desloc_x) < 0 && vx < 0) {
                ww(m, jogador.wrapping_add(OB_INERTIA), 0);
                ww(m, jogador.wrapping_add(OB_VELX), 0);
            }
            ww(
                m,
                jogador.wrapping_add(OB_X),
                x_sonic.wrapping_sub(desloc_x),
            );
            if sonic_status & 2 == 0 {
                wb(m, jogador.wrapping_add(OB_STATUS), sonic_status | 0x20);
                let status = rb(m, objeto.wrapping_add(OB_STATUS));
                wb(m, objeto.wrapping_add(OB_STATUS), status | 0x20);
                return Ok(1);
            }
        }
        let status = rb(m, objeto.wrapping_add(OB_STATUS));
        wb(m, objeto.wrapping_add(OB_STATUS), status & !0x20);
        wb(m, jogador.wrapping_add(OB_STATUS), sonic_status & !0x20);
        return Ok(1);
    }
    if com_sinal(desloc_y) < 0 {
        let vy = com_sinal(rw(m, jogador.wrapping_add(OB_VELY)));
        if vy == 0 && sonic_status & 2 == 0 {
            super::fisica::kill_sonic(m);
            return Ok(-1);
        }
        if vy < 0 {
            ww(
                m,
                jogador.wrapping_add(OB_Y),
                y_sonic.wrapping_sub(desloc_y),
            );
            ww(m, jogador.wrapping_add(OB_VELY), 0);
        }
        return Ok(-1);
    }
    if desloc_y >= 16 {
        return Ok(0);
    }
    let largura_real = u16::from(rb(m, objeto.wrapping_add(OB_ACTWID)));
    let pos = largura_real.wrapping_add(x_sonic).wrapping_sub(x_obj);
    if com_sinal(pos) < 0
        || pos >= largura_real.wrapping_mul(2)
        || com_sinal(rw(m, jogador.wrapping_add(OB_VELY))) < 0
    {
        return Ok(0);
    }
    ww(
        m,
        jogador.wrapping_add(OB_Y),
        y_sonic
            .wrapping_sub(desloc_y.wrapping_sub(4))
            .wrapping_sub(1),
    );
    if sonic_status & 8 != 0 {
        let indice = u32::from(rb(m, jogador.wrapping_add(SON_STANDONOBJECT)));
        let anterior = V_OBJSPACE.wrapping_add(indice.wrapping_mul(OBJECT_SIZE));
        let status = rb(m, anterior.wrapping_add(OB_STATUS));
        wb(m, anterior.wrapping_add(OB_STATUS), status & !8);
        wb(m, anterior.wrapping_add(OB_ROUTINE_2), 0);
    }
    let indice = objeto
        .wrapping_sub(V_OBJSPACE)
        .checked_div(OBJECT_SIZE)
        .unwrap_or(0);
    wb(
        m,
        jogador.wrapping_add(SON_STANDONOBJECT),
        u8::try_from(indice & 0x7F).unwrap_or(0),
    );
    wb(m, jogador.wrapping_add(OB_ANGLE), 0);
    ww(m, jogador.wrapping_add(OB_VELY), 0);
    let vx = rw(m, jogador.wrapping_add(OB_VELX));
    ww(m, jogador.wrapping_add(OB_INERTIA), vx);
    if sonic_status & 2 != 0 {
        super::fisica::sonic_reset_on_floor(m, jogador);
    }
    let status = rb(m, jogador.wrapping_add(OB_STATUS));
    wb(m, jogador.wrapping_add(OB_STATUS), status | 8);
    let status = rb(m, objeto.wrapping_add(OB_STATUS));
    wb(m, objeto.wrapping_add(OB_STATUS), status | 8);
    wb(m, objeto.wrapping_add(OB_ROUTINE_2), 2);
    Ok(-1)
}

/// `Scenery` (`1C`): toco da ponte da Green Hill e lançador da Star Light (`1C GHZ, SYZ Scenery.asm`).
/// Rotina 0 lê a tabela `Scen_Values` e avança para a rotina 2; rotina 2 checa `fora_da_janela` e exibe o sprite.
fn cenario(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    match rb(m, a0.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, a0.wrapping_add(OB_ROUTINE), 2);
            let subtipo = rb(m, a0.wrapping_add(OB_SUBTYPE));
            let base = SCEN_VALUES.wrapping_add(u32::from(subtipo).wrapping_mul(10));
            let (map, gfx, frame, actwid, prio, coltype) =
                if m.rom().len() > (base as usize).wrapping_add(10) && rom_l(m, base) != 0 {
                    (
                        rom_l(m, base),
                        rom_w(m, base.wrapping_add(4)),
                        rom_b(m, base.wrapping_add(6)),
                        rom_b(m, base.wrapping_add(7)),
                        rom_b(m, base.wrapping_add(8)),
                        rom_b(m, base.wrapping_add(9)),
                    )
                } else if subtipo == 3 {
                    (0x7FB2, 0x438E, 1, 16, 1, 0)
                } else {
                    (0x8ED4, 0x44D8, 0, 8, 2, 0)
                };
            wl(m, a0.wrapping_add(OB_MAP), map);
            ww(m, a0.wrapping_add(OB_GFX), gfx);
            let render = rb(m, a0.wrapping_add(OB_RENDER)) | 4;
            wb(m, a0.wrapping_add(OB_RENDER), render);
            wb(m, a0.wrapping_add(OB_FRAME), frame);
            wb(m, a0.wrapping_add(OB_ACTWID), actwid);
            wb(m, a0.wrapping_add(OB_PRIORITY), prio);
            wb(m, a0.wrapping_add(OB_COLTYPE), coltype);
            cenario_chkdel(m, a0);
            Ok(())
        }
        2 => {
            cenario_chkdel(m, a0);
            Ok(())
        }
        _ => Err(ErroAbertura::NaoImplementado("rotina de Scenery")),
    }
}

fn cenario_chkdel(m: &mut Memoria, a0: u32) {
    let x = rw(m, a0.wrapping_add(OB_X));
    if fora_da_janela(m, x) {
        delete_object(m, a0);
    } else {
        display_sprite(m, a0);
    }
}

/// `ExplosionItem` (`27`): explosão cinza de inimigo ou monitor destruído (`27, 3F Explosions.asm`).
fn explosao_item(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    match rb(m, a0.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, a0.wrapping_add(OB_ROUTINE), 2);
            let mut a1 = V_LVLOBJSPACE;
            while a1 < V_LVLOBJEND && rb(m, a1.wrapping_add(OB_ID)) != 0 {
                a1 = a1.wrapping_add(OBJECT_SIZE);
            }
            if a1 < V_LVLOBJEND {
                let x = rw(m, a0.wrapping_add(OB_X));
                let y = rw(m, a0.wrapping_add(OB_Y));
                let pf = rw(m, a0.wrapping_add(0x3E));
                wb(m, a1.wrapping_add(OB_ID), ID_ANIMALS);
                ww(m, a1.wrapping_add(OB_X), x);
                ww(m, a1.wrapping_add(OB_Y), y);
                ww(m, a1.wrapping_add(0x3E), pf);
            }
            explosao_main(m, a0);
            Ok(())
        }
        2 => {
            explosao_main(m, a0);
            Ok(())
        }
        4 => {
            let tempo = rb(m, a0.wrapping_add(OB_TIME_FRAME)).wrapping_sub(1);
            wb(m, a0.wrapping_add(OB_TIME_FRAME), tempo);
            if i8::from_be_bytes([tempo]) < 0 {
                wb(m, a0.wrapping_add(OB_TIME_FRAME), 7);
                let frame = rb(m, a0.wrapping_add(OB_FRAME)).wrapping_add(1);
                wb(m, a0.wrapping_add(OB_FRAME), frame);
                if frame >= 5 {
                    delete_object(m, a0);
                    return Ok(());
                }
            }
            display_sprite(m, a0);
            Ok(())
        }
        _ => Err(ErroAbertura::NaoImplementado("rotina de ExplosionItem")),
    }
}

fn explosao_main(m: &mut Memoria, a0: u32) {
    wb(m, a0.wrapping_add(OB_ROUTINE), 4);
    wl(m, a0.wrapping_add(OB_MAP), 0x9544); // Map_ExplodeItem
    ww(m, a0.wrapping_add(OB_GFX), 0x5A0); // ArtTile_Explosion
    wb(m, a0.wrapping_add(OB_RENDER), 4);
    wb(m, a0.wrapping_add(OB_PRIORITY), 1);
    wb(m, a0.wrapping_add(OB_COLTYPE), 0);
    wb(m, a0.wrapping_add(OB_ACTWID), 12);
    wb(m, a0.wrapping_add(OB_TIME_FRAME), 7);
    wb(m, a0.wrapping_add(OB_FRAME), 0);
    wb(m, V_SOUNDQUEUE1, 0xC1); // sfx_BreakItem
    display_sprite(m, a0);
}

/// `Animals` (`28`): animais libertados dos badniks (`28, 29 Animals and Points.asm`).
fn animal(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    match rb(m, a0.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, a0.wrapping_add(OB_ROUTINE), 2);
            let rnd = super::fisica::random_number(m) & 1;
            let zone = rb(m, V_ZONE);
            let animal_id = match zone {
                1 => {
                    if rnd == 0 {
                        2
                    } else {
                        3
                    }
                }
                2 => {
                    if rnd == 0 {
                        6
                    } else {
                        3
                    }
                }
                3 => {
                    if rnd == 0 {
                        4
                    } else {
                        5
                    }
                }
                4 => {
                    if rnd == 0 {
                        4
                    } else {
                        1
                    }
                }
                5 => {
                    if rnd == 0 {
                        0
                    } else {
                        1
                    }
                }
                _ => {
                    if rnd == 0 {
                        0
                    } else {
                        5
                    }
                }
            };
            wb(m, a0.wrapping_add(0x30), animal_id);

            let (spdx, spdy, map) = match animal_id {
                0 => (-0x200i16, -0x400i16, 0x9AE4u32),
                1 => (-0x200i16, -0x300i16, 0x9AFCu32),
                2 => (-0x180i16, -0x300i16, 0x9AE4u32),
                3 => (-0x140i16, -0x180i16, 0x9AFCu32),
                4 => (-0x1C0i16, -0x300i16, 0x9B14u32),
                5 => (-0x300i16, -0x400i16, 0x9AFCu32),
                _ => (-0x280i16, -0x380i16, 0x9B14u32),
            };
            let gfx = if animal_id & 1 == 0 {
                0x580u16
            } else {
                0x592u16
            };

            ww(m, a0.wrapping_add(0x32), u16_de(spdx)); // animal_speedX
            ww(m, a0.wrapping_add(0x34), u16_de(spdy)); // animal_speedY
            wl(m, a0.wrapping_add(OB_MAP), map);
            ww(m, a0.wrapping_add(OB_GFX), gfx);
            wb(m, a0.wrapping_add(OB_HEIGHT), 12);
            wb(m, a0.wrapping_add(OB_RENDER), 5); // sprite_cam_field | sprite_xflip
            wb(m, a0.wrapping_add(OB_PRIORITY), 6);
            wb(m, a0.wrapping_add(OB_ACTWID), 8);
            wb(m, a0.wrapping_add(OB_TIME_FRAME), 7);
            wb(m, a0.wrapping_add(OB_FRAME), 2);
            ww(m, a0.wrapping_add(OB_VELY), u16_de(-0x400i16));

            if rb(m, 0xFF_F7A7) != 0 {
                wb(m, a0.wrapping_add(OB_ROUTINE), 0x12);
                ww(m, a0.wrapping_add(OB_VELX), 0);
                display_sprite(m, a0);
                return Ok(());
            }

            let mut a1 = V_LVLOBJSPACE;
            while a1 < V_LVLOBJEND && rb(m, a1.wrapping_add(OB_ID)) != 0 {
                a1 = a1.wrapping_add(OBJECT_SIZE);
            }
            if a1 < V_LVLOBJEND {
                let x = rw(m, a0.wrapping_add(OB_X));
                let y = rw(m, a0.wrapping_add(OB_Y));
                let pframe = rw(m, a0.wrapping_add(0x3E)) >> 1;
                wb(m, a1.wrapping_add(OB_ID), ID_POINTS);
                ww(m, a1.wrapping_add(OB_X), x);
                ww(m, a1.wrapping_add(OB_Y), y);
                wb(
                    m,
                    a1.wrapping_add(OB_FRAME),
                    u8::try_from(pframe & 0x7F).unwrap_or(0),
                );
            }
            display_sprite(m, a0);
            Ok(())
        }
        2 => {
            let x = rw(m, a0.wrapping_add(OB_X));
            if fora_da_janela(m, x) {
                delete_object(m, a0);
                return Ok(());
            }
            super::fisica::speed_to_pos(m, a0);
            let vy = com_sinal(rw(m, a0.wrapping_add(OB_VELY))).wrapping_add(0x38);
            ww(m, a0.wrapping_add(OB_VELY), u16_de(vy));
            if vy >= 0 {
                let dist = com_sinal(super::fisica::obj_floor_dist(m, a0));
                if dist < 0 {
                    let y = rw(m, a0.wrapping_add(OB_Y)).wrapping_add(u16_de(dist));
                    ww(m, a0.wrapping_add(OB_Y), y);
                    let spdx = rw(m, a0.wrapping_add(0x32));
                    let spdy = rw(m, a0.wrapping_add(0x34));
                    ww(m, a0.wrapping_add(OB_VELX), spdx);
                    ww(m, a0.wrapping_add(OB_VELY), spdy);
                    wb(m, a0.wrapping_add(OB_FRAME), 1);
                    let anml_id = rb(m, a0.wrapping_add(0x30));
                    let rotina = anml_id.wrapping_mul(2).wrapping_add(4);
                    wb(m, a0.wrapping_add(OB_ROUTINE), rotina);
                    if rb(m, 0xFF_F7A7) != 0 && rb(m, V_VBLANK_BYTE) & 0x10 != 0 {
                        ww(m, a0.wrapping_add(OB_VELX), 0u16.wrapping_sub(spdx));
                        let render = rb(m, a0.wrapping_add(OB_RENDER)) ^ 1;
                        wb(m, a0.wrapping_add(OB_RENDER), render);
                    }
                }
            }
            display_sprite(m, a0);
            Ok(())
        }
        0x12 => {
            if rb(m, a0.wrapping_add(OB_RENDER)) & 0x80 == 0 {
                delete_object(m, a0);
                return Ok(());
            }
            let atraso = rw(m, a0.wrapping_add(0x36)).wrapping_sub(1);
            ww(m, a0.wrapping_add(0x36), atraso);
            if atraso == 0 {
                wb(m, a0.wrapping_add(OB_ROUTINE), 2);
                wb(m, a0.wrapping_add(OB_PRIORITY), 3);
            }
            display_sprite(m, a0);
            Ok(())
        }
        4 | 8 | 10 | 12 | 16 => {
            super::fisica::speed_to_pos(m, a0);
            let vy = com_sinal(rw(m, a0.wrapping_add(OB_VELY))).wrapping_add(0x38);
            ww(m, a0.wrapping_add(OB_VELY), u16_de(vy));
            if vy < 0 {
                wb(m, a0.wrapping_add(OB_FRAME), 1);
            } else {
                wb(m, a0.wrapping_add(OB_FRAME), 0);
                let dist = com_sinal(super::fisica::obj_floor_dist(m, a0));
                if dist < 0 {
                    let y = rw(m, a0.wrapping_add(OB_Y)).wrapping_add(u16_de(dist));
                    ww(m, a0.wrapping_add(OB_Y), y);
                    let spdy = rw(m, a0.wrapping_add(0x34));
                    ww(m, a0.wrapping_add(OB_VELY), spdy);
                }
            }
            let x = rw(m, a0.wrapping_add(OB_X));
            if fora_da_janela(m, x) {
                delete_object(m, a0);
            } else {
                display_sprite(m, a0);
            }
            Ok(())
        }
        6 | 14 => {
            super::fisica::speed_to_pos(m, a0);
            let vy = com_sinal(rw(m, a0.wrapping_add(OB_VELY))).wrapping_add(0x18);
            ww(m, a0.wrapping_add(OB_VELY), u16_de(vy));
            if vy >= 0 {
                let dist = com_sinal(super::fisica::obj_floor_dist(m, a0));
                if dist < 0 {
                    let y = rw(m, a0.wrapping_add(OB_Y)).wrapping_add(u16_de(dist));
                    ww(m, a0.wrapping_add(OB_Y), y);
                    let spdy = rw(m, a0.wrapping_add(0x34));
                    ww(m, a0.wrapping_add(OB_VELY), spdy);
                }
            }
            let tempo = rb(m, a0.wrapping_add(OB_TIME_FRAME)).wrapping_sub(1);
            wb(m, a0.wrapping_add(OB_TIME_FRAME), tempo);
            if i8::from_be_bytes([tempo]) < 0 {
                wb(m, a0.wrapping_add(OB_TIME_FRAME), 1);
                let quadro = (rb(m, a0.wrapping_add(OB_FRAME)).wrapping_add(1)) & 1;
                wb(m, a0.wrapping_add(OB_FRAME), quadro);
            }
            let x = rw(m, a0.wrapping_add(OB_X));
            if fora_da_janela(m, x) {
                delete_object(m, a0);
            } else {
                display_sprite(m, a0);
            }
            Ok(())
        }
        _ => {
            let x = rw(m, a0.wrapping_add(OB_X));
            if fora_da_janela(m, x) {
                delete_object(m, a0);
            } else {
                display_sprite(m, a0);
            }
            Ok(())
        }
    }
}

/// `Points` (`29`): indicador flutuante de pontos (`28, 29 Animals and Points.asm`).
fn pontos_objeto(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    match rb(m, a0.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, a0.wrapping_add(OB_ROUTINE), 2);
            wl(m, a0.wrapping_add(OB_MAP), 0x9B2C); // Map_Points
            ww(m, a0.wrapping_add(OB_GFX), 0x797 | 0x2000); // ArtTile_Points | Tile_Pal2 ($2797)
            wb(m, a0.wrapping_add(OB_RENDER), 4);
            wb(m, a0.wrapping_add(OB_PRIORITY), 1);
            wb(m, a0.wrapping_add(OB_ACTWID), 8);
            ww(m, a0.wrapping_add(OB_VELY), u16_de(-0x300i16));
            display_sprite(m, a0);
            Ok(())
        }
        2 => {
            let vy = com_sinal(rw(m, a0.wrapping_add(OB_VELY)));
            if vy >= 0 {
                delete_object(m, a0);
                return Ok(());
            }
            super::fisica::speed_to_pos(m, a0);
            let novo_vy = vy.wrapping_add(0x18);
            ww(m, a0.wrapping_add(OB_VELY), u16_de(novo_vy));
            display_sprite(m, a0);
            Ok(())
        }
        _ => Err(ErroAbertura::NaoImplementado("rotina de Points")),
    }
}

/// `WaterSound` (`49`): emissor invisível do som de cachoeira da GHZ (`49 GHZ Waterfall Sound.asm`).
fn water_sound(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    match rb(m, a0.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, a0.wrapping_add(OB_ROUTINE), 2);
            wb(m, a0.wrapping_add(OB_RENDER), 4);
            wsnd_play(m, a0);
            Ok(())
        }
        2 => {
            wsnd_play(m, a0);
            Ok(())
        }
        _ => Err(ErroAbertura::NaoImplementado("rotina de WaterSound")),
    }
}

fn wsnd_play(m: &mut Memoria, a0: u32) {
    let vb = rb(m, V_VBLANK_BYTE);
    if vb & 0x3F == 0 {
        wb(m, V_SOUNDQUEUE1, 0xD0); // sfx_Waterfall
    }
    let x = rw(m, a0.wrapping_add(OB_X));
    if fora_da_janela(m, x) {
        delete_object(m, a0);
    }
}

pub const MAP_PROCK: u32 = 0x00D79C;
pub const ART_TILE_PURPLE_ROCK: u16 = 0x63D0;

/// `PurpleRock` (`3B`): pedra roxa sólida da Green Hill Zone (`3B GHZ Purple Rock.asm`).
fn purple_rock(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    if rb(m, a0.wrapping_add(OB_ROUTINE)) == 0 {
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        wl(m, a0.wrapping_add(OB_MAP), MAP_PROCK);
        ww(m, a0.wrapping_add(OB_GFX), ART_TILE_PURPLE_ROCK);
        wb(m, a0.wrapping_add(OB_RENDER), 4);
        wb(m, a0.wrapping_add(OB_ACTWID), 19);
        wb(m, a0.wrapping_add(OB_PRIORITY), 4);
    }

    solid_object(m, a0, 27, 16, 16)?;
    display_sprite(m, a0);
    let x = rw(m, a0.wrapping_add(OB_X));
    if fora_da_janela(m, x) {
        delete_object(m, a0);
    }
    Ok(())
}

/// `Springs`: prepara a orientação, reage ao contato e anima sem alterar os dados da ROM.
fn mola(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    match rotina {
        0 => {
            wb(m, a0.wrapping_add(OB_ROUTINE), 2);
            wl(m, a0.wrapping_add(OB_MAP), MAP_SPRING);
            ww(m, a0.wrapping_add(OB_GFX), 0x523);
            let render = rb(m, a0.wrapping_add(OB_RENDER));
            wb(m, a0.wrapping_add(OB_RENDER), render | 4);
            wb(m, a0.wrapping_add(OB_ACTWID), 16);
            wb(m, a0.wrapping_add(OB_PRIORITY), 4);
            let sub = rb(m, a0.wrapping_add(OB_SUBTYPE));
            if sub & 0x10 != 0 {
                wb(m, a0.wrapping_add(OB_ROUTINE), 8);
                wb(m, a0.wrapping_add(OB_ANIM), 1);
                wb(m, a0.wrapping_add(OB_FRAME), 3);
                ww(m, a0.wrapping_add(OB_GFX), 0x533);
                wb(m, a0.wrapping_add(OB_ACTWID), 8);
            }
            if sub & 0x20 != 0 {
                wb(m, a0.wrapping_add(OB_ROUTINE), 14);
                let status = rb(m, a0.wrapping_add(OB_STATUS));
                wb(m, a0.wrapping_add(OB_STATUS), status | 2);
            }
            if sub & 2 != 0 {
                let gfx = rb(m, a0.wrapping_add(OB_GFX));
                wb(m, a0.wrapping_add(OB_GFX), gfx | 0x20);
            }
            let forca = rom_w(m, SPRING_POWERS.wrapping_add(u32::from(sub & 15)));
            ww(m, a0.wrapping_add(0x30), forca);
        }
        2 | 8 | 14 => {
            let colisao = if rotina == 8 {
                solid_object(m, a0, 19, 14, 15)?
            } else {
                solid_object(m, a0, 27, 8, 16)?
            };
            let status_mola = rb(m, a0.wrapping_add(OB_STATUS));
            let solido = rb(m, a0.wrapping_add(OB_ROUTINE_2));
            let acionar = match rotina {
                2 => solido != 0,
                8 => status_mola & 0x20 != 0,
                _ => solido == 0 && colisao < 0,
            };
            if acionar {
                wb(m, a0.wrapping_add(OB_ROUTINE), rotina.wrapping_add(2));
                let forca = rw(m, a0.wrapping_add(0x30));
                let status = rb(m, V_PLAYER.wrapping_add(OB_STATUS));
                if rotina == 8 {
                    let x = rw(m, V_PLAYER.wrapping_add(OB_X));
                    let virada = status_mola & 1 != 0;
                    ww(
                        m,
                        V_PLAYER.wrapping_add(OB_X),
                        if virada {
                            x.wrapping_add(8)
                        } else {
                            x.wrapping_sub(8)
                        },
                    );
                    let vx = if virada { forca } else { forca.wrapping_neg() };
                    ww(m, V_PLAYER.wrapping_add(OB_VELX), vx);
                    ww(m, V_PLAYER.wrapping_add(OB_INERTIA), vx);
                    ww(m, V_PLAYER.wrapping_add(SON_LOCKTIME), 15);
                    wb(m, V_PLAYER.wrapping_add(OB_STATUS), (status ^ 1) & !0x20);
                    if status & 4 == 0 {
                        wb(m, V_PLAYER.wrapping_add(OB_ANIM), ANIM_WALK);
                    }
                    wb(m, a0.wrapping_add(OB_STATUS), status_mola & !0x20);
                } else {
                    let y = rw(m, V_PLAYER.wrapping_add(OB_Y));
                    ww(
                        m,
                        V_PLAYER.wrapping_add(OB_Y),
                        if rotina == 2 {
                            y.wrapping_add(8)
                        } else {
                            y.wrapping_sub(8)
                        },
                    );
                    ww(
                        m,
                        V_PLAYER.wrapping_add(OB_VELY),
                        if rotina == 2 {
                            forca
                        } else {
                            forca.wrapping_neg()
                        },
                    );
                    wb(m, V_PLAYER.wrapping_add(OB_STATUS), (status | 2) & !8);
                    wb(m, V_PLAYER.wrapping_add(OB_ROUTINE), 2);
                    if rotina == 2 {
                        wb(m, V_PLAYER.wrapping_add(OB_ANIM), 0x10);
                    }
                    wb(m, a0.wrapping_add(OB_STATUS), status_mola & !8);
                    wb(m, a0.wrapping_add(OB_ROUTINE_2), 0);
                }
                wb(m, V_SOUNDQUEUE1, 0xCC);
                animate_sprite(m, a0, ANI_SPRING)?;
            }
        }
        4 | 10 | 16 => animate_sprite(m, a0, ANI_SPRING)?,
        6 | 12 | 18 => {
            wb(
                m,
                a0.wrapping_add(OB_PREV_ANI),
                if rotina == 12 { 2 } else { 1 },
            );
            wb(m, a0.wrapping_add(OB_ROUTINE), rotina.wrapping_sub(4));
        }
        _ => return Err(ErroAbertura::NaoImplementado("rotina de mola")),
    }
    display_sprite(m, a0);
    let x = rw(m, a0.wrapping_add(OB_X));
    if fora_da_janela(m, x) {
        delete_object(m, a0);
    }
    Ok(())
}

/// `Monitor` (`26`): a caixa de item. Rotina 0 prepara; rotina 2 confere o Sonic em cima, ao lado
/// e embaixo (aqui só "longe"), anima e mostra. As rotinas de quebrar ficam para a Etapa 9.
fn monitor(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    match rb(m, a0.wrapping_add(OB_ROUTINE)) {
        0 => {
            wb(m, a0.wrapping_add(OB_ROUTINE), 2);
            wb(m, a0.wrapping_add(OB_HEIGHT), 14);
            wb(m, a0.wrapping_add(OB_WIDTH), 14);
            wl(m, a0.wrapping_add(OB_MAP), MAP_MONITOR);
            ww(m, a0.wrapping_add(OB_GFX), ARTTILE_MONITOR);
            wb(m, a0.wrapping_add(OB_RENDER), 4);
            wb(m, a0.wrapping_add(OB_PRIORITY), 3);
            wb(m, a0.wrapping_add(OB_ACTWID), 15);
            let marca = V_OBJSTATE
                .wrapping_add(2)
                .wrapping_add(u32::from(rb(m, a0.wrapping_add(OB_RESPAWNNO))));
            let b = rb(m, marca) & 0x7F;
            wb(m, marca, b);
            if b & 1 != 0 {
                wb(m, a0.wrapping_add(OB_ROUTINE), 8);
                wb(m, a0.wrapping_add(OB_FRAME), 0xB);
                return Ok(());
            }
            wb(m, a0.wrapping_add(OB_COLTYPE), 0x46);
            let sub = rb(m, a0.wrapping_add(OB_SUBTYPE));
            wb(m, a0.wrapping_add(OB_ANIM), sub);
            monitor_solido(m, a0)
        }
        2 => monitor_solido(m, a0),
        8 => mon_display(m, a0),
        _ => Err(ErroAbertura::NaoImplementado("monitor quebrando")),
    }
}

/// `Mon_Solid` (rotina normal): o Sonic não pode estar tocando o monitor.
fn monitor_solido(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    if rb(m, a0.wrapping_add(OB_ROUTINE_2)) != 0 {
        return Err(ErroAbertura::NaoImplementado("monitor com o Sonic em cima"));
    }
    if mon_solid_sides(m, a0) || rb(m, a0.wrapping_add(OB_STATUS)) & 0x20 != 0 {
        return Err(ErroAbertura::NaoImplementado("Sonic tocando o monitor"));
    }
    animate_sprite(m, a0, ANI_MONITOR)?;
    mon_display(m, a0)
}

/// `Mon_SolidSides`: o Sonic está dentro da caixa do monitor?
fn mon_solid_sides(m: &mut Memoria, a0: u32) -> bool {
    let d1 = 15u16.wrapping_add(11);
    let sx = rw(m, V_PLAYER.wrapping_add(OB_X));
    let d0 = sx
        .wrapping_sub(rw(m, a0.wrapping_add(OB_X)))
        .wrapping_add(d1);
    if com_sinal(d0) < 0 || d0 > d1.wrapping_mul(2) {
        return false;
    }
    let altura = u16::from_be_bytes(
        i16::from(i8::from_be_bytes([rb(m, V_PLAYER.wrapping_add(OB_HEIGHT))])).to_be_bytes(),
    );
    let d2 = 15u16.wrapping_add(altura);
    let sy = rw(m, V_PLAYER.wrapping_add(OB_Y));
    let d3 = sy
        .wrapping_sub(rw(m, a0.wrapping_add(OB_Y)))
        .wrapping_add(d2);
    if com_sinal(d3) < 0 || d3 >= d2.wrapping_mul(2) {
        return false;
    }
    rb(m, F_PLAYERCTRL) & 0x80 == 0
        && rb(m, V_PLAYER.wrapping_add(OB_ROUTINE)) < 6
        && rw(m, V_DEBUGUSE) == 0
}

/// `Mon_Display` (FixBugs = 0): mostra e some fora da janela.
fn mon_display(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    display_sprite(m, a0);
    let x = rw(m, a0.wrapping_add(OB_X));
    if fora_da_janela(m, x) {
        delete_object(m, a0);
    }
    Ok(())
}

/// Deslocamento (X, Y) entre os anéis de uma fileira, por subtipo (`Ring_PosData`).
const RING_POS_DATA: u32 = 0xA1BC;
/// `ring_origX` e `ring_respawnbit` (`objoff_32` e `objoff_34`).
const RING_ORIGX: u32 = 0x32;
const RING_RESPAWNBIT: u32 = 0x34;
/// `col_12x12|col_item`.
const COL_ANEL: u8 = 0x47;

/// Objeto `Rings` (`25`): um grupo de anéis. A rotina 0 cria os anéis da fileira (o primeiro é o
/// próprio objeto), e a rotina 2 mostra o quadro global e some quando sai da janela.
fn aneis(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    match rb(m, a0.wrapping_add(OB_ROUTINE)) {
        0 => ring_main(m, a0),
        2 => {
            ring_animate(m, a0);
            Ok(())
        }
        4 => {
            wb(m, a0.wrapping_add(OB_ROUTINE), 6);
            wb(m, a0.wrapping_add(OB_COLTYPE), 0);
            wb(m, a0.wrapping_add(OB_PRIORITY), 1);
            collect_ring(m);
            let marca = V_OBJSTATE
                .wrapping_add(2)
                .wrapping_add(u32::from(rb(m, a0.wrapping_add(OB_RESPAWNNO))));
            let bit = rb(m, a0.wrapping_add(RING_RESPAWNBIT)) & 7;
            let estado = rb(m, marca) | 1u8.wrapping_shl(u32::from(bit));
            wb(m, marca, estado);
            ring_sparkle(m, a0)
        }
        6 => ring_sparkle(m, a0),
        8 => {
            delete_object(m, a0);
            Ok(())
        }
        _ => Err(ErroAbertura::NaoImplementado("anel coletado")),
    }
}

/// `CollectRing`: o original permite ultrapassar 999 anéis.
pub(super) fn collect_ring(m: &mut Memoria) {
    let aneis = rw(m, V_RINGS).wrapping_add(1);
    ww(m, V_RINGS, aneis);
    let atualizacao = rb(m, F_RINGCOUNT) | 1;
    wb(m, F_RINGCOUNT, atualizacao);
    let mut som = SFX_RING;
    if aneis >= 100 {
        let flags = rb(m, V_LIFECOUNT);
        let bit = if flags & 2 == 0 {
            2
        } else if aneis >= 200 && flags & 4 == 0 {
            4
        } else {
            0
        };
        if bit != 0 {
            wb(m, V_LIFECOUNT, flags | bit);
            let vidas = rb(m, V_LIVES).wrapping_add(1);
            let atualizacao = rb(m, F_LIFECOUNT).wrapping_add(1);
            wb(m, V_LIVES, vidas);
            wb(m, F_LIFECOUNT, atualizacao);
            som = 0x88;
        }
    }
    wb(m, V_SOUNDQUEUE1, som);
}

fn ring_sparkle(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    animate_sprite(m, a0, ANI_RING)?;
    display_sprite(m, a0);
    Ok(())
}

#[cfg(test)]
mod testes_aneis {
    use super::*;

    #[test]
    fn anel_atualiza_contadores_e_vidas_nos_limites() {
        let mut m = Memoria::nova(vec![0; 16]);
        ww(&mut m, V_RINGS, 99);
        wb(&mut m, V_LIVES, 3);
        collect_ring(&mut m);
        assert_eq!(rw(&mut m, V_RINGS), 100);
        assert_eq!(rb(&mut m, V_LIVES), 4);
        assert_eq!(rb(&mut m, V_LIFECOUNT) & 2, 2);
        assert_eq!(rb(&mut m, V_SOUNDQUEUE1), 0x88);

        ww(&mut m, V_RINGS, 199);
        collect_ring(&mut m);
        assert_eq!(rw(&mut m, V_RINGS), 200);
        assert_eq!(rb(&mut m, V_LIVES), 5);
        assert_eq!(rb(&mut m, V_LIFECOUNT) & 6, 6);
        collect_ring(&mut m);
        assert_eq!(rb(&mut m, V_LIVES), 5);
        assert_eq!(rb(&mut m, V_SOUNDQUEUE1), SFX_RING);
    }

    #[test]
    fn fim_de_animacao_avanca_rotina_do_objeto() {
        let mut rom = vec![0; 0x200];
        rom[0x101] = 2;
        rom[0x103] = 0xfc;
        let mut m = Memoria::nova(rom);
        wb(&mut m, V_LVLOBJSPACE.wrapping_add(OB_ROUTINE), 6);
        animate_sprite(&mut m, V_LVLOBJSPACE, 0x100).unwrap();
        assert_eq!(rb(&mut m, V_LVLOBJSPACE.wrapping_add(OB_ROUTINE)), 8);
    }
}

/// `Ring_Animate` (FixBugs = 0): mostra o anel e depois confere se saiu da janela.
fn ring_animate(m: &mut Memoria, a0: u32) {
    let quadro = rb(m, V_ANI1_FRAME);
    wb(m, a0.wrapping_add(OB_FRAME), quadro);
    display_sprite(m, a0);
    let origem = rw(m, a0.wrapping_add(RING_ORIGX));
    if fora_da_janela(m, origem) {
        delete_object(m, a0);
    }
}

/// `out_of_range`: a posição X, em blocos de $80, fica a mais de `128+320+192` da câmera.
pub fn fora_da_janela(m: &mut Memoria, x: u16) -> bool {
    let d0 = x & 0xFF80;
    let d1 = rw(m, V_SCREENPOSX).wrapping_sub(128) & 0xFF80;
    d0.wrapping_sub(d1) > 128 + 320 + 192
}

/// Encontra um slot livre de objeto na faixa de objetos de fase.
pub fn find_free_obj(m: &mut Memoria) -> Option<u32> {
    let mut a1 = V_LVLOBJSPACE;
    while a1 < V_LVLOBJEND {
        if rb(m, a1.wrapping_add(OB_ID)) == 0 {
            return Some(a1);
        }
        a1 = a1.wrapping_add(OBJECT_SIZE);
    }
    None
}

/// `RememberState`: desenha o sprite ou apaga o objeto com atualização do bit de respawn.
pub fn remember_state(m: &mut Memoria, a0: u32) {
    let x = rw(m, a0.wrapping_add(OB_X));
    if fora_da_janela(m, x) {
        let n = rb(m, a0.wrapping_add(OB_RESPAWNNO));
        if n != 0 {
            let marca = V_OBJSTATE.wrapping_add(2).wrapping_add(u32::from(n));
            let valor = rb(m, marca) & 0x7F;
            wb(m, marca, valor);
        }
        delete_object(m, a0);
    } else {
        display_sprite(m, a0);
    }
}

/// `Ring_Main`: cria os anéis da fileira que ainda não foram coletados.
fn ring_main(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let a2 = V_OBJSTATE
        .wrapping_add(2)
        .wrapping_add(u32::from(rb(m, a0.wrapping_add(OB_RESPAWNNO))));
    let mut d4 = rb(m, a2);
    let subtipo = rb(m, a0.wrapping_add(OB_SUBTYPE));
    let mut restam = u16::from(subtipo & 7);
    if restam == 7 {
        restam = 6;
    }
    let pos = RING_POS_DATA.wrapping_add(u32::from(subtipo >> 4).wrapping_mul(2));
    let dx = u16::from_be_bytes(i16::from(i8::from_be_bytes([rom_b(m, pos)])).to_be_bytes());
    let dy = u16::from_be_bytes(
        i16::from(i8::from_be_bytes([rom_b(m, pos.wrapping_add(1))])).to_be_bytes(),
    );
    let mut d2 = rw(m, a0.wrapping_add(OB_X));
    let mut d3 = rw(m, a0.wrapping_add(OB_Y));
    let mut indice = 0u8;
    let mut a1 = a0;
    let mut primeiro = true;
    loop {
        let coletado = d4 & 1 != 0;
        d4 >>= 1;
        if !coletado {
            let b = rb(m, a2) & 0x7F;
            wb(m, a2, b);
            if !primeiro {
                a1 = V_LVLOBJSPACE;
                while a1 < V_LVLOBJEND && rb(m, a1.wrapping_add(OB_ID)) != 0 {
                    a1 = a1.wrapping_add(OBJECT_SIZE);
                }
                if a1 >= V_LVLOBJEND {
                    break;
                }
            }
            wb(m, a1.wrapping_add(OB_ID), ID_RINGS);
            let r = rb(m, a1.wrapping_add(OB_ROUTINE)).wrapping_add(2);
            wb(m, a1.wrapping_add(OB_ROUTINE), r);
            ww(m, a1.wrapping_add(OB_X), d2);
            let origem = rw(m, a0.wrapping_add(OB_X));
            ww(m, a1.wrapping_add(RING_ORIGX), origem);
            ww(m, a1.wrapping_add(OB_Y), d3);
            wl(m, a1.wrapping_add(OB_MAP), MAP_RING);
            ww(m, a1.wrapping_add(OB_GFX), ARTTILE_RING | 0x2000);
            wb(m, a1.wrapping_add(OB_RENDER), 4);
            wb(m, a1.wrapping_add(OB_PRIORITY), 2);
            wb(m, a1.wrapping_add(OB_COLTYPE), COL_ANEL);
            wb(m, a1.wrapping_add(OB_ACTWID), 16 / 2);
            let n = rb(m, a0.wrapping_add(OB_RESPAWNNO));
            wb(m, a1.wrapping_add(OB_RESPAWNNO), n);
            wb(m, a1.wrapping_add(RING_RESPAWNBIT), indice);
        }
        primeiro = false;
        indice = indice.wrapping_add(1);
        d2 = d2.wrapping_add(dx);
        d3 = d3.wrapping_add(dy);
        if restam == 0 {
            break;
        }
        restam = restam.wrapping_sub(1);
    }
    // O primeiro anel já coletado: este objeto não vira anel e some.
    if rb(m, a2) & 1 != 0 {
        delete_object(m, a0);
        return Ok(());
    }
    ring_animate(m, a0);
    Ok(())
}

/// Objeto `HUD` (`21`): o painel de pontos, tempo e anéis. Pisca o quadro dos anéis ou do tempo.
fn hud_objeto(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    if rb(m, a0.wrapping_add(OB_ROUTINE)) == 0 {
        // `HUD_Main`.
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        ww(m, a0.wrapping_add(OB_X), 0x80 + 0x10);
        ww(m, a0.wrapping_add(OB_SCREENY), 0x80 + 0x88);
        wl(m, a0.wrapping_add(OB_MAP), MAP_HUD);
        ww(m, a0.wrapping_add(OB_GFX), ARTTILE_HUD);
        wb(m, a0.wrapping_add(OB_RENDER), 0);
        wb(m, a0.wrapping_add(OB_PRIORITY), 0);
    }
    // `HUD_Flash` (FixBugs = 0): com anéis o painel fica amarelo; sem anéis, pisca o dos anéis.
    if rw(m, V_RINGS) != 0 {
        wb(m, a0.wrapping_add(OB_FRAME), 0);
        display_sprite(m, a0);
        return Ok(());
    }
    let mut quadro = 0u8;
    if rb(m, V_FRAMEBYTE) & 8 == 0 {
        quadro = quadro.wrapping_add(1);
    }
    if rb(m, V_TIMEMIN) == 9 {
        quadro = quadro.wrapping_add(2);
    }
    wb(m, a0.wrapping_add(OB_FRAME), quadro);
    display_sprite(m, a0);
    Ok(())
}

/// Palavra com sinal (`bge` e `bmi` comparam assim).
fn com_sinal(w: u16) -> i16 {
    i16::from_be_bytes(w.to_be_bytes())
}

fn u16_de(i: i16) -> u16 {
    u16::from_be_bytes(i.to_be_bytes())
}

/// Objeto `TitleCard` (`34`): os quatro elementos dos cartões de título da fase (nome da zona,
/// "ZONE", "ACT" e o oval). Rotinas 0 (carga), 2 (entrada), 4 e 6 (espera e saída).
fn cartao_de_titulo(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    match rb(m, a0.wrapping_add(OB_ROUTINE)) {
        0 => {
            card_load_for_zone(m, a0);
            card_move_in(m, a0);
            Ok(())
        }
        2 => {
            card_move_in(m, a0);
            Ok(())
        }
        4 | 6 => card_wait(m, a0),
        _ => Err(ErroAbertura::NaoImplementado("rotina do TitleCard")),
    }
}

/// `Card_LoadForZone`: o objeto raiz vira o nome da zona e cria os outros três, cada um no espaço
/// seguinte, com as posições de `Card_ConData` (início, que também é o fim, e alvo) e com a
/// altura, a rotina e o quadro de `Card_ItemData`.
fn card_load_for_zone(m: &mut Memoria, a0: u32) {
    let zona_e_ato = rw(m, V_ZONE_ACT);
    let mut d0 = u16::from(rb(m, V_ZONE));
    if zona_e_ato == ID_LZ_ACT4 {
        d0 = 5;
    }
    // `d2`: o quadro do nome da zona.
    let mut d2 = d0;
    if zona_e_ato == ID_FZ {
        d0 = 6;
        d2 = 0xB;
    }
    let mut a3 = CARD_CONDATA.wrapping_add(u32::from(d0).wrapping_mul(16));
    let mut a2 = CARD_ITEMDATA;
    let mut a1 = a0;
    for _ in 0..4 {
        wb(m, a1.wrapping_add(OB_ID), ID_TITLE_CARD);
        let inicio = rom_w(m, a3);
        ww(m, a1.wrapping_add(OB_X), inicio);
        ww(m, a1.wrapping_add(CARD_FINALX), inicio);
        let alvo = rom_w(m, a3.wrapping_add(2));
        ww(m, a1.wrapping_add(CARD_MAINX), alvo);
        a3 = a3.wrapping_add(4);
        let y = rom_w(m, a2);
        ww(m, a1.wrapping_add(OB_SCREENY), y);
        let rotina = rom_b(m, a2.wrapping_add(2));
        wb(m, a1.wrapping_add(OB_ROUTINE), rotina);
        let mut quadro = rom_b(m, a2.wrapping_add(3));
        a2 = a2.wrapping_add(4);
        if quadro == 0 {
            quadro = d2.to_be_bytes()[1];
        }
        // O número do ato soma o ato atual; o ato 4 (SBZ3) usa a arte do "3".
        if quadro == 7 {
            let ato = rb(m, V_ACT);
            quadro = quadro.wrapping_add(ato);
            if ato == 3 {
                quadro = quadro.wrapping_sub(1);
            }
        }
        wb(m, a1.wrapping_add(OB_FRAME), quadro);
        wl(m, a1.wrapping_add(OB_MAP), MAP_CARD);
        ww(m, a1.wrapping_add(OB_GFX), ARTTILE_TITLE_CARD | TILE_PRIO);
        wb(m, a1.wrapping_add(OB_ACTWID), 240 / 2);
        // `sprite_cam_screen`: posição de tela, sem câmera.
        wb(m, a1.wrapping_add(OB_RENDER), 0);
        wb(m, a1.wrapping_add(OB_PRIORITY), 0);
        // Um segundo de espera antes de sair (palavra em `obTimeFrame`).
        ww(m, a1.wrapping_add(OB_TIME_FRAME), 60);
        a1 = a1.wrapping_add(OBJECT_SIZE);
    }
}

/// `Card_MoveIn`: anda 16 pixels por quadro rumo a `card_mainX`, e aparece se está na tela.
fn card_move_in(m: &mut Memoria, a0: u32) {
    let alvo = rw(m, a0.wrapping_add(CARD_MAINX));
    let x = rw(m, a0.wrapping_add(OB_X));
    if alvo != x {
        // `bge`: alvo maior ou igual (com sinal) quer dizer que o elemento vem da esquerda.
        let passo = if com_sinal(alvo) >= com_sinal(x) {
            16
        } else {
            0u16.wrapping_sub(16)
        };
        ww(m, a0.wrapping_add(OB_X), x.wrapping_add(passo));
    }
    card_na_tela(m, a0);
}

/// `.checkOffScreen` (FixBugs = 0): só aparece com X não negativo e abaixo de `$80+320+64`.
fn card_na_tela(m: &mut Memoria, a0: u32) {
    let x = rw(m, a0.wrapping_add(OB_X));
    if com_sinal(x) < 0 || x >= 0x80 + 320 + 64 {
        return;
    }
    display_sprite(m, a0);
}

/// `Card_Wait`: espera o tempo do objeto, depois sai.
fn card_wait(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let tempo = rw(m, a0.wrapping_add(OB_TIME_FRAME));
    if tempo == 0 {
        return card_move_out(m, a0);
    }
    ww(m, a0.wrapping_add(OB_TIME_FRAME), tempo.wrapping_sub(1));
    display_sprite(m, a0);
    Ok(())
}

/// `Card_MoveOut`: anda 32 pixels por quadro rumo a `card_finalX`. Fora da tela ou no fim, troca
/// a arte e some.
fn card_move_out(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    // `tst.b obRender(a0)` e `bpl`: o bit 7 diz se o sprite saiu na tela no último quadro.
    if rb(m, a0.wrapping_add(OB_RENDER)) & 0x80 == 0 {
        return card_change_art(m, a0);
    }
    let alvo = rw(m, a0.wrapping_add(CARD_FINALX));
    let x = rw(m, a0.wrapping_add(OB_X));
    if alvo == x {
        return card_change_art(m, a0);
    }
    let passo = if com_sinal(alvo) >= com_sinal(x) {
        32
    } else {
        0u16.wrapping_sub(32)
    };
    ww(m, a0.wrapping_add(OB_X), x.wrapping_add(passo));
    card_na_tela(m, a0);
    Ok(())
}

/// `Card_ChangeArt`: o elemento do nome da zona (rotina 4) devolve o espaço da VRAM às explosões e
/// aos animais da zona; todos os elementos somem (`DeleteObject`).
fn card_change_art(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    if rb(m, a0.wrapping_add(OB_ROUTINE)) == 4 {
        plc::add_plc(m, PLCID_EXPLODE)?;
        let zona = u16::from(rb(m, V_ZONE));
        plc::add_plc(m, zona.wrapping_add(PLCID_GHZANIMALS))?;
    }
    delete_object(m, a0);
    Ok(())
}

/// `AnimateSprite` (`_incObj/sub AnimateSprite.asm`): avança o roteiro de animação do objeto
/// `a0`. `roteiro` é o endereço do roteiro na ROM. Só os códigos de fim `$FF` (recomeça) e `$FE`
/// (volta alguns quadros) existem nesta versão; os outros devolvem `NaoImplementado`.
pub(super) fn animate_sprite(m: &mut Memoria, a0: u32, roteiro: u32) -> Result<(), ErroAbertura> {
    let animacao = rb(m, a0.wrapping_add(OB_ANIM));
    if animacao != rb(m, a0.wrapping_add(OB_PREV_ANI)) {
        wb(m, a0.wrapping_add(OB_PREV_ANI), animacao);
        wb(m, a0.wrapping_add(OB_ANI_FRAME), 0);
        wb(m, a0.wrapping_add(OB_TIME_FRAME), 0);
    }
    let tempo = rb(m, a0.wrapping_add(OB_TIME_FRAME)).wrapping_sub(1);
    wb(m, a0.wrapping_add(OB_TIME_FRAME), tempo);
    // `bpl Anim_Wait`: ainda há tempo neste quadro.
    if tempo & 0x80 == 0 {
        return Ok(());
    }
    // `Anim_LoadNextFrame`: o deslocamento do roteiro vem da tabela de palavras no início.
    // `add.w d0,d0` e `adda.w (a1,d0.w),a1`: a palavra da entrada é somada ao próprio `a1`.
    let entrada = roteiro.wrapping_add(u32::from(animacao).wrapping_mul(2));
    let roteiro = roteiro.wrapping_add(ext(rom_w(m, entrada)));
    wb(m, a0.wrapping_add(OB_TIME_FRAME), rom_byte(m, roteiro));
    let indice = u32::from(rb(m, a0.wrapping_add(OB_ANI_FRAME)));
    let codigo = rom_byte(m, roteiro.wrapping_add(1).wrapping_add(indice));
    if codigo & 0x80 == 0 {
        return animacao_quadro(m, a0, codigo);
    }
    match codigo {
        // `Anim_End_FF`: volta ao começo do roteiro.
        0xFF => {
            wb(m, a0.wrapping_add(OB_ANI_FRAME), 0);
            let primeiro = rom_byte(m, roteiro.wrapping_add(1));
            animacao_quadro(m, a0, primeiro)
        }
        // `Anim_End_FE`: volta um número de quadros dado pelo byte seguinte.
        0xFE => {
            let voltar = rom_byte(m, roteiro.wrapping_add(indice).wrapping_add(2));
            let novo = rb(m, a0.wrapping_add(OB_ANI_FRAME)).wrapping_sub(voltar);
            wb(m, a0.wrapping_add(OB_ANI_FRAME), novo);
            let indice_novo = indice.wrapping_sub(u32::from(voltar)) & 0xFF;
            let quadro = rom_byte(m, roteiro.wrapping_add(indice_novo).wrapping_add(1));
            animacao_quadro(m, a0, quadro)
        }
        0xFC => {
            let rotina = rb(m, a0.wrapping_add(OB_ROUTINE)).wrapping_add(2);
            wb(m, a0.wrapping_add(OB_ROUTINE), rotina);
            Ok(())
        }
        _ => Err(ErroAbertura::NaoImplementado("flag de animação")),
    }
}

/// `Anim_SetFrameAndFlipFlags`: grava o quadro e os espelhamentos do roteiro (bits 5 e 6 do
/// código, girados para 0 e 1), e avança o índice do roteiro.
fn animacao_quadro(m: &mut Memoria, a0: u32, codigo: u8) -> Result<(), ErroAbertura> {
    wb(m, a0.wrapping_add(OB_FRAME), codigo & 0x1F);
    let status = rb(m, a0.wrapping_add(OB_STATUS));
    let espelho = (codigo.rotate_left(3) ^ status) & 0x03;
    let render = rb(m, a0.wrapping_add(OB_RENDER)) & !0x03;
    wb(m, a0.wrapping_add(OB_RENDER), render | espelho);
    let indice = rb(m, a0.wrapping_add(OB_ANI_FRAME)).wrapping_add(1);
    wb(m, a0.wrapping_add(OB_ANI_FRAME), indice);
    Ok(())
}

/// Objeto `TitleSonic` (`0E`, rótulo `TitleSonic`): o Sonic grande que sobe e acena.
fn titulo_sonic(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    match rb(m, a0.wrapping_add(OB_ROUTINE)) {
        // `TSon_Main`: posição, mapa e arte; depois cai em `TSon_Delay`.
        0 => {
            wb(m, a0.wrapping_add(OB_ROUTINE), 2);
            ww(m, a0.wrapping_add(OB_X), 0x80 + 0x70);
            ww(m, a0.wrapping_add(OB_SCREENY), 0x80 + 0x5E);
            wl(m, a0.wrapping_add(OB_MAP), MAP_TSON);
            ww(m, a0.wrapping_add(OB_GFX), 0x300 | 0x2000);
            wb(m, a0.wrapping_add(OB_PRIORITY), 1);
            wb(m, a0.wrapping_add(OB_DELAY_ANI), 30 - 1);
            animate_sprite(m, a0, ANI_TSON)?;
            tson_delay(m, a0)
        }
        2 => tson_delay(m, a0),
        4 => tson_move(m, a0),
        6 => {
            animate_sprite(m, a0, ANI_TSON)?;
            display_sprite(m, a0);
            Ok(())
        }
        _ => Err(ErroAbertura::NaoImplementado("rotina do TitleSonic")),
    }
}

/// `TSon_Delay`: espera meio segundo, depois passa a subir.
fn tson_delay(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let espera = rb(m, a0.wrapping_add(OB_DELAY_ANI)).wrapping_sub(1);
    wb(m, a0.wrapping_add(OB_DELAY_ANI), espera);
    if espera & 0x80 == 0 {
        return Ok(());
    }
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE)).wrapping_add(2);
    wb(m, a0.wrapping_add(OB_ROUTINE), rotina);
    display_sprite(m, a0);
    Ok(())
}

/// `TSon_Move`: sobe Sonic oito pixels por quadro até a posição final.
fn tson_move(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    let y = rw(m, a0.wrapping_add(OB_SCREENY)).wrapping_sub(8);
    ww(m, a0.wrapping_add(OB_SCREENY), y);
    if y == 0x80 + 0x16 {
        let rotina = rb(m, a0.wrapping_add(OB_ROUTINE)).wrapping_add(2);
        wb(m, a0.wrapping_add(OB_ROUTINE), rotina);
    }
    display_sprite(m, a0);
    Ok(())
}

/// Objeto `PSBTM` (`0F`): "PRESS START BUTTON", o "TM" e os sprites que escondem o torso.
/// O quadro carregado decide a variante (`PSB_Main`).
fn psbtm(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    match rb(m, a0.wrapping_add(OB_ROUTINE)) {
        0 => psb_main(m, a0)?,
        2 => psb_prs_start(m, a0)?,
        4 => {}
        _ => return Err(ErroAbertura::NaoImplementado("rotina do PSBTM")),
    }
    display_sprite(m, a0);
    Ok(())
}

/// `PSB_Main`: escolhe a variante pelo quadro e pela posição.
fn psb_main(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    wb(m, a0.wrapping_add(OB_ROUTINE), 2);
    ww(m, a0.wrapping_add(OB_X), 0x80 + 0x50);
    ww(m, a0.wrapping_add(OB_SCREENY), 0x80 + 0xB0);
    wl(m, a0.wrapping_add(OB_MAP), MAP_PSB);
    ww(m, a0.wrapping_add(OB_GFX), 0x200);
    let quadro = rb(m, a0.wrapping_add(OB_FRAME));
    if quadro < 2 {
        return psb_prs_start(m, a0);
    }
    // Não é "PRESS START": é "TM" (quadro 3) ou os sprites que escondem o torso (quadro 2).
    wb(m, a0.wrapping_add(OB_ROUTINE), 4);
    if quadro != 3 {
        return Ok(());
    }
    ww(m, a0.wrapping_add(OB_GFX), 0x510 | 0x2000);
    ww(m, a0.wrapping_add(OB_X), 0x80 + 0xF0);
    ww(m, a0.wrapping_add(OB_SCREENY), 0x80 + 0x78);
    Ok(())
}

/// `PSB_PrsStart`: a palavra "PRESS START" pisca.
fn psb_prs_start(m: &mut Memoria, a0: u32) -> Result<(), ErroAbertura> {
    animate_sprite(m, a0, ANI_PSBTM)
}

/// Objeto `CreditsText` (`8A`): "SONIC TEAM PRESENTS" e os créditos.
fn credits_text(m: &mut Memoria, a0: u32) {
    let rotina = rb(m, a0.wrapping_add(OB_ROUTINE));
    if rotina == 0 {
        // Cred_Main
        wb(m, a0.wrapping_add(OB_ROUTINE), 2);
        ww(m, a0.wrapping_add(OB_X), 160 + 0x80);
        ww(m, a0.wrapping_add(OB_SCREENY), 112 + 0x80);
        wl(m, a0.wrapping_add(OB_MAP), MAP_CRED);
        ww(m, a0.wrapping_add(OB_GFX), ARTTILE_CREDITS_FONT);
        let n = rw(m, V_CREDITSNUM);
        wb(m, a0.wrapping_add(OB_FRAME), n.to_be_bytes()[1]);
        wb(m, a0.wrapping_add(OB_RENDER), 0);
        wb(m, a0.wrapping_add(OB_PRIORITY), 0);
        if rb(m, V_GAMEMODE) == GM_TITLE_ID {
            ww(m, a0.wrapping_add(OB_GFX), ARTTILE_SONIC_TEAM_FONT);
            wb(m, a0.wrapping_add(OB_FRAME), 0xA);
            if rb(m, F_CREDITSCHEAT) != 0 && rb(m, V_JPADHOLD1) == 0x72 {
                ww(m, V_PALETTE_FADING_LINE_3, 0x0EEE);
                ww(m, V_PALETTE_FADING_LINE_3.wrapping_add(2), 0x0880);
                delete_object(m, a0);
                return;
            }
        }
    }
    // Cred_Display
    display_sprite(m, a0);
}

fn sext8(b: u8) -> i16 {
    i16::from(i8::from_be_bytes([b]))
}

/// `BuildSprites`: monta a tabela de sprites a partir das filas de prioridade.
pub fn build_sprites(m: &mut Memoria) -> Result<(), ErroAbertura> {
    let mut a2 = V_SPRITETABLEBUFFER;
    let mut d5: u8 = 0;
    let mut a4 = V_SPRITEQUEUE;
    for _ in 0..SPRITELAYER_NUM {
        if rw(m, a4) != 0 {
            let mut d6: u32 = 2;
            loop {
                let a0 = ext(rw(m, a4.wrapping_add(d6)));
                if rb(m, a0.wrapping_add(OB_ID)) != 0 {
                    desenhar_objeto(m, a0, &mut a2, &mut d5)?;
                }
                d6 = d6.wrapping_add(2);
                let n = rw(m, a4).wrapping_sub(2);
                ww(m, a4, n);
                if n == 0 {
                    break;
                }
            }
        }
        a4 = a4.wrapping_add(SPRITELAYER_SIZE);
    }
    wb(m, V_SPRITECOUNT, d5);
    if d5 == SPRITES_MAX {
        wb(m, a2.wrapping_sub(5), 0);
    } else {
        wl(m, a2, 0);
    }
    Ok(())
}

fn desenhar_objeto(
    m: &mut Memoria,
    a0: u32,
    a2: &mut u32,
    d5: &mut u8,
) -> Result<(), ErroAbertura> {
    // bclr #7: desmarca "renderizado".
    let render = rb(m, a0.wrapping_add(OB_RENDER)) & 0x7F;
    wb(m, a0.wrapping_add(OB_RENDER), render);
    let d4 = render;
    let cam = d4 & 0x0C;
    let (d2, d3): (i16, i16);
    if cam == 0 {
        d2 = i16::from_be_bytes(rw(m, a0.wrapping_add(OB_SCREENY)).to_be_bytes());
        d3 = i16::from_be_bytes(rw(m, a0.wrapping_add(OB_X)).to_be_bytes());
    } else {
        let a1 = match cam {
            0x04 => V_SCREENPOSX,
            0x08 => V_BGSCREENPOSX,
            _ => V_BG3SCREENPOSX,
        };
        let largura = i16::from(rb(m, a0.wrapping_add(OB_ACTWID)));
        let x = i16::from_be_bytes(rw(m, a0.wrapping_add(OB_X)).to_be_bytes());
        let cx = i16::from_be_bytes(rw(m, a1).to_be_bytes());
        let mut d3v = x.wrapping_sub(cx);
        if d3v.wrapping_add(largura) < 0 {
            return Ok(());
        }
        if d3v.wrapping_sub(largura) >= 320 {
            return Ok(());
        }
        d3v = d3v.wrapping_add(0x80);
        let y = i16::from_be_bytes(rw(m, a0.wrapping_add(OB_Y)).to_be_bytes());
        let cy = i16::from_be_bytes(rw(m, a1.wrapping_add(4)).to_be_bytes());
        let mut d2v = y.wrapping_sub(cy);
        if d4 & 0x10 != 0 {
            let h = i16::from(rb(m, a0.wrapping_add(OB_HEIGHT)));
            if d2v.wrapping_add(h) < 0 || d2v.wrapping_sub(h) >= 224 {
                return Ok(());
            }
            d2v = d2v.wrapping_add(0x80);
        } else {
            d2v = d2v.wrapping_add(0x80);
            // Altura suposta de 32 pontos.
            if !(0x80 - 32..0x80 + 224 + 32).contains(&d2v) {
                return Ok(());
            }
        }
        d2 = d2v;
        d3 = d3v;
    }
    let mut a1 = rl(m, a0.wrapping_add(OB_MAP));
    let mut d1: i32;
    if d4 & 0x20 != 0 {
        d1 = 0;
    } else {
        let f = rb(m, a0.wrapping_add(OB_FRAME)).wrapping_add(rb(m, a0.wrapping_add(OB_FRAME)));
        let off = rom_w(m, a1.wrapping_add(u32::from(f)));
        a1 = a1.wrapping_add(ext(off));
        let n = rom_byte(m, a1);
        a1 = a1.wrapping_add(1);
        d1 = i32::from(n).wrapping_sub(1);
        if d1 < 0 {
            let r = rb(m, a0.wrapping_add(OB_RENDER)) | 0x80;
            wb(m, a0.wrapping_add(OB_RENDER), r);
            return Ok(());
        }
    }
    let a3 = rw(m, a0.wrapping_add(OB_GFX));
    let xflip = d4 & 1 != 0;
    let yflip = d4 & 2 != 0;
    while d1 >= 0 {
        if *d5 == SPRITES_MAX {
            break;
        }
        let y = sext8(rom_byte(m, a1));
        let tam = rom_byte(m, a1.wrapping_add(1));
        let mut d0 = y;
        if yflip {
            let d4b = i16::from((tam << 3) & 0x18).wrapping_add(8);
            d0 = (0i16).wrapping_sub(d0).wrapping_sub(d4b);
        }
        d0 = d0.wrapping_add(d2);
        ww(m, *a2, u16::from_be_bytes(d0.to_be_bytes()));
        *a2 = a2.wrapping_add(2);
        wb(m, *a2, tam);
        *a2 = a2.wrapping_add(1);
        *d5 = d5.wrapping_add(1);
        wb(m, *a2, *d5);
        *a2 = a2.wrapping_add(1);
        let tile = u16::from_be_bytes([
            rom_byte(m, a1.wrapping_add(2)),
            rom_byte(m, a1.wrapping_add(3)),
        ]);
        let mut t = tile.wrapping_add(a3);
        if xflip || yflip {
            t ^= (u16::from(xflip) << 11) | (u16::from(yflip) << 12);
        }
        ww(m, *a2, t);
        *a2 = a2.wrapping_add(2);
        let mut x = sext8(rom_byte(m, a1.wrapping_add(4)));
        if xflip {
            let d4b = i16::from(tam.wrapping_add(tam) & 0x18).wrapping_add(8);
            x = (0i16).wrapping_sub(x).wrapping_sub(d4b);
        }
        x = x.wrapping_add(d3);
        let mut xw = u16::from_be_bytes(x.to_be_bytes()) & 0x1FF;
        if xw == 0 {
            xw = 1;
        }
        ww(m, *a2, xw);
        *a2 = a2.wrapping_add(2);
        a1 = a1.wrapping_add(5);
        d1 = d1.wrapping_sub(1);
    }
    let r = rb(m, a0.wrapping_add(OB_RENDER)) | 0x80;
    wb(m, a0.wrapping_add(OB_RENDER), r);
    Ok(())
}

fn rom_byte(m: &Memoria, a: u32) -> u8 {
    super::boot::rom_b(m, a)
}

#[cfg(test)]
mod testes_mola {
    use super::*;

    fn memoria() -> Memoria {
        let mut rom = vec![0; 0x10000];
        let i = usize::try_from(SPRING_POWERS).unwrap();
        rom[i..i.wrapping_add(4)].copy_from_slice(&[0xF0, 0, 0xF6, 0]);
        let i = usize::try_from(ANI_SPRING).unwrap();
        rom[i..i.wrapping_add(6)].copy_from_slice(&[0, 4, 0, 4, 5, 1]);
        Memoria::nova(rom)
    }

    #[test]
    fn mola_inicial_le_forca_e_configura_tipo() {
        let mut m = memoria();
        let a = V_LVLOBJSPACE;
        wb(&mut m, a + OB_SUBTYPE, 0x12);
        ww(&mut m, a + OB_X, 128);
        mola(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a + OB_ROUTINE), 8);
        assert_eq!(rw(&mut m, a + OB_GFX), 0x2533);
        assert_eq!(rw(&mut m, a + 0x30), 0xF600);
        assert_eq!(rb(&mut m, a + OB_ACTWID), 8);
    }

    #[test]
    fn mola_vertical_lanca_sonic_ao_pousar() {
        let mut m = memoria();
        let a = V_LVLOBJSPACE;
        wb(&mut m, a + OB_ROUTINE, 2);
        wb(&mut m, a + OB_RENDER, 0x84);
        wb(&mut m, a + OB_ACTWID, 16);
        ww(&mut m, a + OB_X, 100);
        ww(&mut m, a + OB_Y, 100);
        ww(&mut m, a + 0x30, 0xF000);
        ww(&mut m, V_PLAYER + OB_X, 100);
        ww(&mut m, V_PLAYER + OB_Y, 73);
        ww(&mut m, V_PLAYER + OB_VELY, 0x100);
        wb(&mut m, V_PLAYER + OB_HEIGHT, 19);
        wb(&mut m, V_PLAYER + OB_STATUS, 2);
        mola(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a + OB_ROUTINE), 4);
        assert_eq!(rw(&mut m, V_PLAYER + OB_VELY), 0xF000);
        assert_eq!(rw(&mut m, V_PLAYER + OB_Y), 80);
        assert_eq!(rb(&mut m, V_PLAYER + OB_STATUS) & 0x0A, 2);
        assert_eq!(rb(&mut m, V_SOUNDQUEUE1), 0xCC);
    }

    #[test]
    fn mola_horizontal_lanca_sonic_para_fora() {
        let mut m = memoria();
        let a = V_LVLOBJSPACE;
        wb(&mut m, a + OB_ROUTINE, 8);
        wb(&mut m, a + OB_RENDER, 0x84);
        wb(&mut m, a + OB_ACTWID, 8);
        ww(&mut m, a + OB_X, 100);
        ww(&mut m, a + OB_Y, 100);
        ww(&mut m, a + 0x30, 0xF000);
        ww(&mut m, V_PLAYER + OB_X, 82);
        ww(&mut m, V_PLAYER + OB_Y, 100);
        ww(&mut m, V_PLAYER + OB_VELX, 0x100);
        wb(&mut m, V_PLAYER + OB_HEIGHT, 19);
        mola(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a + OB_ROUTINE), 10);
        assert_eq!(rw(&mut m, V_PLAYER + OB_X), 73);
        assert_eq!(rw(&mut m, V_PLAYER + OB_VELX), 0x1000);
        assert_eq!(rw(&mut m, V_PLAYER + SON_LOCKTIME), 15);
    }

    #[test]
    fn cenario_inicializa_subtipo_ponte_ghz_e_coloca_na_fila() {
        let mut m = memoria();
        let a = V_LVLOBJSPACE;
        wb(&mut m, a + OB_SUBTYPE, 3);
        ww(&mut m, a + OB_X, 100);
        ww(&mut m, V_SCREENPOSX, 100);
        cenario(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a + OB_ROUTINE), 2);
        assert_eq!(rl(&mut m, a + OB_MAP), 0x7FB2);
        assert_eq!(rw(&mut m, a + OB_GFX), 0x438E);
        assert_eq!(rb(&mut m, a + OB_FRAME), 1);
        assert_eq!(rb(&mut m, a + OB_ACTWID), 16);
        assert_eq!(rb(&mut m, a + OB_PRIORITY), 1);
        assert_eq!(rb(&mut m, a + OB_RENDER), 4);
    }

    #[test]
    fn cenario_fora_da_janela_e_removido() {
        let mut m = memoria();
        let a = V_LVLOBJSPACE;
        wb(&mut m, a + OB_ROUTINE, 2);
        ww(&mut m, a + OB_X, 2000);
        ww(&mut m, V_SCREENPOSX, 100);
        cenario(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a + OB_ID), 0);
    }

    #[test]
    fn water_sound_emite_som_a_cada_64_quadros_e_deleta_fora_da_tela() {
        let mut m = memoria();
        let a = V_LVLOBJSPACE;
        wb(&mut m, a + OB_ID, ID_WATER_SOUND);
        ww(&mut m, a + OB_X, 100);
        ww(&mut m, V_SCREENPOSX, 100);
        wb(&mut m, V_VBLANK_BYTE, 0); // multiplo de 64

        water_sound(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a + OB_ROUTINE), 2);
        assert_eq!(rb(&mut m, V_SOUNDQUEUE1), 0xD0);

        // Nao emite som quando nao for multiplo de 64
        wb(&mut m, V_SOUNDQUEUE1, 0);
        wb(&mut m, V_VBLANK_BYTE, 1);
        water_sound(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, V_SOUNDQUEUE1), 0);

        // Fora da janela: deleta
        ww(&mut m, a + OB_X, 2000);
        water_sound(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a + OB_ID), 0);
    }

    #[test]
    fn purple_rock_inicializa_e_e_solida() {
        let mut m = memoria();
        let a = V_LVLOBJSPACE;
        wb(&mut m, a + OB_ID, ID_PURPLE_ROCK);
        ww(&mut m, a + OB_X, 200);
        ww(&mut m, a + OB_Y, 200);
        ww(&mut m, V_SCREENPOSX, 200);

        purple_rock(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a + OB_ROUTINE), 2);
        assert_eq!(rl(&mut m, a + OB_MAP), MAP_PROCK);
        assert_eq!(rw(&mut m, a + OB_GFX), ART_TILE_PURPLE_ROCK);
        assert_eq!(rb(&mut m, a + OB_ACTWID), 19);
        assert_eq!(rb(&mut m, a + OB_PRIORITY), 4);

        // Fora da janela: deleta
        ww(&mut m, a + OB_X, 2000);
        purple_rock(&mut m, a).unwrap();
        assert_eq!(rb(&mut m, a + OB_ID), 0);
    }
}
