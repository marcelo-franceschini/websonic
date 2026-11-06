# Oráculo (Etapa 1)

Registro do oráculo, do formato do trace e do mapa de rótulos. Cada entrada cita o teste ou o comando que a sustenta. Nada aqui tem dado da ROM.

## Veredito de viabilidade do jgenesis (sub-etapa 1.1): GO com ressalvas

Commit fixado: `386db855ec3aaa307df3321bb85df8b5cc20ac95` (workspace 0.14.1, GPLv3). A saída é uma **versão modificada** do jgenesis, só com código de observação (16 arquivos, 440 linhas acrescentadas e 5 trocadas, sem Cargo.lock). Nenhuma lógica de emulação foi alterada. Não houve troca pelo segundo oráculo.

| # | Verificação | Resultado | Evidência |
| --- | --- | --- | --- |
| V1 | Compila com o Rust 1.99.1 | passa | `cargo build -p genesis-core` (host, Rust 1.99.1 igual ao da imagem). Domínios usados: `github.com` (clone), `index.crates.io` e `static.crates.io`. Nenhum domínio novo. Não foi repetido dentro do contêiner |
| V2 | Sem GUI | passa | `cargo tree -p genesis-core` não cita `wgpu`, `sdl3`, `egui` nem `winit` |
| V3 | Roda sem tela | passa | `Renderer`, `AudioOutput`, `InputPoller` e `SaveWriter` nulos em `oracle/runner/src/oraculo.rs`. `sh oracle/scripts/testa.sh`: 13 testes passam |
| V4 | Hooks de observação | passa | 68000: `branch`, `bsr`, `dbcc`, `jmp`, `jsr`, `ret`, `rte`, exceção e interrupção. Z80: todas as instruções de `jump.rs` e a rotina de interrupção. Patch em `oracle/patches/` |
| V5 | Granularidade do ciclo | **ressalva** | O `m68000-emu` devolve os ciclos da instrução inteira (`execute_instruction`). Os acessos dentro da instrução não têm tempo próprio (comentário `TODO` em `timing.rs`: refresh e DMA são aproximados). O Z80 roda por "alcance" depois de cada instrução do 68000. Ver Limites |
| V6 | O 68000 alcança o YM2612 | passa | Escrita em `$A04000` com BUSACK vai para `write_memory` do Z80 (`bus.rs`, bloco `0xA00000..=0xA0FFFF`). Teste `z80_barramento_ym_dac_e_esperas`: 2 escritas do YM com origem 68000. Um resumo anterior da fonte dizia o contrário e estava errado |
| V7 | Determinismo | passa nos casos sintéticos | RAM, VRAM, CRAM, VSRAM e RAM do Z80 começam em zero. Teste `duas_execucoes_geram_o_mesmo_trace` (3 ROMs, 6 quadros, bytes iguais). Com a ROM real: falta a prova do Marcelo (1.9) |
| V8 | Savestate | passa | Testes `estado_salvo_e_recarregado_gera_o_mesmo_trace` e `estado_salvo_no_meio_do_quadro_tambem_volta`. O estado não leva a ROM |
| V9 | Registradores editáveis | passa | `set_data_registers`, `set_address_registers`, `set_status_register` e `set_pc` já são públicos. Teste `rotina_isolada_a_partir_de_um_estado` |
| V10 | Hooks e eventos do Z80 | passa | Eventos `desvio80`, `int80`, e barramento. Testes do programa 3 |

### Limites e ressalvas

- **Tempo dentro da instrução (V5).** O `mclk` de um evento do 68000 é o do **início da instrução** que o causou. O de um evento do Z80 é o relógio do Z80 no início da instrução dele. Uma escrita no meio de um `MOVE.L` não tem ciclo próprio. A VDP aplica as escritas do 68000 depois da instrução, e o evento leva a linha e a coluna da VDP nesse momento. Isso limita o "ciclo de cada escrita de som" do nível 2. A etapa 15 e o segundo oráculo (etapa 2) dizem se a precisão basta.
- **Estado inicial.** O oráculo começa com todas as memórias em zero. O console real é indefinido. Se o jogo depender disso, a etapa 2 aparece com divergência contra o segundo oráculo.
- **DMA.** Não há evento próprio de DMA. As escritas que o disparam aparecem como escritas de controle da VDP.
- **Configuração.** `config_console()` força região Japão e NTSC, controle de 3 botões na porta 1 e nenhum na porta 2. O resto é o padrão do jgenesis. O hash da configuração (texto Debug) vai no cabeçalho do trace.
- **WebAssembly.** `cargo test --target wasm32-wasip1 -p sonic1-fidelidade` não foi rodado: o host não tem o alvo nem o `wasmtime`.

## Como o oráculo é montado

- `oracle/jgenesis-commit.txt`: commit fixado.
- `oracle/patches/*.patch` e `oracle/patches/SHA1`: os patches e o hash do diff contra o commit fixado. `oracle/scripts/prepara.sh` clona em `ORACLE_DIR` (`/mnt/trabalho/oracle` no contêiner, `$SONIC1_EXT_DIR/trabalho/oracle` no host), aplica os patches e recusa se o hash do resultado diferir.
- `oracle/runner`: o executável e a biblioteca. É a raiz do próprio workspace, separado do workspace do jogo. O link `target/oracle-fonte` aponta para o clone. Fica em `target/`, que o `.gitignore` ignora e a IA pode gravar mesmo com `oracle/` congelado.
- `oracle/scripts/gera-patches.sh`: regenera os patches depois de editar o clone na branch `websonic-oracle`.
- O oráculo nunca entra no build do site nem do `core`. O crate `fidelidade` é MIT e não depende do oráculo.

## O que o oráculo faz (sub-etapas 1.3 a 1.7)

Todos os testes usam ROMs sintéticas escritas por nós (`oracle/runner/src/rom_sintetica.rs`). Rodar: `sh oracle/scripts/testa.sh` (prepara, `cargo fmt --check`, clippy com `-D warnings` e `cargo test`). Resultado em 06/11/2026: 13 testes passam.

| Exigência | Teste |
| --- | --- |
| Lê a memória do 68000, do Z80 e da VDP | `ram68k`, `ram_z80`, `vram`, `cram`, `vsram` e os 9 hashes por quadro (`entrada_injetada_chega_a_ram`, `duas_execucoes_geram_o_mesmo_trace`) |
| Injeta entradas | `entrada_injetada_chega_a_ram`: Cima apertado a partir do quadro 3 muda o bit lido em `$A10003` |
| Salva e carrega estados | os dois testes `estado_salvo_*` |
| Para em endereços, rótulos, quadros e relógio | `para_em_endereco_antes_de_executar_a_instrucao`, `para_no_inicio_de_um_quadro_e_em_um_relogio`, `para_em_rotulo_resolvido_pelo_mapa` |
| Ponto de cada interrupção (endereço e ciclo) | `interrupcao_de_vblank_e_registrada_com_endereco_e_ciclo`: nível 6, PC interrompido `$210`, tratador `$400`, contador do tratador igual ao número de eventos |
| Desvios do 68000 e do Z80, tomados e não tomados | `desvios_do_68000_tomados_e_nao_tomados`, `rotina_isolada_a_partir_de_um_estado` (não tomado e RTS), `z80_barramento_ym_dac_e_esperas` (JR do Z80) |
| Escritas de vídeo com a linha | `escrita_na_vdp_e_registrada_com_a_linha` |
| Escritas de som e do DAC | `z80_barramento_ym_dac_e_esperas`: 4 escritas do Z80 na ordem certa, a de dado em `$2A` marcada como DAC, mais 2 escritas do 68000 |
| Esperas pelo Z80 | mesmo teste: BUSREQ, leitura do BUSACK, espera acumulada (causa 1) e consumida |
| Rotina isolada a partir de um estado | `rotina_isolada_a_partir_de_um_estado`: estado salvo, registradores e RAM definidos, retorno por endereço sentinela, duas chamadas iguais |
| Duas execuções, o mesmo trace | `duas_execucoes_geram_o_mesmo_trace` e `sh oracle/scripts/roda-oraculo.sh` com ROM sintética: `cmp` idêntico e `oraculo comparar` diz `IGUAIS` |
| Comparador acusa divergência plantada | `comparador_acusa_entrada_diferente_no_quadro_certo`: quadro 3, categoria `ram68k` |

Detalhes de uso:
- `chamar_rotina` empilha o endereço sentinela, mascara as interrupções (SR com máscara 7, desligável) e para quando o PC chega à sentinela. Os `pokes` fazem parte do estado de partida e não contam como mudança.
- A parada em endereço exige que o PC **chegue** ao endereço. O primeiro passo sempre executa.
- A linha de comando só imprime hashes, contagens e nomes: `oraculo rodar`, `oraculo comparar`, `oraculo rom-sintetica`.

## Mapa de rótulos e ROM montada (sub-etapa 1.8): ferramentas prontas, prova do Marcelo pendente

- `oracle/scripts/monta-disasm.sh` (host, pelo Marcelo). Modo `reconhecer` lista os arquivos que o build cria, só nome e tamanho. Modo `montar ROM LISTAGEM [BASE]` confere o SHA-1 e o MD5 contra `refs/rom-hashes.txt` e imprime `IDENTICA` ou `DIFERENTE`. Se for idêntica, roda `cargo xtask rotulos`.
- `cargo xtask rotulos` (`xtask/src/rotulos.rs`): lê só a seção "Symbol Table" da listagem, extrai nome e valor, classifica (valor de RAM: variável; nome definido como rótulo nos `.asm` de texto: rotina; resto: constante) e grava `refs/rotulos.txt`. A saída passa por `Rotulos::analisar`, que rejeita qualquer linha que não seja metadado. Testes: `cargo test -p xtask` (4 novos, com listagem sintética).
- **O formato da tabela de símbolos do assembler é uma suposição.** Não vi uma listagem real. Se o formato for outro, o comando falha com erro (nunca grava lixo) e o Marcelo cola algumas linhas da tabela de símbolos (são metadado) para eu ajustar o parser.
- Política dos `.asm` (`/mnt/disasm-texto`): a IA lê só para achar nomes de rótulos. Nunca copia tabelas, código ou comentários. A montagem e a listagem ficam em `$SONIC1_EXT_DIR/privado/montagem`, que não é montada no contêiner.

## Prova com a ROM real (sub-etapa 1.9): pendente do Marcelo

`sh oracle/scripts/roda-oraculo.sh ROM QUADROS [ENTRADAS]`, no host. Confere o SHA-1 da ROM, constrói o oráculo, roda duas vezes, compara com `cmp` e com o comparador, e imprime só resumos. As saídas brutas ficam em `$SONIC1_EXT_DIR/privado/oraculo`, fora do repositório e fora da pasta que a IA enxerga. O script recusa saída dentro do repositório ou de `trabalho`.

## Formato do trace (sub-etapa 1.2)

Código: `crates/fidelidade` (`sonic1-fidelidade`). Não depende do jgenesis e não tem dado da ROM.

- Cabeçalho: magia `S1TRACE\0`, versão do formato (1), id do oráculo (commit e hash dos patches), SHA-1 da ROM, SHA-1 da configuração do console, SHA-1 da lista de entradas e modo.
- Quadro: marca `Q`, número, relógio mestre no fim, eventos de 34 bytes cada e 9 hashes SHA-1 por categoria: `ram68k`, `ram_z80`, `vram`, `cram`, `vsram`, `regs_vdp`, `ym`, `psg`, `eventos`. Os hashes de `ym` e `psg` vêm de cópias montadas só com as escritas observadas.
- Tamanho fixo, little-endian, sem relógio de parede nem ponteiros. Eventos na ordem de emissão (`seq`).
- O hash do trace inteiro é o SHA-1 do cabeçalho mais um resumo por quadro. O escritor e o leitor calculam o mesmo valor.
- O comparador (`comparar_leitores`) para na primeira divergência e diz o quadro, a categoria e o primeiro evento diferente.
- Evidência: `cargo test -p sonic1-fidelidade` (19 testes): ida e volta, mesmos bytes para a mesma entrada, magia inválida, trace truncado, marca inválida, divergência plantada em cada categoria, em evento, em relógio e em tamanho, e o formato do mapa de rótulos.

### Eventos

| Tipo | a | b | c | d |
| --- | --- | --- | --- | --- |
| 1 int68 | nível | PC interrompido | endereço do tratador | |
| 2 desvio68 | PC | destino se tomado | classe (1 BRA, 2 Bcc, 3 BSR, 4 DBcc, 5 JMP, 6 JSR, 7 RTS, 8 RTR, 9 RTE, 10 exceção) | tomado |
| 3 int80 | tipo (0 INT, 1 NMI) | PC interrompido | endereço do tratador | |
| 4 desvio80 | PC | destino se tomado | classe (1 JR, 2 JR cc, 3 DJNZ, 4 JP, 5 JP cc, 6 JP (HL), 7 CALL, 8 CALL cc, 9 RET, 10 RET cc, 11 RETI, 12 RST) | tomado |
| 5 vdp | porta (0 dados, 1 controle) | valor | tamanho (1 ou 2) | |
| 6 ym | porta (0 a 3) | valor | registrador (só em escrita de dado) | 1 se registrador `$2A` (DAC) |
| 7 psg | valor | | | |
| 8 barramento_z80 | sinal (0 busreq, 1 reset) | ativo | | |
| 9 espera_acumulada | causa (1 68000 acessa o Z80, 2 Z80 acessa o 68000) | ciclos do 68000 | | |
| 10 espera_consumida | ciclos do 68000 | | | |
| 11 leitura_busack | bit lido | | | |

Cada evento também leva `mclk`, `seq`, `origem` (0 68000, 1 Z80, 2 hardware), `linha` e `coluna`.
