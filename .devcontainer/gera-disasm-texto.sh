#!/bin/sh
# Gera $SONIC1_EXT_DIR/disasm-texto a partir de $SONIC1_EXT_DIR/disasm.
# Roda no computador do Marcelo, nunca pela IA. Copia so os arquivos .asm, mantendo a estrutura
# de pastas. Binarios do disassembly nao sao copiados, e a IA so enxerga esta copia.
# Os .asm trazem dados da ROM em texto: a regra de origem dos dados continua valendo.
set -eu

base=${SONIC1_EXT_DIR:-}
[ -n "$base" ] || { echo "Erro: SONIC1_EXT_DIR nao esta definida." >&2; exit 1; }
origem="$base/disasm"
destino="$base/disasm-texto"
[ -d "$origem" ] || { echo "Erro: $origem nao existe." >&2; exit 1; }

# Recria o destino do zero, para nao sobrar arquivo de uma versao antiga do disassembly.
rm -rf "$destino"
mkdir -p "$destino"
(cd "$origem" && find . -type f -name '*.asm' -exec cp --parents {} "$destino" \;)

echo "Copiados $(find "$destino" -type f | wc -l) arquivos .asm para $destino."
