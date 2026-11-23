# Fase 3 · Reporte

## Bloque 3A · Notificaciones, versión de Claude, memoria y registro (§3.1, §3.2, §3.3, §3.6)

*2026-11-22. Ejecutora. Sin commit: todo queda en stage.*

> **§3.1 cambió en la ronda de correcciones (D-58):** ya no hay notificaciones en pantalla, solo un sonido al terminar y el reporte a la coordinadora. Lo que esta sección dice de las notificaciones queda como historia. Ver «Ronda de correcciones del bloque 3A», más abajo.

### Qué se hizo

| § | Qué | Dónde |
|---|---|---|
| 3.1 | Notificaciones nativas con `osascript` cuando una sesión espera respuesta o termina un turno y no está a la vista. Se apagan por categoría desde su menú y con `lithemux cat set <slug> --notify` | `notifications/`, `platform/process/desktop_notification.rs`, migración 5 |
| 3.2 | Aviso en la barra de estado cuando el `claude` instalado es más nuevo que el de alguna sesión viva, con un botón que reanuda solo las sesiones `idle`. Pregunta al arrancar y cada `version_check_hours` (ajuste nuevo, 6 h por defecto) | `launch/domain/claude_update.rs`, `launch/application/installed_claude.rs`, `app_shell/claude_update.rs`, `status_bar/` |
| 3.3 | Memoria de cada sesión (en su tarjeta) y total (en la barra de estado), medida sobre el árbol de procesos de cada sesión | `platform/process/memory_use.rs`, `app_shell/refresh.rs` |
| 3.6 | `<home>/logs/app.log` con rotación 1 MB × 3, panic hook y registro de los errores del host. Prueba de humo del ciclo completo | `platform/log/`, `session_host/startup.rs`, `tests/integration/smoke_test.rs` |

Las decisiones están en `docs/decisiones.md`, de D-53 a D-57.

### Lo que no salió como decía el encargo

- **Notificaciones, vía (a):** un `.app` sin firmar no consigue permiso de `UNUserNotificationCenter` en este Mac, y además pediría `unsafe` fuera de `detached.rs`. Se usó la vía (b), `osascript` (D-53).
- **El clic en la notificación no enfoca la sesión.** `osascript` no lo permite; según cómo se comporta, el clic abre Script Editor, aunque no lo verifiqué con un clic real. Queda para cuando la app tenga firma (D-53).
- **La notificación lleva el ícono y el nombre de Script Editor.**
- **El subcomando de la CLI es `lithemux cat`, no `lithemux category`** (fase 0). `--notify` se agregó a `cat set`.
- **La memoria es RSS, no *footprint*** (D-54): es lo que `sysinfo` da sin `unsafe`. `darwin-libproc` choca con `vte` por `memchr`.
- **«Reanudar con `--resume`»:** la reapertura sigue la regla de la fase 1. Usa `--resume` si existe el transcript, y si no, `--session-id` con el mismo id. En la prueba manual con el `claude` falso, que no escribe transcript, se vio `--session-id`. La prueba de humo escribe el transcript y comprueba `--resume`.
- **Un pánico en el hilo principal se registra siempre, pero no se puede recuperar** (D-56). Hay un test para el caso en que aborta, dentro de una función `extern "C"`.

### Bug encontrado: el INSERT de categorías ignoraba `collapsed` (regla H-008)

Venía de la fase 2:

- La migración 4 sumó `collapsed` a la lectura (`COLUMNS`), pero el INSERT tenía su propia lista de columnas y no la incluía. Una categoría creada como plegada se guardaba desplegada.
- No se notaba porque la app siempre crea categorías desplegadas.
- Lo destapó el contrato del repositorio, al insertar una categoría con `notify = false` y recibirla con `true`.

**Arreglo:**

- Los INSERT de `categories` y `sessions` usan ahora la misma constante `COLUMNS` que la lectura.
- La regla `H-008` marca un INSERT de esas tablas con columnas escritas a mano. Corrida contra el commit anterior, marca los dos INSERT que había.
- El contrato prueba ahora el viaje de ida y vuelta de una categoría con todos sus campos fuera del valor por defecto.

### Tests

668 en verde en todo el workspace.

**Los que vi fallar antes de arreglar,** y por qué:

| Test | Qué vi | Causa |
|---|---|---|
| `claude_update_test` (5 tests: comparación de versiones y qué sesiones se reanudan) | rojos | no había implementación |
| `policy_test` (2: no avisar a la vista, no avisar con la categoría apagada) | rojos | la primera versión de la política no miraba esos dos datos |
| contrato de categorías (SQLite y memoria) | rojo | dos causas: mi aserción final comparaba contra un valor viejo (bug del test) y el INSERT de SQLite no guardaba `collapsed` ni `notify` (bug real, el de arriba) |
| `memory_use_test::each_tree_counts_only_its_own_processes` | rojo | bug del test: el proceso de ayuda con el rol «no retiene nada» caía en la rama de «arranca uno que retiene» |
| `a_database_made_by_the_app_is_logged` | rojo | bug del test: `HostedWorld` crea la base antes de arrancar el host, así que el registro decía «migrated» y no «created». Se separó en un test que borra la base primero |
| `smoke_test` | rojo dos veces | bugs del test: suponía que el registro del `claude` falso acumulaba todas las corridas y que el id iba en la misma línea que la opción |
| `single_source_guards::no_test_runs_a_real_shell_or_the_real_claude` | rojo | la guarda D-13 no acepta leer una variable de entorno a través de una constante. Se usó el literal en `panic_hook_test` |
| `menu_test::a_category_turns_its_notifications_off_and_on_again` | solo no compilaba | no llegué a verlo rojo en ejecución |

**Los que no vi fallar,** escritos desde el plan sin una implementación rota a la vista:

- `app_log_test` (rotación, aplanado de líneas) y `timestamp_test`;
- `panic_hook_test` (proceso hijo, incluido el aborto en `extern "C"`);
- `desktop_notification_test` (argumentos y tope de longitud);
- `installed_claude_use_cases`;
- `memory_use_test` unitario (`descendants`);
- los de lifecycle: `restarting_a_session_resumes_its_conversation_on_the_installed_version`, `the_installed_claude_version_is_the_one_a_launch_would_use` y `a_session_that_cannot_start_and_one_open_elsewhere_are_logged`;
- `the_news_of_a_hook_carries_its_event`;
- `memory_label_test`;
- la prueba de la CLI `a_category_turns_its_notifications_off_and_on`.

### Comprobación en la app real

Con la app compilada en release, `LITHEMUX_HOME` de prueba (base v4) y el `claude` falso. Los clics se hicieron de verdad sobre la ventana.

1. **Migración:** la base pasó de v4 a v5 al abrir, y `app.log` registró `INFO database migrated from schema v4 to v5`.
2. **Memoria:** cada tarjeta viva muestra su memoria («3 MB») y la barra de estado muestra el total («Memory: 8 MB in 3 sessions»).
3. **Notificación fuera de la vista:** un `Stop` de una sesión en una pestaña no visible mostró «Claude finished / planificadora · Demo app / Listo: el plan del bloque 3A está revisado.». El markdown de la vista previa salió limpio.
4. **Sesión visible y ventana activa:** no salió nada, ni con `Notification` ni con `Stop`.
5. **La misma sesión con la ventana en segundo plano:** salió la notificación.
   - El primer intento no salió porque la sesión estaba `hibernated` en la base: el `claude` falso no manda `SessionStart` y la regla no avisa del `Stop` de una sesión hibernada. Con el `SessionStart` que manda Claude sí salió.
6. **Categoría silenciada:** clic derecho en la categoría → «Turn off notifications». La base pasó a `notify = 0`, el menú pasó a ofrecer «Turn on notifications» y un `Stop` en segundo plano no mostró nada. `lithemux cat set demo --notify` la volvió a encender.
7. **Versión:** con dos sesiones vivas envejecidas a 9.9.8 (el falso dice 9.9.9), la barra mostró el aviso.
   - Con una de ellas en `idle` apareció «Resume 1 idle».
   - El clic reinició solo esa sesión (pid nuevo, versión 9.9.9). La otra, que no estaba en `idle`, siguió con su programa y su 9.9.8.
8. **Al salir:** ningún programa quedó vivo.

### Mediciones (`scripts/accept-phase2`, 20 sesiones, 2 visibles)

El script se corrió sobre el commit anterior (0dcbe3f) y sobre este bloque, en la misma máquina y el mismo día.

| | CPU en reposo, ventana estándar (12–72 s) | CPU en reposo, ventana desde 40 s | Footprint | Pico |
|---|---|---|---|---|
| Antes (0dcbe3f) | 0,25 % | 0,27 %, 0,27 % | 105 MB | 146 MB |
| Bloque 3A, sin redondear la memoria | 0,37 %, 0,43 %, 0,38 % | 0,33 % | 104–107 MB | 147 MB |
| Bloque 3A, final | 0,42 %, 0,40 %, 0,40 % | 0,30 % | 103–106 MB | 147 MB |

**Cómo se leen:**

- **Todo queda por debajo del 1 %.**
- La ventana estándar sube de 0,25 a ~0,40 %. Esa ventana empieza a los 12 s y contiene la pregunta de versión de los 15 s.
- La diferencia entre las dos ventanas apunta a esa pregunta, que es lo único nuevo entre los 12 y los 40 s. No lo aislé más. Pasa una vez por arranque y después cada 6 h.
- En la ventana desde los 40 s, que es el estado estable, el bloque suma unas 0,03 décimas de punto: la medición de memoria cada 3 s.
- Redondear la memoria al megabyte (D-54) bajó esa ventana de 0,33 a 0,30 %, porque la ventana ya no se repinta cuando lo que muestra no cambió.

### Checklist de cierre

Corrida después del último cambio:

| Paso | Resultado |
|---|---|
| `cargo fmt --all --check` | limpio |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 errores, 0 avisos |
| `cargo test --workspace` | 668 pasan, 0 fallan |
| `cargo build --release` | compila |
| `cargo deny check` | advisories, bans, licenses y sources ok |
| `scripts/lint-findings` | 8 reglas, sin hallazgos |
| Tamaños (≲300 líneas por archivo, ≲50 por función) | ningún archivo ni función por encima |
| `.githooks/pre-commit` | pasa sobre el stage (108 archivos) |
| `scripts/accept-phase2` | 2 programas vivos (los visibles), CPU 0,40 %, footprint 103 MB (pico 147), 0 programas vivos al salir, 20 hibernadas |

**Archivos que se partieron por tamaño:**

- `session_host/host.rs` dejó el arranque en `startup.rs` y lo que pasa al lanzar y al terminar el programa en `running.rs`.
- `app_shell/live.rs` dejó el ciclo de refresco en `refresh.rs`.
- `sidebar/tree.rs` separó `card_of`.
- `settings_window/form.rs` separó `texts_of`.

### Reglas del encargo

- **Sin `unsafe` nuevo:** la vía (a) de notificaciones se descartó en parte por eso.
- **Sin red:** `osascript` y `sysinfo` no abren sockets, y `cargo deny` sigue limpio.
- **Ningún archivo de `~/.claude/` leído:** la versión sale de `claude --version` y la vista previa, de campos documentados de los hooks.
- **Nada de git** salvo el `git add -A` final.

### Qué quedó fuera

- **El clic que enfoca la sesión** desde la notificación (necesita firma).
- **Nada de 3B ni 3C.**

### Riesgos detectados

- **El ícono de Script Editor puede confundir:** «¿por qué me avisa Script Editor?». La primera vez, macOS puede pedir permiso de notificaciones a Script Editor, no a lithemux.
- **Si Jhon tiene silenciado Script Editor en Ajustes del Sistema, no verá ninguna notificación de lithemux.**

### Lo que necesita a Jhon

- La prueba manual de `docs/fases/F3-prueba-manual.md`, sobre todo:
  - si las notificaciones le llegan con su configuración de Concentración;
  - qué hace el clic sobre una de ellas.
- Si molestan, quitar a mano las entradas `dev.lithemux.probe`, `probe2` y `probe3` de Ajustes del Sistema → Notificaciones, que quedaron de las pruebas de la vía (a).

## Ronda de correcciones del bloque 3A

*2026-11-22. Reúne la auditoría independiente, la prueba con clics de la planificadora y el cambio de alcance de Jhon en §3.1. Sin commit: todo queda en stage.*

### Cambio de alcance en §3.1 (D-58, reemplaza a D-53)

**El sonido.** Jhon quiere un sonido simple al terminar una tarea, y nada más.

- Suena `afplay /System/Library/Sounds/Glass.aiff` cuando una sesión que corre en la app termina un turno y no está a la vista.
- Como mucho un sonido cada 2 s.
- Se quitaron `osascript`, los banners, «te necesita» y el cuerpo del mensaje. `waiting` no suena.
- El interruptor por categoría pasó a llamarse «Sound when done»/«Sonido al terminar». En la CLI es `cat set <slug> --sound`.

**El reporte a la coordinadora:**

- Cada sesión ya se lanzaba con `-n <categoría>-<slug>` desde la fase 1. Renombrar no cambia esa dirección, porque solo cambia el título.
- El contexto de una sesión con coordinadora le pide reportar con `SendMessage (to: "<dirección>")` qué hizo, qué quedó pendiente y dónde está el detalle, y detenerse. Si el envío falla, se lo dice al usuario.
- El de la coordinadora le dice que encargue con `notify_when_idle: true`.
- Sin coordinadora, nada de eso.

**Permisos:** el `--settings` lleva `permissions.allow: ["SendMessage", "ListAgents"]`.

- La documentación oficial nombra así las dos herramientas.
- Dice que no piden permiso y que se nombran sin especificador.
- También dice que `--settings` combina sus listas con las del usuario.
- **Falta probarlo con el claude real.**

**Guarda nueva:** `claude_sockets_guard.rs` falla si el código fuente nombra `cc-socks`.

**Ya implementados cuando llegó el cambio:** B1, B2 y B3 estaban en verde para las notificaciones. El sonido conserva de ellos el campo `entered` del socket y el filtro «corre en la app». B1 y B2 dejan de aplicar, porque ya no hay banners ni cuerpo.

**Investigación del `.app` firmado ad hoc:** no se hizo. La planificadora la canceló con el cambio de alcance.

### Correcciones

| # | Qué | Cómo quedó |
|---|---|---|
| B4 | El lote reiniciaba sesiones `keep_alive` | Quedan fuera del lote y del aviso. La selección y el lote pasaron al host (`session_host/claude_version.rs`) |
| B5 | Si la versión no se leía al relanzar, quedaba la vieja | Queda en NULL (`forget_claude_version`) |
| B6 | En `--no-window` las noticias crecían sin límite | Se vacían allí, y además el host guarda como mucho 64 sin leer, siempre las más nuevas |
| 7 | El panic hook se instalaba después de `from_environment` | Es lo primero de `run()`, con un solo `AppLog` por archivo (`AppLog::shared`) |
| 8 | El mensaje del pánico podía traer texto de una conversación | Se guardan sus primeros 120 caracteres, con archivo, línea e hilo. El riesgo que queda está en D-56 |
| 9 | Una base ilegible se registraba como «created» | Se distinguen tres casos: no había, versión N, ilegible |
| 10 | La memoria de un programa muerto aparecía como «0 MB»; la cifra saltaba | No se mide. Se muestra el máximo del último minuto. El total y el singular están corregidos |
| 11 | H-008 se esquivaba con espacio, minúsculas, `OR REPLACE` y la lista en la línea siguiente | El patrón nuevo atrapa las 6 variantes de `bad.txt`; el viejo dejaba pasar 5 |
| 12 | `session_host_lifecycle_test.rs` tenía 658 líneas | Se partió en lifecycle (218), hibernation (250), start_quit (96) y version (161) |
| 13 | El lote no tenía prueba con ocupadas y `keep_alive` | `the_batch_resumes_only_the_idle_sessions_that_are_not_kept_alive` |
| 14 | Con solo ocupadas, la barra seguía diciendo «reanudar las idle»; «idle» sin traducir | Ahora dice «1 sesión ocupada sigue en la versión anterior de Claude Code», sin botón. En español, «en reposo» |
| 15 | «en 1 sesiones» | `Texts::count` elige entre `_one` y `_other`. Hay una prueba de que toda clave en singular tiene su plural |

**D-54** ahora incluye el caso del pid reutilizado entre barridos y que la planificadora la aceptó.

### Tests

690 en verde en todo el workspace. **Los que vi fallar antes del arreglo,** y por qué:

| Test | Qué vi |
|---|---|
| `notification_news_test` (3, versión con banners) | B1: un `agent_completed` mientras esperaba volvía a avisar. B2: el cuerpo era el prompt del usuario. B3: avisaba de una sesión de afuera. Los tres fallaban por eso. Después el sonido los reemplazó |
| `hook_outcome_use_cases::the_app_is_told_the_state_an_event_moved_the_session_into` | Rojo por un **error mío en el test**: puse `idle_prompt` entre los avisos que no cambian el estado, pero por la regla de la fase 1 saca de `waiting` |
| `claude_update_test::a_session_kept_alive_is_never_resumed_nor_counted` (B4) | El servidor entraba en `idle` |
| `the_batch_resumes_only_the_idle_sessions_that_are_not_kept_alive` (B4, 13) | El lote reiniciaba también la `keep_alive` |
| `a_launch_whose_version_cannot_be_read_forgets_the_version_of_the_run_before` (B5) | Quedaba «2.1.300» |
| `news_nobody_takes_are_kept_only_up_to_a_limit` (B6) | Se guardaban 300 noticias |
| `a_panic_at_the_start_of_the_app_is_logged` (7) | El registro quedaba vacío |
| `a_long_panic_message_is_cut_to_120_characters` (8) | El mensaje entero, con `SECRET-TAIL`, llegaba al registro |
| `a_program_that_ended_is_not_measured` (10) | La raíz muerta volvía con 0 |
| `messaging_the_other_sessions_is_allowed_and_nothing_else_is` | No había `permissions` |
| `reporting_context_test` (2 de 3) | Sin reporte ni `notify_when_idle` en el contexto. La tercera, sin coordinadora, pasó desde el principio |
| `the_largest_context_without_a_list_fits` | Con las frases nuevas daba 2.063 caracteres (inglés) y 2.078 (español). Se acortó el reporte y se quitó la línea redundante |
| H-008 | El patrón viejo no marcaba 5 de las 6 variantes |

**Los que solo fallaron al compilar,** porque eran funciones nuevas: `AppLog::shared`, `migration_note` (punto 9), `ShownMemory`, `total` y `Texts::count`.

**Los que no vi fallar:**

- la política del sonido y su limitador de 2 s, que escribí después de la política;
- las dos pruebas del sonido por el socket real;
- la guarda de `cc-socks`, porque no hay nada que atrapar;
- `every_counted_text_has_both_forms`.

### Mediciones (`scripts/accept-phase2`)

| | CPU en reposo, ventana estándar | CPU, ventana desde 40 s | Footprint | Pico |
|---|---|---|---|---|
| Commit anterior (0dcbe3f) | 0,25 % | 0,27 % | 105 MB | 146 MB |
| Final del 3A | 0,40–0,42 % | 0,30 % | 103–106 MB | 147 MB |
| Final de la ronda | 0,37 %, 0,40 % | 0,35 %, 0,38 %, 0,30 % | 109–111 MB | 146–147 MB |

**Todo queda por debajo del 1 %.** Entre corridas iguales, la ventana tardía varía de 0,30 a 0,38 %. El footprint sube unos 5 MB, que atribuyo al historial de un minuto del medidor y a la barra de estado, sin haberlo aislado.

### Checklist de cierre

Corrida después del último cambio:

| Paso | Resultado |
|---|---|
| `cargo fmt --all --check` | limpio |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 errores, 0 avisos |
| `cargo test --workspace` | 690 pasan, 0 fallan |
| `cargo build --release` | compila |
| `cargo deny check` | advisories, bans, licenses y sources ok |
| `scripts/lint-findings` | 8 reglas, sin hallazgos |
| Tamaños (código de `src/`) | ningún archivo ni función por encima |
| `.githooks/pre-commit` | pasa sobre el stage |
| `scripts/accept-phase2` | 2 programas vivos (los visibles), CPU 0,37–0,40 %, footprint 109–111 MB, 0 vivos al salir |

### Pendiente

- **Para más adelante,** que viene de la fase 2:
  - Esc no cierra el menú contextual;
  - una sesión abierta desde la terminal aparece «en reposo» hasta que se pulsa Abrir.
- **`tests/integration/hosted_world.rs` tiene 304 líneas,** porque sumó `fire_hook`, `category`, `write_transcript` y `set_claude_version` al quedar compartidos.

### Lo que necesita a Jhon y a la planificadora

- Probar con el claude real:
  - el sonido;
  - el reporte de una sesión a su coordinadora;
  - que `SendMessage` no pida permiso.
- Lo demás está en `docs/fases/F3-prueba-manual.md`, pasos 1 a 4.


## Bloque 3B · Importador de cmux, paleta y teclado (§3.4, §3.5)

*2026-11-22. Decisiones D-59, D-60 y D-61.*

### Qué se hizo

- **Importador de cmux** (D-59):
  - módulo nuevo `core::modules::import`, con su dominio puro, los casos de uso `propose_import` y `apply_import`, y los ports `CmuxSnapshot` (solo lectura) e `ImportWriter` (una transacción);
  - CLI: `lithemux import cmux [--apply] [--into <cat>] [--skip n,…] [--include n,…] [--from <archivo>] [--json]`;
  - app: «File → Import from cmux…» abre una ventana con casillas, un selector de categoría por sesión y el botón Import.
- **«Abierta en otro lado» por la línea de comandos** (D-60):
  - al abrir cualquier sesión, si un proceso vivo tiene su conversación en el argv (`--resume`, `-r` o `--session-id`), no se lanza y se informa el pid;
  - la tarjeta lo muestra sola. Usa el barrido que el medidor de memoria ya hacía cada 30 s, sin barridos extra.
  - Esto cierra el pendiente de la Fase 2: «una sesión abierta desde la terminal aparece en reposo hasta que pulsas Abrir».
- **Paleta y teclado** (D-61):
  - `cmd+p` abre la paleta, con orden exacto > prefijo > aproximado y desempate por actividad reciente;
  - `cmd+alt+↓/↑` va a la sesión siguiente o anterior;
  - `cmd+shift+m` abre el menú de la sesión a la vista;
  - el menú contextual toma el foco: Esc lo cierra, y ↑/↓/Enter lo recorren. Esto cierra el otro pendiente de la Fase 2.

### Contra el archivo real de cmux

Corrí solo el modo que no escribe (`import cmux --json`), con un `LITHEMUX_HOME` desechable para no abrir siquiera la base de Jhon. Conté el resultado con `jq`, sin mirar títulos ni rutas:

| | |
|---|---|
| Sesiones de Claude propuestas | 24 |
| Marcadas para importar | 22 |
| Sin conversación guardada (desmarcadas) | 2 |
| Sin categoría (carpeta home: el usuario elige) | 23 |
| A una categoría nueva | 1 (una categoría nueva) |
| Fijadas | 4 |
| Ya en lithemux / sin id de conversación / sin Claude | 0 / 0 / 0 |

Para importar de verdad hace falta `--into <categoría>`, o elegir la categoría en la ventana, porque 23 de las 24 sesiones trabajan en el home.

### Pruebas que vi fallar antes del arreglo

- **Política de la propuesta:** 10 de 11 fallaron contra una versión vacía. La del home pasaba sin querer, sobre una lista vacía.
- **Ranking de la paleta:** 6 de 7 fallaron contra una versión que devolvía el orden original. La de rendimiento falló después, con la primera implementación real (24 ms por búsqueda con 1.000 sesiones); se arregló plegando el texto una sola vez.
- **Prueba de caso de uso de `prepare_launch`** (`a_conversation_that_runs_in_another_process_is_not_opened_again`): falló con la comprobación desactivada.
- **Integración del importador:**
  - `an_import_that_fails_halfway_stores_nothing` falló sin la transacción. El primer intento de romperla dejó viva una transacción con otro nombre y no probó nada; lo repetí bien;
  - `a_file_caught_half_written_is_read_again_a_moment_later` falló sin el reintento.
- **Guarda de solo lectura sobre cmux:** falló al agregar un `std::fs::write` en el lector.
- **Medidor con líneas de comandos:** falló con la suite completa en paralelo, por dos motivos reales:
  - `sysinfo` no siempre descarta al instante un proceso que acaba de terminar: ahora se pregunta al kernel con `is_alive`;
  - un subshell del script falso muestra un momento la misma línea de comandos que su padre.
  - En la vida real eso es inofensivo: cualquiera de los dos pids es «abierta en otro lado». Las pruebas lo contemplan, y los ayudantes esperan 300 ms después de que el falso escribe su pid.
- **CLI del importador:** 3 fallaron en la primera corrida:
  - esperaba la categoría `alpha`, cuando por especificación se llama como la carpeta de trabajo (`api`);
  - otra prueba corría a la vez un «cmux» con la misma conversación, y la comprobación, que mira toda la máquina, la rechazó con razón. Ahora cada prueba usa su propio id.
- **Las escribí después del código y no las vi fallar:**
  - `conversation_in`;
  - `choose`;
  - `elsewhere`;
  - el orden del teclado y `highlight_after`;
  - el estado de la paleta;
  - el estado de la ventana de importación;
  - la prueba del host con «cmux» vivo;
  - las dos pruebas del barrido real de procesos.

### Comprobado en la app real

Con un HOME falso que tiene el archivo sintético de cmux, un `LITHEMUX_HOME` desechable y el `claude` falso:

- `cmd+p` abre la paleta, que filtra al escribir; ↑/↓ y Enter saltan a la sesión, y Esc la cierra.
- `cmd+alt+↓` recorre las sesiones y da la vuelta al llegar al final.
- `cmd+shift+m` abre el menú de la sesión; ↓ marca una entrada, y Esc lo cierra y devuelve el foco a la terminal.
- La ventana de importación:
  - marca y desmarca;
  - con una sesión del home marcada y sin categoría, el botón no se activa y lo dice;
  - el selector ofrece las categorías existentes, las nuevas y una nueva para la carpeta propia;
  - importó 3 sesiones, hibernadas y en las categorías elegidas, y al abrirla otra vez dice que no queda nada.
- Con un `claude` falso lanzado aparte con `--resume <uuid>` de una sesión importada, la tarjeta pasó sola a «open in another terminal (pid N)», con el pid de ese proceso, en el siguiente barrido.
- **Un detalle de mis herramientas,** no de la app: las pulsaciones sintéticas con texto unicode no llegan al campo; las teclas reales sí.

### Medidas

`scripts/accept-phase2`, build release:

| | CPU en reposo | Footprint | Pico |
|---|---|---|---|
| Final de la ronda del 3A | 0,37–0,40 % | 109–111 MB | 146–147 MB |
| 3B, corrida 1 | 0,47 % | 130 MB | 167 MB |
| 3B, corrida 2 | 0,45 % | 113 MB | 149 MB |
| 3B sin guardar líneas de comandos (solo para medir) | 0,33 % | 107 MB | 148 MB |

Guardar las líneas de comandos en el barrido cuesta unos 6 MB y una décima de punto de CPU: `sysinfo` guarda el argv de todos los procesos, no solo el de los que interesan. Todo sigue por debajo del 1 % y de los 200 MB. La primera corrida, con 130 MB, no se repitió.

### Checklist de cierre

Corrida después del último cambio:

| Paso | Resultado |
|---|---|
| `cargo fmt --all --check` | limpio |
| `cargo clippy --workspace --all-targets` | 0 errores, 0 avisos (con `-D warnings`) |
| `cargo test --workspace` | 747 pasan, 0 fallan |
| `cargo build --release` | compila |
| `cargo deny check` | advisories, bans, licenses y sources ok |
| `scripts/lint-findings` | 8 reglas, sin hallazgos (751 archivos) |
| Tamaños (código de `src/`) | ningún archivo de más de 300 líneas ni función de más de 50 |
| `unsafe` nuevo | ninguno |
| `.githooks/pre-commit` | pasa sobre el stage (104 archivos) |
| `scripts/accept-phase2` | CPU 0,45–0,47 %, 0 programas vivos al salir |

### Pendiente y límites

- **Hasta 30 s** tarda la tarjeta en mostrar una sesión abierta fuera de la app: espera al siguiente barrido. Al abrir, la comprobación es inmediata.
- **La tarjeta dice «open in another terminal»** también cuando quien la tiene es cmux. El texto viene de la Fase 2; si se prefiere «abierta en otro lado», es un cambio de una línea en los locales.
- **El menú de una sesión abierto con el teclado** aparece arriba en la barra lateral, no junto a la tarjeta, porque la vista no guarda dónde quedó dibujada cada tarjeta.
- **Archivos de prueba que ya pasaban de 300 líneas:** `launch_use_cases.rs` sumó 4 (el campo del port nuevo en su `World`); mis pruebas nuevas van en archivos aparte.

### Lo que necesita a Jhon y a la planificadora

- La importación real en un home temporal (`docs/fases/F3-prueba-manual.md`, bloque 3B, paso 1).
- Que la comprobación «abierta en otro lado» reconozca el `claude` real que lanzó cmux (paso 3).
- Los atajos con el teclado Latin American (paso 5).

## Ronda de correcciones del bloque 3B

*2026-11-22. Junta la auditoría independiente y la prueba con clics de la planificadora. Decisiones: ampliaciones de D-59, D-60 y D-61, y D-62 nueva. Hallazgo nuevo: H-009.*

### Correcciones

| # | Qué | Cómo |
|---|---|---|
| 1 | Slug de 50 caracteres que no se podía abrir | `free_slug` recorta la base para que quepa el número, sin dejar `--`; el import valida cada slug con `Slug::parse` antes de guardar |
| 2 | Tildes en los slugs (`migraci-n`) | `fold_latin` en `category::domain`: quita las tildes latinas en NFC y en NFD. Vale para toda la app; los slugs ya guardados no cambian |
| 3 | Propuesta vieja que duplicaba la categoría | al aplicar, `New { root_dir }` se resuelve primero contra las categorías existentes con esa raíz |
| 4 | «Abierta en otro lado» pegado | la fase del host se muestra mientras su pid vive o el barrido ve la conversación; si no, vuelve el estado de la base. El test que fijaba el bug se rehízo |
| 5 | Importación concurrente | relectura y escritura dentro de una transacción `IMMEDIATE` (port `ImportStore`); migración 6 con el índice único parcial; una base con duplicados no se migra y los nombra (D-62). `home-b3` y la base de Jhon, leídas en solo lectura: 0 duplicados |
| 6 | La ventana leía y aplicaba en el hilo de la interfaz | `background_spawn`, cada vez con su propia conexión; botón gris mientras trabaja |
| 7 | Detección por argv | solo cuenta `claude` (`argv[0]`, o `argv[1]` tras un intérprete); un flag sin valor no toma el siguiente. Los falsos negativos (`--continue`, `--resume <nombre>`, `/clear`, `/resume`) quedan documentados en D-60 |
| 8 | Robustez de la lectura | tope de 20 MB por `metadata().len()`; «has no version»; versión no entera es un error de formato, sin reintento; `null` se lee como lista vacía |
| 9 | Tableros huérfanos | se crean después del commit; si uno falla, se avisa (CLI) o se anota en `app.log` (app) y se sigue |
| 10 | `--into` sobre una sesión que ya tenía categoría | no era `--into`: la prueba manual sugería una categoría con raíz `~`, que contenía todas las carpetas. Ahora una categoría con raíz en el home, o más arriba, no cuenta como contenedora. La ayuda de `--into` y la prueba manual dicen que vale solo para las que quedaron sin categoría |
| 11 | «open in another terminal» para cmux | «open elsewhere (pid N)» / «abierta en otro lado (pid N)» |
| 12 | La barra lateral no seguía a cmd+alt+↑/↓ | cada tarjeta es hija directa de la lista; la lista se desplaza a la sesión que gana el foco (`ScrollHandle::scroll_to_item`). Verificado con 37 sesiones |
| 13 | `CmuxSessionFile::path()` público | quitado |
| 14 | Paleta | se desplaza con la selección (`ScrollHandle`); pliega con `fold_latin`, NFD incluido. Verificado bajando 20 filas |
| 15 | La guarda de cmux se esquivaba | lee tokens sin comentarios ni doc comments; ruta por `"cmux"` o `cmuxterm`; marca `write*`, `set_*`, `persist`, `Command`, `append`, `truncate`… Las 8 variantes de la planificadora están en sus ejemplos |
| 16 | Tests | tableros en disco en la prueba de la transacción; largo máximo; propuesta vieja; `-r` en otro programa; archivo sin `version`; la prueba de rendimiento pasó a 100 sesiones con un cuadro (16 ms) de límite, en debug y en paralelo (medido: ~1 ms; criterio en D-61) |
| 17 | `launch_use_cases.rs` de 320 líneas | partido en `launch_world.rs` (72), `launch_use_cases.rs` (186) y `launch_lock_use_cases.rs` (80) |
| 18 | `measure` leía la memoria de los procesos vigilados | solo `is_alive` para ellos |

**Además:**

- **H-009:** toda transacción de SQLite en `src/` es `IMMEDIATE`. Sale del bug 5; la versión anterior del almacén de importación la habría violado.
- **`sqlite_session_repository.rs`** pasaba de 300 líneas con el mapeo del índice nuevo: el paso de fila a `Session` va ahora en `session_row.rs`.
- **Las pruebas con «cmux» vivo** lanzan el falso por un enlace llamado `claude`, porque solo cuenta ese nombre. El nombre sale de `CLAUDE_PROGRAM`, la misma constante que usa la detección.
- **`elsewhere_test` ya no arma líneas de comandos:** la guarda de programas reales tomaba `CommandLine` por `Command`. Prueba `open_elsewhere` con el mapa ya armado; el parseo del argv se prueba en `core`.

### Pruebas que vi fallar antes del arreglo

- **Slugs:**
  - largo con el número;
  - recorte sin `--`;
  - tildes NFC y NFD.
  - La primera versión de una de estas pruebas tenía un predicado que daba por ocupado todo slug de 48, y el bucle no terminaba. Lo corregí.
- **Propuesta:** la categoría con raíz en el home.
- **Integración del importador:**
  - la propuesta vieja;
  - la transacción con los tableros: falló creando el tablero antes del commit;
  - archivo sin `version`;
  - versión no entera;
  - listas `null`;
  - archivo de más de 20 MB.
- **Migración 6** (con el `SCHEMA_VERSION` nuevo en las pruebas):
  - las cinco pruebas de migraciones;
  - las dos nuevas;
  - los dos contratos de «una conversación, una sesión» (SQLite y en memoria).
- **Detección por argv:** `-r` en otro programa y flag sin valor. Las tres pruebas con procesos reales fallaron hasta que el falso se lanzó como `claude`.
- **`elsewhere`:** «termina cuando el otro proceso ya no está».
- **Textos:** «open elsewhere» y «abierta en otro lado».
- **Paleta:** tildes en NFD.
- **Guarda de cmux:** las 8 variantes de la planificadora, corridas contra la versión anterior de la guarda (que tomé del stage): no vio ninguna.
- **H-009:** marca un archivo de prueba con `unchecked_transaction()`.

**No las vi fallar:**

- la prueba de dos importaciones a la vez (con o sin `IMMEDIATE`, SQLite rechaza la segunda escritura, así que no logré provocar el duplicado);
- los dos nombres largos en la propuesta (pasaron porque `free_slug` ya estaba arreglado);
- el desplazamiento de la barra lateral y de la paleta: es código de vista, verificado en la app real;
- el cambio de `measure` (rendimiento, sin prueba de comportamiento).

### Medidas

`scripts/accept-phase2`, build release, después del último cambio:

| | CPU en reposo | Footprint | Pico |
|---|---|---|---|
| Bloque 3B | 0,45–0,47 % | 113–130 MB | 149–167 MB |
| Esta ronda | 0,40 %, 0,38 % | 126 MB, 110 MB | 166 MB, 149 MB |

En ambas rondas, la primera corrida da un footprint alto y la segunda, normal; no lo aislé.

### Checklist de cierre

Corrida después del último cambio:

| Paso | Resultado |
|---|---|
| `cargo fmt --all --check` | limpio |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 errores, 0 avisos |
| `cargo test --workspace` | 769 pasan, 0 fallan |
| `cargo build --release` | compila |
| `cargo deny check` | advisories, bans, licenses y sources ok |
| `scripts/lint-findings` | 9 reglas, sin hallazgos |
| Tamaños (código de `src/`) | ningún archivo de más de 300 líneas ni función de más de 50 |
| `unsafe` nuevo | ninguno |
| `.githooks/pre-commit` | pasa sobre el stage |
| `scripts/accept-phase2` | CPU 0,38–0,40 %, 0 programas vivos al salir |

### Pendiente y límites

- **La detección por argv no ve** `--continue`, `--resume <nombre>`, ni un `/clear` o `/resume` dentro de la sesión (D-60).
- **La tarjeta tarda hasta 30 s** en mostrar algo abierto fuera de la app; deja de mostrarlo en unos 3 s cuando ese proceso termina.
- **El menú de sesión abierto con el teclado** sigue apareciendo arriba en la barra lateral.

### Lo que necesita a Jhon y a la planificadora

- Repetir con el archivo real los pasos 1 a 3 del bloque 3B, en especial `cat add trabajo --root ~` + `--into trabajo`: la sesión de `memory` debe quedar en su categoría nueva.
- La migración 6 sobre la base real: la abre la app nueva. Ya comprobé, solo leyendo, que no tiene conversaciones compartidas.

## Bloque 3C · Arrastrados del 3B y spike del navegador (§3.7)

*2026-11-22. Decisiones D-63 a D-66. Hallazgos nuevos: H-010 y H-011. Sin commit: todo queda en stage.*

### Parte 1: arrastrados del 3B

| Qué | Cómo |
|---|---|
| El panel conservaba «open elsewhere (pid N)» después de que la tarjeta volvía a hibernada | el panel toma la fase de `live.shown_phase`, la misma que la tarjeta, en vez de la del host (`tab_view.rs`). Regla **H-010**: `status_of(host.phase(…))` en la app |
| Un `claude_path` inválido caía en silencio al `claude` del PATH | si `claude_path` está definido, es el único candidato. Si no pasa `checked_value` (un valor con comillas JSON no pasa) o no es ejecutable, `LaunchError::ClaudePathUnusable` y no se consulta ningún shell. Se ve en la tarjeta, el panel, `app.log` y `doctor` (D-63). Regla **H-011**: un texto de `src/` que hable de caer de `claude_path` a otra cosa |

**Pruebas que vi fallar antes del arreglo:**

- `a_claude_path_that_is_not_executable_is_an_error_and_no_other_claude_is_looked_for`;
- `a_claude_path_stored_with_quotes_is_an_error_not_a_fallback`;
- `doctor_reports_an_unusable_claude_path_and_chooses_nothing`;
- `session_host_claude_path_test`, en sus dos variantes (ruta que no existe y ruta con comillas). Usa un shell falso que sí encuentra un `claude` en su PATH: antes del arreglo la sesión quedaba `Running` con ese otro binario.
- H-010 y H-011 marcan su `bad.txt` y dejan pasar su `good.txt`. H-010 marca la línea vieja de `tab_view.rs` (`status_of(live.host.phase(session_id), …)`, tomada con `git show HEAD:`).

**No la vi fallar:** la del panel que sigue a la tarjeta es de vista; la cubre H-010 y el paso 1 de la prueba manual del 3C.

### Parte 2: spike del navegador

**Build con el spike** (no es la de release normal, que no enlaza WebKit):

```sh
cargo build --release -p lithemux-app --features webview-spike
```

- Reemplaza el `target/release/lithemux-app` normal. `cargo build --release -p lithemux-app` lo deja otra vez sin el spike.
- La pestaña se abre con el botón ◎ de la barra de cada panel. `LITHEMUX_SPIKE_URL` elige la página (por defecto `about:blank`); `LITHEMUX_SPIKE_FRAUD_WARNING=on` enciende el aviso de sitios fraudulentos, solo para medirlo.
- Para medir sin teclado: `LITHEMUX_SPIKE_OPEN=1` abre una página al arrancar, `LITHEMUX_SPIKE_CLOSE_AFTER=<s>` la cierra y `LITHEMUX_SPIKE_QUIT_AFTER=<s>` cierra la app.
- Tamaño: 9,67 MiB sin el spike, 9,86 MiB con él.

**Diseño** (D-64): `wry` 0.57.2 crea el `WKWebView` como subvista del NSView de gpui, que gpui da por `HasWindowHandle`. El único `unsafe` está en `crates/webview-spike/src/configuration.rs`, que apaga el aviso de sitios fraudulentos y elige el almacén en memoria. El crate tiene `unsafe_code = "deny"` y H-002 se amplió con esa ruta exacta. **Lo que guarda la página** (D-65): todo en memoria (`nonPersistentDataStore`); nada en disco.

#### Criterios

| # | Criterio | Resultado | Cómo se comprobó |
|---|---|---|---|
| 1 | Ocupa el área de su panel y la sigue | **cumple, salvo el cambio de escala, que no se pudo probar** | capturas: dividir, arrastrar el divisor, redimensionar la ventana y cerrar paneles; la página sigue el área exacta y no queda vista fantasma. Solo hay una pantalla (Retina), así que mover la ventana a otra escala no se probó. La posición va en píxeles lógicos, que es lo que AppKit escala, pero no lo afirmo sin verlo |
| 2 | Se oculta y reaparece sin recargar | **cumple** | capturas, antes del incidente: lo escrito en un campo de la página sigue allí después de pasar a la terminal y volver a la pestaña |
| 3 | Lo de gpui que va encima no queda tapado | **cumple con salida** | la página se oculta mientras están abiertos la paleta, el menú contextual o un arrastre de pestaña, y vuelve al cerrarlos. La ventana principal no tiene diálogos sobre los paneles: los formularios de la barra lateral van dentro de la lista, y la importación y Ajustes son ventanas aparte. **Límite:** los tooltips de gpui que caen sobre el área de la página quedan debajo |
| 4 | Teclado y atajos | **pendiente: lo prueban la planificadora y Jhon** | antes del incidente vi, con lithemux al frente: la escritura va a la página; `cmd+p`, `cmd+alt+↓` y `cmd+w` funcionan con la página enfocada; un clic en la terminal le devuelve el teclado; un clic en la página enfoca su panel. Falta repetirlo a mano y probar `cmd+c/v/a` dentro de la página. Pasos: `F3-prueba-manual.md`, Bloque 3C, paso 3 |
| 5 | Cero red propia | **cumple, con el aviso de sitios fraudulentos apagado** | ver «Red», abajo. Con el aviso encendido, un servicio del sistema habla con Apple: el apagado (el `unsafe` de `configuration.rs`) es necesario |
| 6 | Costo medido; WebKit en el medidor de la §3.3 o por qué no | **cumple (medido y explicado)** | ver «Memoria y CPU», abajo |
| 7 | `cargo-deny` y sin Xcode completo | **cumple** | `cargo deny check` (con `all-features = true` en `deny.toml`, así que revisa `wry` y `objc2-web-kit`): advisories, bans, licenses y sources ok. La guarda de clientes HTTP y telemetría de `single_source_guards` pasa con el `Cargo.lock` nuevo. Compila con las Command Line Tools (`xcode-select -p` = `/Library/Developer/CommandLineTools`; no hay `Xcode.app`) |
| 8 | No rompe la terminal | **CPU en reposo: cumple. Latencia de tecleo: pendiente, la miden la planificadora y Jhon** | ver «Memoria y CPU». La Fase 2 no midió la latencia de tecleo (no está en su reporte), así que no hay cifra de antes: hay que medir las tres variantes el mismo día. Pasos: `F3-prueba-manual.md`, Bloque 3C, paso 4 |

#### Red (criterio 5)

`measure.sh`, en el scratchpad: lanza la app del spike en un `LITHEMUX_HOME` desechable con una página abierta, sin teclado ni ratón. Durante 2 min toma 24 muestras de `lsof -i` sobre la app y los procesos de WebKit que nacieron con ella (GPU, Networking y WebContent), y al final `nettop`.

| Aviso de sitios fraudulentos | Página | Sockets de internet (app + su WebKit) | Servicio Safe Browsing del sistema |
|---|---|---|---|
| apagado | `about:blank` | 0 de 24 muestras | no arranca |
| apagado | localhost | 0 de 24 | no arranca |
| encendido | `about:blank` | 0 de 24 | no arranca |
| encendido | localhost | 0 de 24 | **arranca y conecta con `17.253.13.148:443` (Apple)** en 5 de 24 muestras |

- El servicio de Safe Browsing es del sistema y lo comparte Safari: no es un proceso de la app ni lanzado por ella, pero arranca porque la página navegó con el aviso encendido. Se cuenta aparte.
- Con localhost, el registro del servidor local muestra solo los GET de la página.
- En las cuatro variantes, los procesos de WebKit terminan al salir la app.
- **Disco:** tras las corridas no hay nada de la app en `~/Library/WebKit`, `~/Library/Caches`, `~/Library/HTTPStorages` ni `~/Library/Containers` (D-65).

#### Memoria y CPU (criterios 6 y 8)

`mem.sh`, en el scratchpad: la app del spike en release, en un `LITHEMUX_HOME` vacío (sin sesiones), sin teclado ni ratón. CPU de 15 a 75 s después de arrancar; footprint con `footprint`.

| | App | WebKit GPU | WebKit Networking | WebKit WebContent | Total |
|---|---|---|---|---|---|
| Sin página: CPU en reposo | 0,13 % | — | — | — | 0,13 % |
| Sin página: footprint | 61 MB | — | — | — | 61 MB |
| Con una página en localhost: CPU en reposo | 0,13 % | 0,00 % | 0,02 % | 0,03 % | 0,18 % |
| Con una página en localhost: footprint | 66 MB | 12 MB | 6 MB | 24 MB | 108 MB |
| 20 s después de cerrar la página: footprint | 67 MB | 12 MB | 6 MB | 11 MB | 96 MB |

- **La página cuesta unos 47 MB:** 5 MB en la app y 42 MB en los tres procesos de WebKit.
- **Al cerrar la página, los procesos de WebKit siguen vivos** y WebContent baja a 11 MB. Terminan al salir la app (comprobado en las cuatro corridas de red). Si la pestaña completa sale adelante, conviene ver si soltar la última página puede soltarlos también.
- Con 20 sesiones y dos a la vista, como en la Fase 2, la tabla siguiente.

`scripts/accept-phase2` (20 sesiones, dos a la vista, sin página abierta), con el binario de release normal y con el del spike copiado en `target/release`, en orden normal, spike, spike, normal. Al final volvió el normal (idéntico byte a byte a la copia previa, sin WebKit en `otool -L`):

| Corrida | Build | CPU en reposo (60 s) | Footprint | Pico | Programas vivos al salir |
|---|---|---|---|---|---|
| 1 | normal | 0,20 s (0,33 %) | 107 MB | 149 MB | 0 de 2 |
| 2 | spike | 0,22 s (0,37 %) | 111 MB | 148 MB | 0 de 2 |
| 3 | spike | 0,22 s (0,37 %) | 104 MB | 147 MB | 0 de 2 |
| 4 | normal | 0,21 s (0,35 %) | 107 MB | 149 MB | 0 de 2 |

La diferencia de CPU es de 0,01 a 0,02 s en 60 s, del orden de la resolución de `ps` (0,01 s): **igual dentro de lo que se puede medir así.** La memoria no cambia. Con una página abierta, la CPU de la app tampoco cambia (0,13 % con y sin página en `mem.sh`).

**Por qué los procesos de WebKit no cuentan en el medidor de la §3.3:** el medidor suma el árbol de procesos de cada sesión, y su total el de la app. Los procesos de WebKit no son hijos de la app: los lanza `launchd` como servicios XPC (su padre es el pid 1). Para atribuírselos a la app habría que preguntar al sistema qué proceso es «responsable» de ellos, y esa función no es una API pública de macOS. Si la pestaña completa sale adelante, la opción honesta es mostrar aparte lo que mide WebKit, por los pids que tenga la app, sin sumarlo a las sesiones.

#### Veredicto

**Ningún criterio falló. El GO queda pendiente de tres comprobaciones que no puedo hacer yo** (D-66):
- el teclado (criterio 4);
- la latencia de tecleo (criterio 8);
- el cambio de escala entre pantallas (criterio 1).

Mientras tanto, el código se queda tras el feature, apagado y fuera del release, y la pestaña completa no se implementa.
- **Si las tres pasan, es GO.** La pestaña completa se planifica como bloque aparte, y Jhon decide si se mantiene el `unsafe`.
- **Si alguna falla y no tiene arreglo dentro de los 2 días del spike, es NO-GO.** Se borra lo que lista D-64 y queda este reporte con lo medido.

### Incidente 20:36

**Qué pasó.** Para medir la latencia de tecleo (criterio 8) envié teclas sintéticas con `CGEvent`. La ventana de lithemux quedó detrás de cmux y las teclas cayeron en cmux, en el Mac de Jhon mientras lo usaba. Apareció el diálogo de cerrar sesión del sistema; Jhon lo canceló.

**Cronología** (hora local, por el `mtime` de los archivos de las herramientas):

- **20:35:55 a 20:36:17, corrida 1:**
  - `c3/start.sh` lanzó `c3/app-plain` y su ventana quedó detrás de cmux;
  - `click <pid> 700 400` hizo clic por coordenadas de pantalla, y el clic cayó en cmux;
  - `typecmd.sh` falló por un error de zsh y solo envió un Enter;
  - `keylat 200 50` envió 200 letras sin modificadores, una cada 50 ms, en un ciclo que incluía la `q`.
- **20:36:26 a 20:36:41, corrida 2:** `ctrl+c`, luego `python3 r.py 200 a.txt` con Enter, luego otras 200 letras.
- La planificadora ordenó detener todo. Lo reporté con esta cronología. No quedó nada enviando entrada.

**Causa:**

- Las herramientas enviaban teclas a la app que estuviera al frente, sin comprobar cuál era. El clic por coordenadas no garantizaba que la ventana bajo el punto fuera de lithemux.
- **Sobre el diálogo de cerrar sesión, una hipótesis, no un hecho:** nunca envié `cmd+shift+q` ni otra combinación del sistema. Lo más probable es que una `q` sin modificadores coincidiera con `cmd+shift` pulsados por Jhon: los eventos se crean sin fuente propia y el sistema puede mezclarlos con los modificadores que estén abajo.

**Guardas nuevas:**

- `click` y `keylat` comprueban, en el mismo programa y antes de cada tecla, que el pid de lithemux es la app al frente, y abortan si no lo es.
- Antes de un clic, `click` activa lithemux y comprueba que la ventana más al frente en ese punto sea suya.
- `keylat` solo usa letras sin `q`, `h`, `m` ni `w`, y vacía los modificadores de cada evento.
- **Reglas de la planificadora, vigentes:**
  - ninguna tecla ni clic sintético sin esa comprobación;
  - nunca combinaciones que atiende el sistema (`cmd+shift+q`, `cmd+q`, `cmd+opt+esc`, `ctrl+cmd+q`, `cmd+tab`);
  - las pruebas de foco y teclado las hacen la planificadora y Jhon;
  - la ejecutora mide solo lo que no usa teclado.

**Lección, no regla H.** Las herramientas viven en el scratchpad, fuera del repo, y el lint de hallazgos revisa el repo: una regla H no las vería. Lo que sirve es la regla de trabajo de arriba: nada sintético sin comprobar la app al frente en el mismo script, y las pruebas de teclado a cargo de una persona.

**Segundo tropiezo, sin entrada sintética.** De 20:51 a 22:19, la medición de memoria se quedó colgada en silencio: el temporizador de salida del spike solo corría si había una página abierta, y la variante sin página nunca terminaba. Arreglos:
- el temporizador ya no depende de la página;
- a pedido de la planificadora, `mem.sh` y `measure.sh` tienen ahora un tope de tiempo externo (`bounded_wait.zsh`): si la app sigue viva al llegar el tope, la cierran ellos y escriben «timeout» en el resultado.

### Checklist de cierre

Corrida después del último cambio. Ese cambio partió `Workspace::new` y `render_toolbar`, que pasaban de 50 líneas, y no cambió la conducta. Las mediciones de arriba usan el binario de antes de esa partición, con el temporizador de salida ya arreglado.

| Paso | Resultado |
|---|---|
| `cargo fmt --all --check` | limpio |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 errores, 0 avisos |
| ídem con `--features lithemux-app/webview-spike` | 0 errores, 0 avisos |
| `cargo test --workspace` | 773 pasan, 0 fallan |
| ídem con `--features lithemux-app/webview-spike` | 773 pasan, 0 fallan |
| `cargo build --release` | compila; sin WebKit en `otool -L` |
| `cargo deny check` (todas las features) | advisories, bans, licenses y sources ok |
| `scripts/lint-findings` | 11 reglas, sin hallazgos |
| Tamaños (código de `src/`) | ningún archivo de más de 300 líneas ni función de más de 50 |
| `unsafe` nuevo | solo `crates/webview-spike/src/configuration.rs`, con `// SAFETY:` (D-64) |
| `.githooks/pre-commit` | pasa sobre el stage |
| `scripts/accept-phase2` | CPU 0,33–0,37 %, 0 programas vivos al salir, con y sin el spike |
| Stage | `git add -A`: 47 archivos. `docs/PLAN.md` ya venía en stage, con los criterios de la §3.7 de la planificadora |

### Pendiente y límites

- **Criterios 4 y 8 (latencia):** los prueban la planificadora y Jhon con los pasos 3 y 4 del Bloque 3C de `F3-prueba-manual.md`. Las herramientas de la latencia están en el scratchpad de la ejecutora, que se borra al reiniciar.
- **Criterio 1, cambio de escala:** necesita una segunda pantalla con otra escala.
- **Tooltips de gpui** sobre el área de la página: quedan debajo.
- **`cmd+c/v/a` dentro de la página:** sin probar. Según el código de `wry` 0.57, puede que copiar y pegar no lleguen a la página si el menú Edit de la app no los reenvía.
- Si sale GO, mantener el `unsafe` en la versión final lo decide Jhon al planificar la pestaña completa (D-64).

### Lo que necesita a Jhon y a la planificadora

- Los pasos 1 a 4 del Bloque 3C de `F3-prueba-manual.md`. Con los pasos 3 y 4 se decide el GO o el NO-GO (D-66).
- Cerrar `c3/app-plain` (pid 88214), que quedó abierta desde la medición de las 20:36. El `kill` me fue denegado y no busco otra vía.
- El servidor local de las mediciones (`python3 -m http.server 8765`, pid 85063) sigue vivo, solo en `127.0.0.1`. Se puede cerrar al terminar los pasos 3 y 4, que lo usan.

## Ronda de correcciones del bloque 3C

*2026-11-23. Lo confirmado por la auditoría independiente. Decisiones: ampliaciones de D-63 y D-66, y D-67 y D-68 nuevas. Sin regla H nueva (por qué, en D-67). Sin teclas ni clics sintéticos: no ejecuté la app; los pasos de la app son de la planificadora y Jhon.*

### Correcciones

| # | Qué | Cómo |
|---|---|---|
| 1 | **Página fantasma** (importante): al borrar la única sesión de un panel de una división, su página quedaba encima del otro panel, y el panel siguiente heredaba la página por reutilizar el id | `Layout` recuerda el último id que dio y no lo repite (`last_id`, compatible con lo guardado). Las páginas siguen a la disposición en un solo lugar (`Browsers::follow_the_layout(&Layout)`), llamado desde `follow_the_sessions` y en cada cuadro, desde el `render` de `Workspace` (D-67) |
| 2 | **pid reutilizado:** la tarjeta volvía a decir «open elsewhere (pid N)» si macOS reutilizaba el pid | en cada refresco (3 s) se anotan los procesos vistos terminar, y uno anotado no vuelve (`ended_holders`, D-68). Ya no hay `is_alive` en cada dibujo |
| 3 | **`claude_path` con comillas:** Ajustes lo mostraba vacío, y el mensaje decía «not an executable» de algo que sí lo era | el error lleva el motivo real: comillas, no absoluta, no existe, carpeta o no ejecutable. Cada motivo tiene su texto en los dos idiomas, con cómo arreglarlo y que vacía usa el `claude` del PATH. Ajustes muestra el valor guardado tal cual y el mismo aviso debajo (ampliación de D-63) |
| 4 | **Pruebas que faltaban** | integración con el probe real: carpeta y archivo sin permiso de ejecución. El panel y la tarjeta salen ahora de una sola función (`session_status`, que también usa la paleta), con una prueba de que dicen lo mismo |
| 5 | **Detalles** | referencias «D-63» del spike corregidas a D-64; con el spike, el botón dice «Open a browser tab (spike)» y no «coming later»; `browser_spike` exige `target_os = "macos"` además del feature; `Page::place` y `Page::focus` no llaman a `wry` si la vista no está en una ventana, porque `wry` haría `window().unwrap()` en medio del dibujo |
| 6 | **CI y checklist** | `ci.yml` corre también Clippy y los tests con `--features lithemux-app/webview-spike`. El checklist de este reporte también. El checklist de `docs/PLAN.md` es de la planificadora y no lo toqué |

Los cinco límites que no se arreglan ahora quedaron anotados en D-66, para el bloque de la pestaña completa.

### Pruebas que vi fallar antes del arreglo

- **`pane_ids_test`:** «el id de un panel cerrado no se da a uno nuevo» y «un panel vaciado al borrar su sesión no presta su id».
- **`browsers_test`:** «un panel nuevo no recibe la página de uno cerrado». Lo vi fallar volviendo `next_id` a la regla vieja y pasar con la nueva.
- **`preferences_test`:** «un `claude_path` guardado mal se muestra tal como está».

**No las vi fallar:**

- **Las otras tres de `browsers_test`:** prueban la función de seguimiento nueva, que antes no existía en esa forma. El bug estaba en un camino de la vista que no la llamaba, y la llamada en cada cuadro es código de vista. Por eso, el paso 5 del Bloque 3C de la prueba manual.
- **Las de los motivos de `claude_path`** (política, casos de uso, textos y Ajustes): se escribieron con los tipos nuevos, que antes no tenían motivo. La carpeta y el archivo sin permiso ya los rechazaba el código anterior; lo nuevo es el motivo.
- **`ended_holders` y la prueba del panel y la tarjeta:** nacieron con su función.

### Checklist de cierre

Corrida después del último cambio:

| Paso | Resultado |
|---|---|
| `cargo fmt --all --check` | limpio |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 errores, 0 avisos |
| ídem con `--features lithemux-app/webview-spike` | 0 errores, 0 avisos |
| `cargo test --workspace` | 788 pasan, 0 fallan |
| ídem con `--features lithemux-app/webview-spike` | 792 pasan, 0 fallan |
| `cargo build --release` | compila; sin WebKit en `otool -L` |
| `cargo build --release -p lithemux-app --features webview-spike` | compila. Después, `target/release` volvió al binario sin el spike |
| `cargo deny check` | advisories, bans, licenses y sources ok |
| `scripts/lint-findings` | 11 reglas, sin hallazgos |
| Tamaños (código de `src/`) | ningún archivo de más de 300 líneas ni función de más de 50 |
| `unsafe` | solo `detached.rs` y `crates/webview-spike/src/configuration.rs`, como antes |
| `.githooks/pre-commit` | pasa sobre el stage |

### Lo que necesita a Jhon y a la planificadora

- **Pasos 2 y 5 del Bloque 3C** de `F3-prueba-manual.md`. El 2 se amplió con el aviso de Ajustes y el caso de la carpeta; el 5 es nuevo: ninguna página queda fuera de un panel.
- **Pasos 3 y 4:** siguen igual (criterios 4 y 8).
- **El checklist de `docs/PLAN.md`:** si debe nombrar la corrida con el feature, eso lo decide la planificadora.
