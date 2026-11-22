# Fase 2 · Reporte

**Ejecutora:** Fable 5.2. La fase va en cuatro bloques; este archivo crece con cada uno.

| Bloque | Estado |
|---|---|
| §2.0 Correcciones | **Hecho, en stage, sin commit.** Pendiente de auditoría y de tres comprobaciones de Jhon |
| §2.1 SessionHost y socket | Sin empezar |
| §2.2 + §2.3 Barra lateral y área de trabajo | Sin empezar |
| §2.4 a §2.6 Ciclo de vida, restauración y ajustes | Sin empezar |

Los criterios de aceptación de la fase se verifican al cerrar el último bloque.

## 2.0 · Correcciones

**Fecha:** 2026-11-21. Los 20 puntos de la §2.0 del plan.

### Resumen

- **18 puntos arreglados** y con test.
- **La auditoría independiente** encontró tres guardas que se saltaban, varios tests que no detectaban regresiones y una lista de marcadores incompleta. Todo corregido en la sección «Auditoría de la §2.0», salvo lo que ahí se dice.
- **El arreglo del punto 1 traía una regresión grave**, que encontró la planificadora y ya está corregida (sección propia más abajo).
- **Punto 10 (socket):** arreglado como pide el plan, pero **en macOS el bug no se puede reproducir**, así que ningún test distingue el arreglo.
- **Punto 20 (tecla muerta): no resuelto.** No lo pude reproducir sin teclado. Queda un registro de diagnóstico y los pasos para Jhon.
- `NODE_OPTIONS` y las carpetas de envoltorios de cmux, que en la primera entrega quedaron abiertas, se resolvieron en la ronda de la auditoría.

### Punto por punto

«Visto fallar» dice si vi el test en rojo **antes** de escribir el arreglo.

| # | Punto | Arreglo | Test | Visto fallar |
|---|---|---|---|---|
| 1 | `resolve_claude` se cuelga | Salida leída en un hilo, comando en su propio grupo, `killpg` al terminar o al vencer (D-23) | `lingering_shell_test.rs` (2): `open` y `doctor` con un shell falso que deja `sleep 30 &` | **Sí**: 30,5 s y 60,3 s. Ahora 0,6 s los dos |
| 2 | Migraciones concurrentes | `BEGIN IMMEDIATE` y relectura de `user_version` dentro de la transacción | `concurrent_start_test.rs`: 8 procesos × 20 rondas sobre un home vacío | **Sí**: ronda 0, `table categories already exists` |
| 3 | `compact` pasa a `idle` | Con `compact` solo se toca la actividad (D-21) | Caso de uso `a_compaction_keeps_the_state…`; unitario de la transición; binario del hook con fixture `session_start_compact.json` | **Sí** el caso de uso. Los otros dos los escribí después del arreglo |
| 4 | `waiting` no vuelve a `working` | Sexto evento `PostToolUse`, síncrono, solo `waiting` → `working` (D-20) | Casos de uso (2), unitario, binario del hook en `a_whole_turn…`, JSON de `--settings` | **Sí** los dos casos de uso. p95 medido: 9,6 ms |
| 5 | TOCTOU en `open` | Candado antes de `plan_launch` | `the_transcript_is_looked_up_with_the_session_already_locked` | **Sí**: `Some(false)` |
| 6 | Pid viejo en el candado | Pid propio al adquirir, vacío al soltar; `holder()` lee sin `flock` y comprueba que el proceso vive (D-03) | `session_locks_test.rs` (3 nuevos) | **Uno de tres**: `a_new_holder_never_shows_the_pid…` falló. `asking_for_the_holder_never_blocks_an_acquire` **no reprodujo la carrera** ni con 20.000 intentos; se queda como propiedad. El tercero es una propiedad del diseño nuevo |
| 7 | `LITHEMUX_HOME` relativo | Se vuelve absoluto al leerlo | `a_relative_lithemux_home_reaches_the_child_as_an_absolute_path` | **Sí** |
| 8 | Contexto: saltos de línea y tope | Texto del usuario a una línea con máximos; corte final como último recurso (D-25) | `render_context_test.rs` (2 nuevos) | **Sí** los dos (11 líneas en vez de 8; tope superado) |
| 9 | `open` sin `lithemux-hook` | `plan_launch` exige que sea ejecutable; error `HookNotExecutable` | `open_fails_before_launching_when_the_hook_binary_is_not_executable` | **No**: el test usa una variante de error que no existía, así que lo escribí junto con el arreglo |
| 10 | Socket bloqueante | Socket no bloqueante antes de `connect` (rustix `net`) | `never_in_the_way_test.rs` (2): app que acepta y no lee; app que no acepta, con la cola llena | **No**. Pasaban antes del arreglo: en macOS `connect` a un socket Unix con la cola llena falla al instante. El arreglo protege a Linux |
| 11 | Guarda `real_programs` | Regla `inherited-env` (D-24) | 2 fixtures `bad`, 1 `good` | **Sí**: la guarda marcaba 9 de 11 archivos malos. Al activarla encontró un caso real en `app` |
| 12 | Guarda `arch_boundaries` | Un `domain/` solo toma tipos de dominio de otra puerta (D-24) | 2 fixtures `bad` y `a_domain_cannot_import_a_use_case_of_another_module` | **Sí** |
| 13 | Tests de D-03 que faltaban | Solo tests | `signals_and_lock_test.rs` (3): descendiente vivo sin candado, SIGTERM → 143 y SIGHUP → 129, SIGINT no mata al padre; tope de 48 del slug | No aplica: no había bug. Pasaron a la primera |
| 14 | Test que celebraba el huérfano | El test termina al padre con SIGTERM, comprueba que el hijo murió y reabre. Riesgo de SIGKILL en D-03 y nota en la §2.1 del plan | `opening_the_same_session_twice…` | No aplica |
| 15 | E/S en `doctor` y `category` | `inspect_hook` (launch), `locate_directory` (category), `Home::check_writable` (platform, D-26) | `inspect_hook_use_cases.rs` (3), `directories_test.rs` (2) | No aplica: es orden. Escritos con la implementación |
| 16 | Tipos junto a su caso de uso | `context_ports.rs`, `claude_inspection.rs`, `resolved_claude.rs`, `hook_inspection.rs` | Los existentes | No aplica |
| 17 | Tiempo relativo duplicado | `ActivityAge` en `session/domain`; lo usan la CLI y las dos plantillas | `activity_age_test.rs` (3) | No aplica |
| 18 | Funciones al límite | `handle_hook_event` en tres funciones; `compose` en cuatro | Los existentes (los snapshots del contexto no cambiaron) | No aplica |
| 19 | Entorno heredado de Claude Code | Lista única en `core::platform::process::session_markers`; la usan el lanzador, las sondas y la app (D-22). Regla `H-001` | `inherited_session_test.rs` (2, CLI); `child_env_test.rs` (4, app); `session_markers_test.rs` (4) | **Sí** los dos de la CLI (`CLAUDECODE reached the child`). Los de la app y los de la lista, escritos con el arreglo |
| 20 | Primera tecla muerta | **Sin arreglar.** Registro `LITHEMUX_IME_LOG` | `composition_test.rs` (2 nuevos) | **No se reproduce**: pasan |

**Sobre las reglas `H-00N`.** El plan pedía que los bugs 1, 2, 3 y 5 fueran reglas cuando se pudieran verificar desde el código, o tests de integración cuando no. Ninguno de los cuatro se ve en una línea de código, así que son tests. La única regla nueva es `H-001`, la del punto 19. Para ella el lint aprendió a excluir rutas (archivo `exclude`), con su test.

**Aviso sobre los tests.** Los escribí con la implementación en contexto. Salen de la redacción de la §2.0, pero no son independientes de mi lectura del código.

### Regresión del punto 1, encontrada por la planificadora

**Qué pasó.** Mi arreglo del punto 1 rompió el caso real. Con una terminal de verdad, la sonda `$SHELL -lic` quedaba en un grupo en segundo plano, el shell interactivo se detenía al tocar la terminal y la sonda agotaba el plazo. `lithemux` abría entonces el `claude` viejo (2.1.280). Mis tests corrían sin terminal y no lo vieron.

| Cambio | Test | Visto fallar |
|---|---|---|
| La sonda arranca en una sesión nueva, sin terminal (`setsid`). Es el único `unsafe` del workspace, con la regla `H-002` (D-27) | `on_a_terminal_test.rs` (2): `doctor` y `open` dentro de una PTY real, con un shell falso que cambia los ajustes de su terminal | **Sí**: 9,0 s y 6,0 s. Ahora menos de 1 s |
| El último `claude` que encontró el shell interactivo se recuerda y va antes que el shell no interactivo (D-28) | `resolve_claude_use_cases.rs` (5 nuevos) | **No**: los escribí junto con el cambio |

**Comprobado con el binario de release dentro de una PTY real** (`python3 pty.spawn`, con el zsh y el `claude` de la máquina): elige `~/.local/bin/claude` 2.1.330, sin aviso de respaldo, en 0,28 a 0,35 s. La primera corrida tras compilar tardó 3,5 s, de los que 0,44 s fueron la prueba en seco del hook: es el binario recién compilado, no la sonda.

**Lo que esto enseña sobre mis tests:** todo lo que lanza procesos se probaba sin terminal. Los dos tests nuevos son los primeros que corren la CLI en una PTY.

### Auditoría de la §2.0

La auditoría independiente se hizo sobre el estado anterior al arreglo de la TTY. Su crítico C1 era esa regresión, ya cerrada arriba. Esta es la ronda con el resto.

«Visto fallar» dice si vi el test en rojo antes del arreglo. «Mutación» dice si rompí el código a propósito y el test lo notó.

**Guardas que se saltaban.** Las tres eran de texto. Ahora leen el código como tokens y con `syn` (D-24). Los fixtures `bad` nuevos son los casos de la auditoría.

| Hallazgo | Arreglo | Test | Visto fallar |
|---|---|---|---|
| I1 · guarda de binarios | Los binarios del workspace solo se obtienen en `sandbox.rs`; fuera no puede aparecer `cargo_bin…` (función o macro), `CARGO_BIN_EXE_` ni una ruta a `target/`. Regla nueva `machine-env` contra `env::var("PATH")` y `env::vars()` | 6 fixtures `bad` (los 4 casos de la auditoría y dos más) y 2 `good` | **Sí**, por construcción: los 4 casos son los que la auditoría vio pasar. Al activarla marcó dos lugares reales (tests de `hook` y de `app`), que pasaron a su `sandbox.rs` |
| I2 · guarda de dominio | Toda ruta hacia `crate::modules` se resuelve (alias, módulo entero, `super::`, macros) y se aplica también a la puerta propia. Un `domain/` no re-exporta nada de fuera de su dominio | 5 fixtures `bad` nuevos y 1 `good`; dos tests que nombran cada archivo y su línea | **Sí**, por construcción: son los 5 casos de la auditoría |
| I3 · H-001 | Test con tokens: fuera de tres archivos no se puede ni nombrar `Command` ni `CommandBuilder`. El enum de la CLI pasó a `CliCommand`. `H-001` sigue como patrón rápido y su README remite al test | 8 fixtures `bad` (los 5 de la auditoría, el caso simple, uno dentro de una macro) y 2 `good` | **Sí**, por construcción |
| M8 · `pub use domain::{a::A, b::B}` | Desaparece con `syn` | Fixture `good` con una puerta que exporta así y un dominio que usa los dos nombres | No lo comprobé contra la versión vieja |

**Tests que no detectaban regresiones.**

| Hallazgo | Arreglo | Test | Visto fallar / mutación |
|---|---|---|---|
| I4 · punto 19 en la app | `PtyProcess::spawn_in` recibe el entorno del padre | `pty_environment_test.rs`: lanza un fake en una PTY real con marcadores en el entorno del padre y lee lo que el fake cuenta | **Mutación sí**: pasando el entorno crudo al `CommandBuilder`, el fake ve 3 marcadores y el test falla |
| M3 · `kill_group` | Solo test | `what_the_shell_profile_left_running_is_gone_after_the_lookup`: el fake anota el pid de lo que deja atrás | **Mutación sí**: sin `kill_group`, «process … is still alive» |
| M4 · candado | Solo tests | Dos tests que escriben antes un pid largo «de un muerto» | **Mutación sí**: sin escribir el pid propio falla uno; sin `set_len(0)` fallan los dos |
| M6 · recorte final | Rama quitada | `the_largest_context_without_a_list_fits`: todos los campos en su máximo y 40 hermanas | Pasa: demuestra que la rama no hacía falta |
| M7 · ruta del repo | Las comprobaciones que dependen de poder lanzar el `claude` falso encontrado por el PATH solo se exigen si el repo no está en una carpeta temporal. El tiempo y la lista de `doctor` se exigen siempre | Los mismos tests | **Comprobado:** con el repo copiado bajo `/private/tmp`, los 42 tests de la CLI pasan |
| Lanzador con PTY | Solo tests | `claude_gets_the_terminal_in_the_foreground` y `ctrl_c_on_the_terminal_reaches_claude_and_open_reports_it` (sale con 130) | Pasan con el código que había. No hice mutación |
| M5 · punto 10 | Ninguno | Los dos tests de socket se quedan como comprobación de que el hook no se retrasa | **No se puede reproducir en macOS**: `connect` a un socket Unix con la cola llena falla al instante. El socket no bloqueante antes de `connect` protege a Linux y aquí no tiene test que lo distinga |

**Correcciones.**

| Hallazgo | Arreglo | Test | Visto fallar |
|---|---|---|---|
| I5 · marcadores | `AI_AGENT` siempre; `GIT_EDITOR`, `COREPACK_ENABLE_AUTO_PIN` y `NoDefaultCurrentDirectoryInExePath` solo si `CLAUDECODE` venía del padre; `NODE_OPTIONS` solo si apunta a `cmux-claude-node-options`; las carpetas `cmux-cli-shims` fuera del `PATH` (D-22). La viñeta de la §2.1 del plan se sustituyó por una nota | 4 tests de CLI y 6 unitarios | **Sí** dos de los de CLI (los que piden quitar). Los dos que piden conservar pasaban antes, como debe ser |
| I6 · log del IME | Ya no registra contenido: solo el tipo de evento, si había composición, si la tecla es de texto, muerta o con nombre, y longitudes. Solo se enciende con `=1` | Sin test: es un registro de diagnóstico | No aplica |
| D-20 · herramienta que falla | Séptimo evento, `PostToolUseFailure`, documentado, con la misma transición | Caso de uso, unitario de la transición, JSON de `--settings` | **Sí** el caso de uso |
| M1 · salida tardía | La tubería se lee sin bloquear y nada después de que el shell sale; ya no hay hilo ni ventana de 200 ms (D-23) | `output_that_arrives_after_the_shell_exits_is_not_taken_for_claude`, con un shell falso que escribe `/bin/ls` después de salir | **Sí**: `doctor` elegía `/bin/ls` |
| M2 · presupuesto | 4 s compartidos entre las consultas al shell de un `open` o de un `doctor` (D-29) | 2 tests de CLI con un shell que nunca termina; 3 de caso de uso | **Sí** los de CLI: 6,0 s y 9,0 s. Ahora 4 s |
| M9 · `SQLITE_BUSY` | El `busy_timeout` ya estaba puesto antes del `BEGIN IMMEDIATE`, pero era de 250 ms. Crear y migrar esperan ahora hasta 5 s; después vuelve a 250 ms | `opening_waits_for_a_database_that_is_busy_for_a_moment`: otra conexión retiene la base 700 ms | **Sí**: `DatabaseBusy` |
| M10 · tamaños | `pty.rs::spawn` partido en tres funciones; los manejadores de ratón de `view.rs` pasaron a `pointer.rs` (219 y 101 líneas) | Los existentes | No aplica |

**I6 · el punto 20 sigue abierto.** No está arreglado ni lo doy por arreglado. Lo que hay es un registro para que Jhon capture la prueba B.

**Hook, medido otra vez con siete eventos** (mismas condiciones; carga de la máquina: 9,6):

| Evento | media | p50 | p95 | p99 | máx |
|---|---|---|---|---|---|
| `post-tool-use-failure` (nuevo) | 8,4 | 8,4 | **10,4** | 12,2 | 16,8 |
| `post-tool-use` | 7,7 | 7,2 | 9,8 | 10,6 | 11,8 |
| `session-start` | 7,7 | 7,4 | 9,5 | 11,2 | 16,5 |
| `stop` | 8,3 | 7,6 | 11,1 | 18,3 | 89,2 |

El máximo de 89 ms en `stop` fue una sola corrida de 500. No medí de nuevo `prompt-submit` ni `notification`, que no cambiaron.

**Punto 19 de punta a punta, repetido** desde esta sesión de Claude Code dentro de cmux: el programa de cada panel recibe 0 de las variables de la lista ampliada, sobre 43, y su `PATH` no tiene ninguna carpeta de cmux (ni `cmux-cli-shims` ni la del bundle).

**Dos hallazgos míos durante la ronda, corregidos por decisión de la planificadora.**

| Hallazgo | Arreglo | Test | Visto fallar |
|---|---|---|---|
| La carpeta `bin` del bundle de cmux seguía en el `PATH` heredado, con un `open` que tapa al de macOS | Se quita en los mismos lanzamientos que los envoltorios, identificada por su ruta exacta (`…/cmux.app/Contents/Resources/bin`) | 2 unitarios (se quita; lo que solo se le parece se queda) y el test de CLI del `PATH` | **Sí** el unitario y el de CLI |
| Con un `HOME` inexistente el terminal de la app no arrancaba | El PTY recibe siempre un directorio explícito: el de la sesión si existe, si no `HOME` si existe, si no `/` | 3 tests en `pty_environment_test.rs`, con un fake que dice dónde arrancó | **Sí** el del `HOME` inexistente. Los otros dos pasaban antes |

### Auditoría del delta de la §2.0

Última ronda antes del commit. La auditoría dio por cerrada la regresión de la TTY, mató 16 de 16 mutaciones y no encontró nada crítico. Lo que sigue se corrigió con el criterio nuevo de `CLAUDE.md`: las guardas protegen contra errores honestos, no contra la evasión deliberada.

| # | Hallazgo | Arreglo | Test | Visto fallar / mutación |
|---|---|---|---|---|
| 1 | Guarda de binarios | `env!`/`option_env!`, alias de `std::env`, `var(…)` con un `const`, `current_exe` fuera de `sandbox.rs`, y `env_clear` en el mismo bloque que el binario | 5 fixtures `bad` nuevos | **Sí**, por construcción: son los casos que la auditoría vio pasar |
| 2 | Guarda de dominio | `use crate::modules::*;` y `r#` normalizado | 2 fixtures `bad` y un test | **Sí**, por construcción |
| 3 | Procesos | `command_without_session_markers` devuelve `CleanCommand`: sin `envs` ni `env_clear`, y su `env` descarta las claves marcadoras. `r#` normalizado. La guarda revisa todos los archivos de `src/` | Unitario del envoltorio; 3 fixtures `bad` (`r#`, un `.inc`, un archivo que ni es Rust) | El `.envs(vars_os())` ya no compila. Los fixtures, **sí**, por construcción |
| 4 | Ruta envenenada (`/bin/ls` como `claude`) | Respuesta de la sonda entre dos marcas aleatorias; solo se acepta y se recuerda un archivo llamado `claude`; `doctor` muestra lo recordado y `config unset last_claude_path` lo borra | Shell falso que imprime una ruta-señuelo después de responder; unitarios de las marcas y del nombre; 2 casos de uso; CLI de leer, borrar y no poder fijar | **Mutación sí** para las marcas (devolviendo toda la salida, el señuelo aparece). La regla del nombre la cubren tests deterministas. **El test de carrera no reproduce el fallo**: ver abajo |
| 5 | Presupuesto por lo concedido | Se descuenta `min(transcurrido, concedido)` con el reloj del puerto (D-29) | 2 casos de uso y uno con la sonda real: interactivo que falla al instante y respaldo de 1,5 s | **Mutación sí**: con el descuento viejo fallan los tres |
| 6 | `H-002` y el lint | `H-002` revisa todo `src/`. `lint-findings` falla si no hay archivos que revisar y dice cuántos revisó (445). Test que mantiene los lints de `core` iguales a los del workspace salvo `unsafe_code` | 2 tests del lint y el de los lints | El de «nada que revisar», **sí** por construcción (antes decía «no findings») |
| 7 | `NODE_OPTIONS` | Se quita solo la opción de cmux; el resto del valor se queda | 2 unitarios | Escritos con el arreglo |
| 8 | `remember()` | Mejor esfuerzo: un fallo al escribir no aborta el lanzamiento | Caso de uso con un almacén que no admite escrituras | Escrito con el arreglo |
| 9 | Aserciones que se saltaban con el repo bajo `/tmp` | **A medias.** Las raíces temporales son ahora inyectables en la sonda real, y hay 5 tests con la sonda real y los shells falsos que se cumplen **siempre**, esté donde esté el repo. Los tests de la CLI siguen condicionados | `login_shell_probe_test.rs` | Ver abajo |
| 10 | Log del IME | Test de que solo `=1` lo enciende y de que un texto se describe solo por su tamaño. ctrl o cmd con una letra sale como «modified key» | 3 unitarios | Escritos con el cambio |

**Sobre el punto 4, con franqueza.** La auditoría vio la carrera 11 veces de 30. Yo no la reproduzco en esta máquina: ni con diez escritores a retardos distintos, ni quitando a propósito las dos protecciones. `a_path_printed_by_a_job_of_the_profile_never_becomes_claude` se queda como propiedad, pero **no demuestra el arreglo**. Lo que sí lo demuestra es determinista: el señuelo impreso después de responder (marcas) y las rutas que no se llaman `claude` (nombre).

**Sobre el punto 9, lo que no hice y por qué.** Las dos salidas que proponía la planificadora no alcanzan a los tests de la CLI:

- *Copiar los fakes a una carpeta no temporal.* Si el repo está bajo `/tmp`, también lo están `target/` y cualquier carpeta que un test pueda crear. El único sitio escribible fuera de las raíces temporales sería algo como `/Users/Shared`, y no me pareció aceptable que los tests escriban ahí.
- *Inyectar las raíces.* Hecho en la sonda (`LoginShellProbe::with_temporary_roots`), pero el binario `lithemux` de los tests de la CLI usa las reales, y abrir una variable de entorno que desactive ese filtro en producción sería peor que el problema.

Resultado: «sin respaldo» y «se eligió el `claude` del PATH» se comprueban siempre con la sonda real. Lo que solo la CLI puede comprobar (que `open` lo lance dentro de una PTY) sigue exigiéndose solo si el repo no está en una carpeta temporal. **Necesita decisión de la planificadora.**

**Fuera de alcance, documentado en D-24:** `extern crate self as otro`, `concat!` para armar nombres, una macro `#[macro_export]` definida en otra capa, e `include!`/`#[path]` hacia archivos que no son `.rs` (salvo donde revisar todo `src/` era barato).

### Punto 19: comprobación de punta a punta

Lancé el binario de release desde esta sesión, que es Claude Code dentro de cmux, el mismo entorno donde falló:

| | Marcadores (`CLAUDECODE`, `CLAUDE_CODE_*` de la lista, `CMUX_*`) |
|---|---|
| Entorno de la sesión que lanza la app | 30 |
| Entorno del programa en cada panel | **0** de 48 variables |

Lo que **no** comprobé: que el `claude` real guarde el transcript, porque ningún test ni comprobación mía ejecuta el `claude` real. Es el paso A de `F2-prueba-manual.md`.

### Punto 20: qué sé y qué no

- La lógica de composición es correcta para la secuencia `´` + `a`, también como primera composición de una vista recién creada. Dos tests nuevos lo cubren y pasan.
- Para que a Claude le llegue «´a», macOS tiene que haber insertado `´` y `a` por separado. Eso ocurre cuando el sistema cree que el cliente perdió el texto en composición. No sé por qué pasaría solo la primera vez: sin ver los eventos reales solo puedo suponer.
- Descartado leyendo el código: que la app mande el `´` como tecla (no tiene bytes para esa tecla).
- **Lo que queda:** `LITHEMUX_IME_LOG=1` imprime en stderr cada llamada del sistema de entrada, cada tecla y cada cambio de foco, con milisegundos. Está apagado por defecto. Los pasos están en el paso B de `F2-prueba-manual.md`. Con ese registro se puede arreglar en el siguiente bloque.

### Mediciones

**Hook** (hyperfine, 500 corridas tras 20 de calentamiento, binario de release, base real con 3 sesiones; carga de la máquina: 9):

| Evento | media | p50 | p95 | p99 | máx |
|---|---|---|---|---|---|
| `post-tool-use` (nuevo) | 7,8 | 7,6 | **9,6** | 11,5 | 12,0 |
| `session-start` | 7,8 | 7,6 | 9,3 | 11,1 | 14,6 |
| `prompt-submit` | 7,7 | 7,3 | 10,5 | 15,7 | 20,7 |
| `stop` | 7,8 | 7,5 | 9,3 | 11,0 | 20,8 |
| `notification` | 7,6 | 7,3 | 9,4 | 17,1 | 22,7 |

En milisegundos. La meta es p95 < 20 ms. El `post-tool-use` se midió sobre una sesión que no estaba en `waiting`, que es el caso de casi todas sus llamadas.

**Bug 1:** `open` y `doctor` con el shell que deja un proceso atrás pasaron de 30,5 s y 60,3 s a 0,6 s.

### Checklist de cierre

Ejecutado después del último cambio:

| Paso | Resultado |
|---|---|
| `cargo fmt --all --check` | sin diferencias |
| `cargo clippy --workspace --all-targets -- -D warnings` | sin avisos |
| `cargo test --workspace` | **340 pruebas, 0 fallas** (233 al cerrar la Fase 1 + 107) |
| `cargo build --release` | ok; `lithemux-app --check-text` responde `text rendering: ok` |
| `cargo deny check` | advisories, bans, licenses y sources ok |
| `scripts/lint-findings` | 2 reglas (`H-001`, `H-002`), 445 archivos revisados, sin hallazgos |
| Guarda de procesos (test con tokens) | pasa; 8 casos malos marcados |
| Fronteras de arquitectura | pasa, con la regla de dominio sobre `syn` |
| Guardas de fuente única | pasan, con `workspace-binary`, `inherited-env` y `machine-env` |
| Pre-commit | exit 0 |

### Desviaciones del plan

1. **Bug 1.** El plan pedía devolver `None` si algo retenía la salida. Se devuelve la salida cuando el comando ya terminó bien (D-23). Con `None`, un `.zshrc` con un `&` dejaría a lithemux sin encontrar `claude`.
2. **Bug 4.** `PostToolUse` síncrono y no `async`, por el orden de los eventos (D-20). No verifiqué si `async` está documentado.
3. **Punto 15.** La prueba de escritura del directorio de datos quedó en `platform`, no en un caso de uso (D-26).
4. **Punto 12.** La «lista explícita» de tipos de dominio no está escrita a mano: se deriva de las líneas `pub use domain::…` de cada puerta.
5. **Punto 19.** El plan decía «`core::launch`». La lista vive en `core::platform::process`, porque también la usan las sondas de `platform` y `platform` no puede importar módulos.
6. **`unsafe` en un sitio.** `core` pasó de `forbid` a `deny` para permitir una función de cinco líneas (D-27). La planificadora lo autorizó de antemano si no había otra vía; la hay (un trampolín), y está explicado en D-27 por qué no la elegí.
7. **Edité `docs/PLAN.md`** en una línea de la §2.1 (el SessionHost mata al hijo al cerrar), porque el punto 14 lo pedía.

### Cambios de comportamiento que conviene saber

- `lithemux status` y `doctor` ya no muestran «pid 0»: mientras `claude` arranca se ve el pid del propio `lithemux`.
- `lithemux open` falla con un mensaje claro si `lithemux-hook` no está junto al binario.
- El `--settings` que recibe `claude` tiene siete eventos.
- `crates/app` depende ahora de `lithemux-core`.
- Hay un ajuste interno nuevo, `last_claude_path`, que escribe el lanzador (y `doctor`).
- `rustix` usa también su feature `net`. Es el propio crate, sin dependencias nuevas; la guarda de `Cargo.lock` y `cargo deny` pasan.

### Qué quedó fuera

- El arreglo del punto 20.

### Riesgos detectados

1. **La lista de marcadores envejece.** Una versión nueva de Claude Code puede añadir otro. La auditoría ya encontró cuatro que se me habían pasado.
2. **Pid reciclado en el candado:** `status` puede decir «abierta» por error tras una caída. No afecta a `open`.
3. **Permiso negado a mano:** no hay evento documentado, así que la sesión sigue en `waiting` hasta el `Stop`.
4. **`doctor` puede listar menos instalaciones** si el shell tarda: el presupuesto de 4 s es compartido (D-29).
5. **La carrera de la ruta envenenada no la reproduzco aquí,** así que su test no prueba el arreglo; las protecciones sí tienen tests deterministas.
6. **Tests de la CLI con el repo bajo `/tmp`:** parte de sus aserciones sigue condicionada (punto 9 de la auditoría del delta).
7. `launch_use_cases.rs` quedó en 318 líneas, algo por encima de las ~300.

### Lo que necesita a Jhon

Cuatro comprobaciones en `docs/fases/F2-prueba-manual.md`: `doctor` desde una terminal real (0), que `claude` guarde el transcript (A), el registro de la tecla muerta (B) y `waiting` → `working` (C).
