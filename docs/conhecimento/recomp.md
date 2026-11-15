# Recompilação e referências

Registro da investigação de 13/11/2026. Os repositórios estudados foram clonados em `/tmp/websonic-referencias-20261113`. Os hashes abaixo identificam as versões examinadas, não provam o comportamento do WebSonic.

- SonicTheHedgehogRecomp `e82c95b`: `game/game.toml` e `game/sonic1.disasm_*.toml` registram pontos de entrada e tabelas; `game/sonic1_video.c` mantém estados sucessivos para interpolação. A versão qualificada pelo projeto é a REV00; o WebSonic usa REV01.
- SonicTheHedgehog2Recomp `c54a3fd`: `game/CUSTOM-VIDEO.md` descreve a separação do desenho ampliado e da lógica original, além dos limites de carregamento de objetos.
- O motor compartilhado segagenesisrecomp usa descoberta de funções e emissão antecipada em `external/m68k-recomp-core/profiles/genesis/`. A referência inclui fallback interpretado para alguns destinos, que não é usado pelo gerador planejado aqui. Seu código está sob PolyForm Noncommercial, portanto não foi incorporado ao crate MIT.
- NX2navi `d4778b1`: `recomp/lifter.py` separa decodificação e emissão; `tests/diff/lift.py` compara instruções com um processador de referência. As alterações próprias do repositório não autorizam cópia sem permissão.
- Hookshot `b2d03d5`: `nx2/lift.py` inventaria instruções, destinos indiretos e lacunas da tradução. Sua implementação AArch64 e seu backend Vulkan não são diretamente aplicáveis ao alvo web.
- gt4-staticrecomp `b0008e8`: `native/hw_gs/cpuref/` mantém um caminho de referência e `scripts/hw_compare.py` compara quadros capturados. Para o modo fiel do WebSonic, o critério continua sendo igualdade na cor de 9 bits, não tolerância RGB.

O material de Sonic Retro indicado para disassemblies retornou HTTP 403 nesta sessão. A fonte local de Sonic 1 é o disassembly fixado em `refs/disasm-commit.txt`, acompanhado da ROM REV01 identificada por `refs/rom-hashes.txt`.

Decisões de Marcelo nesta conversa: permitir geração antecipada de lógica Rust e estado interno de registradores, compor a imagem em WebGL2 e seguir o ritmo NTSC real. Os dados gráficos, sonoros e tabelas continuam sendo lidos da ROM do jogador. O gerador deve falhar explicitamente para instruções ou destinos que ainda não conhece.
