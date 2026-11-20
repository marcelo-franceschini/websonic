#!/bin/sh
# Etapa 5, bloco A (Marcelo, fora do Claude): aplica o patch do oraculo (inventarios e tempo),
# gera os inventarios do 68000, do barramento e das interrupcoes e os arquivos de tempo do corpus.
# Imprime so nomes, numeros, hashes e vereditos. Roda em qualquer shell: sh scripts/etapa5-bloco-a.sh
set -eu
repo=$(cd "$(dirname "$0")/.." && pwd -P)
cd "$repo"
ROM="$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen"
ext=${SONIC1_EXT_DIR:-$HOME/sonic1-ext}
acc=$ext/privado/etapa5
mkdir -p "$acc"

esperado=$(sed -n 's/^sha1: *//p' refs/rom-hashes.txt)
obtido=$(sha1sum "$ROM" | cut -d' ' -f1)
[ "$esperado" = "$obtido" ] || { echo "Erro: a ROM nao e a REV01." >&2; exit 1; }

echo "##### 1. PATCH DO ORACULO (patch 0004) E TESTES #####"
if git apply --check docs/pedidos/etapa5-oraculo.patch 2>/dev/null; then
    git apply docs/pedidos/etapa5-oraculo.patch && echo "etapa5-oraculo: aplicado"
else
    echo "etapa5-oraculo: ja aplicado (ou nao aplica; veja git status)"
fi
sh oracle/scripts/testa.sh 2>&1 | grep -E "test result|Testes do oraculo|error|FAILED"

echo "##### 2. ORACULO COM OS INVENTARIOS #####"
sh oracle/scripts/prepara.sh | tail -1
export CARGO_TARGET_DIR="$ext/trabalho/oracle/alvo"
(cd oracle/runner && cargo build --release -q)
bin=$CARGO_TARGET_DIR/release/oraculo
unset CARGO_TARGET_DIR

echo "##### 3. CASOS DO CORPUS (instrucoes, barramento e tempo) #####"
rodar() { # nome quadros [entradas]
    nome=$1; q=$2; ent=${3:-}
    if [ -n "$ent" ]; then set -- --entradas "$ent"; else set --; fi
    "$bin" rodar --rom "$ROM" --quadros "$q" --saida /dev/null "$@" \
        --instrucoes "$acc/$nome.ins" --barramento "$acc/$nome.bar" --tempo "$acc/$nome.s1tempo" \
        | grep -E '^(quadros|mclk_final)='
    echo "caso $nome: $(wc -c < "$acc/$nome.s1tempo") bytes de tempo"
}
rodar boot-demos 7200
rodar lz-agua 2000 corpus/lz-agua.s1in
rodar partida-ghz1 "$(wc -l < corpus/partida-ghz1.s1in)" corpus/partida-ghz1.s1in
rodar demos-quatro 11700

echo "##### 4. INVENTARIOS (docs/) #####"
cargo run -q -p xtask -- inventario-68000 $(for f in "$acc"/*.ins; do printf -- '--entrada %s ' "$f"; done)
cargo run -q -p xtask -- inventario-barramento $(for f in "$acc"/*.bar; do printf -- '--entrada %s ' "$f"; done)
cargo run -q -p xtask -- inventario-interrupcoes $(for f in "$acc"/*.s1tempo; do printf -- '--entrada %s ' "$f"; done)
echo "--- classes de instrucao (nome, tamanho, vezes) ---"
grep '^| ' docs/inventario-68000.md | grep -v 'classe\|---' | tr -d '|' | awk '{print $1, $2, $3}' | head -80
echo "--- barramento fora da RAM ---"
grep '^| ' docs/inventario-barramento.md | grep -v 'regi\|---' | tr -d '|' | awk '{print $1, $2, $3, $4}'
echo "--- interrupcoes: resumo ---"
grep -E '^Quadros:' docs/inventario-interrupcoes.md
grep '^| ' docs/inventario-interrupcoes.md | grep -v 'rotina\|---' | head -25 | tr -d '|' | awk '{print $1, "nivel", $2, "vezes", $3}'
echo "##### FIM #####"
