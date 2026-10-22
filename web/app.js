"use strict";

const status = document.querySelector("#status");
const canvas = document.querySelector("#glcanvas");
const offline = document.querySelector("#offline");
const install = document.querySelector("#install");
const update = document.querySelector("#update");
let failed = false;

function showError(message, error) {
  failed = true;
  console.error(message, error);
  status.textContent = message;
  status.hidden = false;
}

window.addEventListener("error", (event) => {
  showError("No se pudo ejecutar el juego. Recarga la pagina e intenta de nuevo.", event.error);
});
window.addEventListener("unhandledrejection", (event) => {
  showError("Ocurrio un error. Revisa tu conexion y recarga la pagina.", event.reason);
});

canvas.addEventListener("pointerdown", () => canvas.focus({ preventScroll: true }));
canvas.addEventListener("contextmenu", (event) => event.preventDefault());

let installPrompt;
window.addEventListener("beforeinstallprompt", (event) => {
  event.preventDefault();
  installPrompt = event;
  install.hidden = false;
});
install.addEventListener("click", async () => {
  if (!installPrompt) return;
  install.disabled = true;
  try {
    await installPrompt.prompt();
    await installPrompt.userChoice;
  } catch (error) {
    showError("No se pudo abrir la instalacion. Usa el menu del navegador.", error);
  } finally {
    installPrompt = undefined;
    install.hidden = true;
    install.disabled = false;
  }
});
window.addEventListener("appinstalled", () => {
  install.hidden = true;
  installPrompt = undefined;
});

async function registerOffline() {
  if (!("serviceWorker" in navigator) || !window.isSecureContext) {
    offline.textContent = "El modo sin conexion requiere HTTPS o localhost y un navegador compatible.";
    offline.classList.remove("visually-hidden");
    return;
  }
  try {
    const registration = await navigator.serviceWorker.register("./sw.js");
    function offerUpdate() {
      if (registration.waiting && navigator.serviceWorker.controller) update.hidden = false;
    }
    offerUpdate();
    registration.addEventListener("updatefound", () => {
      const worker = registration.installing;
      if (!worker) return;
      worker.addEventListener("statechange", () => {
        if (worker.state === "installed") offerUpdate();
        if (worker.state === "redundant") {
          offline.textContent = "No se pudo guardar la nueva version para jugar sin conexion.";
          offline.classList.remove("visually-hidden");
          console.error("Fallo la instalacion del service worker.");
        }
      });
    });
    update.addEventListener("click", () => {
      if (registration.waiting) {
        update.disabled = true;
        registration.waiting.postMessage({ type: "SKIP_WAITING" });
      }
    });
    let reloading = false;
    navigator.serviceWorker.addEventListener("controllerchange", () => {
      if (update.disabled && !reloading) {
        reloading = true;
        location.reload();
      }
    });
    await navigator.serviceWorker.ready;
    offline.textContent = "Listo para jugar sin conexion. En iPhone/iPad: Compartir > Agregar a inicio.";
  } catch (error) {
    offline.textContent = "No se pudo preparar el modo sin conexion. El juego sigue disponible en linea.";
    offline.classList.remove("visually-hidden");
    console.error("No se pudo registrar la PWA.", error);
  }
}

if (typeof load !== "function" || typeof WebAssembly === "undefined") {
  showError("No se pudo cargar WebAssembly. Usa un navegador actualizado y recarga la pagina.");
} else {
  load("./memorama_arcade.wasm");
  const started = performance.now();
  function waitForGame() {
    if (failed) return;
    if (typeof wasm_exports !== "undefined" && wasm_exports && typeof gl !== "undefined" && gl) {
      status.hidden = true;
      canvas.focus({ preventScroll: true });
    } else if (performance.now() - started > 30000) {
      showError("El juego no pudo iniciar. Revisa tu conexion, el soporte WebGL y el tipo MIME application/wasm del servidor.");
    } else {
      setTimeout(waitForGame, 100);
    }
  }
  setTimeout(waitForGame, 100);
}
registerOffline();
