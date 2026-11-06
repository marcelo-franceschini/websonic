# Oráculo (Etapa 1)

Registro do oráculo e do formato do trace. Cada entrada cita o teste ou o commit que a sustenta.

## Formato do trace (sub-etapa 1.2)

Código: `crates/fidelidade` (`sonic1-fidelidade`). Não depende do jgenesis e não tem dado da ROM.

- Cabeçalho: magia `S1TRACE\0`, versão do formato (1), id do oráculo, SHA-1 da ROM, SHA-1 da configuração do console, SHA-1 da lista de entradas e modo.
- Quadro: marca `Q`, número, relógio mestre no fim, eventos de 34 bytes cada e 9 hashes SHA-1 por categoria: `ram68k`, `ram_z80`, `vram`, `cram`, `vsram`, `regs_vdp`, `ym`, `psg`, `eventos`.
- Tamanho fixo, little-endian, sem relógio de parede nem ponteiros. Eventos ordenados por (mclk, seq).
- O hash do trace inteiro é o SHA-1 do cabeçalho mais um resumo por quadro. O escritor e o leitor calculam o mesmo valor.
- O comparador (`comparar_leitores`) para na primeira divergência e diz o quadro, a categoria e o primeiro evento diferente.
- Evidência: `cargo test -p sonic1-fidelidade` (15 testes): ida e volta, mesmos bytes para a mesma entrada, magia inválida, trace truncado, marca inválida, divergência plantada em cada categoria, em evento, em relógio e em tamanho.
- Não provado: `cargo test --target wasm32-wasip1` para este crate. O host de desenvolvimento não tem o alvo nem o `wasmtime`. O CI só roda WebAssembly para `core` e `rom`.

### Eventos

| Tipo | a | b | c | d |
| --- | --- | --- | --- | --- |
| 1 int68 | nível | PC interrompido | endereço do tratador | |
| 2 desvio68 | PC | destino se tomado | classe | tomado |
| 3 int80 | tipo (0 INT, 1 NMI) | PC interrompido | endereço do tratador | |
| 4 desvio80 | PC | destino se tomado | classe | tomado |
| 5 vdp | porta (0 dados, 1 controle) | valor | tamanho (1 ou 2) | |
| 6 ym | porta (0 a 3) | valor | | |
| 7 psg | valor | | | |
| 8 barramento_z80 | sinal (0 busreq, 1 reset) | ativo | | |
| 9 espera_acumulada | causa (1 68000 acessa o Z80, 2 Z80 acessa o 68000) | ciclos do 68000 | | |
| 10 espera_consumida | ciclos do 68000 | | | |
| 11 leitura_busack | bit lido | | | |

Cada evento também leva `mclk`, `seq`, `origem` (0 68000, 1 Z80, 2 hardware), `linha` e `coluna`.
