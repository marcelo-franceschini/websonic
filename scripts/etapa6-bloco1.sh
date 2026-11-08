#!/bin/sh
# Etapa 6, bloco 1 (Marcelo, fora do Claude): aplica o patch 0005 do oraculo (pontos do Z80 no
# tempo), regenera os tempos do corpus e gera o inventario do som. Os traces completos sao
# temporarios (alguns GB) e sao apagados no fim. Imprime so nomes, numeros, enderecos de codigo do
# Z80 e vereditos. Uso: sh scripts/etapa6-bloco1.sh 2>&1 | tee /tmp/etapa6-bloco1.txt
set -eu
repo=$(cd "$(dirname "$0")/.." && pwd -P)
cd "$repo"
ROM="$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen"
ext=${SONIC1_EXT_DIR:-$HOME/sonic1-ext}
acc=$ext/privado/etapa6
tmp=$ext/privado/trace-som
mkdir -p "$acc"
rm -rf "$tmp"; mkdir -p "$tmp"
trap 'rm -rf "$tmp"' EXIT

esperado=$(sed -n 's/^sha1: *//p' refs/rom-hashes.txt)
obtido=$(sha1sum "$ROM" | cut -d' ' -f1)
[ "$esperado" = "$obtido" ] || { echo "Erro: a ROM nao e a REV01." >&2; exit 1; }

echo "##### 1. PATCH 0005 DO ORACULO E TESTES #####"
if git apply --check docs/pedidos/etapa6-oraculo.patch 2>/dev/null; then
    git apply docs/pedidos/etapa6-oraculo.patch && echo "etapa6-oraculo: aplicado"
else
    echo "etapa6-oraculo: ja aplicado (ou nao aplica; veja git status)"
fi
sh oracle/scripts/testa.sh 2>&1 | grep -E "test result|Testes do oraculo|error|FAILED"

echo "##### 2. ORACULO COM OS PONTOS DO Z80 #####"
sh oracle/scripts/prepara.sh | tail -1
export CARGO_TARGET_DIR="$ext/trabalho/oracle/alvo"
(cd oracle/runner && cargo build --release -q)
bin=$CARGO_TARGET_DIR/release/oraculo
unset CARGO_TARGET_DIR

echo "##### 3. CASOS DO CORPUS (trace temporario e tempo versao 2) #####"
rodar() { # nome quadros [entradas]
    nome=$1; q=$2; ent=${3:-}
    if [ -n "$ent" ]; then set -- --entradas "$ent"; else set --; fi
    "$bin" rodar --rom "$ROM" --quadros "$q" --saida "$tmp/$nome.a.s1t" "$@" \
        --tempo "$acc/$nome.s1tempo" | grep -E '^(quadros|mclk_final)='
    echo "caso $nome: $(du -m "$tmp/$nome.a.s1t" | cut -f1) MB de trace (apagado no fim), $(wc -c < "$acc/$nome.s1tempo") bytes de tempo"
}
rodar boot-demos 7200
rodar lz-agua 2000 corpus/lz-agua.s1in
rodar partida-ghz1 "$(wc -l < corpus/partida-ghz1.s1in)" corpus/partida-ghz1.s1in
rodar demos-quatro 11700

echo "##### 4. INVENTARIO DO SOM (docs/inventario-som.md) #####"
cargo run -q -p xtask -- inventario-som --traces "$tmp" --tempos "$acc"

echo "##### 5. RESUMO (cole daqui ate o FIM) #####"
# Sem as linhas de dado do YM2612 do 68000 (sao muitas e ja estao em inventario-hardware.md).
awk '!/^\| 68000 \| [0-9] \| 0x/' docs/inventario-som.md
echo "##### FIM #####"
