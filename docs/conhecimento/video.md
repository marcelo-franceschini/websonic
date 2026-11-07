# Video (Etapa 4)

## Escopo: o que o Sonic 1 usa da VDP e o que fica fora

Vem de `docs/inventario-hardware.md` (`cargo xtask inventario`, caso `boot-demos`, 7200 quadros, rodado pelo Marcelo em 06/11/2026). Vale so para esse corpus. A tabela com a decisao de cada recurso esta em `docs/plano-etapa-4.md`, secao 2. Resumo:

- Dentro: cor de 9 bits (reg 0 = `0x04`), H40 (reg 12 = `0x81`), 28 linhas de celulas (reg 1 sem o bit 3), rolagem por linha e de plano inteiro (reg 11 = `0x03` e `0x00`), plano 64x32 (reg 16 = `0x01`), sprites, DMA de 68000 para VDP e preenchimento de VRAM, interrupcao de linha (por requisito, ainda sem prova no corpus).
- Fora, com aviso: janela (regs 17 e 18 sempre 0), sombra e realce, entrelacamento, H32, V30, copia de VRAM por DMA.
- Fontes publicas dos limites de sprite (consultadas em 06/11/2026): Sega Retro, pagina Sega_Mega_Drive/Sprites (20 por linha e 80 por quadro em H40, mascara por x = 0); ROM de teste "Sprite Masking and Overflow Test ROM" (mesmos limites e teto de 320 pontos por linha em H40); Plutiedev, pagina vdp-registers (bits dos registradores 0, 1, 11 e 12; nao documenta limites nem mascara). Um post de forum cita 32 por linha; nao adotado.

## Regra de renderizacao do oraculo (jgenesis `386db855`)

Arquivos: `backend/genesis-components/src/vdp.rs` e `vdp/render.rs`. Citacao so de nomes, sem copiar codigo. O VDP do jgenesis e guiado por eventos em posicoes H internas de cada linha (funcao `vdp_event_times`, tratados em `handle_vdp_event`):

| Evento | O que faz |
| --- | --- |
| `VInterrupt` | na linha seguinte a ultima ativa, marca a interrupcao de VBlank e guarda a resolucao horizontal do quadro |
| `ReadHScroll` | le da VRAM a rolagem horizontal da linha (`read_h_scroll`) e guarda em `latched_h_scroll` |
| `RenderLine` | varre sprites uma linha adiante (`scan_sprites_one_line_ahead`) e **renderiza a linha inteira** (`render_scanline(linha, 0)`) |
| `FetchSpriteAttributes` | segunda fase dos sprites (`fetch_sprite_attributes`) |
| `HBlankStart` | sinaliza o inicio do HBlank e do VBlank |
| `HInterrupt` | trava a tabela de sprites em cache e decrementa o contador da interrupcao de linha |
| `LatchRegisters` | copia os registradores para `latched_registers` e trava a rolagem vertical de tela inteira |

Regras de escrita (funcao `update_latched_registers_if_necessary`, comentarios do proprio codigo):
- Registradores 2, 3 e 4 (enderecos das tabelas de nomes), 1 (display ligado) e 7 (cor de fundo) valem **na hora**. Se a escrita cair na area ativa, a linha e **re-renderizada a partir do primeiro ponto afetado** (`maybe_render_partial_line`). Os planos A e B esperam a proxima coluna de 16 pontos. Ha um `+16` comentado como possivel erro.
- Os outros registradores so valem na proxima linha (travados em `LatchRegisters`).
- `render_scanline` **nao re-renderiza** se o ponto inicial estiver nos ultimos 16 pontos da area ativa.
- Escritas na CRAM no meio da linha **nao re-renderizam**: a linha ja foi desenhada com a CRAM do instante de `RenderLine`. O efeito no meio da linha aparece pelos "pontos da CRAM" (`vdp/cramdots.rs`), que o oraculo trata como camada a parte (`CramDotBuffer`, `check_for_dot`).
- A VDP simula a FIFO e os slots de acesso (`handle_access_slot`, `pop_fifo`), e o DMA de memoria para VRAM le a memoria durante o tempo (`progress_memory_to_vram_dma`). Consequencia para o renderizador: uma escrita da CPU chega a VRAM ou CRAM alguns slots depois do evento do trace.

Consequencia de projeto: para igualar o oraculo o renderizador precisa de (1) a mesma agenda de eventos por linha, (2) a conversao de `coluna` (ciclos do relogio mestre desde o inicio da linha) para a posicao H interna, (3) os mesmos limites de sprites. A 4.6 trata (1) e (2).

## Ferramentas de lint do crate `video`

- `crates/video` usa a mesma tabela de lints do nucleo. Prova (06/11/2026, copia do repositorio com `docs/pedidos/ci-video.patch` aplicado, alvo nativo): `sh .github/checa-core.sh` imprime `checa-core: OK`; `sh .github/lints-core.sh` passa no alvo nativo. Violacoes plantadas em `crates/video/src/lib.rs` com os mesmos `-F` do script: float (3 erros), soma sem checagem (3), cast que trunca (3), `HashMap` (4). O `#[allow]` passa pelo clippy e e barrado pelo `checa-core.sh` ("atributo allow ou expect no nucleo").
- Os alvos wasm do `lints-core.sh` nao rodaram no host (falta o alvo instalado). Rodam na imagem.
- Cuidado ao escrever o crate: os testes tambem sao examinados (`--all-targets`), entao nada de `+`, `-` ou `*` crus nem de casts que cortam nos testes, e nenhum numero decimal em comentario no fim de linha de codigo.
