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
- **Correcao (4.6):** a primeira versao deste arquivo dizia que a CRAM no meio da linha nao re-renderiza. Estava errado. Ao sair da FIFO, uma escrita na CRAM chama `maybe_render_partial_line` (re-renderiza o resto da linha) e tambem marca um "ponto da CRAM" (`vdp/cramdots.rs`). A escrita na VSRAM faz o mesmo so quando a rolagem vertical e por duas celulas.
- A VDP simula a FIFO e os slots de acesso (`handle_access_slot`, `pop_fifo`), e o DMA de memoria para VRAM le a memoria durante o tempo (`progress_memory_to_vram_dma`). Consequencia para o renderizador: uma escrita da CPU chega a VRAM ou CRAM alguns slots depois do evento do trace.

Consequencia de projeto: para igualar o oraculo o renderizador precisa de (1) a mesma agenda de eventos por linha, (2) a conversao de `coluna` (ciclos do relogio mestre desde o inicio da linha) para a posicao H interna, (3) os mesmos limites de sprites. A 4.6 trata (1) e (2).

## Ferramentas de lint do crate `video`

- `crates/video` usa a mesma tabela de lints do nucleo. Prova (06/11/2026, copia do repositorio com `docs/pedidos/ci-video.patch` aplicado, alvo nativo): `sh .github/checa-core.sh` imprime `checa-core: OK`; `sh .github/lints-core.sh` passa no alvo nativo. Violacoes plantadas em `crates/video/src/lib.rs` com os mesmos `-F` do script: float (3 erros), soma sem checagem (3), cast que trunca (3), `HashMap` (4). O `#[allow]` passa pelo clippy e e barrado pelo `checa-core.sh` ("atributo allow ou expect no nucleo").
- Os alvos wasm do `lints-core.sh` nao rodaram no host (falta o alvo instalado). Rodam na imagem.
- Cuidado ao escrever o crate: os testes tambem sao examinados (`--all-targets`), entao nada de `+`, `-` ou `*` crus nem de casts que cortam nos testes, e nenhum numero decimal em comentario no fim de linha de codigo.

## Modelo de tempo do renderizador (4.6), como o oraculo

Codigo: `crates/video/src/tempo.rs`, `portas.rs` e `renderizador.rs`. Prova: testes do crate `sonic1-video` e o vetor tirado do oraculo (abaixo).

- **Agenda por linha (40 colunas, H interno):** VBlank em `0x002`, leitura da rolagem horizontal em `0x002`, renderizar a linha em `0x01A`, buscar atributos de sprites em `0x142`, inicio do HBlank em `0x14A`, interrupcao de linha em `0x14A` e travar registradores em `0x169`. Um evento dispara antes de uma escrita no ponto `P` se o seu H for menor que `P`. A coluna do evento do trace (ciclos mestres desde o inicio da linha) vira ponto por `mclk_para_ponto` (mclk/8 ate o H `0x16C`).
- **Quando a escrita vale:** o evento do trace tem a linha e a coluna do instante em que a VDP aplicou a escrita, depois de a instrucao andar (o `mclk` do evento e o do inicio da instrucao, a coluna e o do fim). Uma escrita na porta de dados entra na FIFO de 4 entradas com latencia de 3 slots e so chega a memoria ao sair dela.
- **Slots:** a cada ponto par ha um slot. Com imagem ativa so servem a FIFO os slots externos (6, 14, 22, 38, 46, 54, 70, 78, 86, 102, 110, 118, 134, 142, 150, 165, 166 e 190). Em VBlank ou com a imagem desligada servem todos menos os de atualizacao da memoria (26, 58, 90, 122, 154 e 204). Escrita de VRAM em palavra leva dois slots para sair.
- **DMA de memoria para VDP:** um slot par sem atualizacao le uma palavra e a poe na FIFO; a latencia da primeira palavra e 8 slots (5 para a VSRAM). As escritas do 68000 ficam presas ate o DMA acabar.
- **Ligar a imagem com a FIFO nao vazia** adia a ligacao ate a FIFO esvaziar.
- **Pontos da CRAM:** a escrita que sai da FIFO marca um ponto na imagem na coluna `ponto - 31`, desenhado no inicio da renderizacao da linha seguinte. Com a imagem desligada e a cor de fundo mudando, os 3 pontos seguintes (ou ate o fim da linha) tambem recebem a cor.
- **Re-renderizacao parcial** (registradores 1, 2, 3, 4 e 7, CRAM, e VSRAM com rolagem por duas celulas): o resto da linha e desenhado de novo a partir do ponto do evento, com as regras do oraculo (esperar a proxima coluna de 16 pontos para os planos, nao re-renderizar nos ultimos 16 pontos).

## Prova com a ROM sintetica de video (06/11/2026)

ROM `programa_video` (escrita por nos): liga o display, grava a cor 0 da CRAM no meio da linha 0 e faz um DMA de 4 palavras. Oraculo com os patches P6 e P7, 40 quadros:
- `cargo xtask video-estado`: `IGUAL em 40 quadros` (hashes de VRAM, CRAM e VSRAM por quadro).
- `cargo xtask video-comparar`: 39 de 40 quadros iguais. Os quadros 1 a 39 sao azuis por inteiro nos dois lados. A linha 0 do quadro 0 e igual ponto a ponto: azul a partir do ponto 130, ponto da CRAM azul em x = 125, quatro pontos pretos entre os dois (teste `vetor_do_oraculo_para_cram_no_meio_da_linha`).
- **Esquisitice do oraculo OQ1 (quadro 0):** no primeiro quadro o jgenesis ainda nao travou a resolucao horizontal e desenha com largura de 256 pontos por linha. A imagem entregue do quadro 0 fica embaralhada ao ser lida com largura 320 (aqui, pretos a partir da linha 179). Nao e um defeito do renderizador. Com a tela preta no quadro 0 (caso do Sonic) os dois lados coincidem. Se um dia o quadro 0 tiver imagem, o veredito `DIFERE` do quadro 0 e esperado e fica registrado como OQ1.
- A imagem entregue pelo oraculo (`vdp.frame_buffer()`) tem tamanho fixo maximo; so os primeiros largura x altura pontos valem. A suposicao de que a linha tem passo igual a largura so foi provada com imagem uniforme; a primeira comparacao com a ROM real (planos e sprites) confirma ou nega.

## Desempenho (4.9, medida na maquina de desenvolvimento)

`cargo test -p sonic1-video --release --test desempenho -- --ignored --nocapture`: cena aleatoria com 24 sprites encadeados, rolagem por linha e 64 trocas de CRAM no meio do quadro. Mediana de 200 quadros entre 0,95 ms e 1,10 ms no nativo (orcamento 1,0 ms). O custo vem do desenho dos planos e dos sprites por linha. Otimizacoes ja feitas: reuso dos planos quando so a CRAM muda, tabela de 64 cores por linha, caminho rapido da celula inteira. Wasm: nao medido no host (falta o alvo); roda na imagem.

## Decisoes da etapa

- O renderizador copia o comportamento do oraculo, inclusive o que o proprio codigo dele marca como duvidoso. A diferenca para o hardware real fica para o desempate (plano da Etapa 4, secao 5).
- Janela, sombra e realce, entrelacamento, H32, V30, copia de VRAM por DMA, VRAM de 128 KiB e coluna da esquerda apagada: so contam um aviso.
- O quadro e fechado na linha 232, como o oraculo.
