// Registro de Service Worker, instalacao PWA no app (Chrome/Edge + guia iOS Safari) e status offline.

export function detectarModoStandalone() {
  if (typeof window === "undefined") return false;
  return Boolean(
    (window.matchMedia && window.matchMedia("(display-mode: standalone)").matches) ||
      window.navigator?.standalone === true,
  );
}

export function detectarIosSafari() {
  if (typeof navigator === "undefined") return false;
  const ua = navigator.userAgent || "";
  const ehIos = /iphone|ipad|ipod/i.test(ua);
  const ehWebkit = /webkit/i.test(ua);
  const naoEhOutroNav = !/crios|fxios|edgios|opios/i.test(ua);
  return ehIos && ehWebkit && naoEhOutroNav;
}

export function inicializarPwa({
  btnInstalar,
  badgeOffline,
  modalIos,
  btnFecharModalIos,
}) {
  let promptInstalacao = null;
  const standalone = detectarModoStandalone();
  const ehIosSafari = detectarIosSafari();

  // Registra Service Worker se suportado em contexto seguro (HTTPS ou localhost).
  if (typeof navigator !== "undefined" && "serviceWorker" in navigator) {
    const registrar = () => {
      navigator.serviceWorker.register("./sw.js").catch(() => {
        // Silencioso em ambientes sem suporte a SW (ex: iframes restritos).
      });
    };
    if (document.readyState === "complete") {
      registrar();
    } else {
      window.addEventListener("load", registrar, { once: true });
    }
  }

  function atualizarStatusRede(onlineForcado) {
    if (!badgeOffline) return;
    const online = typeof onlineForcado === "boolean"
      ? onlineForcado
      : (typeof navigator === "undefined" ? true : navigator.onLine !== false);
    badgeOffline.hidden = online;
  }

  atualizarStatusRede();
  window.addEventListener("online", () => atualizarStatusRede(true));
  window.addEventListener("offline", () => atualizarStatusRede(false));

  let ultimoFocoIos = null;

  function abrirModalIos() {
    if (!modalIos) return;
    ultimoFocoIos = typeof document !== "undefined" ? document.activeElement : null;
    modalIos.hidden = false;
    if (btnFecharModalIos) btnFecharModalIos.focus();
  }

  function fecharModalIos() {
    if (!modalIos) return;
    modalIos.hidden = true;
    if (ultimoFocoIos && typeof ultimoFocoIos.focus === "function") {
      ultimoFocoIos.focus();
    }
    ultimoFocoIos = null;
  }

  if (btnFecharModalIos) {
    btnFecharModalIos.addEventListener("click", fecharModalIos);
  }
  if (modalIos) {
    modalIos.addEventListener("click", (e) => {
      if (e.target === modalIos) fecharModalIos();
    });
    window.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && !modalIos.hidden) {
        fecharModalIos();
      }
    });
  }

  // Botao de instalacao no cabecalho.
  if (btnInstalar && !standalone) {
    if (ehIosSafari) {
      btnInstalar.hidden = false;
    }

    window.addEventListener("beforeinstallprompt", (e) => {
      e.preventDefault();
      promptInstalacao = e;
      btnInstalar.hidden = false;
    });

    window.addEventListener("appinstalled", () => {
      promptInstalacao = null;
      btnInstalar.hidden = true;
    });

    btnInstalar.addEventListener("click", async () => {
      if (promptInstalacao) {
        try {
          promptInstalacao.prompt();
          const escolha = await promptInstalacao.userChoice;
          if (escolha && escolha.outcome === "accepted") {
            btnInstalar.hidden = true;
          }
        } catch {
          // Ignora cancelamentos do navegador.
        } finally {
          promptInstalacao = null;
        }
        return;
      }

      if (ehIosSafari) {
        abrirModalIos();
      }
    });
  }
}
