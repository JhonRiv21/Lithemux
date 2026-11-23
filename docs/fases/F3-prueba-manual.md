# Fase 3 · Pruebas manuales

Lo que necesita ojos, teclado y un Claude Code real. Se irá ampliando con cada bloque.

## Bloque 3A

**Antes de empezar:** compila con `cargo build --release` y abre la app con tu home de siempre. La primera vez migrará la base a la versión 5. Revisa que `~/Library/Application Support/lithemux/logs/app.log` (o tu `LITHEMUX_HOME/logs/app.log`) diga `INFO database migrated from schema v4 to v5`.

- [ ] PASA  - [ ] FALLA

### 1. Sonido cuando Claude termina y no lo estás mirando

1. Abre una sesión de Claude, pídele algo que tarde unos segundos («cuenta hasta 30 despacio») y **cambia a otra aplicación** antes de que termine.
2. Al terminar debe sonar **Glass**, el sonido del sistema. No aparece ningún banner.
3. Repite con la app al frente y esa sesión en su pestaña visible: **no** debe sonar.
4. Repite con la app al frente pero la sesión en una pestaña de atrás: **sí** debe sonar.
5. Haz terminar dos sesiones casi a la vez, fuera de la vista: suena **una** vez.
6. Cuando Claude pide un permiso (`waiting`), **no** suena; la tarjeta muestra el contador.
7. Una sesión abierta con `lithemux open <cat>/<sesión>` en otra terminal **no** hace sonar la app.

- [ ] PASA  - [ ] FALLA

### 2. Silencio por categoría

1. Clic derecho en el nombre de una categoría → «Turn off sound when done».
2. Repite el paso 1 con una sesión de esa categoría: **no** debe sonar.
3. El menú de la categoría ofrece ahora «Turn on sound when done».
4. Desde una terminal, `lithemux cat list` muestra `(sound when done off)` en esa categoría. `lithemux cat set <slug> --sound` lo vuelve a encender, y el menú lo refleja en unos segundos.

- [ ] PASA  - [ ] FALLA

### 3. La coordinadora encarga una tarea y recibe el reporte

Necesita una categoría con una sesión coordinadora y al menos otra sesión, las dos de Claude y **abiertas en la app** (lanzadas de nuevo después de compilar esta versión, para que tengan el contexto nuevo).

1. En la sesión que no es coordinadora, pregúntale: «¿a quién reportas y cómo?». Debe nombrar la dirección de su coordinadora (`<categoría>-<slug>`) y decir que lo hace con SendMessage.
2. En la coordinadora, pídele que le encargue a esa sesión una tarea corta («pídele a `<categoría>-<sesión>` que liste los archivos de su carpeta y te reporte»).
   - La coordinadora debe usar SendMessage con `notify_when_idle: true`.
   - **Ninguna de las dos sesiones debe quedar esperando un permiso por SendMessage o ListAgents.** Si alguna pide permiso, anota cuál y en qué modo de permisos estaba.
3. La sesión hace la tarea y le reporta a la coordinadora: qué hizo, qué quedó pendiente y dónde está el detalle. Después se detiene.
4. La coordinadora recibe el reporte (y el aviso de que la sesión quedó libre).
5. Hiberna la coordinadora y pídele a la otra sesión que le reporte algo: el envío falla y la sesión **te lo dice** en su respuesta. Nadie despierta a la coordinadora.

- [ ] PASA  - [ ] FALLA

### 4. Sin coordinadora

1. En una categoría sin sesión coordinadora, pregúntale a una sesión «¿a quién reportas?». No debe hablar de reportarle a una coordinadora.

- [ ] PASA  - [ ] FALLA

### 5. Memoria

1. Cada sesión viva muestra su memoria en la tarjeta («312 MB»). Las hibernadas, y las que acaban de terminar, no muestran nada.
2. La barra de abajo dice «Memory: … in N sessions», con N igual a las sesiones vivas, y «in 1 session» cuando hay una sola. El total es la suma de las tarjetas.
3. La cifra de una tarjeta es el **máximo del último minuto**: no baja enseguida después de un pico, y no debe saltar de un lado a otro cada pocos segundos.
4. Compara a ojo con Monitor de Actividad: la cifra de la tarjeta es la suma del `claude` de esa sesión y sus procesos hijos (memoria residente). No tiene que coincidir con la columna «Memoria» de Monitor de Actividad, que es otra medida, pero debe ser del mismo orden.

- [ ] PASA  - [ ] FALLA

### 6. Aviso de versión nueva de Claude Code

Solo se puede probar cuando salga una versión nueva de Claude Code. También se puede simular:

1. Con la app abierta desde hace más de 20 s (ya preguntó su versión al `claude` instalado) y una sesión viva y en reposo (`idle`), envejece la versión de esa sesión desde una terminal:

   ```sh
   sqlite3 "<tu home>/lithemux.db" "update sessions set claude_version='0.0.1' where slug='<sesión>'"
   ```

2. En unos segundos, la barra de abajo debe mostrar «A new Claude Code version is available: resume idle sessions» y el botón «Resume 1 idle session».
3. Pulsa el botón: la sesión se reinicia con la **misma conversación** (pregúntale algo de antes) y el aviso desaparece.
4. Una sesión que esté trabajando cuando pulses el botón **no** se toca. Si es la única que queda en la versión vieja, la barra dice «1 busy session is still on the previous Claude Code version», sin botón.
5. Una sesión con «Keep alive» **no** se toca, ni cuenta en el aviso.
6. En Ajustes, «Check the Claude version every (hours)» acepta de 1 a 168.

- [ ] PASA  - [ ] FALLA

### 7. El registro

1. `cat "<tu home>/logs/app.log"`: solo líneas con fecha UTC, nivel y mensaje. **Ningún** texto de tus conversaciones.
2. Si la app se cierra por un pánico, la última línea es `ERROR panic in thread … at <archivo>:<línea>: …`, con el mensaje cortado a 120 caracteres.
3. Abre la misma sesión desde una terminal con `lithemux open <cat>/<sesión>` y después en la app. La app dice que está abierta en otro lado, y el registro tiene una línea `WARN … is open elsewhere (pid N): not started here`.

- [ ] PASA  - [ ] FALLA

### 8. CPU en reposo

```sh
cd ~/Personal/lithemux
scripts/accept-phase2
```

La CPU en reposo debe quedar por debajo del 1 %. En mis corridas, al cerrar la ronda de correcciones, dio 0,37–0,40 % en la ventana estándar y 0,30–0,38 % desde los 40 s (ver el reporte).

- [ ] PASA  - [ ] FALLA

## Bloque 3B

**Antes de empezar:** compila con `cargo build --release`. Para el importador, usa un home de lithemux temporal, para no tocar el tuyo:

```sh
export LITHEMUX_HOME=$(mktemp -d)/lithemux
```

### 1. Importar desde cmux, por la CLI

1. `lithemux import cmux`: muestra la propuesta y dice «Nothing was written». Cada sesión tiene:
   - un número;
   - «yes» o «no»;
   - su nombre;
   - su categoría, o «(no category: use --into)»;
   - su carpeta;
   - y, si corresponde, «pinned» o «no saved conversation».
2. Comprueba que **nada cambió**: `lithemux status` sigue vacío, y el archivo de cmux conserva su fecha:

   ```sh
   stat -f %m ~/Library/Application\ Support/cmux/session-com.cmuxterm.app.json
   ```

   Ojo: cmux lo reescribe solo, así que compara con cmux cerrado.
3. Crea una categoría para las del home y aplica:

   ```sh
   lithemux cat add trabajo --name Trabajo --root ~
   lithemux import cmux --apply --into trabajo
   ```

   Debe decir cuántas sesiones y categorías importó.

   `--into` vale **solo** para las sesiones que quedaron sin categoría (las del home). Las demás conservan la que les propuso: una carpeta de un proyecto recibe su propia categoría nueva, aunque `trabajo` tenga su raíz en el home. Las categorías con raíz en el home, o más arriba, no cuentan como contenedoras.
4. `lithemux status`: las sesiones aparecen `hibernated`, con su nombre de cmux. Las que estaban fijadas en cmux salen fijadas en la app.
   - Un nombre con tildes da un slug sin ellas: «Migración» da `migracion` (antes daba `migraci-n`).
   - Ningún slug pasa de 48 caracteres, aunque el nombre sea larguísimo, y todas se pueden abrir.
5. `lithemux import cmux` otra vez: «0 Claude sessions to import (N already in lithemux…)». Reimportar no duplica nada.
6. Con **cmux cerrado**, `lithemux open <cat>/<sesión>` de una importada: Claude retoma la conversación de cmux. Pregúntale algo de antes.

- [ ] PASA  - [ ] FALLA

### 2. Importar desde cmux, en la app

Con otro `LITHEMUX_HOME` temporal y la app abierta sobre él:

1. «File → Import from cmux…» abre la ventana. Arriba dice cuántas sesiones quedan fuera y por qué.
2. Las sesiones sin conversación guardada salen desmarcadas. Las del home dicen «Choose a category».
3. Marca una del home sin elegirle categoría: el botón «Import N sessions» queda gris y abajo dice «Choose a category for each marked session».
4. Clic en «Choose a category»: aparecen las categorías existentes, las nuevas propuestas («New: …») y una nueva para su propia carpeta. Elige una y el botón se activa.
5. Import: abajo dice cuántas importó, la lista queda vacía y la barra lateral muestra las sesiones hibernadas.

- [ ] PASA  - [ ] FALLA

### 3. Una sesión que sigue viva en cmux

1. Con cmux abierto y una de sus sesiones de Claude corriendo, importa esa sesión (paso 1 o 2).
2. Espera hasta 30 s sin tocar nada: la tarjeta de esa sesión pasa sola a «open elsewhere (pid N)» («abierta en otro lado» en español). N es el `claude` de cmux; compruébalo con `ps -p N -o command`.
3. Clic en la tarjeta: **no** arranca un segundo `claude`, y sigue diciendo que está abierta en otro lado.
4. `lithemux open <cat>/<sesión>` en una terminal: «the session is already open (pid N)».
5. Cierra esa sesión en cmux. En unos segundos la tarjeta vuelve sola a «hibernated», aunque le hayas hecho clic en el paso 3. Ahora un clic la abre y retoma la conversación.
6. `grep -r <uuid de esa sesión> ~/algún-directorio` corriendo un rato en una terminal **no** la marca como abierta en otro lado: solo cuenta un programa llamado `claude`.

- [ ] PASA  - [ ] FALLA

### 4. Una sesión abierta en otra terminal

1. `lithemux open <cat>/<sesión>` en una terminal, con la app abierta.
2. Sin pulsar nada en la app, en menos de 30 s la tarjeta dice «open elsewhere (pid N)». Antes decía «en reposo».
3. Cierra esa terminal: en unos segundos la tarjeta deja de decirlo.

- [ ] PASA  - [ ] FALLA

### 5. Paleta y atajos (con tu teclado Latin American)

| Atajo | Qué hace |
|---|---|
| `cmd+p` | Abre la paleta: escribe parte del nombre, del slug, de la categoría o del rol; ↑/↓ eligen, Enter salta, Esc cierra |
| `cmd+alt+↓` | Sesión siguiente, en el orden de la barra lateral (salta las categorías plegadas) |
| `cmd+alt+↑` | Sesión anterior |
| `cmd+shift+m` | Menú de la sesión a la vista (se abre arriba en la barra lateral); ↑/↓ y Enter lo usan, Esc lo cierra |
| Esc, con un menú contextual abierto | Lo cierra y el foco vuelve a la terminal; la terminal **no** recibe el Esc |

Los mismos atajos están en el menú View.

1. Prueba cada atajo de la tabla.
2. En la paleta, «revision» encuentra «Revisión»: no importan mayúsculas ni tildes. Una búsqueda exacta sale antes que una que solo empieza igual, y esa antes que una aproximada.
3. Enter sobre una sesión hibernada la despierta y la muestra.
4. Clic derecho en una tarjeta y Esc: el menú se cierra, y Claude no recibe un Esc (no interrumpe lo que hacía).
5. **Con 100 sesiones** (o las que tengas), la paleta abre y filtra sin retraso visible al escribir. Las pruebas miden 100 sesiones en debug en alrededor de 1 ms por búsqueda; el límite es un cuadro (16 ms).
6. **Con más sesiones de las que caben** (las 22 importadas, por ejemplo):
   - `cmd+alt+↓` repetido: la barra lateral se desplaza para que la sesión elegida quede siempre a la vista;
   - en la paleta, ↓ más allá de unas 15 filas: la lista se desplaza con la selección.
7. Mientras importa, la ventana «Import from cmux» no se congela: el botón queda gris hasta que termina.

- [ ] PASA  - [ ] FALLA

### 6. CPU en reposo

```sh
cd ~/Personal/lithemux
scripts/accept-phase2
```

Debe quedar por debajo del 1 %. En mis corridas del 3B dio 0,45–0,47 %. Guardar las líneas de comandos le suma alrededor de una décima de punto (ver el reporte).

- [ ] PASA  - [ ] FALLA
