"use strict";

(() => {
  const key = `memorama:${new URL("./", location.href).pathname}:highscores`;
  const encoder = new TextEncoder();
  const decoder = new TextDecoder("utf-8", { fatal: true });
  let pending;

  function report(error) {
    console.error("No se pudo acceder a los records JSON.", error);
    document.querySelector("#storage").textContent =
      "No se pudieron cargar o guardar los records. Revisa los permisos de almacenamiento del navegador.";
  }

  miniquad_add_plugin({
    name: "memorama_highscores",
    version: 1,
    register_plugin(imports) {
      imports.env.highscores_size = () => {
        try {
          const json = localStorage.getItem(key);
          if (json === null) {
            pending = undefined;
            return 0;
          }
          if (json.length === 0) throw new Error("JSON de records vacio.");
          pending = encoder.encode(json);
          return pending.length;
        } catch (error) {
          report(error);
          return -1;
        }
      };
      imports.env.highscores_read = (ptr, len) => {
        if (!pending || pending.length !== len) {
          report(new Error("Longitud de records inconsistente."));
          return -1;
        }
        new Uint8Array(wasm_memory.buffer, ptr, len).set(pending);
        pending = undefined;
        return len;
      };
      imports.env.highscores_write = (ptr, len) => {
        try {
          const json = decoder.decode(new Uint8Array(wasm_memory.buffer, ptr, len));
          JSON.parse(json);
          localStorage.setItem(key, json);
          document.querySelector("#storage").textContent = "";
          return 0;
        } catch (error) {
          report(error);
          return -1;
        }
      };
    },
  });
})();
