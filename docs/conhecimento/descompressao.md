# Descompressores (Etapa 3)

Registro dos três formatos, de onde veio cada afirmação e o que está provado. Nada aqui tem dado da ROM. Nada é "provado contra o jogo" antes de o Marcelo rodar `descompressao/prova.sh` e colar o relatório.

## Formatos e usos

| Formato | Para que o Sonic 1 usa | Rotina do disassembly | Fonte da afirmação |
| --- | --- | --- | --- |
| Nemesis | Arte (tiles de 32 bytes). `NemDec` grava na VDP, `NemDecToRAM` na RAM | `NemDec`, `NemDecToRAM`, `NemDecMain`, `NemDec_BuildCodeTable` | Cabeçalhos e comentários dos `.asm` de texto (`_inc/Decompression/`); s3unlocked (arte) |
| Enigma | Mapas de plano, **blocos de 16x16 de cada zona**, layouts da special stage (`SS_1` a `SS_6`), telas (título, Sega, nomes japoneses) | `EniDec` | Comentários dos pontos de chamada (`LevelLayoutLoad`, special stage, `sonic.asm`) |
| Kosinski | **Blocos de 256x256**, flores do final, o **programa do Z80** (`DACDriver`, descomprimido na RAM do Z80) | `KosDec` | Comentários dos pontos de chamada (`LevelLayoutLoad`, `DACDriverLoad`) |

Respostas às perguntas do plano: Q2 (16x16 é Enigma e 256x256 é Kosinski; special stage é Enigma) saiu dos comentários dos pontos de chamada. Q3 (registradores): `KosDec` recebe a0 origem e a1 destino; `EniDec` recebe d0 tile inicial somado a cada palavra, a0 e a1; `NemDec` só a0; `NemDecToRAM` a0 e a4 destino. `EniDec` e `NemDecToRAM` restauram o registrador de destino ao sair (movem), então o tamanho da saída não vem do registrador.

## Especificação dos formatos (Q1)

As páginas do Sonic Retro e do Sega Retro bloquearam o acesso automático. A descrição usada vem do código-fonte público do `mdcomp` (flamewing, `src/lib/nemesis.cc`, `kosinski.cc`, `enigma.cc`, lido em 06/11/2026 por um resumo da ferramenta de busca, não linha a linha). Foi usada só como descrição do formato. Nada foi copiado.

- **Kosinski:** descrição de 16 bits little-endian, bit menos significativo primeiro, recarga antecipada (a próxima palavra vem antes dos bytes de dados do código que gastou o 16º bit). Bit 1 literal. Bits 0,1: cópia com distância de 13 bits em 2 bytes e contagem de 3 bits (3 a 9) ou, com contagem 0, um terceiro byte (0 fim, 1 sem saída, n dá n+1). Bits 0,0 e 2 bits: cópia curta de 2 a 5 com 1 byte de distância (1 a 256). Terminador `00 F0 00`.
- **Enigma:** cabeçalho de 6 bytes (bits por valor, máscara de 5 flags, palavra incremental, palavra comum). Fluxo de bits: `00`+4 bits incremental, `01`+4 bits comum, `1`+modo de 2 bits+contagem de 4 bits (modos 0 a 2 repetem em linha, modo 3 lista; modo 3 com contagem 15 termina). Saída em palavras big-endian.
- **Nemesis:** cabeçalho de 2 bytes (bit 15 XOR, 15 bits de tiles). Tabela de códigos até `0xFF`. Prefixo `111111` reservado para corrida em linha (3 bits de contagem menos 1 e 4 bits de nibble). XOR aplicado por linha de 4 bytes.

O que **não** está confirmado em fonte primária: o valor 1 do terceiro byte no Kosinski (só a página resumida do `mdcomp` o cita); a soma do tile inicial também aos valores em linha do Enigma (suposição minha, coerente com o comentário do cabeçalho de `EniDec` "added to each 8x8"); e se corridas do Nemesis podem passar da fronteira de tile (o código aceita, o fim exato é checado). O relatório contra o oráculo decide cada um.

## Código

- `crates/core/src/descompressao/` (`kosinski.rs`, `enigma.rs`, `nemesis.rs`). Lints do núcleo valem, sem `#[allow]`. Nemesis é uma estrutura retomável (`executar_ate(n_tiles)`), decisão D3, porque o progresso da arte fica na memória comparada no original. Os passos exatos por quadro ficam para a Etapa 5.
- `crates/teste-descompressao/`: compressores de teste e os 11 testes. Crate separado porque os lints do núcleo (aritmética sem estouro explícito) tornariam os compressores ilegíveis.
- `xtask/src/descompressao.rs`: `cargo xtask descompressao-relatorio`.
- `descompressao/manifesto.txt`: 174 blocos (151 Nemesis, 17 Enigma, 6 Kosinski). Só nomes de rótulo e formato, tirados dos nomes dos arquivos incluídos pelos `.asm` de texto. Rótulos de ramos REV00 saem como `AUSENTE`.
- `docs/pedidos/descomprimir-oraculo.patch`: subcomando `oraculo descomprimir` (oracle/ é congelado).

## Evidência (06/11/2026, IA, dados sintéticos)

- `cargo test -p sonic1-teste-descompressao`: 11 passam. Ida e volta dos três formatos (com e sem XOR no Nemesis, três tiles iniciais no Enigma), fronteiras de 16 bits da descrição Kosinski, distância 1 e 8192, cópia longa, vetor manual com contagem 1, erros, Nemesis retomável igual ao inteiro, 3000 entradas aleatórias, todos os prefixos e todo bit invertido de quatro fluxos válidos: nunca pânico, saída dentro do limite, consumo dentro da entrada, determinismo.
- `cargo clippy --workspace --all-targets -- -D warnings`: sem aviso. `cargo test --workspace`: sem falha.
- `cargo test -p xtask`: 21 passam (relatório com ROM sintética: `IGUAL`, `DIFERE`, `ERRO`, `AUSENTE`, bloco que passa do próximo rótulo, manifesto ruim; mapa de rótulos na base 16 e trava de sanidade).
- Oráculo com o patch aplicado numa cópia: 14 testes passam (13 antigos e `descomprimir_chama_a_rotina_certa_e_mede_o_tamanho`, com rotinas falsas de 68000 que provam os registradores de entrada e a medida de tamanho por duplo preenchimento).
- **Não rodado:** `wasm32-wasip1` (o host não tem o alvo, como na Etapa 1) e tudo com a ROM real.

## Como o oráculo mede o "carregado"

`oraculo descomprimir` parte do estado de ligar, põe a origem (rótulo do bloco) e o destino (`$FF0000`) nos registradores, chama a rotina do jogo isolada, e lê a RAM. Como as rotinas restauram o destino, o tamanho vem de duas execuções com a área preenchida de `00` e de `FF`: o que a rotina escreveu é o prefixo em que as duas terminam iguais. Imprime rótulo, formato, tile, endereço, tamanho e SHA-1. Limite: o jogo usa `NemDec` (porta da VDP) e o oráculo chama `NemDecToRAM`; a decodificação é a mesma, só a rotina de escrita da linha muda. Conferir a rotina de escrita da VDP fica no backlog.

## Limites

Os compressores de teste e os descompressores foram escritos pela mesma sessão. Um mal-entendido do formato pode estar nos dois. Só o relatório contra o oráculo (ROM real, pelo Marcelo) fecha a fidelidade.
