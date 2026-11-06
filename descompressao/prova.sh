#!/bin/sh
# Prova da Etapa 3: compara os descompressores do nucleo com o oraculo, bloco a bloco.
#
# Roda no computador do Marcelo, FORA do Claude Code (usa a ROM real). A saida bruta fica em
# $SONIC1_EXT_DIR/privado/descompressao. O que aparece no terminal e so o relatorio: rotulos,
# enderecos, tamanhos, hashes e vereditos. Nunca bytes.
#
# Pre-requisitos:
#   1. refs/rotulos.txt refeito na base 16 (sh oracle/scripts/monta-disasm.sh montar ... 16).
#   2. patch docs/pedidos/descomprimir-oraculo.patch aplicado em oracle/.
#
# Uso: sh descompressao/prova.sh "<caminho da ROM REV01>"
set -eu

rom=${1:?uso: sh descompressao/prova.sh ROM}
repo=$(cd "$(dirname "$0")/.." && pwd -P)
ext=${SONIC1_EXT_DIR:?defina SONIC1_EXT_DIR}
saida=$ext/privado/descompressao

mkdir -p "$saida"
case "$(cd "$saida" && pwd -P)/" in
    "$repo"/*) echo "Erro: a saida esta dentro do repositorio." >&2; exit 1 ;;
esac

esperado=$(sed -n 's/^sha1: *//p' "$repo/refs/rom-hashes.txt")
obtido=$(sha1sum "$rom" | cut -d' ' -f1)
[ "$esperado" = "$obtido" ] || { echo "Erro: a ROM nao e a REV01 de referencia." >&2; exit 1; }
echo "ROM REV01 conferida (sha1 $obtido)."

# O mapa corrigido tem a variavel da RAM que o descompressor de arte grava.
grep -q '^v_plc_ptrnemcode	' "$repo/refs/rotulos.txt" || {
    echo "Erro: refs/rotulos.txt sem v_plc_ptrnemcode. Refaca o mapa com a base 16:" >&2
    echo "  sh oracle/scripts/monta-disasm.sh montar s1built.bin sonic.lst 16" >&2
    exit 1
}
grep -q 'descomprimir_lote' "$repo/oracle/runner/src/descompressao.rs" 2>/dev/null || {
    echo "Erro: aplique antes o patch: patch -p1 < docs/pedidos/descomprimir-oraculo.patch" >&2
    exit 1
}

sh "$repo/oracle/scripts/prepara.sh" >/dev/null
base=${ORACLE_DIR:-$ext/trabalho/oracle}
export CARGO_TARGET_DIR="$base/alvo"
(cd "$repo/oracle/runner" && cargo build --release -q)

"$CARGO_TARGET_DIR/release/oraculo" descomprimir --rom "$rom" \
    --rotulos "$repo/refs/rotulos.txt" --manifesto "$repo/descompressao/manifesto.txt" \
    --saida "$saida/oraculo.tsv" > /dev/null
echo "Oraculo terminou ($(grep -vc '^#' "$saida/oraculo.tsv") linhas)."

cd "$repo"
cargo xtask descompressao-relatorio --rom "$rom" --rotulos refs/rotulos.txt \
    --manifesto descompressao/manifesto.txt --oraculo "$saida/oraculo.tsv" \
    --saida "$saida/relatorio.tsv"
