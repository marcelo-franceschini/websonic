# Inventário de hardware

Gerado por `cargo xtask inventario`. Só nomes, números e contagens. Cada item cita o primeiro caso, quadro e `seq` em que apareceu no corpus.

Casos lidos (1): boot-demos

Fora deste inventário: DMA como evento próprio (o trace não o registra) e E/S do 68000 (pedido P2).

## VDP: registradores (registrador, valor)

| Item | Vezes | Primeiro caso | Quadro | seq |
| --- | --- | --- | --- | --- |
| reg 0 = 0x04 | 11 | boot-demos | 0 | 4 |
| reg 1 = 0x04 | 1 | boot-demos | 2 | 6660 |
| reg 1 = 0x14 | 1 | boot-demos | 0 | 6 |
| reg 1 = 0x34 | 4 | boot-demos | 55 | 5020 |
| reg 1 = 0x74 | 6 | boot-demos | 89 | 5588 |
| reg 2 = 0x30 | 11 | boot-demos | 0 | 9 |
| reg 3 = 0x28 | 1 | boot-demos | 55 | 5024 |
| reg 3 = 0x3c | 1 | boot-demos | 0 | 11 |
| reg 4 = 0x07 | 11 | boot-demos | 0 | 13 |
| reg 5 = 0x6c | 1 | boot-demos | 0 | 15 |
| reg 5 = 0x7c | 4 | boot-demos | 55 | 5028 |
| reg 6 = 0x00 | 2 | boot-demos | 0 | 17 |
| reg 7 = 0x00 | 5 | boot-demos | 0 | 20 |
| reg 7 = 0x20 | 6 | boot-demos | 354 | 7668 |
| reg 8 = 0x00 | 2 | boot-demos | 0 | 22 |
| reg 9 = 0x00 | 2 | boot-demos | 0 | 24 |
| reg 10 = 0x00 | 1 | boot-demos | 55 | 5039 |
| reg 10 = 0xdf | 4749 | boot-demos | 61 | 6793 |
| reg 10 = 0xff | 1 | boot-demos | 0 | 26 |
| reg 11 = 0x00 | 5 | boot-demos | 0 | 29 |
| reg 11 = 0x03 | 6 | boot-demos | 354 | 7667 |
| reg 12 = 0x81 | 2 | boot-demos | 0 | 31 |
| reg 13 = 0x37 | 1 | boot-demos | 0 | 33 |
| reg 13 = 0x3f | 1 | boot-demos | 55 | 5046 |
| reg 14 = 0x00 | 2 | boot-demos | 0 | 35 |
| reg 15 = 0x01 | 26 | boot-demos | 0 | 38 |
| reg 15 = 0x02 | 27 | boot-demos | 2 | 6661 |
| reg 16 = 0x01 | 8 | boot-demos | 0 | 40 |
| reg 17 = 0x00 | 2 | boot-demos | 0 | 42 |
| reg 18 = 0x00 | 5 | boot-demos | 0 | 44 |
| reg 19 = 0x40 | 12564 | boot-demos | 61 | 6551 |
| reg 19 = 0x70 | 1771 | boot-demos | 1141 | 19 |
| reg 19 = 0xc0 | 6282 | boot-demos | 61 | 6637 |
| reg 19 = 0xff | 26 | boot-demos | 0 | 47 |
| reg 20 = 0x00 | 6282 | boot-demos | 61 | 6550 |
| reg 20 = 0x01 | 14335 | boot-demos | 61 | 6559 |
| reg 20 = 0x0f | 24 | boot-demos | 82 | 906 |
| reg 20 = 0xff | 2 | boot-demos | 0 | 49 |
| reg 21 = 0x00 | 14336 | boot-demos | 0 | 51 |
| reg 21 = 0x80 | 6282 | boot-demos | 61 | 6553 |
| reg 22 = 0x00 | 1 | boot-demos | 0 | 53 |
| reg 22 = 0xe4 | 1771 | boot-demos | 1141 | 20 |
| reg 22 = 0xe6 | 6282 | boot-demos | 61 | 6638 |
| reg 22 = 0xfc | 6282 | boot-demos | 61 | 6561 |
| reg 22 = 0xfd | 6282 | boot-demos | 61 | 6552 |
| reg 23 = 0x7f | 20617 | boot-demos | 61 | 6554 |
| reg 23 = 0x80 | 26 | boot-demos | 0 | 56 |

## VDP: comandos de endereço

| Item | Vezes | Primeiro caso | Quadro | seq |
| --- | --- | --- | --- | --- |
| escrita em CRAM | 6284 | boot-demos | 2 | 6663 |
| escrita em VRAM | 73910 | boot-demos | 0 | 59 |
| escrita em VSRAM | 6286 | boot-demos | 2 | 6767 |

## VDP: DMA disparado

| Item | Vezes | Primeiro caso | Quadro | seq |
| --- | --- | --- | --- | --- |
| 68000 para VDP | 20617 | boot-demos | 61 | 6557 |
| preenchimento de VRAM | 26 | boot-demos | 0 | 59 |

## VDP: escritas na porta de dados (bytes por escrita)

| Item | Vezes | Primeiro caso | Quadro | seq |
| --- | --- | --- | --- | --- |
| 2 byte(s) | 665746 | boot-demos | 0 | 60 |

## YM2612: registradores escritos (porta, registrador)

| Item | Vezes | Primeiro caso | Quadro | seq |
| --- | --- | --- | --- | --- |
| porta 0, reg 0x27 | 6 | boot-demos | 61 | 6869 |
| porta 0, reg 0x28 | 3966 | boot-demos | 61 | 7167 |
| porta 0, reg 0x2a | 796736 | boot-demos | 164 | 9274 |
| porta 0, reg 0x2b | 303 | boot-demos | 61 | 6841 |
| porta 0, reg 0x30 | 7 | boot-demos | 505 | 1533 |
| porta 0, reg 0x31 | 8 | boot-demos | 505 | 2270 |
| porta 0, reg 0x32 | 12 | boot-demos | 505 | 3008 |
| porta 0, reg 0x34 | 7 | boot-demos | 505 | 1586 |
| porta 0, reg 0x35 | 8 | boot-demos | 505 | 2324 |
| porta 0, reg 0x36 | 12 | boot-demos | 505 | 3062 |
| porta 0, reg 0x38 | 7 | boot-demos | 505 | 1559 |
| porta 0, reg 0x39 | 8 | boot-demos | 505 | 2297 |
| porta 0, reg 0x3a | 12 | boot-demos | 505 | 3035 |
| porta 0, reg 0x3c | 7 | boot-demos | 505 | 1613 |
| porta 0, reg 0x3d | 8 | boot-demos | 505 | 2350 |
| porta 0, reg 0x3e | 12 | boot-demos | 505 | 3088 |
| porta 0, reg 0x40 | 19 | boot-demos | 61 | 7337 |
| porta 0, reg 0x41 | 20 | boot-demos | 61 | 7566 |
| porta 0, reg 0x42 | 24 | boot-demos | 61 | 7796 |
| porta 0, reg 0x44 | 33 | boot-demos | 61 | 7396 |
| porta 0, reg 0x45 | 20 | boot-demos | 61 | 7625 |
| porta 0, reg 0x46 | 24 | boot-demos | 61 | 7853 |
| porta 0, reg 0x48 | 33 | boot-demos | 61 | 7454 |
| porta 0, reg 0x49 | 20 | boot-demos | 61 | 7683 |
| porta 0, reg 0x4a | 31 | boot-demos | 61 | 7909 |
| porta 0, reg 0x4c | 51 | boot-demos | 61 | 7510 |
| porta 0, reg 0x4d | 40 | boot-demos | 61 | 7738 |
| porta 0, reg 0x4e | 52 | boot-demos | 61 | 7968 |
| porta 0, reg 0x50 | 7 | boot-demos | 505 | 1640 |
| porta 0, reg 0x51 | 8 | boot-demos | 505 | 2377 |
| porta 0, reg 0x52 | 12 | boot-demos | 505 | 3115 |
| porta 0, reg 0x54 | 7 | boot-demos | 505 | 1692 |
| porta 0, reg 0x55 | 8 | boot-demos | 505 | 2431 |
| porta 0, reg 0x56 | 12 | boot-demos | 505 | 3169 |
| porta 0, reg 0x58 | 7 | boot-demos | 505 | 1666 |
| porta 0, reg 0x59 | 8 | boot-demos | 505 | 2405 |
| porta 0, reg 0x5a | 12 | boot-demos | 505 | 3143 |
| porta 0, reg 0x5c | 7 | boot-demos | 505 | 1720 |
| porta 0, reg 0x5d | 8 | boot-demos | 505 | 2458 |
| porta 0, reg 0x5e | 12 | boot-demos | 505 | 3196 |
| porta 0, reg 0x60 | 7 | boot-demos | 505 | 1746 |
| porta 0, reg 0x61 | 8 | boot-demos | 505 | 2484 |
| porta 0, reg 0x62 | 12 | boot-demos | 505 | 3222 |
| porta 0, reg 0x64 | 7 | boot-demos | 505 | 1800 |
| porta 0, reg 0x65 | 8 | boot-demos | 505 | 2538 |
| porta 0, reg 0x66 | 12 | boot-demos | 505 | 3276 |
| porta 0, reg 0x68 | 7 | boot-demos | 505 | 1773 |
| porta 0, reg 0x69 | 8 | boot-demos | 505 | 2511 |
| porta 0, reg 0x6a | 12 | boot-demos | 505 | 3249 |
| porta 0, reg 0x6c | 7 | boot-demos | 505 | 1827 |
| porta 0, reg 0x6d | 8 | boot-demos | 505 | 2564 |
| porta 0, reg 0x6e | 12 | boot-demos | 505 | 3302 |
| porta 0, reg 0x70 | 7 | boot-demos | 505 | 1854 |
| porta 0, reg 0x71 | 8 | boot-demos | 505 | 2591 |
| porta 0, reg 0x72 | 12 | boot-demos | 505 | 3329 |
| porta 0, reg 0x74 | 7 | boot-demos | 505 | 1907 |
| porta 0, reg 0x75 | 8 | boot-demos | 505 | 2644 |
| porta 0, reg 0x76 | 12 | boot-demos | 505 | 3382 |
| porta 0, reg 0x78 | 7 | boot-demos | 505 | 1880 |
| porta 0, reg 0x79 | 8 | boot-demos | 505 | 2617 |
| porta 0, reg 0x7a | 12 | boot-demos | 505 | 3355 |
| porta 0, reg 0x7c | 7 | boot-demos | 505 | 1933 |
| porta 0, reg 0x7d | 8 | boot-demos | 505 | 2670 |
| porta 0, reg 0x7e | 12 | boot-demos | 505 | 3408 |
| porta 0, reg 0x80 | 7 | boot-demos | 505 | 1960 |
| porta 0, reg 0x81 | 8 | boot-demos | 505 | 2697 |
| porta 0, reg 0x82 | 12 | boot-demos | 505 | 3435 |
| porta 0, reg 0x84 | 7 | boot-demos | 505 | 2013 |
| porta 0, reg 0x85 | 8 | boot-demos | 505 | 2751 |
| porta 0, reg 0x86 | 12 | boot-demos | 505 | 3489 |
| porta 0, reg 0x88 | 7 | boot-demos | 505 | 1986 |
| porta 0, reg 0x89 | 8 | boot-demos | 505 | 2724 |
| porta 0, reg 0x8a | 12 | boot-demos | 505 | 3462 |
| porta 0, reg 0x8c | 7 | boot-demos | 505 | 2040 |
| porta 0, reg 0x8d | 8 | boot-demos | 505 | 2777 |
| porta 0, reg 0x8e | 12 | boot-demos | 505 | 3515 |
| porta 0, reg 0xa0 | 747 | boot-demos | 580 | 154 |
| porta 0, reg 0xa1 | 391 | boot-demos | 565 | 166 |
| porta 0, reg 0xa2 | 202 | boot-demos | 565 | 277 |
| porta 0, reg 0xa4 | 747 | boot-demos | 580 | 129 |
| porta 0, reg 0xa5 | 391 | boot-demos | 565 | 140 |
| porta 0, reg 0xa6 | 202 | boot-demos | 565 | 252 |
| porta 0, reg 0xb0 | 7 | boot-demos | 505 | 1507 |
| porta 0, reg 0xb1 | 8 | boot-demos | 505 | 2245 |
| porta 0, reg 0xb2 | 12 | boot-demos | 505 | 2983 |
| porta 0, reg 0xb4 | 36 | boot-demos | 505 | 2164 |
| porta 0, reg 0xb5 | 8 | boot-demos | 505 | 2898 |
| porta 0, reg 0xb6 | 14 | boot-demos | 505 | 3636 |
| porta 1, reg 0x30 | 72 | boot-demos | 505 | 3742 |
| porta 1, reg 0x31 | 81 | boot-demos | 505 | 4424 |
| porta 1, reg 0x34 | 72 | boot-demos | 505 | 3789 |
| porta 1, reg 0x35 | 81 | boot-demos | 505 | 4472 |
| porta 1, reg 0x38 | 72 | boot-demos | 505 | 3766 |
| porta 1, reg 0x39 | 81 | boot-demos | 505 | 4447 |
| porta 1, reg 0x3c | 72 | boot-demos | 505 | 3812 |
| porta 1, reg 0x3d | 81 | boot-demos | 505 | 4499 |
| porta 1, reg 0x40 | 84 | boot-demos | 61 | 7367 |
| porta 1, reg 0x41 | 93 | boot-demos | 61 | 7597 |
| porta 1, reg 0x42 | 18 | boot-demos | 61 | 7826 |
| porta 1, reg 0x44 | 84 | boot-demos | 61 | 7426 |
| porta 1, reg 0x45 | 93 | boot-demos | 61 | 7655 |
| porta 1, reg 0x46 | 18 | boot-demos | 61 | 7881 |
| porta 1, reg 0x48 | 99 | boot-demos | 61 | 7481 |
| porta 1, reg 0x49 | 99 | boot-demos | 61 | 7710 |
| porta 1, reg 0x4a | 18 | boot-demos | 61 | 7940 |
| porta 1, reg 0x4c | 181 | boot-demos | 61 | 7540 |
| porta 1, reg 0x4d | 119 | boot-demos | 61 | 7769 |
| porta 1, reg 0x4e | 18 | boot-demos | 61 | 7998 |
| porta 1, reg 0x50 | 72 | boot-demos | 505 | 3836 |
| porta 1, reg 0x51 | 81 | boot-demos | 505 | 4523 |
| porta 1, reg 0x54 | 72 | boot-demos | 505 | 3883 |
| porta 1, reg 0x55 | 81 | boot-demos | 505 | 4571 |
| porta 1, reg 0x58 | 72 | boot-demos | 505 | 3860 |
| porta 1, reg 0x59 | 81 | boot-demos | 505 | 4547 |
| porta 1, reg 0x5c | 72 | boot-demos | 505 | 3906 |
| porta 1, reg 0x5d | 81 | boot-demos | 505 | 4597 |
| porta 1, reg 0x60 | 72 | boot-demos | 505 | 3930 |
| porta 1, reg 0x61 | 81 | boot-demos | 505 | 4621 |
| porta 1, reg 0x64 | 72 | boot-demos | 505 | 3978 |
| porta 1, reg 0x65 | 81 | boot-demos | 505 | 4670 |
| porta 1, reg 0x68 | 72 | boot-demos | 505 | 3954 |
| porta 1, reg 0x69 | 81 | boot-demos | 505 | 4648 |
| porta 1, reg 0x6c | 72 | boot-demos | 505 | 4002 |
| porta 1, reg 0x6d | 81 | boot-demos | 505 | 4696 |
| porta 1, reg 0x70 | 72 | boot-demos | 505 | 4025 |
| porta 1, reg 0x71 | 81 | boot-demos | 505 | 4719 |
| porta 1, reg 0x74 | 72 | boot-demos | 505 | 4072 |
| porta 1, reg 0x75 | 81 | boot-demos | 505 | 4767 |
| porta 1, reg 0x78 | 72 | boot-demos | 505 | 4049 |
| porta 1, reg 0x79 | 81 | boot-demos | 505 | 4743 |
| porta 1, reg 0x7c | 72 | boot-demos | 505 | 4095 |
| porta 1, reg 0x7d | 81 | boot-demos | 505 | 4794 |
| porta 1, reg 0x80 | 72 | boot-demos | 505 | 4120 |
| porta 1, reg 0x81 | 81 | boot-demos | 505 | 4817 |
| porta 1, reg 0x84 | 72 | boot-demos | 505 | 4170 |
| porta 1, reg 0x85 | 81 | boot-demos | 505 | 4864 |
| porta 1, reg 0x88 | 72 | boot-demos | 505 | 4147 |
| porta 1, reg 0x89 | 81 | boot-demos | 505 | 4840 |
| porta 1, reg 0x8c | 72 | boot-demos | 505 | 4194 |
| porta 1, reg 0x8d | 81 | boot-demos | 505 | 4888 |
| porta 1, reg 0xa0 | 990 | boot-demos | 565 | 390 |
| porta 1, reg 0xa1 | 537 | boot-demos | 580 | 593 |
| porta 1, reg 0xa4 | 990 | boot-demos | 565 | 363 |
| porta 1, reg 0xa5 | 537 | boot-demos | 580 | 568 |
| porta 1, reg 0xb0 | 72 | boot-demos | 505 | 3719 |
| porta 1, reg 0xb1 | 81 | boot-demos | 505 | 4402 |
| porta 1, reg 0xb4 | 106 | boot-demos | 505 | 4314 |
| porta 1, reg 0xb5 | 116 | boot-demos | 505 | 5007 |
| porta 1, reg 0xb6 | 6 | boot-demos | 505 | 1248 |

## YM2612: uso do DAC

| Item | Vezes | Primeiro caso | Quadro | seq |
| --- | --- | --- | --- | --- |
| escritas no registrador $2A (DAC) | 796736 | boot-demos | 164 | 9274 |

## PSG

| Item | Vezes | Primeiro caso | Quadro | seq |
| --- | --- | --- | --- | --- |
| dado de tom | 1294 | boot-demos | 565 | 479 |
| ruído, canal 3 | 42 | boot-demos | 505 | 5112 |
| tom, canal 0 | 739 | boot-demos | 1060 | 7221 |
| tom, canal 1 | 143 | boot-demos | 1060 | 7282 |
| tom, canal 2 | 412 | boot-demos | 565 | 478 |
| volume, canal 0 | 1088 | boot-demos | 2 | 6847 |
| volume, canal 1 | 1326 | boot-demos | 2 | 6852 |
| volume, canal 2 | 49 | boot-demos | 3 | 4 |
| volume, canal 3 | 3378 | boot-demos | 3 | 10 |

## Interrupções do 68000 (nível)

| Item | Vezes | Primeiro caso | Quadro | seq |
| --- | --- | --- | --- | --- |
| nível 6 | 6381 | boot-demos | 61 | 6517 |

## Z80: BUSREQ e RESET (sinal 0=busreq 1=reset, ativo)

| Item | Vezes | Primeiro caso | Quadro | seq |
| --- | --- | --- | --- | --- |
| sinal 0, ativo 0 | 13404 | boot-demos | 0 | 183 |
| sinal 0, ativo 1 | 13414 | boot-demos | 0 | 61 |
| sinal 1, ativo 0 | 10 | boot-demos | 0 | 62 |
| sinal 1, ativo 1 | 6 | boot-demos | 0 | 182 |
