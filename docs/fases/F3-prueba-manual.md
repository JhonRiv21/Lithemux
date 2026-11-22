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
