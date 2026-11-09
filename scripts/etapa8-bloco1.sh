#!/bin/sh
# Etapa 8, bloco 1 (Marcelo, host): prepara os chips de referencia e confere o ambiente.
# Nao usa a ROM. Imprime so commits, hashes, licencas e versoes.
# Uso: sh scripts/etapa8-bloco1.sh
set -u

EXT="${SONIC1_EXT_DIR:-$HOME/sonic1-ext}"
DEST="$EXT/trabalho/nuked"
OPN2_COMMIT=335747d78cb0abbc3b55b004e62dad9763140115
PSG_COMMIT=d15a168c676f4669e23660be9225b34ad7c1764e

mkdir -p "$DEST" || exit 1

# Clona (se faltar) e fixa o commit. Fica fora do repositorio.
prepara() {
  nome="$1"; url="$2"; commit="$3"
  if [ ! -d "$DEST/$nome/.git" ]; then
    git clone --quiet "$url" "$DEST/$nome" || { echo "FALHA clone $nome"; return 1; }
  fi
  git -C "$DEST/$nome" fetch --quiet origin 2>/dev/null
  git -C "$DEST/$nome" checkout --quiet "$commit" || { echo "FALHA checkout $nome"; return 1; }
  echo "$nome commit: $(git -C "$DEST/$nome" rev-parse HEAD)"
  echo "$nome esperado: $commit"
}

prepara Nuked-OPN2 https://github.com/nukeykt/Nuked-OPN2 "$OPN2_COMMIT"
prepara Nuked-PSG https://github.com/nukeykt/Nuked-PSG "$PSG_COMMIT"

echo "--- licencas (sha1 do arquivo e primeira linha)"
for d in Nuked-OPN2 Nuked-PSG; do
  echo "$d LICENSE sha1: $(sha1sum "$DEST/$d/LICENSE" | cut -d' ' -f1)"
  echo "$d LICENSE linha1: $(sed -n '1,3p' "$DEST/$d/LICENSE" | tr -s ' ' | tr '\n' '|')"
done
echo "--- fontes (nome, tamanho, sha1)"
for f in Nuked-OPN2/ym3438.c Nuked-OPN2/ym3438.h Nuked-PSG/ympsg.c Nuked-PSG/ympsg.h; do
  if [ -f "$DEST/$f" ]; then
    echo "$f $(wc -c < "$DEST/$f") $(sha1sum "$DEST/$f" | cut -d' ' -f1)"
  else
    echo "$f AUSENTE"
  fi
done
echo "--- licenca declarada nos cabecalhos"
grep -h -i -m1 "License\|Lesser" "$DEST/Nuked-OPN2/ym3438.h" "$DEST/Nuked-PSG/ympsg.h" 2>/dev/null

echo "--- ambiente"
cc --version 2>&1 | sed -n '1p'
cargo --version 2>&1
rustc --version 2>&1
rustup target list --installed 2>&1 | tr '\n' ' '; echo
command -v wasmtime >/dev/null 2>&1 && wasmtime --version || echo "wasmtime: ausente no host"
echo "repo HEAD: $(git rev-parse --short HEAD)  status: $(git status --short | wc -l) arquivos alterados"
echo "--- fim do bloco 1"
