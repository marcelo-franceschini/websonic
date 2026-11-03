#!/bin/sh
# Checagens em shell que o clippy nao faz. Rode pela imagem:
#   sh .github/rodar-na-imagem.sh sh .github/checa-core.sh
#
# 1. Configuracao do cargo e do rustc: so existe a config e o toolchain esperados (copias
#    congeladas em .github/). Um .cargo/config.toml aninhado, [env], rustflags, rustc-wrapper
#    ou um rust-toolchain novo mudariam o que o CI mede.
# 2. Dependencias do sonic1-core (normais, dev e build, em qualquer alvo, com todas as features)
#    iguais a lista fechada .github/deps-core.txt. Sem build.rs no nucleo.
# 3. Fontes do nucleo sem os atalhos que escondem codigo dos lints: allow, cfg_attr, include!,
#    #[path], cfg de plataforma ou de debug, tipos de ponto flutuante.
#
# Falso positivo: se uma linha legitima for barrada, o Marcelo ajusta este script (congelado).
set -u

aqui=$(cd "$(dirname "$0")/.." && pwd)
cd "$aqui" || exit 1
falhou=0
falha() {
    echo "FALHA: $*" >&2
    falhou=1
}

# ---- 1. Configuracao do cargo e do toolchain ----
# Todo diretorio .cargo e todo arquivo rust-toolchain* do repositorio (sem target/ e .git/).
achados=$(find . \( -path ./target -o -path ./.git -o -path ./dist \) -prune -o \
    \( -name .cargo -o -name 'rust-toolchain*' -o -name '.rustc-wrapper*' \) -print | sort)
esperado=$(printf '%s\n' ./.cargo ./rust-toolchain.toml | sort)
[ "$achados" = "$esperado" ] || falha "configuracao do cargo ou do toolchain fora do lugar. Achados:
$achados"
arquivos=$(find .cargo -type f 2>/dev/null | sort)
[ "$arquivos" = ".cargo/config.toml" ] || falha ".cargo deve ter so config.toml. Achados: $arquivos"
cmp -s .cargo/config.toml .github/esperado-cargo-config.txt \
    || falha ".cargo/config.toml difere de .github/esperado-cargo-config.txt."
cmp -s rust-toolchain.toml .github/esperado-toolchain.txt \
    || falha "rust-toolchain.toml difere de .github/esperado-toolchain.txt."
# [patch] e [replace] trocariam uma dependencia da lista por outra.
if grep -rnE '^\[(patch|replace)' --include=Cargo.toml . 2>/dev/null | grep -v '^./target/'; then
    falha "[patch] ou [replace] em algum Cargo.toml."
fi

# ---- 2. Dependencias do nucleo ----
if [ -e crates/core/build.rs ] || find crates/core -name build.rs | grep -q .; then
    falha "crates/core nao pode ter build.rs."
fi
if grep -nE '^[[:space:]]*(build|links)[[:space:]]*=' crates/core/Cargo.toml; then
    falha "crates/core/Cargo.toml nao pode declarar build ou links."
fi
deps=$(cargo tree --locked -p sonic1-core -e normal,build,dev --target all --all-features \
    --prefix none 2>&1) || { echo "$deps" >&2; falha "cargo tree falhou."; deps=""; }
atuais=$(printf '%s\n' "$deps" | sed -e 's/ .*//' | grep -v '^$' | sort -u | grep -vx 'sonic1-core')
lista=$(sed -e 's/#.*//' -e 's/[[:space:]]//g' .github/deps-core.txt | sed '/^$/d' | sort -u)
[ "$atuais" = "$lista" ] || falha "dependencias do sonic1-core diferem de .github/deps-core.txt.
  no cargo tree: $(echo $atuais)
  na lista:      $(echo $lista)"

# ---- 3. Fontes do nucleo ----
# Procura nos .rs e no Cargo.toml do nucleo. Comentarios de linha ficam de fora do teste de
# ponto flutuante, para a documentacao poder falar de numeros.
proibe() { # <descricao> <regex>
    if grep -rnE --include='*.rs' --include=Cargo.toml -- "$2" crates/core; then
        falha "$1"
    fi
}
proibe "atributo allow ou expect no nucleo." '#!?\[[[:space:]]*(allow|expect)\b'
proibe "cfg_attr no nucleo." 'cfg_attr'
proibe "include! ou #[path] no nucleo." '(include|include_str|include_bytes)[[:space:]]*!|#!?\[[[:space:]]*path[[:space:]]*='
proibe "cfg de plataforma ou de compilacao no nucleo (use so cfg(test))." \
    'cfg[[:space:]]*\(.*(target_|windows|unix|debug_assertions|miri|panic)'
proibe "extern crate, macro_use ou proc_macro no nucleo." 'extern[[:space:]]+crate|macro_use|proc_macro'
proibe "lints do Cargo.toml do nucleo com allow ou expect." '=[[:space:]]*"(allow|expect)"|level[[:space:]]*=[[:space:]]*"(allow|expect)"'
if grep -rnE --include='*.rs' '(^|[^[:alnum:]_])f(32|64)([^[:alnum:]_]|$)' crates/core; then
    falha "tipo de ponto flutuante no nucleo."
fi
if grep -rn --include='*.rs' -vE '^[[:space:]]*//' crates/core | grep -E '[^[:alnum:]_.][0-9]+\.[0-9]+([eE][+-]?[0-9]+)?'; then
    falha "literal de ponto flutuante no nucleo."
fi

if [ "$falhou" -eq 0 ]; then
    echo "checa-core: OK"
fi
exit "$falhou"
