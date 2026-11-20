#!/bin/sh
# Monta o disassembly fixado e gera o mapa de rotulos. Roda no computador do Marcelo, FORA do
# Claude Code. A IA nunca roda este script e nunca ve o binario nem a listagem.
#
# A montagem acontece numa copia em $SONIC1_EXT_DIR/privado/montagem (pasta que nao e montada
# no contêiner). O que sai para o repositorio e para a conversa e so metadado: hashes, nomes de
# arquivo, tamanhos e o refs/rotulos.txt (nome, tipo e valor).
#
# Dois modos:
#   sh oracle/scripts/monta-disasm.sh reconhecer
#       Monta e lista os arquivos que o build criou (nome e tamanho). Serve para descobrir qual
#       e a ROM montada e qual e a listagem com a tabela de simbolos.
#   sh oracle/scripts/monta-disasm.sh montar ROM_RELATIVA LISTAGEM_RELATIVA [BASE]
#       Monta, confere a ROM contra refs/rom-hashes.txt (IDENTICA ou DIFERENTE) e, se for
#       identica, gera refs/rotulos.txt. BASE e a base dos numeros da tabela (10 ou 16, padrao 10).
#
# Variavel opcional: MONTAR_CMD (padrao "lua build.lua"), o comando de build do disassembly.
set -eu

modo=${1:-}
ext=${SONIC1_EXT_DIR:?defina SONIC1_EXT_DIR}
repo=$(cd "$(dirname "$0")/../.." && pwd -P)
disasm=$ext/disasm
texto=$ext/disasm-texto
privado=$ext/privado
montagem=$privado/montagem

# A pasta privada nunca pode ficar dentro do repositorio.
mkdir -p "$privado"
case "$(cd "$privado" && pwd -P)/" in
    "$repo"/*) echo "Erro: $privado esta dentro do repositorio." >&2; exit 1 ;;
esac

# 1. O commit do disassembly tem de ser o fixado.
fixado=$(sed -n 's/^commit: *//p' "$repo/refs/disasm-commit.txt")
if [ -d "$disasm/.git" ]; then
    atual=$(git -C "$disasm" rev-parse HEAD)
    [ "$atual" = "$fixado" ] || { echo "Erro: o disassembly esta em $atual, e o fixado e $fixado." >&2; exit 1; }
    echo "Disassembly no commit fixado ($fixado)."
else
    echo "AVISO: $disasm nao e um repositorio git; o commit nao foi conferido." >&2
fi

# 2. Revision = 1 (REV01).
grep -q '^Revision = 1' "$disasm/sonic.asm" || { echo "Erro: sonic.asm nao tem 'Revision = 1'." >&2; exit 1; }

# 3. Copia limpa e build.
rm -rf "$montagem"
mkdir -p "$montagem"
cp -a "$disasm/." "$montagem/"
rm -rf "$montagem/.git"
: > "$privado/.marca"
sleep 1
if (cd "$montagem" && ${MONTAR_CMD:-lua build.lua}) > "$privado/montagem.log" 2>&1; then
    echo "Build terminou com sucesso (log em $privado/montagem.log, nao colado)."
else
    echo "Erro: o build falhou. Veja $privado/montagem.log (nao cole o conteudo sem olhar)." >&2
    exit 1
fi

case "$modo" in
reconhecer)
    echo "Arquivos criados pelo build (nome<TAB>tamanho):"
    (cd "$montagem" && find . -type f -newer "$privado/.marca" -printf '%P\t%s\n' | sort)
    ;;
montar)
    rom=${2:?uso: monta-disasm.sh montar ROM_RELATIVA LISTAGEM_RELATIVA [BASE]}
    lst=${3:?uso: monta-disasm.sh montar ROM_RELATIVA LISTAGEM_RELATIVA [BASE]}
    base=${4:-10}
    esp_sha1=$(sed -n 's/^sha1: *//p' "$repo/refs/rom-hashes.txt")
    esp_md5=$(sed -n 's/^md5: *//p' "$repo/refs/rom-hashes.txt")
    got_sha1=$(sha1sum "$montagem/$rom" | cut -d' ' -f1)
    got_md5=$(md5sum "$montagem/$rom" | cut -d' ' -f1)
    echo "sha1 montada:    $got_sha1"
    echo "sha1 referencia: $esp_sha1"
    echo "md5 montada:     $got_md5"
    echo "md5 referencia:  $esp_md5"
    if [ "$got_sha1" = "$esp_sha1" ] && [ "$got_md5" = "$esp_md5" ]; then
        echo "ROM montada: IDENTICA a de referencia."
    else
        echo "ROM montada: DIFERENTE da de referencia." >&2
        exit 1
    fi
    (cd "$repo" && cargo xtask rotulos --entrada "$montagem/$lst" --fontes "$texto" \
        --saida "$repo/refs/rotulos.txt" --base "$base")
    tam=$(wc -c < "$repo/refs/rotulos.txt")
    limite=$(sed -n 's/^max_bytes: *//p' "$repo/refs/limite-tamanho.txt")
    echo "refs/rotulos.txt: $tam bytes (limite do hook: $limite), sha1 $(sha1sum "$repo/refs/rotulos.txt" | cut -d' ' -f1)"
    [ "$tam" -lt "$limite" ] || echo "AVISO: rotulos.txt passa do limite do hook; divida por categoria ou abra excecao." >&2
    ;;
*)
    echo "uso: monta-disasm.sh reconhecer | montar ROM_REL LISTAGEM_REL [BASE]" >&2
    exit 2
    ;;
esac
