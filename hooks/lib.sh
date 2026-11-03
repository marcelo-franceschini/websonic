# Funcoes comuns dos hooks do git. Shell POSIX, sem bashismos.
# Quem usa: hooks/pre-commit e hooks/pre-push (via ". lib.sh").
#
# Regra de falha: se um arquivo de refs faltar ou estiver ilegivel, o hook BLOQUEIA.
# Em duvida, o hook recusa. Nunca libera por falta de dado.

raiz=$(git rev-parse --show-toplevel) || exit 1
refs="$raiz/refs"

# Le o valor de uma linha "chave: valor" de refs/<arquivo>. Vazio se nao achar.
ler_ref() {
    sed -n "s/^$2:[[:space:]]*//p" "$refs/$1" 2>/dev/null | head -n 1
}

# Verdadeiro se o caminho esta na lista de excecoes (uma linha por caminho).
# A excecao vale para extensao e tamanho. Nunca vale para o hash da ROM.
em_excecao() {
    grep -v '^[[:space:]]*#' "$refs/excecoes-extensoes.txt" 2>/dev/null | grep -Fxq -- "$1"
}

# Verdadeiro se a extensao do caminho esta na lista de bloqueio.
# A lista espelha o .gitignore (ROMs, compactados, estados, midia).
extensao_proibida() {
    base=${1##*/}
    case $base in *.*) ;; *) return 1 ;; esac
    ext=$(printf '%s' "${base##*.}" | tr 'A-Z' 'a-z')
    # O git cita caminhos com caracteres especiais; tira a aspa final.
    ext=${ext%\"}
    case $ext in
        gen|bin|smd|rom|32x|sms|iso|zip|7z|rar|tar|gz|bz2|xz|sav|srm|state*|trace|gst|wav|mp3|ogg|flac|vgm)
            return 0 ;;
    esac
    return 1
}

# Le linhas "<id do objeto> <caminho>" na entrada e imprime uma linha por problema.
# Saida vazia significa que esta tudo certo. O tamanho e o tipo vem do proprio git,
# pelo id do objeto, entao um nome de arquivo disfarcado nao muda o resultado.
verificar_objetos() {
    max=$(ler_ref limite-tamanho.txt max_bytes)
    rom=$(ler_ref rom-hashes.txt blob_id)
    if [ -z "$max" ] || [ -z "$rom" ]; then
        echo "  refs/limite-tamanho.txt ou refs/rom-hashes.txt ausente ou ilegivel"
        return 0
    fi
    git cat-file --batch-check='%(objectname) %(objecttype) %(objectsize) %(rest)' |
    while IFS=' ' read -r id tipo tam caminho; do
        [ "$tipo" = blob ] || continue
        if [ "$id" = "$rom" ]; then
            echo "  $caminho: e a ROM de referencia (blob $id)"
            continue
        fi
        em_excecao "$caminho" && continue
        [ "$tam" -ge "$max" ] && echo "  $caminho: $tam bytes (limite $max)"
        extensao_proibida "$caminho" && echo "  $caminho: extensao proibida"
    done
    return 0
}

# Mensagem final de bloqueio. $1 e o nome da operacao.
recusar() {
    {
        echo "hook: $1 BLOQUEADO. Nada com conteudo da ROM pode entrar no git:"
        printf '%s\n' "$2"
        echo "Remova o arquivo. Nao use --no-verify nem desligue os hooks."
    } >&2
    exit 1
}
