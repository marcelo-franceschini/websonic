# Imagem base fixada por digest (obtido com "docker pull debian:bookworm-slim").
FROM debian:bookworm-slim@sha256:99c0b25c46d0214d5a2091092e7908907dcaecd6a962cde0ac700acee222b6f3

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

# Versoes fixadas. Para atualizar, mude aqui e reconstrua a imagem.
# Rust: estavel de 15/10/2026. O wasm-bindgen-cli 0.2.129 exige Rust 1.86 ou mais novo.
ARG RUST_VERSION=1.99.1
ARG WASM_BINDGEN_VERSION=0.2.129
ARG WASMTIME_VERSION=v50.0.1
ARG CLAUDE_CODE_VERSION=2.1.317

ENV DEBIAN_FRONTEND=noninteractive
# Durante o build, o cargo instala em /usr/local/cargo. No fim, o usuario dev troca CARGO_HOME.
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH="/usr/local/cargo/bin:$PATH"

# Sistema: linker C (build-essential), sandbox (bubblewrap, socat), firewall e Lua do disassembly.
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    curl \
    git \
    xz-utils \
    bubblewrap \
    socat \
    iptables \
    ipset \
    dnsmasq \
    sudo \
    lua5.2 \
    liblua5.2-dev \
    && rm -rf /var/lib/apt/lists/*

# Usuario dev (UID e GID 1000) e as pastas que o volume e o cargo precisam escrever.
RUN groupadd -g 1000 dev && \
    useradd -u 1000 -g 1000 -m -s /bin/bash dev && \
    mkdir -p /home/dev/.claude /home/dev/.cargo/registry /home/dev/.cargo/git && \
    chown -R dev:dev /home/dev/.claude /home/dev/.cargo

# O sudo do dev vale so para o script de firewall.
RUN echo "dev ALL=(root) NOPASSWD: /usr/local/bin/init-firewall.sh" > /etc/sudoers.d/dev-firewall && \
    chmod 0440 /etc/sudoers.d/dev-firewall

# Rust: o rustup roda como root e grava em /usr/local/cargo e /usr/local/rustup.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    --no-modify-path \
    --default-toolchain "${RUST_VERSION}" \
    --profile minimal \
    -c rustfmt -c clippy \
    -t wasm32-unknown-unknown -t wasm32-wasip1 && \
    rustc --version | grep -q "${RUST_VERSION}"

# Gerador de ligacao com o navegador. --locked usa as dependencias do proprio crate.
RUN cargo install wasm-bindgen-cli --version "${WASM_BINDGEN_VERSION}" --locked && \
    rm -rf /usr/local/cargo/registry /usr/local/cargo/git && \
    wasm-bindgen --version | grep -q "${WASM_BINDGEN_VERSION}"

# Executor de WebAssembly para os testes em wasm32-wasip1 (runner em .cargo/config.toml).
RUN arch="$(uname -m)" && \
    curl -fsSL "https://github.com/bytecodealliance/wasmtime/releases/download/${WASMTIME_VERSION}/wasmtime-${WASMTIME_VERSION}-${arch}-linux.tar.xz" \
        | tar -xJ -C /tmp && \
    install -m 0755 /tmp/wasmtime-*/wasmtime /usr/local/bin/wasmtime && \
    rm -rf /tmp/wasmtime-* && \
    wasmtime --version

# Claude Code, versao fixa. O instalador nativo grava em /root/.local; copia-se so o binario.
RUN curl -fsSL https://claude.ai/install.sh | bash -s -- "${CLAUDE_CODE_VERSION}" && \
    install -m 0755 "$(readlink -f /root/.local/bin/claude)" /usr/local/bin/claude && \
    rm -rf /root/.local /root/.claude && \
    claude --version | grep -q "${CLAUDE_CODE_VERSION}"

# O Claude Code espera achar o proprio binario em ~/.local/bin. O link evita avisos do
# "claude doctor". O binario de verdade continua em /usr/local/bin, que o dev nao escreve.
RUN mkdir -p /home/dev/.local/bin && \
    ln -s /usr/local/bin/claude /home/dev/.local/bin/claude && \
    chown -R dev:dev /home/dev/.local

# Firewall e lista de dominios. Ficam em caminhos do root, fora do alcance da IA.
COPY .devcontainer/init-firewall.sh /usr/local/bin/init-firewall.sh
COPY .devcontainer/dominios.txt /etc/firewall/dominios.txt
RUN chown root:root /usr/local/bin/init-firewall.sh /etc/firewall/dominios.txt && \
    chmod 755 /usr/local/bin/init-firewall.sh && \
    chmod 644 /etc/firewall/dominios.txt

# Permissoes gerenciadas do Claude Code, com prioridade sobre o repositorio.
COPY .devcontainer/managed-settings.json /etc/claude-code/managed-settings.json
RUN chown root:root /etc/claude-code/managed-settings.json && \
    chmod 644 /etc/claude-code/managed-settings.json

# Pontos de montagem e o workspace (caminho unico).
RUN mkdir -p /workspaces/websonic /mnt/rom /mnt/disasm /mnt/disasm-texto /mnt/trabalho && \
    chown dev:dev /workspaces/websonic /mnt/trabalho

USER dev
# Em execucao, o cache de pacotes do cargo fica na home do dev. Os binarios continuam no PATH.
ENV CARGO_HOME=/home/dev/.cargo
ENV USER=dev
ENV PATH="/home/dev/.local/bin:$PATH"
WORKDIR /workspaces/websonic

CMD ["/bin/bash"]
