# Sintetizador (Etapa 8)

Plano: `docs/plano-etapa-8.md`. Cada afirmação cita o teste ou o comando. Nada aqui tem dado da ROM.

## Licenças e fontes (8.0, bloco 1 do Marcelo, 09/11/2026)

| Referência | Commit | Licença | SHA-1 do `LICENSE` |
| --- | --- | --- | --- |
| Nuked-OPN2 (`ym3438.c`, `ym3438.h`) | `335747d78cb0abbc3b55b004e62dad9763140115` | LGPL 2.1 ou superior (cabeçalho e `LICENSE`) | `b386b371ce94933e63ced1052aa72a60da5485ff` |
| Nuked-PSG (`ympsg.c`, `ympsg.h`) | `d15a168c676f4669e23660be9225b34ad7c1764e` | GPL 2 ou superior (cabeçalho e `LICENSE`) | `4cc77b90af91e615a64ae04893fdffa7939db84c` |

Comando: `sh scripts/etapa8-bloco1.sh` (commits conferidos iguais aos fixados; `cc` GCC 16.2.1; Rust 1.99.1; o host não tem `rustup` nem `wasmtime`, então o alvo wasm e os testes wasm rodam na imagem). Uso permitido: comparação local, no workspace `diferencial-sintese/` (GPLv3), fora do `core`, do `crates/audio` e do site. O fonte dos chips fica em `$SONIC1_EXT_DIR/trabalho/nuked`, fora do git.

## Decisões

- **D8.2 (origem do código, com divulgação).** O `crates/audio` não copia código do Nuked. Ao desenhar o formato de comparação do PSG, a IA **leu o `ympsg.c` inteiro** (9 KB), porque o `YMPSG_GetOutput` devolve `float` e o formato digital só se define vendo o estado interno. O PSG do `crates/audio` foi escrito depois, em estrutura própria (nomes, funções e decomposição diferentes), mas segue a mesma semântica de ritmo de 4 ciclos, porque é isso que a identidade amostra a amostra exige. Quem revisar deve tratar o PSG como **derivado do comportamento do Nuked-PSG**, não como escrito só de documentação pública. Para o FM, a IA **não abriu o corpo do `ym3438.c`**: só o cabeçalho público (`ym3438.h`, nomes dos campos) e o comportamento medido por sondas sintéticas (`diferencial-sintese/examples/sonda*.rs`).
- **D8.7 (quem roda o quê).** O `D1` (execução com a ROM só pelo Marcelo) não proíbe rodar o Nuked com **escritas sintéticas**. A IA roda a bateria sintética núcleo contra Nuked para calibrar. O veredito da etapa (8.5) vem das escritas reais do corpus e roda só pelo Marcelo.
- **D8.8 (amostra do FM).** No modo YM2612 o pino do Nuked-OPN2 é **multiplexado**: cada canal aparece em um ciclo próprio dentro dos 24 ciclos de uma amostra, e os outros ciclos mostram o valor de repouso. Logo a "amostra do FM" para comparar é o fluxo de 24 pares (esquerda, direita) por amostra, não uma soma. Ordem do pico por canal, medida (`examples/sonda3.rs`): ciclo 3 canal 1, 7 canal 5, 11 canal 3, 15 canal 0, 19 canal 4, 23 canal 2 (contando a partir de `cycles == 0`). A mistura para o navegador soma os 6 picos.
- **D8.9 (amostra do PSG).** A saída digital do Nuked-PSG é o que `YMPSG_Read` devolve: 4 níveis de 4 bits (atenuação de cada canal, 15 = mudo), empacotados em 16 bits, lidos a cada 16 ciclos do PSG (mestre/15). O `float` do `YMPSG_GetOutput` é só a conversão para mistura e não entra na comparação.

## PSG: prova (8.1 e 8.2)

- `diferencial-sintese/tests/psg.rs` (4 testes, `cargo test` no workspace, rodado pela IA com escritas sintéticas): silêncio depois do reset; tom simples; tons mais ruído; 40 sementes de 300 escritas aleatórias em ciclos aleatórios (6000 amostras cada). Resultado em 09/11/2026: **iguais ao Nuked-PSG em todas as amostras**, com checagem de que a saída varia (não é silêncio nos dois lados).
- Unidade: um ciclo do PSG = mestre/15 (15 mclk); uma amostra = 16 ciclos (240 mclk). O reset (16 ciclos com IC alto) é o estado de partida de `Psg::novo()`.
- Falta: modelo independente simples e propriedades (determinismo, nunca panica) no `crates/audio`.

## FM: o que foi medido do Nuked-OPN2 (8.3, sondas sintéticas)

- **Saída do pino (modo YM2612):** `pino = 3 * saida_do_canal + (saida >= 0 ? 3 : 0)`, com a saída do canal em 9 bits com sinal (-256 a 255). Canal com o lado desligado no panning mostra o repouso `3`. `examples/sonda2.rs` e `sonda3.rs`.
- **Processamento por slot:** 24 slots; o slot de índice `i` (grupo `i / 6`, canal `i % 6`) é processado no ciclo `(5 + i) mod 24` (contagem do campo `cycles` antes do ciclo). Grupo 0 é o operador 1 (endereço `+0`). A ordem dos grupos 1 a 3 ainda não foi medida.
- **Operador:** a saída `fm_out` usa a fase **antes** do incremento da amostra. O canal latcha a soma dos operadores no ciclo `6 + canal`, mas com a saída do grupo 0 da amostra **anterior**. Resultado: fase no instante T, saída do canal em T+2 amostras. `diag_latencia_tecla` em `diferencial-sintese/tests/fm.rs`.
- **Tecla (key-on):** o nível do envelope vai a 0 cerca de 49 ciclos depois da escrita do dado e a fase zera 52 ciclos depois (canal 0, grupo 0). A fase corre livre antes disso.
- **Latência de escrita:** o efeito de uma escrita depende do ciclo em que ela chega, em degraus de 24 ciclos. Registradores de operador (multiplicador, detune, TL) são lidos num ponto do ciclo e o registrador de F-number baixo seis ciclos antes (`examples/sonda4.rs`: limiar j=11 contra j=17). Isto indica que cada registrador é gravado quando o seu slot passa.
- **Tabelas log-seno e exponencial:** formula publicada das ROMs do OPL2 (`diferencial-sintese/examples/tabelas.rs`); a operação `exp(x)` indexa `EXP[x & 255]` direto e desloca por `x >> 8`.

## Estado das provas (atualizado ao fim do trabalho desta sessão)

| Item | Estado |
| --- | --- |
| 8.0 licenças, pins, ambiente | feito (bloco 1) |
| 8.1 harness (FFI dos dois Nuked, `diferencial-sintese/`) | feito para PSG e FM em nível de ciclo; falta o leitor de traces e o relatório por bloco |
| 8.2 PSG | **igual ao Nuked-PSG** na bateria sintética |
| 8.3 FM | **em andamento, ainda não igual**: estrutura e saída do pino medidas, latências de escrita e tecla ainda sem modelo |
| 8.4 a 8.10 | não feitos |
