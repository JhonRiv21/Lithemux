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
