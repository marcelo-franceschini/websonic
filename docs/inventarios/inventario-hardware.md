# Inventário de hardware

Gerado por `cargo xtask inventario`. Só nomes, números e contagens. Cada item cita o primeiro caso, quadro e `seq` em que apareceu no corpus.

Casos lidos (2): boot-demos, lz-agua

Fora deste inventário: DMA como evento próprio (o trace não o registra) e E/S do 68000 (pedido P2).

## VDP: registradores (registrador, valor)

| Item | Vezes | Primeiro caso | Quadro | seq |
| --- | --- | --- | --- | --- |
| reg 0 = 0x04 | 16 | boot-demos | 0 | 4 |
| reg 0 = 0x14 | 1 | lz-agua | 771 | 1502 |
| reg 1 = 0x04 | 2 | boot-demos | 2 | 6660 |
| reg 1 = 0x14 | 2 | boot-demos | 0 | 6 |
| reg 1 = 0x34 | 6 | boot-demos | 55 | 5020 |
| reg 1 = 0x74 | 8 | boot-demos | 89 | 5588 |
| reg 2 = 0x30 | 16 | boot-demos | 0 | 9 |
| reg 3 = 0x28 | 2 | boot-demos | 55 | 5024 |
| reg 3 = 0x3c | 2 | boot-demos | 0 | 11 |
| reg 4 = 0x07 | 16 | boot-demos | 0 | 13 |
| reg 5 = 0x6c | 2 | boot-demos | 0 | 15 |
| reg 5 = 0x7c | 6 | boot-demos | 55 | 5028 |
| reg 6 = 0x00 | 4 | boot-demos | 0 | 17 |
| reg 7 = 0x00 | 8 | boot-demos | 0 | 20 |
| reg 7 = 0x20 | 8 | boot-demos | 354 | 7668 |
| reg 8 = 0x00 | 4 | boot-demos | 0 | 22 |
| reg 9 = 0x00 | 4 | boot-demos | 0 | 24 |
| reg 10 = 0x00 | 2 | boot-demos | 55 | 5039 |
| reg 10 = 0x02 | 1 | lz-agua | 1859 | 8280 |
| reg 10 = 0x07 | 1 | lz-agua | 1858 | 8031 |
| reg 10 = 0x0b | 1 | lz-agua | 1857 | 8295 |
| reg 10 = 0x0f | 1 | lz-agua | 1856 | 8303 |
| reg 10 = 0x13 | 1 | lz-agua | 1855 | 8409 |
| reg 10 = 0x16 | 1 | lz-agua | 1854 | 8222 |
| reg 10 = 0x1a | 1 | lz-agua | 1853 | 7851 |
| reg 10 = 0x1e | 1 | lz-agua | 1852 | 7629 |
| reg 10 = 0x21 | 1 | lz-agua | 1851 | 7740 |
| reg 10 = 0x25 | 1 | lz-agua | 1850 | 7386 |
| reg 10 = 0x28 | 1 | lz-agua | 1849 | 7777 |
| reg 10 = 0x2c | 1 | lz-agua | 1848 | 7787 |
| reg 10 = 0x2f | 1 | lz-agua | 1847 | 7497 |
| reg 10 = 0x33 | 1 | lz-agua | 1846 | 7835 |
| reg 10 = 0x36 | 1 | lz-agua | 1845 | 7635 |
| reg 10 = 0x39 | 1 | lz-agua | 1844 | 7738 |
| reg 10 = 0x3c | 1 | lz-agua | 1843 | 7908 |
| reg 10 = 0x3d | 5 | lz-agua | 1738 | 8557 |
| reg 10 = 0x3e | 8 | lz-agua | 1743 | 8556 |
| reg 10 = 0x3f | 7 | lz-agua | 1751 | 7938 |
| reg 10 = 0x40 | 6 | lz-agua | 1737 | 8099 |
| reg 10 = 0x41 | 5 | lz-agua | 1762 | 8601 |
| reg 10 = 0x42 | 6 | lz-agua | 1767 | 7895 |
| reg 10 = 0x43 | 4 | lz-agua | 1772 | 7950 |
| reg 10 = 0x44 | 4 | lz-agua | 1776 | 7931 |
| reg 10 = 0x45 | 6 | lz-agua | 1780 | 7922 |
| reg 10 = 0x46 | 6 | lz-agua | 1736 | 7749 |
| reg 10 = 0x47 | 2 | lz-agua | 1790 | 6569 |
| reg 10 = 0x48 | 1 | lz-agua | 1839 | 7819 |
| reg 10 = 0x4a | 3 | lz-agua | 1792 | 7541 |
| reg 10 = 0x4b | 7 | lz-agua | 1795 | 7798 |
| reg 10 = 0x4c | 9 | lz-agua | 1735 | 7901 |
| reg 10 = 0x4d | 10 | lz-agua | 1809 | 7957 |
| reg 10 = 0x4e | 19 | lz-agua | 1819 | 8558 |
| reg 10 = 0x52 | 1 | lz-agua | 1734 | 7925 |
| reg 10 = 0x53 | 4 | lz-agua | 1268 | 8370 |
| reg 10 = 0x54 | 5 | lz-agua | 1267 | 8262 |
| reg 10 = 0x55 | 5 | lz-agua | 1276 | 8148 |
| reg 10 = 0x56 | 5 | lz-agua | 1281 | 9001 |
| reg 10 = 0x57 | 5 | lz-agua | 1286 | 9010 |
| reg 10 = 0x58 | 7 | lz-agua | 1291 | 8609 |
| reg 10 = 0x59 | 8 | lz-agua | 1297 | 9040 |
| reg 10 = 0x5a | 11 | lz-agua | 1266 | 8064 |
| reg 10 = 0x5b | 24 | lz-agua | 1315 | 8309 |
| reg 10 = 0x5d | 1 | lz-agua | 1732 | 7646 |
| reg 10 = 0x5e | 1 | lz-agua | 1339 | 8201 |
| reg 10 = 0x5f | 1 | lz-agua | 1731 | 7972 |
| reg 10 = 0x60 | 2 | lz-agua | 1265 | 8370 |
| reg 10 = 0x62 | 1 | lz-agua | 1729 | 7937 |
| reg 10 = 0x64 | 37 | lz-agua | 1340 | 8318 |
| reg 10 = 0x66 | 1 | lz-agua | 1264 | 8261 |
| reg 10 = 0x67 | 1 | lz-agua | 1692 | 7930 |
| reg 10 = 0x69 | 1 | lz-agua | 1691 | 7736 |
| reg 10 = 0x6a | 2 | lz-agua | 1341 | 8053 |
| reg 10 = 0x6c | 6 | lz-agua | 1225 | 8241 |
| reg 10 = 0x6d | 16 | lz-agua | 1229 | 8343 |
| reg 10 = 0x6e | 17 | lz-agua | 1224 | 8376 |
| reg 10 = 0x6f | 13 | lz-agua | 1247 | 8195 |
| reg 10 = 0x70 | 10 | lz-agua | 1253 | 8334 |
| reg 10 = 0x71 | 10 | lz-agua | 1258 | 8234 |
| reg 10 = 0x72 | 5 | lz-agua | 1654 | 8012 |
| reg 10 = 0x73 | 3 | lz-agua | 1651 | 8004 |
| reg 10 = 0x74 | 2 | lz-agua | 1223 | 7816 |
| reg 10 = 0x78 | 1 | lz-agua | 1650 | 8548 |
| reg 10 = 0x79 | 1 | lz-agua | 1344 | 8217 |
| reg 10 = 0x7a | 1 | lz-agua | 1222 | 8184 |
| reg 10 = 0x7e | 1 | lz-agua | 1345 | 8450 |
| reg 10 = 0x7f | 1 | lz-agua | 1649 | 8267 |
| reg 10 = 0x80 | 1 | lz-agua | 1221 | 8360 |
| reg 10 = 0x82 | 1 | lz-agua | 1346 | 8445 |
| reg 10 = 0x85 | 1 | lz-agua | 1648 | 8676 |
| reg 10 = 0x86 | 2 | lz-agua | 1220 | 8154 |
| reg 10 = 0x8a | 1 | lz-agua | 1348 | 8124 |
| reg 10 = 0x8b | 1 | lz-agua | 1647 | 8706 |
| reg 10 = 0x8c | 14 | lz-agua | 1206 | 7200 |
| reg 10 = 0x8e | 6 | lz-agua | 1349 | 8526 |
| reg 10 = 0x8f | 6 | lz-agua | 1417 | 9101 |
| reg 10 = 0x90 | 5 | lz-agua | 1412 | 9058 |
| reg 10 = 0x91 | 6 | lz-agua | 1407 | 9038 |
| reg 10 = 0x92 | 8 | lz-agua | 1205 | 7083 |
| reg 10 = 0x93 | 4 | lz-agua | 1398 | 9071 |
| reg 10 = 0x94 | 4 | lz-agua | 1394 | 9067 |
| reg 10 = 0x95 | 4 | lz-agua | 1351 | 9019 |
| reg 10 = 0x97 | 1 | lz-agua | 1429 | 8386 |
| reg 10 = 0x98 | 3 | lz-agua | 1204 | 8186 |
| reg 10 = 0x9a | 1 | lz-agua | 1607 | 8243 |
| reg 10 = 0x9b | 33 | lz-agua | 1353 | 9184 |
| reg 10 = 0x9c | 1 | lz-agua | 1430 | 8175 |
| reg 10 = 0x9e | 3 | lz-agua | 1203 | 8505 |
| reg 10 = 0x9f | 2 | lz-agua | 1355 | 9079 |
| reg 10 = 0xa0 | 2 | lz-agua | 1431 | 8515 |
| reg 10 = 0xa1 | 2 | lz-agua | 1356 | 9134 |
| reg 10 = 0xa2 | 1 | lz-agua | 1202 | 8421 |
| reg 10 = 0xa3 | 2 | lz-agua | 1357 | 9216 |
| reg 10 = 0xa4 | 1 | lz-agua | 1432 | 8571 |
| reg 10 = 0xa5 | 2 | lz-agua | 1358 | 9203 |
| reg 10 = 0xa6 | 2 | lz-agua | 1201 | 8399 |
| reg 10 = 0xa7 | 1 | lz-agua | 1359 | 8722 |
| reg 10 = 0xa8 | 8 | lz-agua | 1360 | 9181 |
| reg 10 = 0xa9 | 6 | lz-agua | 1361 | 9255 |
| reg 10 = 0xaa | 8 | lz-agua | 1200 | 8492 |
| reg 10 = 0xab | 10 | lz-agua | 1363 | 8416 |
| reg 10 = 0xac | 64 | lz-agua | 930 | 8458 |
| reg 10 = 0xad | 37 | lz-agua | 977 | 6871 |
| reg 10 = 0xae | 23 | lz-agua | 987 | 8149 |
| reg 10 = 0xaf | 18 | lz-agua | 995 | 8411 |
| reg 10 = 0xb0 | 21 | lz-agua | 1001 | 8413 |
| reg 10 = 0xb1 | 30 | lz-agua | 1006 | 9065 |
| reg 10 = 0xb2 | 28 | lz-agua | 1011 | 9071 |
| reg 10 = 0xb3 | 17 | lz-agua | 1016 | 8721 |
| reg 10 = 0xb4 | 16 | lz-agua | 1020 | 8402 |
| reg 10 = 0xb5 | 11 | lz-agua | 1024 | 8500 |
| reg 10 = 0xb6 | 11 | lz-agua | 1029 | 8347 |
| reg 10 = 0xb7 | 12 | lz-agua | 1034 | 8350 |
| reg 10 = 0xb8 | 49 | lz-agua | 1039 | 8295 |
| reg 10 = 0xb9 | 23 | lz-agua | 1045 | 8245 |
| reg 10 = 0xba | 26 | lz-agua | 1053 | 8889 |
| reg 10 = 0xbb | 46 | lz-agua | 1063 | 8877 |
| reg 10 = 0xbc | 8 | lz-agua | 1617 | 8400 |
| reg 10 = 0xbd | 2 | lz-agua | 1615 | 8428 |
| reg 10 = 0xdf | 6411 | boot-demos | 61 | 6793 |
| reg 10 = 0xff | 2 | boot-demos | 0 | 26 |
| reg 11 = 0x00 | 8 | boot-demos | 0 | 29 |
| reg 11 = 0x03 | 8 | boot-demos | 354 | 7667 |
| reg 12 = 0x81 | 4 | boot-demos | 0 | 31 |
| reg 13 = 0x37 | 2 | boot-demos | 0 | 33 |
| reg 13 = 0x3f | 2 | boot-demos | 55 | 5046 |
| reg 14 = 0x00 | 4 | boot-demos | 0 | 35 |
| reg 15 = 0x01 | 36 | boot-demos | 0 | 38 |
| reg 15 = 0x02 | 38 | boot-demos | 2 | 6661 |
| reg 16 = 0x01 | 12 | boot-demos | 0 | 40 |
| reg 17 = 0x00 | 4 | boot-demos | 0 | 42 |
| reg 18 = 0x00 | 8 | boot-demos | 0 | 44 |
| reg 19 = 0x40 | 15904 | boot-demos | 61 | 6551 |
| reg 19 = 0x70 | 1936 | boot-demos | 1141 | 19 |
| reg 19 = 0xc0 | 7938 | boot-demos | 61 | 6637 |
| reg 19 = 0xff | 36 | boot-demos | 0 | 47 |
| reg 20 = 0x00 | 7966 | boot-demos | 61 | 6550 |
| reg 20 = 0x01 | 17812 | boot-demos | 61 | 6559 |
| reg 20 = 0x0f | 32 | boot-demos | 82 | 906 |
| reg 20 = 0xff | 4 | boot-demos | 0 | 49 |
| reg 21 = 0x00 | 17814 | boot-demos | 0 | 51 |
| reg 21 = 0x40 | 140 | lz-agua | 1860 | 8296 |
| reg 21 = 0x80 | 7826 | boot-demos | 61 | 6553 |
| reg 22 = 0x00 | 2 | boot-demos | 0 | 53 |
| reg 22 = 0xe4 | 1936 | boot-demos | 1141 | 20 |
| reg 22 = 0xe6 | 7938 | boot-demos | 61 | 6638 |
| reg 22 = 0xfc | 7938 | boot-demos | 61 | 6561 |
| reg 22 = 0xfd | 7966 | boot-demos | 61 | 6552 |
| reg 23 = 0x7f | 25778 | boot-demos | 61 | 6554 |
| reg 23 = 0x80 | 36 | boot-demos | 0 | 56 |

## VDP: comandos de endereço

| Item | Vezes | Primeiro caso | Quadro | seq |
| --- | --- | --- | --- | --- |
| escrita em CRAM | 9198 | boot-demos | 2 | 6663 |
| escrita em VRAM | 89042 | boot-demos | 0 | 59 |
| escrita em VSRAM | 7944 | boot-demos | 2 | 6767 |

## VDP: DMA disparado

| Item | Vezes | Primeiro caso | Quadro | seq |
| --- | --- | --- | --- | --- |
| 68000 para VDP | 25777 | boot-demos | 61 | 6557 |
| preenchimento de VRAM | 36 | boot-demos | 0 | 59 |

## VDP: escritas na porta de dados (bytes por escrita)

| Item | Vezes | Primeiro caso | Quadro | seq |
| --- | --- | --- | --- | --- |
| 2 byte(s) | 837676 | boot-demos | 0 | 60 |

## YM2612: registradores escritos (porta, registrador)

| Item | Vezes | Primeiro caso | Quadro | seq |
| --- | --- | --- | --- | --- |
| porta 0, reg 0x27 | 8 | boot-demos | 61 | 6869 |
| porta 0, reg 0x28 | 4609 | boot-demos | 61 | 7167 |
| porta 0, reg 0x2a | 975308 | boot-demos | 164 | 9274 |
| porta 0, reg 0x2b | 366 | boot-demos | 61 | 6841 |
| porta 0, reg 0x30 | 9 | boot-demos | 505 | 1533 |
| porta 0, reg 0x31 | 10 | boot-demos | 505 | 2270 |
| porta 0, reg 0x32 | 14 | boot-demos | 505 | 3008 |
| porta 0, reg 0x34 | 9 | boot-demos | 505 | 1586 |
| porta 0, reg 0x35 | 10 | boot-demos | 505 | 2324 |
| porta 0, reg 0x36 | 14 | boot-demos | 505 | 3062 |
| porta 0, reg 0x38 | 9 | boot-demos | 505 | 1559 |
| porta 0, reg 0x39 | 10 | boot-demos | 505 | 2297 |
| porta 0, reg 0x3a | 14 | boot-demos | 505 | 3035 |
| porta 0, reg 0x3c | 9 | boot-demos | 505 | 1613 |
| porta 0, reg 0x3d | 10 | boot-demos | 505 | 2350 |
| porta 0, reg 0x3e | 14 | boot-demos | 505 | 3088 |
| porta 0, reg 0x40 | 25 | boot-demos | 61 | 7337 |
| porta 0, reg 0x41 | 26 | boot-demos | 61 | 7566 |
| porta 0, reg 0x42 | 30 | boot-demos | 61 | 7796 |
| porta 0, reg 0x44 | 39 | boot-demos | 61 | 7396 |
| porta 0, reg 0x45 | 26 | boot-demos | 61 | 7625 |
| porta 0, reg 0x46 | 30 | boot-demos | 61 | 7853 |
| porta 0, reg 0x48 | 39 | boot-demos | 61 | 7454 |
| porta 0, reg 0x49 | 26 | boot-demos | 61 | 7683 |
| porta 0, reg 0x4a | 37 | boot-demos | 61 | 7909 |
| porta 0, reg 0x4c | 63 | boot-demos | 61 | 7510 |
| porta 0, reg 0x4d | 52 | boot-demos | 61 | 7738 |
| porta 0, reg 0x4e | 64 | boot-demos | 61 | 7968 |
| porta 0, reg 0x50 | 9 | boot-demos | 505 | 1640 |
| porta 0, reg 0x51 | 10 | boot-demos | 505 | 2377 |
| porta 0, reg 0x52 | 14 | boot-demos | 505 | 3115 |
| porta 0, reg 0x54 | 9 | boot-demos | 505 | 1692 |
| porta 0, reg 0x55 | 10 | boot-demos | 505 | 2431 |
| porta 0, reg 0x56 | 14 | boot-demos | 505 | 3169 |
| porta 0, reg 0x58 | 9 | boot-demos | 505 | 1666 |
| porta 0, reg 0x59 | 10 | boot-demos | 505 | 2405 |
| porta 0, reg 0x5a | 14 | boot-demos | 505 | 3143 |
| porta 0, reg 0x5c | 9 | boot-demos | 505 | 1720 |
| porta 0, reg 0x5d | 10 | boot-demos | 505 | 2458 |
| porta 0, reg 0x5e | 14 | boot-demos | 505 | 3196 |
| porta 0, reg 0x60 | 9 | boot-demos | 505 | 1746 |
| porta 0, reg 0x61 | 10 | boot-demos | 505 | 2484 |
| porta 0, reg 0x62 | 14 | boot-demos | 505 | 3222 |
| porta 0, reg 0x64 | 9 | boot-demos | 505 | 1800 |
| porta 0, reg 0x65 | 10 | boot-demos | 505 | 2538 |
| porta 0, reg 0x66 | 14 | boot-demos | 505 | 3276 |
| porta 0, reg 0x68 | 9 | boot-demos | 505 | 1773 |
| porta 0, reg 0x69 | 10 | boot-demos | 505 | 2511 |
| porta 0, reg 0x6a | 14 | boot-demos | 505 | 3249 |
| porta 0, reg 0x6c | 9 | boot-demos | 505 | 1827 |
| porta 0, reg 0x6d | 10 | boot-demos | 505 | 2564 |
| porta 0, reg 0x6e | 14 | boot-demos | 505 | 3302 |
| porta 0, reg 0x70 | 9 | boot-demos | 505 | 1854 |
| porta 0, reg 0x71 | 10 | boot-demos | 505 | 2591 |
| porta 0, reg 0x72 | 14 | boot-demos | 505 | 3329 |
| porta 0, reg 0x74 | 9 | boot-demos | 505 | 1907 |
| porta 0, reg 0x75 | 10 | boot-demos | 505 | 2644 |
| porta 0, reg 0x76 | 14 | boot-demos | 505 | 3382 |
| porta 0, reg 0x78 | 9 | boot-demos | 505 | 1880 |
| porta 0, reg 0x79 | 10 | boot-demos | 505 | 2617 |
| porta 0, reg 0x7a | 14 | boot-demos | 505 | 3355 |
| porta 0, reg 0x7c | 9 | boot-demos | 505 | 1933 |
| porta 0, reg 0x7d | 10 | boot-demos | 505 | 2670 |
| porta 0, reg 0x7e | 14 | boot-demos | 505 | 3408 |
| porta 0, reg 0x80 | 9 | boot-demos | 505 | 1960 |
| porta 0, reg 0x81 | 10 | boot-demos | 505 | 2697 |
| porta 0, reg 0x82 | 14 | boot-demos | 505 | 3435 |
| porta 0, reg 0x84 | 9 | boot-demos | 505 | 2013 |
| porta 0, reg 0x85 | 10 | boot-demos | 505 | 2751 |
| porta 0, reg 0x86 | 14 | boot-demos | 505 | 3489 |
| porta 0, reg 0x88 | 9 | boot-demos | 505 | 1986 |
| porta 0, reg 0x89 | 10 | boot-demos | 505 | 2724 |
| porta 0, reg 0x8a | 14 | boot-demos | 505 | 3462 |
| porta 0, reg 0x8c | 9 | boot-demos | 505 | 2040 |
| porta 0, reg 0x8d | 10 | boot-demos | 505 | 2777 |
| porta 0, reg 0x8e | 14 | boot-demos | 505 | 3515 |
| porta 0, reg 0xa0 | 810 | boot-demos | 580 | 154 |
| porta 0, reg 0xa1 | 448 | boot-demos | 565 | 166 |
| porta 0, reg 0xa2 | 1099 | boot-demos | 565 | 277 |
| porta 0, reg 0xa4 | 810 | boot-demos | 580 | 129 |
| porta 0, reg 0xa5 | 448 | boot-demos | 565 | 140 |
| porta 0, reg 0xa6 | 1099 | boot-demos | 565 | 252 |
| porta 0, reg 0xb0 | 9 | boot-demos | 505 | 1507 |
| porta 0, reg 0xb1 | 10 | boot-demos | 505 | 2245 |
| porta 0, reg 0xb2 | 14 | boot-demos | 505 | 2983 |
| porta 0, reg 0xb4 | 38 | boot-demos | 505 | 2164 |
| porta 0, reg 0xb5 | 10 | boot-demos | 505 | 2898 |
| porta 0, reg 0xb6 | 17 | boot-demos | 505 | 3636 |
| porta 1, reg 0x30 | 85 | boot-demos | 505 | 3742 |
| porta 1, reg 0x31 | 101 | boot-demos | 505 | 4424 |
| porta 1, reg 0x34 | 85 | boot-demos | 505 | 3789 |
| porta 1, reg 0x35 | 101 | boot-demos | 505 | 4472 |
| porta 1, reg 0x38 | 85 | boot-demos | 505 | 3766 |
| porta 1, reg 0x39 | 101 | boot-demos | 505 | 4447 |
| porta 1, reg 0x3c | 85 | boot-demos | 505 | 3812 |
| porta 1, reg 0x3d | 101 | boot-demos | 505 | 4499 |
| porta 1, reg 0x40 | 101 | boot-demos | 61 | 7367 |
| porta 1, reg 0x41 | 117 | boot-demos | 61 | 7597 |
| porta 1, reg 0x42 | 24 | boot-demos | 61 | 7826 |
| porta 1, reg 0x44 | 101 | boot-demos | 61 | 7426 |
| porta 1, reg 0x45 | 117 | boot-demos | 61 | 7655 |
| porta 1, reg 0x46 | 24 | boot-demos | 61 | 7881 |
| porta 1, reg 0x48 | 116 | boot-demos | 61 | 7481 |
| porta 1, reg 0x49 | 123 | boot-demos | 61 | 7710 |
| porta 1, reg 0x4a | 24 | boot-demos | 61 | 7940 |
| porta 1, reg 0x4c | 204 | boot-demos | 61 | 7540 |
| porta 1, reg 0x4d | 150 | boot-demos | 61 | 7769 |
| porta 1, reg 0x4e | 24 | boot-demos | 61 | 7998 |
| porta 1, reg 0x50 | 85 | boot-demos | 505 | 3836 |
| porta 1, reg 0x51 | 101 | boot-demos | 505 | 4523 |
| porta 1, reg 0x54 | 85 | boot-demos | 505 | 3883 |
| porta 1, reg 0x55 | 101 | boot-demos | 505 | 4571 |
| porta 1, reg 0x58 | 85 | boot-demos | 505 | 3860 |
| porta 1, reg 0x59 | 101 | boot-demos | 505 | 4547 |
| porta 1, reg 0x5c | 85 | boot-demos | 505 | 3906 |
| porta 1, reg 0x5d | 101 | boot-demos | 505 | 4597 |
| porta 1, reg 0x60 | 85 | boot-demos | 505 | 3930 |
| porta 1, reg 0x61 | 101 | boot-demos | 505 | 4621 |
| porta 1, reg 0x64 | 85 | boot-demos | 505 | 3978 |
| porta 1, reg 0x65 | 101 | boot-demos | 505 | 4670 |
| porta 1, reg 0x68 | 85 | boot-demos | 505 | 3954 |
| porta 1, reg 0x69 | 101 | boot-demos | 505 | 4648 |
| porta 1, reg 0x6c | 85 | boot-demos | 505 | 4002 |
| porta 1, reg 0x6d | 101 | boot-demos | 505 | 4696 |
| porta 1, reg 0x70 | 85 | boot-demos | 505 | 4025 |
| porta 1, reg 0x71 | 101 | boot-demos | 505 | 4719 |
| porta 1, reg 0x74 | 85 | boot-demos | 505 | 4072 |
| porta 1, reg 0x75 | 101 | boot-demos | 505 | 4767 |
| porta 1, reg 0x78 | 85 | boot-demos | 505 | 4049 |
| porta 1, reg 0x79 | 101 | boot-demos | 505 | 4743 |
| porta 1, reg 0x7c | 85 | boot-demos | 505 | 4095 |
| porta 1, reg 0x7d | 101 | boot-demos | 505 | 4794 |
| porta 1, reg 0x80 | 85 | boot-demos | 505 | 4120 |
| porta 1, reg 0x81 | 101 | boot-demos | 505 | 4817 |
| porta 1, reg 0x84 | 85 | boot-demos | 505 | 4170 |
| porta 1, reg 0x85 | 101 | boot-demos | 505 | 4864 |
| porta 1, reg 0x88 | 85 | boot-demos | 505 | 4147 |
| porta 1, reg 0x89 | 101 | boot-demos | 505 | 4840 |
| porta 1, reg 0x8c | 85 | boot-demos | 505 | 4194 |
| porta 1, reg 0x8d | 101 | boot-demos | 505 | 4888 |
| porta 1, reg 0xa0 | 1699 | boot-demos | 565 | 390 |
| porta 1, reg 0xa1 | 605 | boot-demos | 580 | 593 |
| porta 1, reg 0xa4 | 1699 | boot-demos | 565 | 363 |
| porta 1, reg 0xa5 | 605 | boot-demos | 580 | 568 |
| porta 1, reg 0xb0 | 85 | boot-demos | 505 | 3719 |
| porta 1, reg 0xb1 | 101 | boot-demos | 505 | 4402 |
| porta 1, reg 0xb4 | 124 | boot-demos | 505 | 4314 |
| porta 1, reg 0xb5 | 139 | boot-demos | 505 | 5007 |
| porta 1, reg 0xb6 | 8 | boot-demos | 505 | 1248 |

## YM2612: uso do DAC

| Item | Vezes | Primeiro caso | Quadro | seq |
| --- | --- | --- | --- | --- |
| escritas no registrador $2A (DAC) | 975308 | boot-demos | 164 | 9274 |

## PSG

| Item | Vezes | Primeiro caso | Quadro | seq |
| --- | --- | --- | --- | --- |
| dado de tom | 1605 | boot-demos | 565 | 479 |
| ruído, canal 3 | 53 | boot-demos | 505 | 5112 |
| tom, canal 0 | 885 | boot-demos | 1060 | 7221 |
| tom, canal 1 | 193 | boot-demos | 1060 | 7282 |
| tom, canal 2 | 527 | boot-demos | 565 | 478 |
| volume, canal 0 | 1713 | boot-demos | 2 | 6847 |
| volume, canal 1 | 2028 | boot-demos | 2 | 6852 |
| volume, canal 2 | 64 | boot-demos | 3 | 4 |
| volume, canal 3 | 3838 | boot-demos | 3 | 10 |

## Interrupções do 68000 (nível)

| Item | Vezes | Primeiro caso | Quadro | seq |
| --- | --- | --- | --- | --- |
| nível 4 | 1530 | lz-agua | 771 | 1678 |
| nível 6 | 8068 | boot-demos | 61 | 6517 |

## Z80: BUSREQ e RESET (sinal 0=busreq 1=reset, ativo)

| Item | Vezes | Primeiro caso | Quadro | seq |
| --- | --- | --- | --- | --- |
| sinal 0, ativo 0 | 16910 | boot-demos | 0 | 183 |
| sinal 0, ativo 1 | 16923 | boot-demos | 0 | 61 |
| sinal 1, ativo 0 | 16 | boot-demos | 0 | 62 |
| sinal 1, ativo 1 | 10 | boot-demos | 0 | 182 |
