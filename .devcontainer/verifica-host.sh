#!/bin/sh
# Roda no computador do Marcelo, antes de criar o contêiner (initializeCommand).
# Confere se as 4 variaveis existem e apontam para pastas. Nao lista nem le o conteudo delas.
set -eu

falhou=0
for var in SONIC1_ROM_DIR SONIC1_DISASM_DIR SONIC1_DISASM_TEXTO_DIR SONIC1_TRABALHO_DIR; do
    # Expansao indireta em POSIX: eval so sobre o nome da variavel, que e fixo acima.
    valor=$(eval "printf '%s' \"\${$var:-}\"")
    if [ -z "$valor" ]; then
        echo "Erro: a variavel $var nao esta definida. Veja o README." >&2
        falhou=1
    elif [ ! -d "$valor" ]; then
        echo "Erro: $var aponta para '$valor', que nao e uma pasta. Crie-a (veja o README)." >&2
        falhou=1
    fi
done
exit "$falhou"
