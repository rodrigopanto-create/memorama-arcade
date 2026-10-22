# Memorama Arcade

Juego en Rust y Macroquad, disponible como aplicación nativa o PWA/WebAssembly.

## Escritorio

```sh
cargo run --release
```

## Compilar la PWA

Requisitos: Rust instalado con `rustup` y una shell POSIX (macOS,
Linux o WSL). La primera compilación necesita internet para descargar las
dependencias y, si falta, el target `wasm32-unknown-unknown`.

```sh
sh scripts/build-web.sh
cargo install miniserve --locked # Solo si no lo tienes instalado
miniserve dist --index index.html --port 8085
```

Abre <http://localhost:8085>. No abras el HTML con `file://`: WASM y los service
workers necesitan un servidor HTTP. Para detener el servidor, usa Ctrl+C.

El script genera `dist/` con el WASM optimizado, el loader **de la misma versión
de Miniquad que usa Macroquad según Cargo.lock**, HTML, estilos, manifiesto, iconos PNG y un
service worker versionado por contenido. El empaquetador está escrito en Rust;
no necesita Python, Node, wasm-pack,
wasm-bindgen ni un CDN; respeta `Cargo.lock` y `CARGO_TARGET_DIR`.

## Publicar e instalar

- Sube **todo el contenido de `dist/`** a un hosting estático con **HTTPS**,
  por ejemplo GitHub Pages o tu servidor. También funciona bajo una subruta
  como `/memorama/`, sin cambiar los archivos.
- Sirve `.wasm` como `application/wasm`, `.js` como JavaScript y
  `.webmanifest` como `application/manifest+json`. `miniserve`
  permite probarlo localmente.
- Evita caché HTTP prolongada para `sw.js`, `index.html` y los recursos sin
  hash en su nombre; usa `Cache-Control: no-cache` en producción y publica
  todos los archivos juntos para no mezclar versiones.
- En navegadores compatibles aparece **Instalar**. También puedes usar el
  menú del navegador; en iPhone/iPad, **Compartir > Agregar a inicio**.
- Espera el mensaje **Listo para jugar sin conexion** antes de cerrar la
  primera visita. Después podrás volver a abrir o recargar sin internet.
  El navegador puede eliminar ese almacenamiento si falta espacio.
- Una nueva compilación cambia la versión de la caché. Al detectarla, se
  ofrece **Actualizar juego**; aceptarlo recarga y reinicia la partida.
  Cierra otras pestañas del juego antes de actualizar.

## Controles y comportamiento web

Haz clic o toca las cartas. Enter/Espacio inicia el juego; las flechas y
Enter/Espacio permiten registrar iniciales. Los botones bajo el canvas
ofrecen esos controles sin teclado físico.

En web se conserva un área lógica de 800x600 que se ajusta proporcionalmente
a la pantalla, incluido el mapeo de clics y toques. En escritorio se mantiene
el tamaño de ventana original. Se usa la aleatoriedad de Macroquad para
barajar en ambos destinos sin dependencias de JavaScript adicionales.
El loader incluido cubre la configuración actual sin audio (los efectos son
stubs). Si activas la feature `audio` de Macroquad u otros plugins, también
deberás incluir sus puentes JavaScript correspondientes en el empaquetado.

## Persistencia de récords en JSON

Se cargan los récords al iniciar y se guardan al confirmar las tres iniciales.
Se conserva el top 10 ordenado por puntaje. Si todavía no hay datos, se usa
la tabla inicial RET/ARC/CPU hasta guardar el primer récord.

- **Escritorio:** `highscores.json` en el directorio desde el que ejecutas el
  juego. Se escribe primero un temporal y se renombra para reemplazar la tabla.
- **PWA:** JSON en `localStorage`; la clave para la ruta raíz es
  `memorama:/:highscores` y para `/memorama/` es `memorama:/memorama/:highscores`.
  Persiste al recargar, cerrar y actualizar, también sin conexión.
  Está separado por origen (incluido el puerto) y ruta: cambiar de 8080 a 8085
  no migra los récords del origen anterior.
- El formato es un arreglo, por ejemplo:

  ```json
  [
    { "name": "RET", "score": 2500 },
    { "name": "ARC", "score": 2000 },
    { "name": "CPU", "score": 1500 }
  ]
  ```

Solo se aceptan iniciales de tres letras A-Z y puntajes enteros no negativos.
Un JSON corrupto no se sobrescribe: se muestra una advertencia y se permite
jugar sin guardar hasta corregirlo y reiniciar. Los fallos de guardado también
se muestran en pantalla y se registran en la consola.

`miniserve` solo sirve archivos; la PWA no modifica un JSON del servidor ni
sincroniza dispositivos. Borrar los datos del sitio elimina los récords web;
el modo privado o las restricciones de almacenamiento pueden impedir guardar.

## Verificación

```sh
cargo check --locked
cargo test --locked
sh scripts/build-web.sh
```

Prueba en el navegador las tres dificultades, las iniciales con los botones
y la recarga sin conexión después de que el service worker esté listo.
Para comprobar una actualización, recompila con un cambio, vuelve a visitar
la página en línea y acepta **Actualizar juego**.
