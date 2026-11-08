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
