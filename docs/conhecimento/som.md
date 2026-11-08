# Som (Etapa 6)

Plano: `docs/plano-etapa-6.md`. Cada afirmação cita o teste ou o comando. Nada aqui tem dado da ROM. Estado de cada prova no fim do arquivo.

## Como o Z80 do oráculo avança (sub-etapa 6.0)

- O Z80 do oráculo executa a instrução **inteira no primeiro ciclo** e depois só espera `custo - 1` ciclos. Os efeitos (escrita no YM, leitura da ROM, escrita no banco) acontecem no início do trecho. Fonte: `tick` e `execute_instruction` em `cpu/z80-emu/src/core.rs` do jgenesis fixado. Consequência para o núcleo: a máquina do Z80 tem um trecho por instrução, com efeito no início e custo depois (decisão D6.2 do plano).
- Com BUSREQ ativo ou RESET ativo, `execute_instruction` devolve 4 ciclos sem executar nada; com RESET o PC volta a zero. O contador de ciclos restantes (`t_cycles_wait`) continua caindo mesmo com BUSREQ. Teste: `z80_parado_pelo_barramento_nao_anda_mas_os_ciclos_correm` e `depois_do_reset_o_z80_volta_ao_endereco_zero` em `oracle/runner/tests/pontos_z80.rs` (patch 0005).
- O BUSACK do oráculo vale na hora: `busreq && !reset` (`bus.rs`). Não espera o fim da instrução do Z80. Por isso a leitura do BUSACK não precisa de ponto do Z80.
- O Z80 só avança **depois** de cada instrução do 68000 (alcance, `api.rs`): enquanto `z80_mclk + 15 <= m68k_mclk`, um tick. As escritas do 68000 na janela do Z80 são aplicadas depois desse alcance (`tick_components`), e as leituras na janela são feitas durante a instrução, antes dele. O ponto do Z80 grava a posição vista em cada caso.
- O Z80 para quando a VDP tem o barramento (DMA). Esses ciclos não são contados como ativos. Cada leitura do Z80 na ROM do 68000 soma 3 ciclos de relógio ao Z80 sem tick e uma parada ao 68000.

## Patch 0005 do oráculo: pontos do Z80 (6.0)

- `.s1tempo` versão 2 ganha o tipo 4, **ponto do Z80**, um por acesso do 68000 que toca o Z80: BUSREQ pedido ou liberado, RESET ligado ou solto, leitura ou escrita da janela da RAM do Z80. Campos: PC visto, ciclos que faltam da instrução, ciclos ativos desde o ponto anterior, registrador de banco (9 bits), endereço e byte da janela. O leitor do núcleo aceita as versões 1 e 2 (`crates/core/src/quadro/tempo.rs`, testes `ida_e_volta`, `versao_1_e_aceita_sem_pontos_do_z80`, `ponto_do_z80_com_acesso_desconhecido_e_recusado`).
- O formato do trace (`crates/fidelidade`, congelado) não mudou.
- Prova no oráculo, com ROM sintética (08/11/2026): `sh oracle/scripts/testa.sh` passa, com 5 testes novos em `pontos_z80.rs`: um ponto por acesso e na ordem do programa; endereço e byte da janela; PC e ciclos parados com o barramento cedido; PC zero depois do reset; a soma dos ciclos dos pontos não passa do tempo decorrido. Resultado da rodada de 08/11/2026 na cópia limpa: todos os testes do oráculo passam (14 + 5 + 3 + 3 e os demais).
- Patch pronto em `docs/pedidos/etapa6-oraculo.patch` (aplica limpo com `git apply --check`).

## IDs de som

- `cargo xtask ids-som` (3 testes) lê `refs/rotulos.txt` e os nomes dos arquivos de texto do som: 73 IDs (19 músicas `81` a `93`, 48 efeitos `A0` a `CF`, 1 especial `D0`, 5 sinalizadores `E0` a `E4`). A lista fica em `crates/teste-som/ids.txt`. Cinco IDs de efeito não têm arquivo de texto próprio (`sem_arquivo`).

## Inventário do som

- `cargo xtask inventario-som` (2 testes) lê traces e tempos e escreve `docs/inventario-som.md`. Teste de ponta a ponta com a ROM sintética do Z80 (`oraculo rodar` com `--tempo`, depois o xtask): 25 pontos, 9 posições distintas, DAC e desvios do Z80 contados por origem. Com a ROM real: bloco 1 do Marcelo.

## Estado das provas

| Item | Estado |
| --- | --- |
| 6.0 patch 0005 | provado em ROM sintética; aplicar no bloco 1 |
| 6.0 inventário do som | gerador provado em ROM sintética; dados reais no bloco 1 |

## Barramento e programa do Z80 no núcleo (6.1 e 6.2)

- `crates/core/src/som/z80.rs`: `Banco` (9 bits), `MaquinaZ80` com os dois programas, `PontoZ80` e `DivergenciaZ80`. `Memoria` consome um ponto do Z80 gravado a cada acesso do 68000 que toca o Z80 (BUSREQ, RESET, janela), avança a máquina pelos ciclos gravados e confere PC, ciclos que faltam e banco. Sem ponto na fila, o Z80 fica parado (comportamento da Etapa 5 intacto).
- O layout dos trechos do programa B foi montado a partir da ordem das instruções e do tamanho de cada uma (manual do Z80). **Conferência com dados reais:** todos os endereços de desvio do Z80 que o oráculo executou no corpus (`docs/inventario-som.md`) caem em endereços de instrução que o layout prevê (`1E` DJNZ, `20` JR, `37` JP P, `3F` JR NC, `8E`, `A4`, `CA` DJNZ, `AC`, `B3`, `B6`, `D0`, `D3` e, no programa A, `25` JP (HL)).
- Prova com o oráculo (ROM sintética, programas montados à mão a partir do manual, tabelas e amostras inventadas): `diferencial-som/tests/z80.rs`, 3 testes passam: o programa A inteiro (LDIR de 8153 voltas, salto para si mesmo), o programa B tocando as três amostras de DPCM em instantes variados, e o programa B tocando a voz SEGA pela janela de ROM. Em todos: zero divergências de posição em todos os pontos (PC, ciclos que faltam, banco), as escritas do Z80 no YM2612 idênticas e na mesma ordem, e a RAM do Z80 idêntica no fim.
- Falha achada e corrigida no jogo: o reconhecimento do programa no endereço zero só valia depois de um reset, então o programa A, ao saltar para zero, recomeçava em vez de cair no salto para si mesmo. Agora todo trecho em zero reconhece o programa de novo.
- `diferencial-som/` é um workspace novo (GPLv3, como o oráculo) porque `diferencial/` foi congelado na Etapa 5.
- Modelo independente do barramento: `crates/teste-som/src/modelo_barramento.rs`, cruzado com o núcleo em 200 sequências pseudoaleatórias (BUSACK e visibilidade da RAM do Z80) e 500 sequências do registrador de banco (`crates/teste-som/tests/barramento.rs`).
- WebAssembly: o alvo `wasm32-wasip1` não está instalado no host; roda na imagem (bloco do Marcelo).

## Driver do 68000 (6.3) e harness (6.5)

- `crates/core/src/som/driver.rs`: `Driver` sobre a `Memoria`, uma função por rótulo (`UpdateMusic`, `DACUpdateTrack`, `FMUpdateTrack`, `PSGUpdateTrack`, `CoordFlag` e as 26 flags `cf*`, `SetVoice`, `SendVoiceTL`, `Sound_PlayBGM`, `Sound_PlaySFX`, `Sound_PlaySpecial`, `StopSFX`, `StopSpecialSFX`, `FadeOutMusic`, `DoFadeOut`, `DoFadeIn`, `PauseMusic`, `StopAllSound`, `InitMusicPlayback`, `CycleSoundQueue`, `PlaySoundID`, `PlaySegaSound` e as filas). Cada decisão passa por `decisao!`. Tabelas (frequências, vozes, prioridades, envelopes, endereços de trilhas) são lidas da ROM nos endereços dos rótulos.
- **Retornos adulterados do original:** o original faz `addq.w #4,sp` e `addq.w #8,sp` para sair de mais de uma rotina. Viram `Fluxo::SairDoUpdate` (BGM carregada, voz SEGA, flag de coordenação na trilha de DAC: `UpdateMusic` acaba na hora, **sem o `startZ80` do fim**) e `Cf::Sair8` (flag de coordenação nas trilhas de FM e PSG: só a atualização da trilha acaba). A sutileza: a mesma flag que encerra só a trilha de FM encerra o `UpdateMusic` inteiro quando vem da trilha de DAC, porque a pilha tem um nível a menos. Isto é leitura do código, a prova é o relatório do bloco 2.
- **Bugs conhecidos reproduzidos no código:** a fila 3 é ignorada por `UpdateMusic`; `PlaySoundID` aceita música até `$9F` e especial até `$DF`; `StopAllSound` não apaga os últimos `$10` bytes; `SendVoiceTL` lê o ponteiro de voz relativo a A6 (cai em `v_special_voice_ptr`); `StopSFX` perde o ponteiro da trilha de efeito no caso do FM4 com efeito especial; `InitMusicPlayback` silencia todos os canais; `PSGDoVolFX` só trata o terminador `$80`; `cfFadeInToPrevious` não reescreve o registrador `$2B`. Testes sintéticos: `a_terceira_fila_e_ignorada_pelo_update` e `comando_de_parar_tudo_limpa_a_ram_do_driver_menos_os_ultimos_16_bytes` (`crates/teste-som/tests/driver.rs`). Os testes `BK_nnnn` por item vêm na 6.6.
- **Risco aceito:** os registradores de dados do original que guardam lixo na parte alta (por exemplo `d0` em `SetVoice` chamado por `StopSFX`) são tratados como limpos. Se o relatório mostrar divergência nesses caminhos, é onde olhar.
- IDs `94` a `9F` e `D1` a `DF`: o original trata como música ou especial e trava ou executa lixo. Ficam **fora** do relatório (não há comportamento definido a comparar). Entram `80`, `00`, `7F`, `E5` e `FF`.
- Layout: `crates/teste-som/tests/layout.rs` confere todas as constantes de RAM, de trilha e de endereço de tabela contra `refs/rotulos.txt` (2 testes passam).
- Harness `diferencial-som` (binário `som`): o oráculo roda o jogo real a partir de um estado calmo (som parado), o ID entra pela fila, e cada chamada de `UpdateMusic` é medida por entrada e retorno (o endereço de retorno vem da pilha). O núcleo roda a mesma chamada, de forma contínua, com o tempo gravado do trecho (pontos do Z80, leituras do BUSACK e do YM). Acessos ao Z80 feitos fora do driver são aplicados à máquina do Z80 (`aplicar_ponto_externo`), o que confere a posição do Z80 também fora de `UpdateMusic`. Escritas na RAM do driver feitas por fora são contadas como entradas externas. Comparado por trecho: escritas no YM e no PSG (fonte, porta, valor, na ordem, até o último BUSREQ liberado do trecho), região do driver na RAM, RAM do Z80 e posição do Z80. Smoke sintético: `diferencial-som/tests/harness.rs`.
- Estado inicial do Z80 do núcleo: a posição vem do oráculo (PC), os ciclos que faltam são calibrados para bater com o primeiro ponto, e o banco é `$00F` (valor visto em todos os pontos do corpus fora da carga).

## Estado das provas (atualizado)

| Item | Estado |
| --- | --- |
| 6.1, 6.2 | provado contra o oráculo (ROM sintética, programas A e B montados à mão) |
| 6.3 | testes sintéticos do núcleo; comparação com o oráculo pendente (bloco 2) |
| 6.4 | integração do DAC e da voz SEGA: pela máquina do Z80 (provada) e pelo driver; comparação real pendente (bloco 2) |
| 6.5 | harness roda numa ROM sintética (negativo); relatório com a ROM pendente (bloco 2) |

## Resultado do bloco 2 e bugs conhecidos do som (6.5 e 6.6)

- **Relatório do som com a ROM real: 78 IGUAL, 0 DIVERGE** (bloco 2, terceira rodada; hash do relatório `ff49435d25246d1734cc344369bfe58f0c104735`). Cobre as 19 músicas (2400 quadros cada, com as escritas do DAC), os 48 efeitos, o especial `D0`, os comandos `E0` a `E4` e os IDs `80`, `00`, `7F`, `E5` e `FF`. Em cada um: sequência de escritas no YM2612 e no PSG igual em conteúdo e ordem (hash igual), região do driver na RAM igual, RAM do Z80 igual (menos as posições `1FFD` e `1FFF`, que o Z80 muda sozinho) e posição do Z80 igual em todo acesso do 68000 (PC, ciclos que faltam, banco). Entradas externas: 0, exceto 2 nos IDs `89` e `93` (escritas de outro código do jogo na região do driver durante a música).
- Falhas do relatório achadas e corrigidas no caminho (não do driver): o fim do trecho confundia o BUSREQ liberado do retry do DAC com o fim; a RAM do Z80 na saída de um trecho longo (voz SEGA) e as posições `1FFD` e `1FFF` não são comparáveis no instante da saída (o Z80 do oráculo anda sozinho depois do último acesso do 68000).
- Bugs conhecidos: 8 dos 20 itens (`BK_0276`, `0277`, `0278`, `0286`, `0287`, `0290`, `0291`, `0292`) têm teste com o nome certo; `cobertura/bugs-som.txt` diz como cada um dos 20 é provado (teste, relatório do som, inalcançável ou pendente). Seis itens dependem de roteiros de fade com efeito ativo, que o harness ainda não tem (backlog).
