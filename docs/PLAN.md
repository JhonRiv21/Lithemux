# lithemux · Plan por fases

Gestor de sesiones de terminal y Claude Code, ligero, sin telemetría, con categorías jerárquicas (madre e hijas).
Primero macOS; Windows después. Open source (MIT), repo público. No se distribuye firmado.

> **Protocolo de trabajo (obligatorio)**
> - Se ejecuta una fase a la vez. **Al cerrar cada fase te detienes**: corres el checklist de cierre, escribes `docs/fases/F<N>-reporte.md`, dejas todo en stage (`git add`), reportas y esperas. No empiezas la siguiente fase sin orden expresa.
> - **Nunca `git commit` ni `git push`.** Jhon los hace.
> - Código, identificadores, comentarios y README en **inglés**. Los documentos de `docs/` van en español.
> - Si una decisión del plan resulta inviable, **no improvises otra**: documenta el problema con evidencia en `docs/decisiones.md` y repórtalo.

---

## 0. Por qué existe y principios

cmux (la herramienta actual) traba el Mac cada ~15 días. Medido el 21-nov-2026:

- La app ocupaba 910 MB, de ellos 449 MB en superficies de GPU.
- Las 23 sesiones de `claude` estaban siempre vivas (~3,9 GB).
- El swap estaba casi lleno.
- Trae PostHog y Sentry.
- Ejecuta un binario de 55 MB en cada hook.

Tampoco organiza las sesiones por app ni les da contexto entre sí.

Principios no negociables:

1. **Ligero.** La interfaz solo pinta lo visible. Los procesos `claude` que no se usan se hibernan. Los hooks son un binario mínimo que nunca bloquea.
2. **Cero telemetría.** La app no hace ninguna llamada de red. Ni analítica, ni reporte de errores, ni comprobación de actualizaciones. CI falla si entra una dependencia de ese tipo.
3. **Confiable.** El estado vive en SQLite (WAL). La persistencia de las conversaciones la da `claude --resume`, no mantener procesos vivos. Un reinicio del Mac, un cierre de la app y una hibernación se tratan igual.
4. **No envolver a Claude Code.** Se lanza el `claude` del usuario, resuelto desde su PATH en cada arranque, para recibir sus actualizaciones automáticas. Solo se usan interfaces documentadas: flags de la CLI, hooks y `claude agents --json`. **Nunca** se leen archivos internos de `~/.claude/` (`sessions/*.json`, `jobs/`, el formato de los `.jsonl`). La única excepción es comprobar si existe el transcript, cuya ruta sí está documentada (ver 1.3).
5. **Licencias.** No copiar código de cmux (GPL-3.0) ni del `terminal_view` de Zed (GPL). gpui y alacritty_terminal son Apache-2.0 y se pueden usar como dependencias.

## Hechos verificados de Claude Code (v2.1.330)

Son la base del diseño. Fuente: code.claude.com/docs (cli-reference, sessions, hooks, cross-session-messaging, agent-view).

- `-n/--name <nombre>` fija el nombre visible **y la dirección** de la sesión en `ListAgents`/`SendMessage`. Además, `claude --resume <nombre>` reanuda por nombre.
- `--session-id <uuid>` crea la conversación con ese ID. `--resume <uuid>` la reanuda.
- El transcript vive en `~/.claude/projects/<cwd codificado>/<uuid>.jsonl`. `--resume <uuid>` lo encuentra aunque cambie el cwd, siempre que haya un único transcript con ese ID.
- `--settings <json>` se fusiona con el settings.json del usuario. **No se restaura al reanudar**: hay que volver a pasarlo en cada lanzamiento, y lo mismo con `--add-dir`.
- El hook `SessionStart` se dispara con `source` = `startup`, `resume`, `clear`, `compact` o `fork`. Su salida `hookSpecificOutput.additionalContext` llega a Claude como recordatorio del sistema (si pasa de 10.000 caracteres, se trunca a un archivo). Debe redactarse **en forma factual**: el texto que parece una orden del sistema puede activar las defensas contra inyección.
- Si dos procesos reanudan la misma sesión a la vez, se pierden mensajes. Hace falta un candado por sesión.
- Las actualizaciones se aplican en el siguiente arranque del binario. Un proceso abierto sigue con su versión.
- No usar `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`: apaga también la actualización automática.

---

## Fase 0 — Núcleo: categorías, contexto y lanzador (sin interfaz)

**Objetivo:** que la jerarquía madre/hijas y la reanudación funcionen desde ya en cualquier terminal, incluida cmux. La app de la Fase 2 reutilizará este núcleo tal cual.

### 0.1 Andamiaje

- `git init -b main` en `~/Personal/lithemux`.
- Workspace de Cargo con tres crates:
  - `crates/core` (lib): modelo, almacenamiento, contexto y lanzador;
  - `crates/cli` (bin `lithemux`);
  - `crates/hook` (bin `lithemux-hook`).
- Rust estable con edición 2024 y `rust-toolchain.toml` fijado.
- `LICENSE` MIT, `README.md` en inglés con una sección en español al final, y `.gitignore`.
- CI en GitHub Actions sobre `macos-latest`: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` y `cargo build --release`.
- `cargo-deny` con una lista de crates prohibidos: `sentry*`, `posthog*`, `opentelemetry*`, `segment*`, `rudderstack*`, `mixpanel*`, `amplitude*`, `bugsnag*`, `datadog*`. Además, los clientes HTTP (`reqwest`, `hyper`, `ureq`, `isahc`, `surf`) quedan prohibidos en `core`, `cli` y `hook`. Si hiciera falta red algún día, se decide aparte.
- `docs/fases/` (un reporte por fase) y `docs/decisiones.md`.
- Andamiaje de pruebas según la sección «Pruebas» de `CLAUDE.md`: carpetas `test/` por capa, `tests/support`, `tests/fakes/fake-claude`, `tests/arch-fixtures`, test de fronteras, guardas de fuente única, lint de hallazgos vacío y `.githooks/pre-commit`.

### 0.2 Modelo y almacenamiento

Base de datos en `~/Library/Application Support/lithemux/lithemux.db`, usando el crate `directories`. La variable `LITHEMUX_HOME` la sobrescribe, para tests. `rusqlite` con `bundled`, WAL y `busy_timeout` de 250 ms. Migraciones por `PRAGMA user_version`.

```
categories(
  id TEXT PK,                 -- uuid v4
  slug TEXT UNIQUE NOT NULL,  -- [a-z0-9-], ej. "selektal"
  name TEXT NOT NULL,         -- "Selektal"
  root_dir TEXT NOT NULL,     -- cwd por defecto de sus sesiones
  color TEXT, sort INTEGER, created_at INTEGER)

sessions(
  id TEXT PK,                 -- uuid v4 interno de lithemux
  category_id TEXT NOT NULL REFERENCES categories ON DELETE CASCADE,
  slug TEXT NOT NULL,         -- "planificadora"
  role TEXT NOT NULL,         -- descripción libre del rol: "Planifica, revisa y redacta prompts"
  kind TEXT NOT NULL CHECK(kind IN ('mother','child')),
  profile TEXT NOT NULL CHECK(profile IN ('claude','shell')),
  cwd TEXT,                   -- NULL = root_dir de la categoría
  claude_session_id TEXT,     -- uuid que se pasa a --session-id/--resume; solo profile=claude
  model TEXT,                 -- NULL = el default del usuario
  keep_alive INTEGER NOT NULL DEFAULT 0,
  state TEXT NOT NULL DEFAULT 'new', -- new|working|idle|waiting|hibernated|ended
  claude_version TEXT,        -- versión con la que arrancó por última vez
  last_activity_at INTEGER, created_at INTEGER, sort INTEGER,
  UNIQUE(category_id, slug))

CREATE UNIQUE INDEX one_mother_per_category ON sessions(category_id) WHERE kind='mother';

settings(key TEXT PK, value TEXT)   -- language, hibernate_after_hours, etc.
```

- **Nombre de mensajería** = `<category.slug>-<session.slug>`, por ejemplo `selektal-planificadora`. Se calcula, no se guarda, y debe ser único en todo el sistema. Validarlo al crear.
- Una categoría puede no tener madre. En ese caso todas sus sesiones son hermanas.

### 0.3 Lanzador (`core::launch`)

Construye el comando (`plan_launch`), **no lo ejecuta**. Quien lo ejecuta es `open_session`, a través del puerto `SessionRunner`: la CLI lo corre como hijo en la terminal actual y la app lo usará en un PTY.

- **Resolver `claude`** en cada lanzamiento, sin cachearlo nunca. Orden (decisión D-05):
  1. el ajuste `claude_path`, si está definido y es ejecutable;
  2. `$SHELL -lic 'command -v claude'`, con stdin en `/dev/null` y timeout de 3 s; se toma la última línea que sea una ruta absoluta y ejecutable;
  3. si falla o se pasa del tiempo, `$SHELL -lc` como respaldo, con advertencia en `hook.log` y en la salida de `open`.

  En los pasos 2 y 3 se descarta cualquier candidato bajo `$TMPDIR`, `/tmp`, `/private/tmp`, `/var/folders`, `/private/var/folders` o cuya ruta contenga `shims`: son envoltorios que otra herramienta inyectó en el PATH.
- **¿Nueva o reanudar?** Si existe algún archivo `~/.claude/projects/*/<claude_session_id>.jsonl`, se usa `--resume <id>`. Si no, `--session-id <id>`.
- **Argumentos siempre presentes:** `-n <nombre-mensajería>`, `--settings <json-hooks>`, y `--model <m>` si está definido.
- **JSON de hooks** con rutas absolutas a `lithemux-hook`, ubicado junto al binario actual (`std::env::current_exe()`). Solo cinco eventos; nada de `PreToolUse`, que es demasiado frecuente:
  ```json
  {"hooks":{
    "SessionStart":[{"hooks":[{"type":"command","command":"<abs>/lithemux-hook session-start","timeout":5}]}],
    "UserPromptSubmit":[{"hooks":[{"type":"command","command":"<abs>/lithemux-hook prompt-submit","timeout":2}]}],
    "Stop":[{"hooks":[{"type":"command","command":"<abs>/lithemux-hook stop","timeout":2}]}],
    "Notification":[{"hooks":[{"type":"command","command":"<abs>/lithemux-hook notification","timeout":2}]}],
    "SessionEnd":[{"hooks":[{"type":"command","command":"<abs>/lithemux-hook session-end","timeout":2}]}]}}
  ```
- **Entorno del hijo:** `LITHEMUX_SESSION_ID=<id interno>` y `LITHEMUX_HOME` si está definido.
- **Candado:** `flock` sobre `<home>/locks/<session_id>.lock`. Si está tomado, el comando falla con «la sesión ya está abierta (pid N)». Así se evita reanudar dos veces la misma conversación.
- **Spawn + wait, no `exec`** (decisión D-03):
  - `open` toma el candado con un descriptor `CLOEXEC`, lanza `claude` como hijo que hereda el TTY, espera y sale con el código de salida del hijo (128 + señal si lo mató una señal);
  - mientras espera ignora SIGINT y SIGQUIT (los recibe el hijo, que comparte el grupo de foreground) y reenvía SIGHUP y SIGTERM al hijo;
  - escribe el pid del hijo dentro del archivo de candado, para `doctor`, `status` y el mensaje de error;
  - el candado nunca se hereda: un descriptor heredado por `claude` llegaría a los procesos que este deje vivos y la sesión quedaría inabrible.

  En la Fase 2 el candado lo retendrá la propia app, que es dueña del PTY.
- **Versión de Claude** (decisión D-02): antes de lanzar, `open` ejecuta `<claude> --version` con timeout de 2 s y guarda el resultado en `claude_version`. Si falla o se pasa del tiempo, conserva el valor anterior y no bloquea la apertura.
- **Perfil `shell`:** `$SHELL -l` en el cwd, sin hooks ni contexto.

### 0.4 Binario de hook (`lithemux-hook`)

Reglas duras:
- **Siempre `exit 0`** y nunca escribir a stderr: un hook roto no debe estorbar a Claude.
- Termina en menos de 50 ms típicos.
- Los errores van a `<home>/logs/hook.log`, rotado a 1 MB.
- Sin `LITHEMUX_SESSION_ID`, no hace nada y sale con 0. Así es inofensivo fuera de lithemux.

| Evento | Acción |
|---|---|
| `session-start` | Actualiza `state=idle`, `last_activity_at` y `claude_version`. Si el JSON de stdin trae la versión, la usa; si no, la deja tal cual. Imprime el JSON con `additionalContext`, según 0.5. |
| `prompt-submit` | `state=working` y `last_activity_at`. |
| `stop` | `state=idle` y `last_activity_at`. |
| `notification` | Si es una solicitud de permiso o espera de input (mirar `notification_type`/matcher en el JSON de stdin, según la doc de hooks), `state=waiting`. |
| `session-end` | `state=ended`. Si la app la hibernó a propósito, la app ya habrá escrito `hibernated` y el hook no lo pisa. |

Además, después de escribir en la BD, intenta una conexión no bloqueante al socket `<home>/app.sock` y le manda una línea JSON `{"session_id":..,"event":..}`. Si no hay app escuchando, ignora el error. Esto se usará en la Fase 2.

### 0.5 Contexto inyectado (`core::context`)

Texto **factual**, corto (menos de 2.000 caracteres), en el idioma del ajuste `context_language` (por defecto el de la interfaz, y por defecto inglés). Se renderiza desde la BD en cada `SessionStart`, así que siempre refleja el estado actual. Las plantillas viven en `core` y se prueban con snapshots.

Contenido según el caso:

- **Madre:**
  - categoría y carpeta raíz;
  - su nombre de mensajería y su rol;
  - lista de hijas: nombre de mensajería, rol, estado y última actividad en tiempo relativo;
  - que ella coordina la categoría y es la única que escribe el tablero;
  - ruta del tablero;
  - que la lista es una foto del arranque y `lithemux status <categoria>` da el estado vivo.
- **Hija:**
  - categoría;
  - su nombre y su rol;
  - su madre (nombre de mensajería y rol);
  - sus hermanas;
  - que reporta a la madre al terminar o al bloquearse, y que lo que esté fuera de su rol pasa por la madre;
  - ruta del tablero, de solo lectura salvo que la madre indique otra cosa;
  - `lithemux status`.
- **Sin madre:** las sesiones se presentan como hermanas que coordinan directamente entre sí.

El **tablero** es `<home>/categories/<slug>/board.md`. Se crea vacío, con un encabezado, al crear la categoría. No vive en el repo de la app de usuario.

Ejemplo en español (como referencia de tono, no literal):

```
lithemux: esta sesión pertenece a la categoría «Selektal» (carpeta /Users/x/Linktic/RecruitLine).
Nombre de esta sesión: selektal-ejecutora. Rol: implementa las fases que define la planificadora.
Madre de la categoría: selektal-planificadora (rol: planifica, revisa y redacta prompts).
Hermanas: selektal-documental (rol: requerimientos; estado: hibernada, última actividad hace 3 días).
Las sesiones se comunican con SendMessage usando esos nombres. Al terminar una tarea o al bloquearse, esta sesión reporta a la madre.
Tablero de la categoría (lo mantiene la madre): /Users/x/Library/Application Support/lithemux/categories/selektal/board.md
Esta lista es del momento del arranque; `lithemux status selektal` muestra el estado actual.
```

### 0.6 CLI (`lithemux`)

Con `clap`. Salida legible por defecto y `--json` donde aplique.

```
lithemux cat add <slug> --name <Nombre> --root <dir> [--color]
lithemux cat list | cat rm <slug> [--yes]
lithemux new <cat>/<slug> --role "<texto>" [--mother] [--profile claude|shell] [--cwd <dir>] [--model <m>]
lithemux open <cat>/<slug>            # corre la sesión en la terminal actual y sale con su código de salida
lithemux status [<cat>] [--json]      # árbol madre → hijas con estado y última actividad
lithemux context <cat>/<slug>         # imprime el contexto que recibiría (depuración)
lithemux set <cat>/<slug> --role|--mother|--keep-alive|--model ...
lithemux rm <cat>/<slug> [--yes]      # no borra el transcript de Claude
lithemux config get|set|unset <clave> # claves: claude_path (ruta absoluta), context_language (es|en)
lithemux doctor                       # claude elegido y todos los instalados, BD, permisos, sesiones abiertas, prueba en seco del hook
```

- `set` acepta `--mother [true|false]`, `--keep-alive [true|false]`, `--model <m>` y `--default-model` (vuelve al modelo por defecto del usuario).
- `cat rm` conserva el `board.md` e imprime su ruta.
- `config` rechaza cualquier otra clave con la lista de las válidas.
- `doctor` lista **todos** los `claude` del PATH (`which -a claude` dentro de `-lic`) con su versión, marca el que se usaría y advierte si hay más de una instalación con versiones distintas.
- `status` muestra además si la sesión está abierta y el pid que la retiene.

`lithemux open` es lo que permite usarlo **hoy dentro de cmux**: una pestaña por sesión, `lithemux open selektal/ejecutora`.

### 0.7 Pruebas

Forma y capas de las pruebas: sección «Pruebas» de `CLAUDE.md`. Regla del equipo: **los tests se escriben desde esta especificación, no desde la implementación.** Escríbelos antes o en paralelo, mirando este documento.

Prueba solo lógica pura que puede fallar:
- unicidad de la madre;
- validación de slugs y de nombres de mensajería;
- elección `--session-id` vs `--resume` según exista o no el transcript (con un `HOME` temporal);
- construcción exacta de los argumentos;
- render del contexto para madre, hija y sin madre (snapshots con `insta`);
- el hook sin `LITHEMUX_SESSION_ID` sale con 0 sin salida;
- el hook con la BD bloqueada sale con 0.

### Cierre de la Fase 0 · criterios de aceptación

1. CI en verde en local: fmt, clippy `-D warnings`, test, build `--release`, `cargo deny check`.
2. **Prueba manual en cmux,** documentada en el reporte con capturas de texto:
   - crear la categoría `demo` con `planificadora` (madre), `ejecutora` y `documental`, y abrir las tres con `lithemux open`;
   - en la ejecutora, `/list-agents` muestra `demo-planificadora` y `demo-documental` con esos nombres;
   - preguntar a la ejecutora «¿quién es tu madre y quiénes tus hermanas?»: responde correctamente sin que se lo hayas dicho;
   - cerrar las tres (`/exit`) y volver a abrirlas con `lithemux open`: reanudan la misma conversación (recuerdan algo dicho antes) y conservan el nombre;
   - abrir dos veces la misma sesión: la segunda falla por el candado.
3. Medir y anotar el tiempo del hook (`hyperfine` sobre `lithemux-hook stop` con BD real). Objetivo: p95 por debajo de 20 ms.
4. **Detente y reporta.**

---

## Fase 1 — Spike de interfaz nativa (máximo 3 días)

**Objetivo:** decidir con evidencia si la interfaz nativa en Rust con **gpui** + **alacritty_terminal** es viable. Es un go/no-go: el código del spike puede tirarse, salvo el `TerminalView` si sale bien.

### 1.1 Base

- Crate `crates/app`.
- gpui como dependencia git **fijada a un `rev` concreto** del repo de Zed. El crate de crates.io está desactualizado. Anota en `docs/decisiones.md` el `rev` y el motivo.
- `alacritty_terminal` desde crates.io (última estable) y `portable-pty` para el proceso.
- Escribir un `TerminalView` propio sobre la grilla de `alacritty_terminal`: celdas, colores, cursor, selección y scroll. Se puede leer el código de Zed o de `gpui-terminal` para orientarse, **sin copiar** el de Zed (GPL). Revisa la licencia de cualquier otro crate antes de reutilizarlo.

### 1.2 Pruebas del spike

Cada una se marca PASA / FALLA, con nota:

1. Abrir zsh y `claude` dentro: el TUI de Claude se ve completo, con colores truecolor, sin parpadeo ni basura al redibujar.
2. **Teclado en español:** teclas muertas (`´` + `a` = `á`), `ñ`, `¿`, `¡`, y composición de IME (mantener presionada una vocal y elegir el acento).
3. `shift+enter` inserta una línea nueva en el prompt de Claude.
4. Copiar y pegar con cmd+c/v; pegado entre corchetes (bracketed paste) con texto multilínea; selección con el ratón.
5. Redimensionar la ventana: Claude se reacomoda sin romperse.
6. **Dos paneles lado a lado,** cada uno con su PTY, y cambio de foco con el teclado.
7. Rendimiento:
   - `cat` de un archivo de 50 MB no congela la interfaz;
   - CPU en reposo con 2 terminales por debajo de 1 % (sin bucle de repintado);
   - footprint de la app con 2 terminales menor de 150 MB (`footprint <pid>`).

### 1.3 Decisión

Informe en `docs/fases/F1-reporte.md` con la tabla de resultados.

- **GO:** pasan 1–6 y 7 queda en metas o cerca.
- **NO-GO:** falla 1, 2 o 6 y no hay arreglo razonable en el tiempo. El plan B es Tauri 2 + TypeScript sin framework + xterm.js, con el mismo `core`.

**Detente y reporta.** La decisión la toma Jhon con la planificadora.

---

## Fase 2 — App MVP

Se ejecuta con la tecnología que salga de la Fase 1. Las subfases se describen para gpui.

### 2.1 Gestor de sesiones (sin interfaz)

- Un `SessionHost` en la app es dueño de los PTYs. Cada sesión viva tiene su `Term` de alacritty, que **sigue parseando la salida aunque no se vea**.
- Scrollback por defecto: 10.000 líneas, configurable.
- Solo los paneles visibles se pintan. Las terminales ocultas no consumen GPU.
- El lanzamiento usa `core::launch`, con candado incluido.
- Servidor de socket en `<home>/app.sock`: recibe los eventos del hook y actualiza el estado en vivo. La fuente de verdad sigue siendo la BD.

### 2.2 Barra lateral

- Categorías colapsables. La **madre es la cabecera** de su categoría y las hijas cuelgan de ella. Una categoría sin madre muestra a sus sesiones al mismo nivel.
- Indicador de estado por sesión: trabajando, idle, esperando permiso (resaltado), hibernada y terminada. También indicador de no leído: hubo actividad mientras no se veía.
- Crear, renombrar, reordenar y borrar categorías y sesiones. Marcar una sesión como madre (respetando una por categoría). Activar `keep_alive`.

### 2.3 Área de trabajo

- Un panel o **split en 2**, horizontal o vertical. Abrir una sesión en el panel activo o en «el otro panel». Atajos de teclado para dividir, cambiar de foco y cerrar el panel.
- La disposición y el contenido de los paneles se guardan en la BD.

### 2.4 Ciclo de vida y hibernación

- **Hibernar:** la sesión no está visible, `keep_alive=0`, `state=idle` (nunca `working` ni `waiting`) y lleva **más de `hibernate_after_hours`** sin actividad. Ese valor es **48 h por defecto**, configurable.
  - Acción: escribir `state=hibernated` en la BD, mandar SIGHUP al grupo de procesos y SIGKILL a los 5 s si sigue vivo.
- **Válvula de memoria (activada por defecto, desactivable):** se escucha la presión de memoria de macOS (`DISPATCH_SOURCE_TYPE_MEMORYPRESSURE`). En estado **crítico**, se hiberna la sesión idle no visible más antigua (que no sea `keep_alive`). Como máximo una cada 60 s.
- **Despertar:** al enfocar una sesión hibernada se relanza con `core::launch` (que resuelve `--resume`). El usuario solo ve un indicador de «reanudando».
- Si el proceso termina solo, la sesión queda en `ended` con un botón para reanudar.
- **Nada en la app debe hacer perder una conversación:** nunca se borra un transcript y nunca se relanza con `--session-id` si el transcript existe.

### 2.5 Arranque y restauración

Al abrir la app:
- se restauran ventana, barra lateral y paneles;
- todas las sesiones de `claude` aparecen hibernadas;
- **solo las de los paneles visibles se reanudan solas;**
- las de perfil shell se relanzan en su cwd.

### 2.6 Ajustes e idioma

- Ventana de Ajustes y menú nativo de macOS (Archivo, Edición, Visualización, Ventana, Ayuda).
- **Idioma de la interfaz: inglés por defecto, español opcional.** Los textos van en archivos de recursos (`locales/en.toml` y `locales/es.toml`) y nunca en el código. Si falta una clave en español, se usa la inglesa. El cambio aplica sin reiniciar, o como mucho con un reinicio avisado.
- Otros ajustes:
  - `context_language` (por defecto, el de la interfaz);
  - horas para hibernar y la válvula de memoria;
  - scrollback;
  - fuente y tamaño;
  - `claude_path` (opcional);
  - tema claro, oscuro o del sistema.

### Cierre de la Fase 2 · criterios de aceptación

1. Checklist de CI en verde.
2. Prueba con **20 sesiones** en 4 categorías, 2 visibles y el resto hibernadas:
   - footprint de la app por debajo de 200 MB;
   - solo los procesos `claude` visibles están vivos;
   - CPU en reposo por debajo de 1 %.
3. Cerrar la app con `cmd+q` y abrirla: la disposición se restaura, las visibles se reanudan y las demás quedan hibernadas sin perder nada.
4. **Reiniciar el Mac** y repetir el punto 3.
5. Madre e hijas funcionan en la app igual que en la Fase 0.
6. La interfaz completa funciona en español y en inglés.
7. `nettop`/Little Snitch con la app abierta 10 min: **cero conexiones salientes** de la app. Las de los procesos `claude` son de Claude, no nuestras.
8. **Detente y reporta.**

---

## Fase 3 — Robustez y pulido

- **3.1 Notificaciones nativas de macOS** cuando una sesión pasa a `waiting` (permiso) o termina un turno, **solo si no está a la vista**. Se configuran por categoría.
- **3.2 Versión de Claude:** comparar `claude --version` con el `claude_version` de cada sesión viva. Si hay una versión nueva, mostrar un aviso discreto: «Hay una versión nueva de Claude Code: reanudar las sesiones idle», en lote y solo con las idle.
- **3.3 Medidor de memoria** por sesión y total, a partir del árbol de procesos (`libproc`), en la barra de estado.
- **3.4 Importador desde cmux:** lee `~/Library/Application Support/cmux/session-com.cmuxterm.app.json`, que trae workspaces con `customTitle`, `currentDirectory` y `terminal.agent.sessionId`. Propone categorías y sesiones que conservan su `claude_session_id`, para no perder las conversaciones actuales. Solo lectura sobre cmux.
- **3.5 Búsqueda y salto rápido** de sesiones (paleta con cmd+p) y navegación completa con teclado.
- **3.6** Panic hook que escribe en `<home>/logs/app.log` (rotado), sin enviar nada. Smoke test automatizado del ciclo: crear sesión → abrir → hibernar → reanudar.
- **Cierre:** checklist de CI, reporte de la fase, **detente y reporta.**

## Fase 4 — Extensiones y Windows (se planifica en detalle al llegar)

- Perfiles genéricos definidos por configuración (comando, args, si soporta resume, adaptador de hooks): Codex, modelos locales (Ollama) y comandos propios.
- Layouts de hasta 4 paneles.
- Port a Windows: ConPTY (portable-pty), rutas, socket → named pipe, señales → `TerminateProcess`, y presión de memoria con la API de Windows.

---

## Checklist de cierre de cada fase

1. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build --release` y `cargo deny check` en verde.
2. `docs/fases/F<N>-reporte.md` con:
   - qué se hizo, por subfase;
   - qué quedó fuera y por qué;
   - mediciones con números;
   - desviaciones del plan, con su justificación;
   - riesgos detectados.
3. `git add -A` (revisando que no entren binarios, `target/` ni secretos). **Sin commit.**
4. Reporte breve a la sesión planificadora y a Jhon: resultado de los criterios de aceptación, punto por punto.
5. Esperar.
