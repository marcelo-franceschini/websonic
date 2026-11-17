use sonic1_console::Console;

#[test]
#[ignore = "requer SONIC1_ROM"]
fn simular_partida_ghz1() {
    let caminho = std::env::var_os("SONIC1_ROM").expect("SONIC1_ROM");
    let rom = std::fs::read(caminho).expect("leitura da ROM");
    let mut console = Console::novo(rom);

    let entradas_txt = std::fs::read_to_string("/ssd400/Repos/websonic/corpus/partida-ghz1.s1in")
        .or_else(|_| std::fs::read_to_string("../../corpus/partida-ghz1.s1in"))
        .expect("partida-ghz1");
    let entradas: Vec<u8> = entradas_txt
        .lines()
        .filter_map(|l| {
            let s = l.trim();
            if s.is_empty() || s.starts_with('#') {
                None
            } else {
                Some(u8::from_str_radix(s, 16).expect("hex"))
            }
        })
        .collect();

    assert_eq!(entradas.len(), 8309);

    let mut gamemode_anterior = 0xFF;
    let mut sonic_x_max = 0u16;

    for (quadro, &botoes) in entradas.iter().enumerate() {
        console.definir_botoes(botoes);
        console.simular_quadro();

        if let Some(err) = &console.ab.erro {
            panic!("erro no console no quadro {quadro}: {err:?}");
        }

        let ram = console.ab.mem.ram();
        let gamemode = ram[0xF600];
        if gamemode != gamemode_anterior {
            gamemode_anterior = gamemode;
        }

        if gamemode == 0x0C {
            let x = u16::from_be_bytes([ram[0xD008], ram[0xD009]]);
            if x > sonic_x_max {
                sonic_x_max = x;
            }
        }
    }

    assert!(sonic_x_max > 4500, "Sonic deve ter percorrido a fase ate o final da Green Hill Zone Act 1 (x={sonic_x_max})");
}

#[test]
#[ignore = "requer SONIC1_ROM"]
fn inspecionar_gameplay_ghz1_manual() {
    let caminho = std::env::var_os("SONIC1_ROM").expect("SONIC1_ROM");
    let rom = std::fs::read(caminho).expect("leitura da ROM");
    let mut console = Console::novo(rom);

    let mut quadro_gameplay_iniciou = None;
    let mut x_maximo = 0u16;

    for quadro in 0..2000 {
        let ram = console.ab.mem.ram();
        let gamemode = ram[0xF600];

        // Segura Start na tela de titulo (0x04) para iniciar
        if gamemode == 0x04 {
            console.definir_botoes(0x80);
        } else if gamemode == 0x0C && quadro > 450 {
            if quadro_gameplay_iniciou.is_none() {
                println!("GAMEPLAY REALMENTE INICIOU no quadro {quadro}!");
                quadro_gameplay_iniciou = Some(quadro);
            }

            let rel = quadro - quadro_gameplay_iniciou.unwrap();
            let mut botoes = 0x08; // Segura DIREITA

            // Sonic alcanca x=760 por volta do quadro rel=190-210. Pula sobre o obstaculo!
            if (190..=205).contains(&rel) {
                botoes |= 0x20; // Pulo (C)
            }

            // Sonic alcanca x=1150 por volta de rel=260-275. Pula sobre o segundo obstaculo!
            if (260..=275).contains(&rel) {
                botoes |= 0x20; // Pulo (C)
            }

            // Pula nos quadros 400-415
            if (400..=415).contains(&rel) {
                botoes |= 0x20; // Pulo (C)
            }

            console.definir_botoes(botoes);
        } else {
            console.definir_botoes(0);
        }

        console.simular_quadro();
        if let Some(err) = &console.ab.erro {
            panic!("erro no console no quadro {quadro}: {err:?}");
        }

        if let Some(inicio) = quadro_gameplay_iniciou {
            let rel = quadro - inicio;
            let ram = console.ab.mem.ram();
            let x = u16::from_be_bytes([ram[0xD008], ram[0xD009]]);
            let y = u16::from_be_bytes([ram[0xD00C], ram[0xD00D]]);
            let vx = i16::from_be_bytes([ram[0xD010], ram[0xD011]]);
            let vy = i16::from_be_bytes([ram[0xD012], ram[0xD013]]);
            let inercia = i16::from_be_bytes([ram[0xD014], ram[0xD015]]);
            let angulo = ram[0xD026];
            let status = ram[0xD022];
            let anim = ram[0xD01C];
            let rotina = ram[0xD024];
            let lock = ram[0xF7DA]; // f_lockctrl
            if x > x_maximo {
                x_maximo = x;
            }
            if rel % 25 == 0 && rel <= 600 {
                println!("rel={rel:03} x={x:04} y={y:04} vx={vx:5} vy={vy:5} in={inercia:5} ang={angulo:02X} st={status:#04x} an={anim:02} rot={rotina} lk={lock}");
            }
        }
    }

    println!("X maximo alcancado: {x_maximo}");
    assert!(x_maximo > 1500, "Sonic deve ter ultrapassado x=1500 com os pulos nos obstaculos (x={x_maximo})");
}
