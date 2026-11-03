# Imagem base Debian Bookworm Slim com digest fixado para reproducibilidade
FROM debian:bookworm-slim@sha256:d6b46ba7833f6a27e1f6e245a4980072b2260ff053f3e1b0682fa05b53eb901c

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

ENV DEBIAN_FRONTEND=noninteractive
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME_GLOBAL=/usr/local/cargo
ENV PATH="/usr/local/cargo/bin:/home/dev/.cargo/bin:$PATH"

# Instalacao de dependencias do sistema, ferramentas de sandbox, firewall e Lua
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    gcc \
    libc6-dev \
    git \
    ca-certificates \
    curl \
    bubblewrap \
    socat \
    iptables \
    ipset \
    dnsmasq \
    sudo \
    lua5.2 \
    liblua5.2-dev \
    && rm -rf /var/lib/apt/lists/*

# Criacao do usuario dev (UID e GID 1000)
RUN groupadd -g 1000 dev && \
    useradd -u 1000 -g 1000 -m -s /bin/bash dev

# Configuracao de permissao restrita de sudo apenas para o script de firewall
RUN echo "dev ALL=(ALL) NOPASSWD: /usr/local/bin/init-firewall.sh" > /etc/sudoers.d/dev-firewall && \
    chmod 0440 /etc/sudoers.d/dev-firewall

# Instalacao do Rust fixado na versao 1.85.0
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    --default-toolchain 1.85.0 \
    --profile minimal \
    --component rustfmt clippy \
    --target wasm32-unknown-unknown wasm32-wasip1

# Instalacao do wasm-bindgen-cli na versao exata 0.2.100
RUN cargo install wasm-bindgen-cli --version 0.2.100 --root /usr/local/cargo

# Instalacao do wasmtime para execucao dos testes no alvo wasm32-wasip1
RUN curl -fsSL https://wasmtime.dev/install.sh | bash && \
    mv /root/.wasmtime/bin/wasmtime /usr/local/bin/wasmtime && \
    chmod 755 /usr/local/bin/wasmtime && \
    rm -rf /root/.wasmtime

# Configuracao global do cargo para usar wasmtime como runner de wasm32-wasip1
RUN mkdir -p /usr/local/cargo && \
    printf '[target.wasm32-wasip1]\nrunner = "wasmtime"\n' > /usr/local/cargo/config.toml

# A toolchain e os utilitarios instalados ficam apenas para leitura para dev
RUN chmod -R a+rX /usr/local/rustup /usr/local/cargo

# Instalacao nativa do Claude Code sem Node.js
RUN curl -fsSL https://claude.ai/install.sh | bash && \
    if [ -f /root/.local/bin/claude ]; then \
        mv /root/.local/bin/claude /usr/local/bin/claude; \
    elif [ -f /root/.claude/bin/claude ]; then \
        mv /root/.claude/bin/claude /usr/local/bin/claude; \
    fi && \
    chmod 755 /usr/local/bin/claude 2>/dev/null || true

# Copia e permissao do script de firewall
COPY .devcontainer/init-firewall.sh /usr/local/bin/init-firewall.sh
RUN chmod 755 /usr/local/bin/init-firewall.sh

# Copia da lista de dominios autorizados para o firewall
RUN mkdir -p /etc/firewall
COPY .devcontainer/dominios.txt /etc/firewall/dominios.txt
RUN chmod 644 /etc/firewall/dominios.txt

# Copia das configuracoes gerenciadas do Claude Code (propriedade do root)
RUN mkdir -p /etc/claude-code
COPY .devcontainer/managed-settings.json /etc/claude-code/managed-settings.json
RUN chown root:root /etc/claude-code/managed-settings.json && \
    chmod 644 /etc/claude-code/managed-settings.json

# Diretorios de trabalho e montagem
RUN mkdir -p /workspaces/sonic1 /workspaces/websonic /mnt/rom /mnt/disasm /mnt/disasm-texto /mnt/trabalho && \
    chown -R dev:dev /workspaces/sonic1 /workspaces/websonic /mnt/trabalho

# Link simbolico para garantir compatibilidade entre caminhos
RUN ln -s /workspaces/sonic1 /workspaces/sonic1-alias 2>/dev/null || true

USER dev
ENV CARGO_HOME=/home/dev/.cargo
ENV USER=dev
WORKDIR /workspaces/sonic1

CMD ["/bin/bash"]
