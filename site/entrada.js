// Entrada deterministica de teclado, gamepad e toque para o controle de 3 botoes do Mega Drive.

export const MAPA_TECLAS_CODE = {
  // Direcionais
  ArrowUp: 0x01, KeyW: 0x01, Numpad8: 0x01,
  ArrowDown: 0x02, KeyS: 0x02, Numpad2: 0x02,
  ArrowLeft: 0x04, KeyA: 0x04, Numpad4: 0x04,
  ArrowRight: 0x08, KeyD: 0x08, Numpad6: 0x08,

  // Botoes A, B, C (No Sonic 1, todos sao pulo)
  KeyX: 0x10, KeyK: 0x10, // B
  KeyC: 0x20, KeyL: 0x20, // C
  KeyZ: 0x40, KeyJ: 0x40, // A
  Space: 0x20, // Padrao C / Pulo

  // Start / Pausa
  Enter: 0x80, NumpadEnter: 0x80, KeyP: 0x80,
};

export const MAPA_TECLAS_KEY = {
  ArrowUp: 0x01, w: 0x01, W: 0x01,
  ArrowDown: 0x02, s: 0x02, S: 0x02,
  ArrowLeft: 0x04, a: 0x04, A: 0x04,
  ArrowRight: 0x08, d: 0x08, D: 0x08,
  x: 0x10, X: 0x10, k: 0x10, K: 0x10,
  c: 0x20, C: 0x20, l: 0x20, L: 0x20,
  z: 0x40, Z: 0x40, j: 0x40, J: 0x40,
  " ": 0x20, Spacebar: 0x20,
  Enter: 0x80, p: 0x80, P: 0x80,
};

export function ehModoTituloOuDemo(modo) {
  return modo === 0x00 || modo === 0x04 || modo === 0x08;
}

export function cancelarOpostas(mascara) {
  let limpa = mascara;
  if ((limpa & 0x0C) === 0x0C) limpa &= ~0x0C; // Esquerda + Direita
  if ((limpa & 0x03) === 0x03) limpa &= ~0x03; // Cima + Baixo
  return limpa;
}

export function calcularMascaraTeclado(teclasSeguradas, modo = 4) {
  let mascara = 0;
  const telaTituloOuDemo = ehModoTituloOuDemo(modo);

  for (const chave of teclasSeguradas) {
    let b = MAPA_TECLAS_CODE[chave] || MAPA_TECLAS_KEY[chave] || 0;
    if (chave === "Space" || chave === " " || chave === "Spacebar") {
      b = telaTituloOuDemo ? 0x80 : 0x20;
    }
    mascara |= b;
  }

  return cancelarOpostas(mascara);
}

export function calcularMascaraGamepad(gamepads, modo = 4) {
  if (!gamepads) return 0;
  const telaTituloOuDemo = ehModoTituloOuDemo(modo);
  let mascara = 0;

  for (let idx = 0; idx < gamepads.length; idx++) {
    const controle = gamepads[idx];
    if (!controle || !controle.connected) continue;

    const botao = (i) => Boolean(controle.buttons?.[i]?.pressed || (controle.buttons?.[i]?.value > 0.45));
    const ax = (i) => controle.axes?.[i] || 0;

    if (botao(12) || ax(1) < -0.38) mascara |= 0x01; // Cima
    if (botao(13) || ax(1) > 0.38) mascara |= 0x02;  // Baixo
    if (botao(14) || ax(0) < -0.38) mascara |= 0x04; // Esquerda
    if (botao(15) || ax(0) > 0.38) mascara |= 0x08;  // Direita

    // Suporte a eixos adicionais (DirectInput D-pad axes 4/5)
    if (controle.axes && controle.axes.length >= 6) {
      if (ax(5) < -0.38) mascara |= 0x01;
      if (ax(5) > 0.38) mascara |= 0x02;
      if (ax(4) < -0.38) mascara |= 0x04;
      if (ax(4) > 0.38) mascara |= 0x08;
    }

    if (botao(0)) mascara |= 0x10; // B
    if (botao(1)) mascara |= 0x20; // C
    if (botao(2)) mascara |= 0x40; // A
    if (botao(3)) mascara |= 0x20; // C
    if (botao(4) || botao(5) || botao(6) || botao(7)) mascara |= 0x10;

    if (botao(9) || botao(8) || botao(16)) mascara |= 0x80;

    if (telaTituloOuDemo && (botao(0) || botao(1) || botao(2) || botao(3))) {
      mascara |= 0x80;
    }

    if (mascara !== 0) break;
  }

  return cancelarOpostas(mascara);
}

export function calcularMascaraToque(toquesAtivos, modo = 4) {
  if (!toquesAtivos || toquesAtivos.size === 0) return 0;
  let mascara = 0;
  const telaTituloOuDemo = ehModoTituloOuDemo(modo);

  if (toquesAtivos.has("cima")) mascara |= 0x01;
  if (toquesAtivos.has("baixo")) mascara |= 0x02;
  if (toquesAtivos.has("esquerda")) mascara |= 0x04;
  if (toquesAtivos.has("direita")) mascara |= 0x08;

  if (toquesAtivos.has("b")) mascara |= 0x10;
  if (toquesAtivos.has("c")) mascara |= 0x20;
  if (toquesAtivos.has("a")) mascara |= 0x40;
  if (toquesAtivos.has("start")) mascara |= 0x80;

  if (telaTituloOuDemo && (mascara & 0x70) !== 0) {
    mascara |= 0x80;
  }

  return cancelarOpostas(mascara);
}

export function combinarMascaras(teclado, gamepad, toque) {
  return cancelarOpostas(teclado | gamepad | toque);
}

export function criarControladorEntrada({
  obterTela,
  badgeGamepad,
  controlesToque,
  btnToque,
  onAcaoEspecial,
  onInteracao,
  estaJogando,
  onMudancaEntrada,
}) {
  const teclasSeguradas = new Set();
  const toquesAtivos = new Set();
  const temSuporteToque = ("ontouchstart" in window) || (navigator.maxTouchPoints > 0);
  let toqueAtivo = temSuporteToque;
  let temGamepadConectado = false;

  const cacheKeycaps = new Map();
  document.querySelectorAll(".tecla-cap[data-code]").forEach((cap) => {
    const code = cap.getAttribute("data-code");
    if (!code) return;
    let lista = cacheKeycaps.get(code);
    if (!lista) {
      lista = [];
      cacheKeycaps.set(code, lista);
    }
    lista.push(cap);
  });

  function iluminarKeycap(code, ativa) {
    if (!code) return;
    const caps = cacheKeycaps.get(code);
    if (!caps) return;
    for (let i = 0; i < caps.length; i++) {
      caps[i].classList.toggle("ativa", ativa);
    }
  }

  function notificarMudanca() {
    if (onMudancaEntrada) onMudancaEntrada();
  }

  window.addEventListener("keydown", (e) => {
    if (e.repeat) return;
    if (e.code === "KeyF" && !e.ctrlKey && !e.metaKey && !e.altKey) {
      if (onAcaoEspecial) onAcaoEspecial("fullscreen");
      e.preventDefault();
      return;
    }
    if (e.code === "F3" && !e.ctrlKey && !e.metaKey && !e.altKey) {
      if (onAcaoEspecial) onAcaoEspecial("hud");
      e.preventDefault();
      return;
    }

    const b = MAPA_TECLAS_CODE[e.code] || MAPA_TECLAS_KEY[e.key];
    if (b) {
      teclasSeguradas.add(e.code || e.key);
      iluminarKeycap(e.code, true);
      if (onInteracao) onInteracao();

      const tela = obterTela ? obterTela() : null;
      if (
        document.activeElement &&
        document.activeElement !== document.body &&
        document.activeElement !== tela &&
        document.activeElement.tagName !== "INPUT"
      ) {
        document.activeElement.blur();
      }

      e.preventDefault();
      notificarMudanca();
    }
  });

  window.addEventListener("keyup", (e) => {
    const b = MAPA_TECLAS_CODE[e.code] || MAPA_TECLAS_KEY[e.key];
    if (b) {
      teclasSeguradas.delete(e.code);
      teclasSeguradas.delete(e.key);
      iluminarKeycap(e.code, false);
      e.preventDefault();
      notificarMudanca();
    }
  });

  window.addEventListener("blur", () => {
    teclasSeguradas.clear();
    toquesAtivos.clear();
    document.querySelectorAll(".tecla-cap.ativa").forEach((cap) => cap.classList.remove("ativa"));
    if (controlesToque) {
      controlesToque.querySelectorAll(".btn-toque.ativo").forEach((b) => b.classList.remove("ativo"));
    }
    notificarMudanca();
  });

  function atualizarBadgeGamepad(gp) {
    if (!badgeGamepad) return;
    if (gp && gp.connected) {
      const nome = gp.id.length > 22 ? gp.id.slice(0, 20) + "..." : gp.id;
      badgeGamepad.className = "pill ativo";
      badgeGamepad.textContent = `🎮 ${nome}`;
    } else {
      badgeGamepad.className = "pill";
      badgeGamepad.textContent = "🎮 Gamepad";
    }
  }

  window.addEventListener("gamepadconnected", (e) => {
    temGamepadConectado = true;
    atualizarBadgeGamepad(e.gamepad);
  });

  window.addEventListener("gamepaddisconnected", () => {
    const gps = navigator.getGamepads ? navigator.getGamepads() : [];
    temGamepadConectado = false;
    let primeiro = null;
    for (let i = 0; i < gps.length; i++) {
      if (gps[i]?.connected) {
        temGamepadConectado = true;
        primeiro = gps[i];
        break;
      }
    }
    atualizarBadgeGamepad(primeiro);
    notificarMudanca();
  });

  function tratarPonteiroBotao(el, ativo) {
    const chave = el.getAttribute("data-dir") || el.getAttribute("data-botao");
    if (!chave) return;
    if (ativo) {
      toquesAtivos.add(chave);
      el.classList.add("ativo");
      if (onInteracao) onInteracao();
    } else {
      toquesAtivos.delete(chave);
      el.classList.remove("ativo");
    }
    notificarMudanca();
  }

  if (controlesToque) {
    const botoesToque = controlesToque.querySelectorAll(".btn-toque");
    botoesToque.forEach((btn) => {
      btn.addEventListener("pointerdown", (e) => {
        e.preventDefault();
        try { btn.setPointerCapture(e.pointerId); } catch (_) {}
        tratarPonteiroBotao(btn, true);
      });
      btn.addEventListener("pointerup", (e) => {
        e.preventDefault();
        try { btn.releasePointerCapture(e.pointerId); } catch (_) {}
        tratarPonteiroBotao(btn, false);
      });
      btn.addEventListener("pointercancel", (e) => {
        e.preventDefault();
        try { btn.releasePointerCapture(e.pointerId); } catch (_) {}
        tratarPonteiroBotao(btn, false);
      });
    });
  }

  if (btnToque) {
    if (toqueAtivo) {
      btnToque.classList.add("btn-azul");
      btnToque.classList.remove("btn-cinza");
    }
    btnToque.addEventListener("click", () => {
      toqueAtivo = !toqueAtivo;
      if (controlesToque && estaJogando && estaJogando()) {
        controlesToque.hidden = !toqueAtivo;
      }
      btnToque.classList.toggle("btn-azul", toqueAtivo);
      btnToque.classList.toggle("btn-cinza", !toqueAtivo);
      const tela = obterTela ? obterTela() : null;
      if (tela) tela.focus();
      notificarMudanca();
    });
  }

  function lerMascara(modo = 0x0C) {
    const gp = temGamepadConectado && navigator.getGamepads
      ? calcularMascaraGamepad(navigator.getGamepads(), modo)
      : 0;
    const tk = calcularMascaraTeclado(teclasSeguradas, modo);
    const tq = toqueAtivo ? calcularMascaraToque(toquesAtivos, modo) : 0;
    return combinarMascaras(tk, gp, tq);
  }

  return {
    lerMascara,
    lerParMascaras() {
      return {
        jogo: lerMascara(0x0C),
        titulo: lerMascara(0x04),
      };
    },
    temGamepad() {
      return temGamepadConectado;
    },
    toqueEstaAtivo() {
      return toqueAtivo;
    },
  };
}
