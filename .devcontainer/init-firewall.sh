#!/bin/sh
set -eu

# Script de inicializacao do firewall estrito
# Politica padrao: bloqueia todo o trafego de entrada e saida.
# Permite apenas loopback, DNS e conexoes HTTPS/HTTP para os dominios autorizados.

DOMINIOS_FILE=""
if [ -f "/etc/firewall/dominios.txt" ]; then
    DOMINIOS_FILE="/etc/firewall/dominios.txt"
elif [ -f "$(dirname "$0")/dominios.txt" ]; then
    DOMINIOS_FILE="$(dirname "$0")/dominios.txt"
elif [ -f "/workspaces/websonic/.devcontainer/dominios.txt" ]; then
    DOMINIOS_FILE="/workspaces/websonic/.devcontainer/dominios.txt"
elif [ -f "/workspaces/sonic1/.devcontainer/dominios.txt" ]; then
    DOMINIOS_FILE="/workspaces/sonic1/.devcontainer/dominios.txt"
fi

if [ -z "$DOMINIOS_FILE" ] || [ ! -f "$DOMINIOS_FILE" ]; then
    echo "Erro: arquivo dominios.txt nao encontrado." >&2
    exit 1
fi

echo "Iniciando configuracao do firewall com base em $DOMINIOS_FILE..."

# Cria o conjunto de IPs no ipset para acomodar IPs de CDNs e servicos
ipset create allowed-hosts hash:ip family inet -exist
ipset flush allowed-hosts

# Configura regras basicas no iptables
iptables -F
iptables -X
iptables -t nat -F
iptables -t nat -X

# Politica padrao restritiva
iptables -P INPUT DROP
iptables -P FORWARD DROP
iptables -P OUTPUT DROP

# Permite trafego local em loopback
iptables -A INPUT -i lo -j ACCEPT
iptables -A OUTPUT -o lo -j ACCEPT

# Permite conexoes ja estabelecidas e relacionadas
iptables -A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT
iptables -A OUTPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT

# Permite saida para resolucao DNS nas portas 53 (UDP e TCP)
iptables -A OUTPUT -p udp --dport 53 -j ACCEPT
iptables -A OUTPUT -p tcp --dport 53 -j ACCEPT

# Permite conexoes de saida HTTP e HTTPS apenas para os IPs do ipset
iptables -A OUTPUT -p tcp -m multiport --dports 80,443 -m set --match-set allowed-hosts dst -j ACCEPT

# Configura dnsmasq para manter o ipset atualizado dinamicamente com respostas DNS de CDNs
DNSMASQ_CONF="/etc/dnsmasq.d/sonic-firewall.conf"
mkdir -p /etc/dnsmasq.d
rm -f "$DNSMASQ_CONF"
touch "$DNSMASQ_CONF"

# Adiciona cada dominio de dominios.txt no dnsmasq e resolve o IP inicial
while IFS= read -r linha || [ -n "$linha" ]; do
    # Remove espacos em branco e ignora linhas vazias ou comentarios
    dominio=$(echo "$linha" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')
    case "$dominio" in
        \#*|"") continue ;;
    esac

    echo "ipset=/$dominio/allowed-hosts" >> "$DNSMASQ_CONF"

    # Resolucao inicial para popular o ipset de imediato
    for ip in $(getent ahostsv4 "$dominio" 2>/dev/null | awk '{print $1}' | sort -u); do
        ipset add allowed-hosts "$ip" -exist
    done
done < "$DOMINIOS_FILE"

# Inicia ou reinicia o servico dnsmasq se instalado
if command -v dnsmasq >/dev/null 2>&1; then
    service dnsmasq restart 2>/dev/null || dnsmasq --conf-file=/etc/dnsmasq.conf --conf-dir=/etc/dnsmasq.d 2>/dev/null || true
fi

echo "Verificando integridade das regras de isolamento..."

# Teste 1: Um dominio fora da lista deve falhar obrigatoriamente
if curl --connect-timeout 3 --max-time 5 -sI https://example.com >/dev/null 2>&1; then
    echo "Falha de seguranca: dominio nao autorizado (example.com) conseguiu conexao." >&2
    exit 1
fi
echo "Confirmado: dominio nao autorizado bloqueado com sucesso."

# Teste 2: Um dominio da lista autorizada deve responder com sucesso
if ! curl --connect-timeout 5 --max-time 10 -sI https://index.crates.io >/dev/null 2>&1; then
    if ! curl --connect-timeout 5 --max-time 10 -sI https://api.anthropic.com >/dev/null 2>&1; then
        echo "Falha de rede: dominios autorizados nao responderam." >&2
        exit 1
    fi
fi
echo "Confirmado: dominio autorizado conectou com sucesso."

echo "Firewall configurado e validado."
exit 0

