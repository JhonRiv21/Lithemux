# Fase 1 · Prueba manual del spike

Estas seis pruebas necesitan teclado y ojos. Marca cada una con PASA o FALLA y, si falla, anota qué viste (una captura ayuda). La prueba 7 ya está medida en `F1-reporte.md`.

## Cómo lanzar la app

Desde cualquier terminal:

```sh
cd ~/Personal/lithemux
./target/release/lithemux-app
```

Se abre una ventana «lithemux» con **dos terminales lado a lado**, cada uno con tu zsh de login en la carpeta desde la que lanzaste la app. El panel con foco tiene el borde azul.

Si el binario no existe o está viejo:

```sh
export PATH=/opt/homebrew/opt/rustup/bin:$PATH
cargo build --release -p lithemux-app
```

Antes de abrir la ventana puedes comprobar que el binario sabe dibujar texto:

```sh
./target/release/lithemux-app --check-text
```

Debe responder `text rendering: ok`. Si no, no sigas: avísame con lo que imprima.

Atajos del spike:

| Atajo | Qué hace |
|---|---|
| `cmd+option+→` / `cmd+option+←` | pasa el foco al otro panel |
| clic en un panel | le da el foco |
| `cmd+c` / `cmd+v` | copiar la selección / pegar |
| `cmd+q` | salir |

Para cerrar basta `cmd+q` o cerrar la ventana.

## 1 · Claude dentro del terminal

1. En el panel izquierdo escribe `claude` y pulsa enter.
2. Escribe un mensaje corto, por ejemplo `hola`, y envíalo.
3. Pídele algo que pinte bastante: `muéstrame un diff de ejemplo con colores`.
4. Abre `/help` y ciérralo con `esc`.

5. Sal de Claude y pega esto en zsh; pinta un degradado con colores de 24 bits:

   ```sh
   for i in $(seq 0 79); do printf '\033[48;2;%d;%d;255m ' $((i*3)) $((255-i*3)); done; printf '\033[0m\n'
   ```

**Se espera:** la pantalla de Claude completa (logo, caja del prompt con sus bordes, barra de estado), con colores, sin parpadeo mientras responde y sin restos de texto al redibujar. El degradado del paso 5 es una franja continua, de verde azulado a magenta, sin escalones ni rayas verticales.

- [ ] PASA
- [ ] FALLA — nota:

## 2 · Teclado en español

**Dentro de Claude** (ábrelo con `claude`), en su prompt, escribe sin enviar:

1. `´` y luego `a`, `e`, `i`, `o`, `u` → `á é í ó ú`.
2. `ñ`, `Ñ`, `¿`, `¡`, `ü` (con `¨` + `u`).
3. Símbolos con option: `@`, `#`, `[`, `]`, `{`, `}`, `~`, `\`, `|`.
4. Mantén pulsada la `a` hasta que salga el menú de acentos y elige `á` con el número o las flechas.

**Se espera:** cada carácter aparece una sola vez y bien. Con la tecla muerta, el acento se ve subrayado en el cursor hasta que pulsas la vocal. En el paso 4 el menú sale junto al cursor y queda `á`, no `aá`.

- [ ] PASA
- [ ] FALLA — nota:

## 3 · `shift+enter`

1. Con Claude abierto, escribe `uno`, pulsa `shift+enter`, escribe `dos`.
2. Pulsa enter.

**Se espera:** tras `shift+enter` el cursor baja a una línea nueva dentro del prompt, sin enviar. Al pulsar enter se envía un solo mensaje con las dos líneas.

- [ ] PASA
- [ ] FALLA — nota:

## 4 · Copiar, pegar y seleccionar

1. En zsh ejecuta `printf 'uno\ndos\ntres\n'`.
2. Arrastra con el ratón sobre las tres líneas; quedan resaltadas. Pulsa `cmd+c`.
3. Abre Claude y pulsa `cmd+v`.
4. Doble clic sobre una palabra y triple clic sobre una línea.
5. Copia algo con acentos desde otra app (`año ¿qué?`) y pégalo.

6. Pegado hostil. En **otra terminal** (no en lithemux) deja esto en el portapapeles:

   ```sh
   printf 'echo a\e[201~echo PWNED\n' | pbcopy
   ```

   Vuelve a lithemux y, en zsh, pulsa `cmd+v`. No pulses enter.

**Se espera:** en el paso 3 Claude recibe las tres líneas como un solo pegado (las muestra en el prompt o como «[Pasted text]»), sin enviar nada por su cuenta. El doble clic selecciona la palabra y el triple la línea. Los acentos llegan intactos. En el paso 6 el texto queda escrito en la línea, sin ejecutarse: **no debe aparecer una línea de salida que diga `PWNED`** hasta que tú pulses enter.

- [ ] PASA
- [ ] FALLA — nota:

## 5 · Redimensionar

1. Con Claude abierto y algo de conversación en pantalla, arrastra el borde de la ventana: más ancha, más estrecha, más baja.
2. Ponla en pantalla completa y vuelve.

**Se espera:** Claude se reacomoda al nuevo ancho, la caja del prompt sigue entera y no quedan líneas cortadas ni duplicadas.

- [ ] PASA
- [ ] FALLA — nota:

## 6 · Dos paneles y foco

1. Abre `claude` en el izquierdo y deja zsh en el derecho.
2. Pulsa `cmd+option+→`: el borde azul pasa al derecho. Escribe `ls` y enter.
3. Pulsa `cmd+option+←` y escribe en Claude.
4. Cambia de foco también con clic.

5. Con Claude abierto en un panel, sal con `cmd+q`. En otra terminal:

   ```sh
   pgrep -fl claude
   ```

**Se espera:** lo que escribes va solo al panel con foco; el otro no cambia. El cursor del panel sin foco se ve hueco. Cada panel es independiente: cerrar el shell de uno (`exit`) no afecta al otro. En el paso 5 no queda ningún `claude` lanzado desde lithemux (si tienes otras sesiones de Claude abiertas en cmux, esas sí aparecen; compara con la lista de antes de abrir la app).

- [ ] PASA
- [ ] FALLA — nota:

## 7 · Rendimiento, medido por ti (opcional)

Mis números están en `F1-reporte.md`. Para repetirlos, con la app abierta y los dos terminales en reposo, en otra terminal:

```sh
PID=$(pgrep -x lithemux-app)
top -pid $PID -stats pid,cpu,mem -l 10 -s 1 | grep "^ *$PID"   # CPU: debe rondar 0.0
footprint $PID | grep phys_footprint                            # meta: menos de 150 MB
lsof -nP -i -a -p $PID                                          # no debe imprimir nada
```

Para el volcado, dentro de un panel de lithemux:

```sh
yes 'the quick brown fox jumps over the lazy dog 0123456789' | head -c 50000000 > /tmp/lithemux-50mb.txt
cat /tmp/lithemux-50mb.txt
```

Mientras corre, cambia de panel con `cmd+option+→` y escribe: la ventana debe seguir respondiendo. Repite `footprint` al terminar y borra el archivo con `rm /tmp/lithemux-50mb.txt`.

- [ ] PASA
- [ ] FALLA — nota:

## Lo que el spike no hace (no cuenta como falla)

- El cursor no parpadea.
- No hay pestañas, barra lateral ni título por sesión; la ventana es solo para decidir si la tecnología sirve.
- El ratón no se le reporta al programa (selecciona texto siempre).
- Cuando el shell de un panel termina, el panel queda vacío en vez de cerrarse.
- `option` + letra escribe el carácter de la distribución, así que los atajos de Claude con `option` (`option+p`, `option+t`) no funcionan en el spike. Es una decisión anotada (D-17), con ajuste previsto para la Fase 2.
