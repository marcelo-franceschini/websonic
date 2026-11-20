#!/bin/sh
# Etapa 7, bloco 5 (Marcelo, fora do Claude): o caminho do navegador com a ROM real.
# 1) Hash da imagem por quadro, nativo e em WebAssembly (wasmtime na imagem), e comparacao.
# 2) Monta o site na imagem. 3) Imagens da tela da Sega para voce ver (ficam fora do repositorio).
# Imprime so quadros, hashes e vereditos.
# Uso: sh scripts/etapa7-bloco5.sh "$HOME/sonic1-ext/rom/Sonic The Hedgehog (W) (REV01) [!].gen" [QUADROS]
set -u
rom=${1:?uso: etapa7-bloco5.sh ROM [QUADROS]}
quadros=${2:-330}
repo=$(cd "$(dirname "$0")/.." && pwd -P)
ext=${SONIC1_EXT_DIR:-$HOME/sonic1-ext}
saida=$ext/privado/etapa7
mkdir -p "$saida"
cd "$repo"
esperado=$(sed -n 's/^sha1: *//p' refs/rom-hashes.txt)
obtido=$(sha1sum "$rom" | cut -d' ' -f1)
[ "$esperado" = "$obtido" ] || { echo "Erro: a ROM nao e a REV01." >&2; exit 1; }

echo "=== NATIVO ==="
cargo run -q --release -p sonic1-console --bin console_hashes -- "$rom" "$quadros" \
    --ppm 40 "$saida/sega-quadro40.ppm" > "$saida/hashes-nativo.txt"
tail -2 "$saida/hashes-nativo.txt"
cargo run -q --release -p sonic1-console --bin console_hashes -- "$rom" "$quadros" \
    --ppm 150 "$saida/sega-quadro150.ppm" > "$saida/hashes-nativo2.txt"
cmp -s "$saida/hashes-nativo.txt" "$saida/hashes-nativo2.txt" && echo "determinismo: PASSOU" || echo "determinismo: FALHOU"

echo "=== WEBASSEMBLY (wasmtime na imagem) ==="
sh .github/rodar-na-imagem.sh cargo build -q --release --target wasm32-wasip1 -p sonic1-console --bin console_hashes
romdir=$(dirname "$rom")
romnome=$(basename "$rom")
docker run --rm -v "$repo:/workspaces/websonic:ro" -v "$romdir:/rom:ro" -w /workspaces/websonic websonic-dev:ci \
    wasmtime run --dir /rom target/wasm32-wasip1/release/console_hashes.wasm "/rom/$romnome" "$quadros" \
    > "$saida/hashes-wasm.txt" 2>"$saida/wasm-erros.txt" || echo "wasmtime falhou (veja wasm-erros.txt)"
if cmp -s "$saida/hashes-nativo.txt" "$saida/hashes-wasm.txt"; then echo "wasm_igual_ao_nativo: SIM"; else echo "wasm_igual_ao_nativo: NAO"; fi
echo "linhas: nativo=$(wc -l < "$saida/hashes-nativo.txt") wasm=$(wc -l < "$saida/hashes-wasm.txt")"
awk '{print $3}' "$saida/hashes-nativo.txt" | sort -u | wc -l | sed 's/^/imagens_distintas: /'

echo "=== SITE ==="
sh .github/rodar-na-imagem.sh cargo xtask site 2>&1 | tail -2
ls dist | tr '\n' ' '; echo

echo "=== TESTES EM WASM (na imagem) ==="
sh .github/rodar-na-imagem.sh cargo test -q --target wasm32-wasip1 -p sonic1-core -p sonic1-video -p sonic1-console -p sonic1-teste-som 2>&1 | grep -E "test result|FAILED|error" | head
echo "=== IMAGENS (para voce abrir; nao entram no git) ==="
ls -1 "$saida"/*.ppm
echo "=== FIM ==="
