#!/bin/sh
# Etapa 4, passo do host (Marcelo, fora do Claude): oraculo com video, traces do corpus com
# imagem e DMA, perfil da Labyrinth, estado e imagem do renderizador contra o oraculo.
# Imprime so nomes, numeros, hashes e coordenadas.
# Uso: sh scripts/etapa4-host.sh "$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen"
set -eu
rom=${1:?uso: etapa4-host.sh ROM}
repo=$(cd "$(dirname "$0")/.." && pwd -P)
ext=${SONIC1_EXT_DIR:-$HOME/sonic1-ext}
traces=$ext/privado/traces
mkdir -p "$traces"
cd "$repo"

esperado=$(sed -n 's/^sha1: *//p' refs/rom-hashes.txt)
obtido=$(sha1sum "$rom" | cut -d' ' -f1)
[ "$esperado" = "$obtido" ] || { echo "Erro: a ROM nao e a REV01." >&2; exit 1; }

echo "=== TESTES DO ORACULO (com o video novo) ==="
sh oracle/scripts/testa.sh 2>&1 | tail -4

echo "=== OFICINA DO ORACULO ==="
sh oracle/scripts/prepara.sh | tail -1
export CARGO_TARGET_DIR="$ext/trabalho/oracle/alvo"
(cd oracle/runner && cargo build --release -q)
bin=$CARGO_TARGET_DIR/release/oraculo
unset CARGO_TARGET_DIR

echo "=== PERFIL DA LABYRINTH (so as linhas em que modo, zona, ato ou demo mudam) ==="
"$bin" rodar --rom "$rom" --quadros 2000 --saida /dev/null --entradas corpus/lz-agua.s1in \
    --perfil-a-cada 60 --rotulos refs/rotulos.txt | grep '^perfil ' \
    | awk '{k=$3" "$4" "$5" "$6; if (k!=u) print; u=k}'

echo "=== CORPUS-TRACE (traces, imagem e DMA) ==="
for caso in lz-agua boot-demos; do
    cargo fidelidade corpus-trace --manter-trace --rom "$rom" --oraculo "$bin" \
        --rotulos refs/rotulos.txt --saida "$traces" --caso "$caso"
done | tee "$traces/resumo4.tsv"

echo "=== INVENTARIO (interrupcoes e DMA do corpus inteiro) ==="
cargo run -q -p xtask -- inventario --traces "$traces"
grep -E "nível|68000 para VDP|preenchimento|DAC" docs/inventario-hardware.md || true

for caso in boot-demos lz-agua; do
    entradas=
    [ "$caso" = lz-agua ] && entradas="--entradas corpus/lz-agua.s1in"
    echo "=== VIDEO-ESTADO: $caso ==="
    cargo run -q --release -p xtask -- video-estado --trace "$traces/$caso.a.s1t" --video "$traces/$caso.a.s1v"
    echo "=== VIDEO-COMPARAR: $caso ==="
    # shellcheck disable=SC2086
    cargo run -q --release -p xtask -- video-comparar --trace "$traces/$caso.a.s1t" \
        --video "$traces/$caso.a.s1v" --imagens "$traces/$caso.a.s1i" \
        --saida "$traces/$caso.video.tsv" --rom "$rom" --oraculo "$bin" \
        --estados "$traces" --caso "$caso" $entradas
    awk -F'\t' 'NR>1 && NF==4 {c[$4]++} END {for (k in c) print "veredito " k ": " c[k]}' "$traces/$caso.video.tsv"
done

echo "=== DESEMPENHO NATIVO ==="
WEBSONIC_BENCH=1 cargo test -p sonic1-video --release --test desempenho -- --nocapture 2>&1 | grep video-bench

echo "=== MANIFESTO (sem assinar) ==="
grep -hv '^determinismo' "$traces/resumo4.tsv" > "$traces/hashes4.tsv"
# Mantem os hashes do corpus anterior (boot-demos) e acrescenta o novo caso.
cargo fidelidade manifesto gerar --traces "$traces/hashes4.tsv"
echo "=== FIM ==="
