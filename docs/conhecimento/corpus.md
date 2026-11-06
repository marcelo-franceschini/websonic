# Corpus e traces (Etapa 2)

## Formato `.s1in` e `.meta` (2.3)

- `corpus/<nome>.s1in`: uma linha por quadro do emulador, mascara de 2 digitos hexadecimais, `#` comenta. Bits: Cima 01, Baixo 02, Esquerda 04, Direita 08, B 10, C 20, A 40, Start 80. Leitor estrito: recusa linha com tamanho diferente de 2.
- `corpus/<nome>.meta`: `chave: valor` com `fonte`, `licenca`, `rom_sha1`, `quadros`, `entradas_sha1`, `autor`. O `corpus-trace` recusa o caso se `quadros` ou `entradas_sha1` nao baterem com o `.s1in`.
- Quadro de lag: a linha vale para um quadro de video. O jogo ignora a linha sozinho. Nenhuma conversao corrige lag.
- Codigo: `crates/corpus` (MIT). Prova: `cargo test -p sonic1-corpus`, 9 testes com dados sinteticos (ida e volta, malformados, GMV inventado).

## GMV (conversor feito) e BK2 (NAO feito)

- Fonte do GMV: https://tasvideos.org/EmulatorResources/Gens/GMV, lida em 06/11/2026. Cabecalho de 64 bytes, 3 bytes por quadro, bits ativos em zero, A=10, B=20, C=40, Start=80. A pagina nao separa 3 e 6 botoes: so aceitamos controle 1 de 3 botoes ('3') e 60 Hz.
- Risco aberto: o mapa A/B/C vem so dessa pagina. Um TAS real precisa confirmar (a conferencia contra a REV01 da sub-etapa 2.5 mostraria dessincronia).
- BK2: a pagina https://tasvideos.org/BizHawk/BK2Format (lida em 06/11/2026) diz que e um zip com Input Log, mas nao da o nome exato do arquivo nem a ordem dos botoes do Mega Drive. Pela regra do plano ("se um formato nao for confirmavel, nao entra"), o BK2 ficou fora. Backlog.

## Traces do corpus (2.4)

- `cargo xtask corpus-trace --rom ROM --oraculo BIN --rotulos refs/rotulos.txt --saida DIR [--corpus corpus] [--intervalo 600] [--caso NOME]`.
- Casos: `corpus/demos.txt` (`nome<TAB>quadros`, sem botoes) e cada `corpus/*.s1in`.
- Por caso: duas execucoes (cmp byte a byte e hash) e retomada do ultimo estado salvo antes do fim, comparando os quadros restantes com o trace completo. Imprime `nome hash quadros` e `determinismo: PASSOU`.
- Saida recusada se estiver dentro do repositorio.
- Mudanca no oraculo (congelado), entregue como patch: `docs/pedidos/estado-a-cada.patch` com `--estado-a-cada N`, `--carregar-estado ARQ` (`--quadros` vira o quadro final absoluto) e `--perfil-a-cada N` (imprime modo, zona, ato e demo, so numeros).
- Evidencia (06/11/2026, ROM sintetica `vblank`, 45 quadros, K=10): `determinismo: PASSOU`, retomada sem divergencia. Testes: `cargo test -p xtask` (25), `comparar_cauda` com quadro plantado, trace curto e longo.
- A evidencia com a ROM real ainda nao existe (so o Marcelo roda).

## Manifesto, ocultos e inventario (2.6 a 2.8, lado da IA)

- `cargo xtask manifesto gerar|verificar`: `arquivo/trace/zona/oculto/teste` com SHA-256 (implementacao propria, vetores do padrao conferidos com `sha256sum`). Assinatura SSHSIG por `ssh-keygen -Y`, namespace `websonic-manifesto`. Prova (06/11/2026, chave descartavel): `assinatura: OK`, `referencias: OK`, `testes: OK`; os casos plantados (assinatura adulterada ou ausente, arquivo mudado ou ausente, teste ignorado ou faltando, teste novo "nao assinado") estao em `cargo test -p xtask` (40 testes).
- `cargo xtask ocultos cifrar|auditar|chaves`: crate `age` 0.12.1 (so a chave publica cifra). A auditoria decifra para `/dev/shm`, apaga e imprime so `zona: passou|falhou (N casos)`; erros viram `erro: E1|E2|E3`. Nao existe `decifrar` de proposito. A identidade e um arquivo 0600 sem senha (decisao: o crate so oferece senha com a feature `cli-common`; protecao em repouso fica por conta do disco do Marcelo). Custo: o xtask passou a ter ~340 crates na arvore de dependencias.
- `cargo xtask inventario --traces DIR`: decodifica os eventos de VDP (registradores, comandos, DMA pelo reg 23), YM, PSG, nivel de interrupcao e BUSREQ/RESET. Nao cobre DMA como evento (R6) nem E/S do 68000 (P2). Teste: 4 casos sinteticos.
- Casos de lag e DMA do VDP que o renderizador da Etapa 4 vai precisar: o trace so tem as escritas nas portas, nao o conteudo dos DMAs. Registrado para o plano da Etapa 4.
