# Fase 2 · Pruebas manuales

Lo que necesita teclado y ojos. Se irá ampliando con cada bloque.

## Bloque 2.0

### 0. `doctor` desde una terminal real elige el `claude` bueno

Es la comprobación de la regresión que encontró la planificadora: con una terminal de verdad, `lithemux` caía al `claude` viejo.

1. En una pestaña de terminal normal (no desde un script):

   ```sh
   cd ~/Personal/lithemux
   time ./target/release/lithemux doctor
   ```

2. La línea `claude` debe decir `~/.local/bin/claude`, y esa instalación debe ser la **2.1.330** y llevar la marca `<- used by lithemux`.
3. **No** debe aparecer ninguna línea `warn  claude lookup`.
   - Aparece una línea nueva, `claude remembered`, con la misma ruta del paso 2. Si alguna vez muestra otra cosa, bórrala con `lithemux config unset last_claude_path` y avísame.
4. Debe tardar **menos de 1 s**. La primera corrida de un binario recién compilado puede tardar unos segundos por la verificación de macOS; mide la segunda.

- [ ] PASA  - [ ] FALLA

### A. El `claude` de adentro guarda su conversación (punto 19)

Es la comprobación del fallo más grave de la prueba de la Fase 1.

1. Desde **esta misma terminal de cmux con Claude Code**, que es donde falló:

   ```sh
   cd ~/Personal/lithemux
   ./target/release/lithemux-app
   ```

2. En un panel escribe `env | grep -c -E '^(CLAUDECODE|CLAUDE_CODE_CHILD_SESSION|CMUX_)'`. Debe responder **0**.
3. Escribe `claude`. **No** debe aparecer el aviso «Transcript saving is off — inherited CLAUDE_CODE_CHILD_SESSION marker».
4. Dile algo corto, sal con `/exit` y escribe `claude --resume`. La conversación debe estar en la lista.

- [ ] PASA  - [ ] FALLA

Si falla, copia el aviso que salga y el resultado del paso 2.

### B. La primera tecla muerta (punto 20)

No pude reproducirlo sin teclado, así que hace falta un registro de lo que macOS le entrega a la app.

1. Cierra la app si está abierta y lánzala con el registro activado, guardando la salida:

   ```sh
   cd ~/Personal/lithemux
   LITHEMUX_IME_LOG=1 ./target/release/lithemux-app 2> /tmp/lithemux-ime.log
   ```

2. **Sin hacer clic en ningún lado**, escribe en el panel izquierdo: `´` y luego `a`. Anota si salió `á` o `´a`.
3. Escribe `´` + `e`, y después `´` + `i`.
4. Pasa al panel derecho con `cmd+option+→` y repite `´` + `a`. Anota qué salió.
5. Haz clic en el panel izquierdo y repite `´` + `a`.
6. Cierra con `cmd+q` y pásame `/tmp/lithemux-ime.log`, junto con lo que anotaste en los pasos 2, 4 y 5.

El registro **no guarda lo que tecleas**: solo qué evento llegó, si había una composición en curso o una tecla muerta, y longitudes. Solo se activa con `LITHEMUX_IME_LOG=1` exactamente.

- [ ] La primera salió bien  - [ ] La primera salió `´a` (en el paso: ___)

### C. `waiting` vuelve a `working` (bug 4)

1. `lithemux open <categoría>/<sesión>` de una sesión con perfil `claude`.
2. Pídele algo que necesite permiso (por ejemplo, crear un archivo).
3. Con el diálogo de permiso abierto, en otra terminal: `lithemux status`. La sesión debe estar en `waiting`.
4. Aprueba el permiso. Mientras Claude sigue trabajando, repite `lithemux status`: debe decir `working`.
5. Cuando termine el turno, `idle`.

- [ ] PASA  - [ ] FALLA

## Bloque 2.1 · SessionHost

La ventana de este bloque es **temporal**: `lithemux-app --open <categoría>/<sesión>` abre esas sesiones. Las dos primeras se ven lado a lado; las demás corren sin panel y aparecen en la línea de abajo. Cada panel dice `sesión · estado · running`; mientras se prepara dice `opening…`.

Necesitas una categoría con al menos dos sesiones de Claude. Si no la tienes:

```sh
cd ~/Personal/lithemux
./target/release/lithemux cat add prueba --name Prueba --root ~/Personal/lithemux
./target/release/lithemux new prueba/uno --role "prueba uno"
./target/release/lithemux new prueba/dos --role "prueba dos"
```

> **Desde el bloque 3, `--open` ya no existe con ventana.** En los pasos D, E y F, donde dice `lithemux-app --open prueba/uno`, abre `./target/release/lithemux-app` y haz clic en la tarjeta de la sesión.

### D. Las sesiones viven en la app y la CLI las ve

1. `./target/release/lithemux-app --open prueba/uno --open prueba/dos`
   - La ventana aparece al instante y los paneles dicen `opening…` un momento.
2. En cada panel arranca tu `claude` de verdad, con el nombre `prueba-uno` y `prueba-dos`.
3. En otra terminal: `./target/release/lithemux status prueba`. Las dos deben decir `open (pid …)`.
4. En esa misma terminal: `./target/release/lithemux open prueba/uno`. Debe negarse con `the session is already open (pid …)`.
5. Escríbele algo a `prueba/uno`. La etiqueta del panel debe pasar a `working` y, al terminar, a `idle`. `lithemux status prueba` debe decir lo mismo.

- [ ] PASA  - [ ] FALLA

### E. Una sola instancia y «abierta en otra terminal»

1. Con la app abierta, en otra terminal: `./target/release/lithemux-app --open prueba/dos`. Debe salir con `lithemux is already running (pid …)`.
2. Cierra la app con `cmd+q`. En una terminal: `./target/release/lithemux open prueba/uno` y déjalo abierto.
3. En otra: `./target/release/lithemux-app --open prueba/uno --open prueba/dos`. El panel de `prueba/uno` no debe arrancar `claude`: debe decir `open in another terminal (pid …)`. El de `prueba/dos` arranca normal.

- [ ] PASA  - [ ] FALLA

### F. Cerrar no deja huérfanos y reabrir reanuda

1. Con dos sesiones abiertas en la app, anota algo que le hayas dicho a `prueba/uno`.
2. `cmd+q`. Después: `pgrep -fl claude`. (Repítelo otra vez cerrando la terminal desde la que lanzaste la app, en vez de `cmd+q`.) No debe quedar ningún `claude` de esas sesiones (los de tus otras terminales sí estarán).
3. `./target/release/lithemux status prueba`: las dos `closed`.
4. Vuelve a abrir: `./target/release/lithemux-app --open prueba/uno`. Debe **reanudar la misma conversación**, con lo que le dijiste antes.

- [ ] PASA  - [ ] FALLA


## Bloque 3 · Barra lateral y área de trabajo

Preparación: `cargo build --release` y abre `./target/release/lithemux-app` (sin argumentos). Usa la categoría `prueba` de los pasos anteriores o crea una desde la app.

Lo que sigue **no tiene test automático** (es el cableado de la interfaz, D-44), así que estos pasos son la comprobación.

### G. Barra lateral: crear, renombrar, ordenar, borrar

1. «Nueva categoría»: escribe un nombre, «Elegir carpeta», «Crear». Aparece la categoría.
2. El `+` de la categoría: escribe `Mi API` y Enter. Se crea la sesión, se abre en el panel y arranca `claude`.
3. Clic derecho en la tarjeta: «Renombrar», «Fijar» (sube a «Fijadas»), «Marcar como madre» (pasa a ser la cabecera y las demás cuelgan), «Mantener viva», «Subir»/«Bajar».
4. Marca otra sesión de la misma categoría como madre: debe negarse con un aviso abajo.
5. Arrastra una tarjeta sobre otra de su categoría: toma su lugar. Arrastra una categoría sobre otra.
6. Clic en el nombre de la categoría: se contrae y se expande.
7. Clic derecho, «Borrar»: pide confirmación; al aceptar, la sesión se cierra y desaparece.

- [ ] PASA  - [ ] FALLA

### H. Tarjetas: estado, vista previa, no leídos, puertos

1. Abre dos sesiones de `claude` en pestañas del mismo panel. Escríbele algo a una y cambia a la otra pestaña.
2. Mientras trabaja, su tarjeta dice «working» con tu mensaje como vista previa. Al terminar: «idle», la vista previa es lo último que dijo Claude y aparece el contador de no leídos.
3. Vuelve a su pestaña: el contador desaparece.
4. Haz que pida un permiso (por ejemplo, un comando de Bash): la tarjeta se resalta y dice «Claude is waiting for your input».
   - Con la sesión a la vista pero **la ventana de lithemux en segundo plano**, el contador de no leídos sube; al volver a la ventana, se pone en cero.
5. En una terminal de la app: `python3 -m http.server 8765`. En unos segundos su tarjeta muestra `:8765`; un clic abre `http://localhost:8765` en tu navegador. Al parar el servidor, el puerto desaparece.

- [ ] PASA  - [ ] FALLA

### I. Área de trabajo: paneles, pestañas y atajos

1. La app arranca con un solo panel.
2. `cmd+d` divide a la derecha y `cmd+shift+d` hacia abajo, hasta 4 paneles; el quinto se niega con un aviso.
3. Los tres botones de cada panel hacen lo mismo (nueva terminal, dividir a la derecha, dividir hacia abajo); el cuarto, el del navegador, está apagado.
4. Arrastra un divisor: los paneles cambian de tamaño y las terminales se reajustan.
5. Clic en una tarjeta: se abre en el panel que tiene el foco (el del borde azul).
6. Arrastra una pestaña a otro panel y a otro lugar entre las pestañas.
7. `cmd+]`/`cmd+[` cambian de panel; `cmd+shift+]`/`cmd+shift+[` de pestaña; `cmd+w` cierra la pestaña y `cmd+shift+w` el panel.
8. `ctrl+tab` dentro de una terminal le llega al programa de dentro (ya no cambia de pestaña).

- [ ] PASA  - [ ] FALLA

### J. Reabrir mientras se cierra y «abriendo»

1. Con una sesión de `claude` abierta, cierra su pestaña (`cmd+w`) y haz clic en su tarjeta enseguida.
2. La tarjeta puede decir «closing» un instante y después «opening»; la sesión vuelve a arrancar sola, **con ese único clic**. **Nunca** debe decir «open in another terminal» ni quedarse esperando un segundo clic.
3. Al abrir una sesión que estaba «ended», la tarjeta dice «opening» hasta que Claude arranca, no «ended».

- [ ] PASA  - [ ] FALLA

### K. El layout se restaura

1. Deja dos o tres paneles con varias pestañas y un divisor movido. `cmd+q`.
2. Abre la app de nuevo: mismos paneles, pestañas, tamaños y foco. **Ningún `claude` arranca solo:** cada pestaña muestra su estado y el botón «Open».
3. Abre una segunda `./target/release/lithemux-app` desde otra terminal: sale con `lithemux is already running (pid …)` y no aparece otra ventana.
4. Español. El cambio de idioma llega con la ventana de Ajustes (§2.6) y `lithemux config` todavía no acepta `language`. Para verlo ahora, con la app cerrada:
   `sqlite3 "$HOME/Library/Application Support/lithemux/lithemux.db" "insert or replace into settings(key,value) values('language','es')"`
   (ajusta la ruta a la que diga `lithemux doctor` en «database»). Al abrir la app, todo sale en español. Para volver: el mismo comando con `'en'`.

- [ ] PASA  - [ ] FALLA

### L. Correcciones de la auditoría del bloque 3

1. **Home largo.** `LITHEMUX_HOME="$HOME/una-carpeta-con-un-nombre-muy-largo/otra-carpeta-con-un-nombre-muy-largo/otra-mas-con-un-nombre-largo" ./target/release/lithemux-app`: la app arranca (antes salía con «path must be shorter than SUN_LEN»). Crea una categoría y una sesión: su tarjeta cambia de estado al usarla, señal de que el hook encuentra el socket.
2. **Borrar lo abierto.** Con una sesión de `claude` abierta en la app: `./target/release/lithemux rm prueba/uno --yes` debe negarse con «already open (pid …)». Con una **terminal** abierta en la app, `lithemux rm` de esa sesión de shell sí la borra, y en unos segundos la app cierra su pestaña y su programa.
3. **Subir y bajar.** En una categoría con madre y con alguna sesión fijada, «Subir»/«Bajar» mueven la tarjeta un lugar entre las que se ven debajo de la categoría, sin «saltos que no hacen nada».
4. **Errores en español.** Con la interfaz en español, marca una segunda madre: el aviso sale en español.

- [ ] PASA  - [ ] FALLA
