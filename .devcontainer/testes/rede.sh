#!/bin/bash
# Prova de rede da sub-etapa 0.2. Roda DENTRO do contêiner, como o usuario dev,
# depois que o postStartCommand configurou o firewall. Nao toca na ROM nem no disassembly.
# Cada teste imprime PASSOU ou FALHOU com o comando. O codigo de saida e o numero de falhas.
export GIT_TERMINAL_PROMPT=0
falhas=0

# passa DESCRICAO COMANDO...: o comando precisa terminar com sucesso.
passa() {
    local desc="$1"; shift
    if "$@" >/dev/null 2>&1; then echo "PASSOU: $desc"; else echo "FALHOU: $desc  [$*]"; falhas=$((falhas + 1)); fi
}
# bloqueia DESCRICAO COMANDO...: o comando precisa terminar com erro.
bloqueia() {
    local desc="$1"; shift
    if "$@" >/dev/null 2>&1; then echo "FALHOU: $desc  [$*]"; falhas=$((falhas + 1)); else echo "PASSOU: $desc"; fi
}

# Dominios autorizados conectam (IPv4).
for d in api.anthropic.com index.crates.io api.github.com; do
    passa "dominio autorizado conecta por IPv4: $d" curl -4 -s -o /dev/null --connect-timeout 5 --max-time 15 "https://$d"
done

# Dominios nao autorizados falham, nos dois protocolos.
for d in example.com tasvideos.org en.wikipedia.org; do
    bloqueia "dominio nao autorizado falha por IPv4: $d" curl -4 -s -o /dev/null --connect-timeout 3 --max-time 6 "https://$d"
    bloqueia "dominio nao autorizado falha por IPv6: $d" curl -6 -s -o /dev/null --connect-timeout 3 --max-time 6 "https://$d"
    bloqueia "DNS recusa o nome: $d" getent hosts "$d"
done

# IPv6 literal (sem DNS): nada sai.
bloqueia "IPv6 literal bloqueado (2606:4700:4700::1111)" curl -6 -s -o /dev/null --connect-timeout 3 --max-time 6 "https://[2606:4700:4700::1111]"
# IPv4 literal fora da lista (sem DNS): nada sai.
bloqueia "IPv4 literal bloqueado (1.1.1.1)" curl -4 -s -o /dev/null --connect-timeout 3 --max-time 6 "https://1.1.1.1"

# DNS direto a um servidor externo: nao pode haver resposta (o pacote e rejeitado).
dns_direto() {
    exec 3<>/dev/udp/8.8.8.8/53 || return 1
    printf '\x12\x34\x01\x00\x00\x01\x00\x00\x00\x00\x00\x00\x07example\x03com\x00\x00\x01\x00\x01' >&3
    read -r -t 3 -u 3 _ 2>/dev/null
}
bloqueia "DNS UDP direto a 8.8.8.8 sem resposta" dns_direto
bloqueia "DNS TCP direto a 8.8.8.8 recusado" bash -c 'exec 3<>/dev/tcp/8.8.8.8/53'

# O dev nao tem sudo alem do script de firewall.
bloqueia "sudo generico recusado" sudo -n true

# git push sem credenciais falha (repositorio temporario, remoto = origin do projeto).
tmp=$(mktemp -d)
(
    cd "$tmp" && git init -q && git -c user.name=t -c user.email=t@t commit -q --allow-empty -m teste \
        && git remote add origin "$(git -C /workspaces/websonic remote get-url origin)"
)
bloqueia "git push sem credenciais falha" git -C "$tmp" push origin HEAD:refs/heads/teste-sem-credencial
rm -rf "$tmp"

# O volume de configuracao do Claude e gravavel pelo dev.
passa "/home/dev/.claude gravavel" bash -c 't=/home/dev/.claude/.teste; : > "$t" && rm "$t"'

echo "---"
echo "falhas: $falhas"
exit "$falhas"
