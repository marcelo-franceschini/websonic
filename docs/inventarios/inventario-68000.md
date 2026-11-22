# Inventário de instruções do 68000

Gerado por `cargo xtask inventario-68000`. Só nome de classe, tamanho e contagem.

Fonte: `oraculo rodar --instrucoes`, casos: boot-demos, demos-quatro, lz-agua, partida-ghz1.

A classe é o nome da variante do decodificador do oráculo (jgenesis). O tamanho é `-` quando a classe não tem.

56 classes distintas.

| classe | tamanho | vezes |
| --- | --- | --- |
| Add | Byte | 1995773 |
| Add | LongWord | 2411293 |
| Add | Word | 14149397 |
| And | Byte | 435240 |
| And | LongWord | 566 |
| And | Word | 6568719 |
| ArithmeticShiftMemory | - | 13 |
| ArithmeticShiftRegister | - | 1538956 |
| BitTest | - | 2789491 |
| BitTestAndChange | - | 2250 |
| BitTestAndClear | - | 559652 |
| BitTestAndSet | - | 196324 |
| Branch | - | 102451760 |
| BranchDecrement | - | 27304097 |
| BranchToSubroutine | - | 2185293 |
| Clear | - | 214236 |
| Compare | Byte | 2600443 |
| Compare | LongWord | 1065348 |
| Compare | Word | 3254798 |
| DivideSigned | - | 21060 |
| DivideUnsigned | - | 25270 |
| ExchangeData | - | 142 |
| ExclusiveOr | Byte | 177340 |
| ExclusiveOr | LongWord | 253962 |
| ExclusiveOr | Word | 76585 |
| Extend | - | 2858698 |
| Jump | - | 900287 |
| JumpToSubroutine | - | 1707722 |
| LoadEffectiveAddress | - | 7188290 |
| LogicalShiftRegister | - | 6338356 |
| Move | Byte | 18126534 |
| Move | LongWord | 9725318 |
| Move | Word | 17926885 |
| MoveFromSr | - | 215960 |
| MoveMultiple | - | 713788 |
| MoveQuick | - | 4458640 |
| MoveToCcr | - | 215960 |
| MoveToSr | - | 27670 |
| MoveUsp | - | 4 |
| MultiplySigned | - | 145831 |
| MultiplyUnsigned | - | 18608 |
| Negate | Byte | 16172 |
| Negate | Word | 1399350 |
| NoOp | - | 12387850 |
| Not | - | 62617 |
| Or | Byte | 2262147 |
| Or | Word | 143802 |
| Return | - | 3719460 |
| ReturnFromException | - | 27942 |
| RotateRegister | - | 368352 |
| RotateThruExtendRegister | - | 39340 |
| Subtract | Byte | 1063717 |
| Subtract | LongWord | 36146 |
| Subtract | Word | 7084807 |
| Swap | - | 2684253 |
| Test | - | 80504142 |
