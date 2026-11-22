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

