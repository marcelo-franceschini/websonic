# Inventário do barramento do 68000

Gerado por `cargo xtask inventario-barramento`. Só região, operação, tamanho em bytes e contagem.

Fonte: `oraculo rodar --barramento`, casos: boot-demos, demos-quatro, lz-agua, partida-ghz1.

A região `ram` conta os acessos fora do DMA; `sem_dispositivo` é o acesso a endereço sem nada ligado (leitura devolve barramento aberto, escrita é ignorada). Na linha `excecao`, o campo de bytes é o número do vetor.

| região | operação | bytes | vezes |
| --- | --- | --- | --- |
| io | escrita | 1 | 104192 |
| io | leitura | 1 | 104192 |
| io | leitura | 2 | 12 |
| ram | escrita | 1 | 4821355 |
| ram | escrita | 2 | 28774216 |
| ram | leitura | 1 | 92148811 |
| ram | leitura | 2 | 20720690 |
| rom | leitura | 1 | 5819241 |
| rom | leitura | 2 | 649347077 |
| vdp | escrita | 1 | 36244 |
| vdp | escrita | 2 | 3299489 |
| vdp | leitura | 2 | 143421 |
| z80_barramento | escrita | 2 | 111063 |
| z80_barramento | leitura | 1 | 55534 |
| z80_janela | escrita | 1 | 265410 |
| z80_janela | leitura | 1 | 362264 |
| z80_reset | escrita | 2 | 61 |
