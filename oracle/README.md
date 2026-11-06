# Oráculo

O oráculo é o jgenesis (GPLv3), em um commit fixado, com patches só de observação, rodando sem tela. Este diretório inteiro é GPLv3 (ver `LICENSE`) e **nunca entra no build do site nem do crate `core`**.

Detalhes, evidência e limites: `docs/conhecimento/oraculo.md`.

| Caminho | O que é |
| --- | --- |
| `jgenesis-commit.txt` | Commit fixado do jgenesis |
| `patches/` | Patches sobre o commit fixado e o hash do resultado (`SHA1`) |
| `runner/` | O executável `oraculo` e a biblioteca (raiz do próprio workspace) |
| `scripts/prepara.sh` | Clona, aplica os patches e cria o link `target/oracle-fonte` (ignorado pelo git) |
| `scripts/testa.sh` | Testes com ROMs sintéticas |
| `scripts/gera-patches.sh` | Regenera os patches a partir do clone |
| `scripts/monta-disasm.sh` | Monta o disassembly e gera o mapa de rótulos (só o Marcelo, no host) |
| `scripts/roda-oraculo.sh` | Prova de determinismo com qualquer ROM (só o Marcelo, no host, para a ROM real) |

A IA nunca roda o oráculo com a ROM real. Traces e estados reais ficam em `$SONIC1_EXT_DIR/privado`, que não é montada no contêiner.
