#!/bin/sh
# Firewall do contêiner de trabalho. Roda como root (sudo) no postStartCommand.
#
# Desenho, adaptado do init-firewall.sh de referencia do Claude Code:
# - Politica padrao DROP em IPv4 e IPv6 (a referencia so cobre IPv4).
# - O dnsmasq local (127.0.0.1) e o unico resolvedor. Ele so resolve os dominios da lista
#   e, a cada resposta, grava os IPs no ipset. Assim os IPs de CDN acompanham o DNS.
# - Nomes fora da lista recebem REFUSED, e so o root alcanca o resolvedor do contêiner.
#   Isso fecha o DNS como canal de vazamento.
# - A tabela nat nao e tocada, para nao apagar as regras do DNS embutido do Docker (127.0.0.11).
set -eu

DOMINIOS=/etc/firewall/dominios.txt
UPSTREAM_SALVO=/etc/firewall/resolvedor-original
DNSMASQ_CONF=/etc/firewall/dnsmasq.conf
DNSMASQ_PID=/run/dnsmasq-sonic.pid

erro() {
    echo "Erro: $*" >&2
    exit 1
}

[ "$(id -u)" -eq 0 ] || erro "execute como root."
[ -r "$DOMINIOS" ] || erro "$DOMINIOS nao encontrado."

# Resolvedor original do contêiner. Num reinicio o resolv.conf ja aponta para 127.0.0.1,
# entao o valor anterior fica guardado em UPSTREAM_SALVO.
atual=$(awk '/^nameserver/ {print $2; exit}' /etc/resolv.conf)
if [ -n "$atual" ] && [ "$atual" != "127.0.0.1" ]; then
    echo "$atual" > "$UPSTREAM_SALVO"
fi
[ -s "$UPSTREAM_SALVO" ] || erro "resolvedor original desconhecido."
UPSTREAM=$(cat "$UPSTREAM_SALVO")
echo "Resolvedor original: $UPSTREAM"

# Lista de dominios, sem comentarios nem linhas vazias.
lista=$(sed -e 's/#.*//' -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' "$DOMINIOS" | sed '/^$/d')
[ -n "$lista" ] || erro "lista de dominios vazia."

# Regras IPv4 (tabela filter). O ipset e preenchido pelo dnsmasq.
ipset create allowed4 hash:ip family inet -exist
ipset flush allowed4

iptables -F
iptables -X
iptables -P INPUT DROP
iptables -P FORWARD DROP
iptables -P OUTPUT DROP
iptables -A INPUT -i lo -j ACCEPT
iptables -A OUTPUT -o lo -j ACCEPT
iptables -A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT
iptables -A OUTPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT
# DNS: so o root (o dnsmasq) fala com o resolvedor do contêiner.
iptables -A OUTPUT -p udp -d "$UPSTREAM" --dport 53 -m owner --uid-owner 0 -j ACCEPT
iptables -A OUTPUT -p tcp -d "$UPSTREAM" --dport 53 -m owner --uid-owner 0 -j ACCEPT
# HTTPS so para os IPs que o DNS filtrado devolveu.
iptables -A OUTPUT -p tcp --dport 443 -m set --match-set allowed4 dst -j ACCEPT
# Falha rapida, em vez de esperar o tempo limite.
iptables -A OUTPUT -j REJECT --reject-with icmp-admin-prohibited

# Regras IPv6: nada sai, so o loopback. Sem ip6tables a falha e fatal (fecha por padrao).
ip6tables -F
ip6tables -X
ip6tables -P INPUT DROP
ip6tables -P FORWARD DROP
ip6tables -P OUTPUT DROP
ip6tables -A INPUT -i lo -j ACCEPT
ip6tables -A OUTPUT -o lo -j ACCEPT
ip6tables -A OUTPUT -j REJECT --reject-with icmp6-adm-prohibited

# dnsmasq: sem resolvedor padrao (no-resolv), so os dominios da lista tem servidor.
{
    echo "no-resolv"
    echo "no-hosts"
    echo "user=root"
    echo "listen-address=127.0.0.1"
    echo "bind-interfaces"
    echo "pid-file=$DNSMASQ_PID"
    for dominio in $lista; do
        echo "server=/$dominio/$UPSTREAM"
        echo "ipset=/$dominio/allowed4"
    done
} > "$DNSMASQ_CONF"

if [ -s "$DNSMASQ_PID" ]; then
    kill "$(cat "$DNSMASQ_PID")" 2>/dev/null || true
    sleep 1
fi
dnsmasq --conf-file="$DNSMASQ_CONF" || erro "dnsmasq nao iniciou."

# O contêiner passa a usar o dnsmasq. (cat >, porque o Docker monta o arquivo.)
printf 'nameserver 127.0.0.1\n' > /etc/resolv.conf

echo "Verificando as regras..."

# 1. Politicas DROP nos dois protocolos.
iptables -S OUTPUT | grep -q '^-P OUTPUT DROP' || erro "IPv4 OUTPUT sem DROP."
ip6tables -S OUTPUT | grep -q '^-P OUTPUT DROP' || erro "IPv6 OUTPUT sem DROP."

# 2. Dominio fora da lista: o DNS recusa e a conexao falha.
if getent hosts example.com >/dev/null 2>&1; then
    erro "o DNS resolveu example.com, que nao esta na lista."
fi
if curl -4 --connect-timeout 3 --max-time 5 -s -o /dev/null https://example.com; then
    erro "example.com conectou por IPv4."
fi
if curl -6 --connect-timeout 3 --max-time 5 -s -o /dev/null https://example.com; then
    erro "example.com conectou por IPv6."
fi
echo "OK: dominio fora da lista bloqueado (DNS, IPv4 e IPv6)."

# 3. Dominios da lista: todos os testados precisam responder (sem ||).
for url in https://api.anthropic.com https://index.crates.io https://api.github.com; do
    curl -4 --connect-timeout 5 --max-time 15 -s -o /dev/null "$url" \
        || erro "dominio autorizado nao respondeu: $url"
done
echo "OK: dominios autorizados conectam."

echo "Firewall configurado."
