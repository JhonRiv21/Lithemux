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

