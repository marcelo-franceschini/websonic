#!/bin/sh
# Prova de determinismo do oraculo: roda a mesma ROM e as mesmas entradas duas vezes, compara os
# traces byte a byte (cmp) e pelo comparador, e imprime so resumos (hashes e contagens).
#
# Roda no computador do Marcelo, FORA do Claude Code. As saidas brutas (traces e estados) ficam
# em $SONIC1_EXT_DIR/privado/oraculo, que nao e montada no contêiner. A IA so ve o resumo que
# o Marcelo colar.
#
# Uso: sh oracle/scripts/roda-oraculo.sh ROM QUADROS [ENTRADAS]
# Com SEM_CONFERIR_ROM=1 (ROM sintetica), nao confere o SHA-1 da REV01.
set -eu

rom=${1:?uso: roda-oraculo.sh ROM QUADROS [ENTRADAS]}
quadros=${2:?uso: roda-oraculo.sh ROM QUADROS [ENTRADAS]}
entradas=${3:-}
aqui=$(cd "$(dirname "$0")/.." && pwd -P)
repo=$(cd "$aqui/.." && pwd -P)
ext=${SONIC1_EXT_DIR:?defina SONIC1_EXT_DIR}
saida=$ext/privado/oraculo

# A saida bruta nunca pode ficar dentro do repositorio nem na pasta de trabalho da IA.
mkdir -p "$saida"
saida_real=$(cd "$saida" && pwd -P)
case "$saida_real/" in
    "$repo"/*) echo "Erro: a saida ($saida_real) esta dentro do repositorio." >&2; exit 1 ;;
    "$(cd "$ext/trabalho" && pwd -P)"/*) echo "Erro: a saida esta na pasta de trabalho." >&2; exit 1 ;;
esac

if [ "${SEM_CONFERIR_ROM:-0}" != 1 ]; then
    esperado=$(sed -n 's/^sha1: *//p' "$repo/refs/rom-hashes.txt")
    obtido=$(sha1sum "$rom" | cut -d' ' -f1)
    [ "$esperado" = "$obtido" ] || { echo "Erro: a ROM nao e a REV01 de referencia." >&2; exit 1; }
    echo "ROM REV01 conferida (sha1 $obtido)."
fi

sh "$aqui/scripts/prepara.sh" >/dev/null
base=${ORACLE_DIR:-$ext/trabalho/oracle}
export CARGO_TARGET_DIR="$base/alvo"
(cd "$aqui/runner" && cargo build --release -q)
bin=$CARGO_TARGET_DIR/release/oraculo

extra=
[ -z "$entradas" ] || extra="--entradas $entradas"
for n in a b; do
    echo "--- execucao $n ---"
    # shellcheck disable=SC2086
    "$bin" rodar --rom "$rom" --quadros "$quadros" --saida "$saida/$n.s1t" $extra
done

echo "--- comparacao ---"
if cmp -s "$saida/a.s1t" "$saida/b.s1t"; then echo "cmp: arquivos identicos"; else echo "cmp: ARQUIVOS DIFEREM"; exit 1; fi
"$bin" comparar "$saida/a.s1t" "$saida/b.s1t"
echo "Prova de determinismo: PASSOU."
