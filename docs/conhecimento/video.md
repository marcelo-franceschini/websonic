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

## Rodada do host de 06/11/2026 (Marcelo, com a ROM real)

- Oraculo com P6 e P7 aplicados: `Testes do oraculo: OK`, patches `9e065ee5`. Hooks 14 de 14. O `checa-core`, o `lints-core` e os testes wasm **nao rodaram**: a imagem `websonic-dev:ci` nao existe no host (precisa `docker build -t websonic-dev:ci -f .devcontainer/Dockerfile .`, como o CI).
- O hash do trace do `boot-demos` mudou de `d0acd7fe` para `ee5d2725`. Explicacao: o cabecalho do trace carrega o hash dos patches do oraculo (`ID_ORACULO`), que mudou com o P7. O conteudo dos eventos nao foi comparado quadro a quadro com o trace antigo (foi sobrescrito). `determinismo: PASSOU`.
- **`video-comparar`: IGUAL em 7200 quadros (`boot-demos`) e em 1500 quadros (`lz-agua`).** Isto vale para a imagem de 320x224 em 9 bits da Sega, do titulo e das demos de GHZ, MZ e do inicio de SYZ. O criterio da etapa esta cumprido para esse corpus, mas ele nao tem interrupcao de linha (ver abaixo).
- `lz-agua` (1a versao): o perfil mostrou zona 0 (GHZ), nao a Labyrinth. O codigo do seletor foi digitado cedo demais, aos quadros 420 a 435 (o titulo comeca perto do quadro 360 e demora a ler o controle), e o A + Start aos 450 iniciou a fase 1 normal. Corrigido: sequencia a partir do quadro 560 e 2000 quadros.
- `video-estado`: CRAM e VSRAM iguais em todos os quadros, mas a VRAM difere em 1129 dos 7200 quadros (a primeira no 354) e as imagens batem. Hipotese: o oraculo fecha o quadro no fim da instrucao que cruza para a linha 232, e nao no comeco dela; com DMA em andamento a VRAM nesse instante tem palavras a mais ou a menos. Correcao feita: `fechar_quadro_em(coluna)` com a coluna `mclk_fim - inicio_da_linha_232`. Ainda nao verificado com a ROM.
- `avisos_fora_do_inventario` deu 2406 e 440. O detalhe por campo agora e impresso (`avisos=...`). Hipotese: `dma_sem_dados`, que conta quadros que terminam com DMA parado sem palavras, e nao um recurso fora do inventario de fato.
- Desempenho nativo: mediana 982 us, maximo 1016 us (orcamento 1000 us).

## Rodada 2 do host (06/11/2026): criterio da etapa cumprido para o corpus atual

Comando: `sh scripts/etapa4-host.sh <ROM REV01>` depois de `docker build -t websonic-dev:ci -f .devcontainer/Dockerfile .`. Resultados colados pelo Marcelo:

| Prova | Resultado |
| --- | --- |
| `checa-core.sh` na imagem (com `ci-video.patch`) | `checa-core: OK` |
| `lints-core.sh` na imagem (nativo e os dois alvos wasm) | `lints do nucleo: OK`, com `sonic1-core` e `sonic1-video` |
| `cargo test --target wasm32-wasip1 -p sonic1-video` | 44 testes da biblioteca, 1 do `desempenho` (sem medir) e 4 da referencia, todos ok |
| Perfil de `lz-agua` (2000 quadros) | modo 4 (titulo) no quadro 360, modo 140 no 780, modo 12 (fase) com zona 1 (Labyrinth) e ato 0 a partir do 960 |
| Inventario do corpus inteiro | interrupcao de **nivel 4: 1530 vezes**, primeiro caso `lz-agua`, quadro 771 (a interrupcao de linha da agua), alem de nivel 6: 8068 |
| `video-estado` `boot-demos` (7200) e `lz-agua` (2000) | VRAM, CRAM e VSRAM iguais ao trace em todos os quadros, `avisos_fora_do_inventario=0` |
| `video-comparar` `boot-demos` | **`IGUAL em 7200 quadros`** (imagem de 320x224 em 9 bits, hash por quadro) |
| `video-comparar` `lz-agua` | **`IGUAL em 2000 quadros`**, incluindo a fase da Labyrinth com a interrupcao de linha |
| `determinismo` dos dois casos | `PASSOU` (duas execucoes e retomada de estado) |
| Desempenho nativo (`WEBSONIC_BENCH=1`) | mediana 928 us, maximo 1940 us, orcamento 1000 us: dentro (variacao entre rodadas: 928 a 1100 us) |
| Manifesto | assinado, `assinatura: OK`, `referencias: OK`, `testes: OK (0 nao assinados)` |

- As duas correcoes da rodada 1 funcionaram: fechar o quadro na coluna em que o oraculo o fecha (VRAM igual em 7200 de 7200 quadros, antes 6071) e o aviso de DMA que atravessa o quadro deixou de contar como recurso fora do inventario (`dma_sem_dados` agora e so informativo: 6269 quadros do `boot-demos` terminam com um DMA esperando palavras do proximo quadro).
- Hash do trace atual: `lz-agua` `099144a211c1ec07c92d73c346bab97cade04dc8` (2000 quadros) e `boot-demos` `ee5d2725afdf10a996399208a30ba140e799ff30` (7200 quadros). O `boot-demos` mudou de `d0acd7fe` porque o cabecalho carrega o hash dos patches do oraculo.
- **O que a rodada nao prova:** (1) a Labyrinth so cobre os primeiros 2000 quadros (inicio da fase, nao a imersao completa na agua nem os outros atos); (2) special stage, creditos, finais, game over e as outras zonas nao estao no corpus; (3) o desempate de cor entre os dois oraculos nao foi exercitado porque nao houve divergencia; (4) a medida em wasm nao foi feita (o teste so mede com `WEBSONIC_BENCH=1` e a variavel nao foi passada ao wasmtime); (5) a OQ1 (quadro 0 do oraculo com largura 256) nao apareceu porque o quadro 0 e preto.

## Medições de HBlank da Labyrinth (15/11/2026)

Um trace de 1100 quadros de `lz-agua` contém 330 interrupções de nível 4. Foram identificadas 328 transferências completas de paleta. Em 326 delas, a interrupção foi aceita na linha anterior à programada no registrador 10. O comando de escrita na CRAM começou 994 ciclos mestres após a aceitação em 319 ocorrências e 1008 em 9; duas ocorreram aproximadamente 10300 ciclos depois. Essas exceções não sustentam uma latência fixa universal.

Nas 169 transferências com linha programada abaixo de 220, a sequência de escritas observada abrangeu três linhas. Em 158 das 159 restantes abrangeu uma linha; a exceção abrangeu quatro. Os intervalos entre pares de palavras variam com as esperas, inclusive durante a área visível. Isso impede representar a transferência inteira como instantânea ou inferir a espera da CPU apenas pelo número de palavras.

`video-comparar` reproduziu as imagens dos 1100 quadros usando linha e coluna de cada escrita do trace. Isso valida a reprodução do vídeo com tempos fornecidos, não a previsão dos tempos de CPU ou a integração nominal. O relatório mostrou 618 avisos informativos `dma_sem_dados` e nenhum aviso fora do inventário.

## Medição de desempenho em WebAssembly e WebGL2 (17/11/2026)

Com a remoção do compositor de candidatos para GPU e a restauração da renderização em software pela CPU, a imagem de 320x224 pontos em 9 bits é enviada diretamente ao WebGL2 via textura `R16UI`.
- Transferência por quadro: 143.360 bytes constantes (71.680 palavras de 16 bits).
- Chamadas `texImage2D`: zero após a inicialização.
- Tempo de quadro em Wasm (`partida-ghz1`, comando `SONIC1_ROM=... node --test site/tests/desempenho-wasm.test.mjs`):
  - Média: 6,70 ms por quadro (meta <= 8,0 ms; antes da remediação: 15,46 ms).
  - Percentil 95: 11,56 ms (meta <= 13,0 ms; antes da remediação: 23,50 ms).
  - Quadros acima de 16,67 ms: 0,6% (meta <= 2,0%; antes da remediação: 27,2%).
- Desempenho nativo do renderizador (`WEBSONIC_BENCH=1 cargo test -p sonic1-video --release --test desempenho`): mediana de 2569 us.

## Otimizações de renderização na rodada 2 (19/11/2026)

- **Sinalizadores de pontos da CRAM (item V1):** em `crates/video/src/renderizador.rs`, as rotinas `trocar_pontos_da_cram` e `aplicar_pontos_da_linha_anterior` varriam 347 entradas vazias duas a tres vezes por linha ativa. A introducao dos sinalizadores booleanos `marcados_atual` e `marcados_anterior` evita essas varreduras quando nao ha pontos registrados na linha. O ganho medido em Wasm foi de cerca de 0,3 ms por quadro (razao de 0,94 a 0,95 contra o S1 no nucleo P), validado pelas guardas de fidelidade de imagem e testes unitarios do crate `sonic1-video`.
