# Decisiones

Decisiones tomadas durante la ejecución, con fecha y porqué. Las aprueba la sesión planificadora; el plan (`docs/PLAN.md`) se actualiza cuando una decisión lo contradice.

| ID | Fecha | Fase | Decisión |
|---|---|---|---|
| D-01 | 2026-11-21 | 0 | El hook sigue el `session_id` que reporta Claude Code |
| D-02 | 2026-11-21 | 0 | `open` guarda la versión de Claude |
| D-03 | 2026-11-21 | 0 | `open` hace spawn + wait, no `exec` |
| D-04 | 2026-11-21 | 0 | Qué notificaciones pasan a `waiting` |
| D-05 | 2026-11-21 | 0 | Orden de resolución de `claude` y filtro de envoltorios |
| D-06 | 2026-11-21 | 0 | `lithemux config` con dos claves validadas |
| D-07 | 2026-11-21 | 0 | Slugs sin guion inicial ni final |
| D-08 | 2026-11-21 | 0 | `cat rm` conserva el tablero |
| D-09 | 2026-11-21 | 0 | Telemetría y clientes HTTP: lista en `deny.toml` + guarda sobre `Cargo.lock` |
| D-10 | 2026-11-21 | 0 | Capas como módulos privados; la puerta re-exporta nombres concretos |
| D-11 | 2026-11-21 | 0 | El tiempo entra a los casos de uso como parámetro |
| D-12 | 2026-11-21 | 0 | Toolchain de Rust instalado por Homebrew |
| D-13 | 2026-11-21 | 0 | Guarda: ningún test corre un shell real ni el `claude` real |

---

## D-01 · El hook sigue el `session_id` que reporta Claude Code

**Contexto.** El plan guardaba `claude_session_id` una sola vez, al crear la sesión. Pero `/clear`, un fork o un `/resume` hacia otra conversación cambian el ID de sesión de Claude. Tras un `/clear`, el siguiente `lithemux open` habría reanudado la conversación anterior al clear.

**Decisión.** Regla: lithemux sigue la conversación que la pestaña tiene abierta. En `session-start`, con cualquier `source` (`startup`, `resume`, `clear`, `compact`, `fork`), si el `session_id` del stdin difiere del guardado, el hook lo actualiza y registra el cambio viejo → nuevo en `hook.log`. Nunca se borra nada. Solo se acepta un ID con forma de UUID.

**Consecuencia.** Dos sesiones de lithemux pueden terminar apuntando a la misma conversación si desde una se hace `/resume` hacia la conversación de la otra. El candado es por sesión de lithemux, así que ese caso no queda protegido (ver riesgos del reporte de la Fase 0).

## D-02 · `open` guarda la versión de Claude

**Contexto.** El plan esperaba la versión en el stdin del hook, pero la documentación de hooks no define ningún campo de versión. `claude_version` habría quedado siempre vacío.

**Decisión.** `lithemux open` ejecuta `<claude> --version` antes de lanzar, con timeout de 2 s, y guarda el primer token de la salida. Si falla o se pasa del tiempo, conserva el valor anterior y no bloquea la apertura. El hook mantiene la regla «si viene, la usa».

## D-03 · `open` hace spawn + wait, no `exec`

**Contexto.** Para que un `flock` sobreviva a un `exec` hay que quitarle `FD_CLOEXEC` al descriptor. Entonces lo heredan también los procesos que `claude` lance; uno que sobreviva al `/exit` mantendría el candado y la sesión quedaría inabrible sin explicación.

**Decisión.** `open` retiene el candado (descriptor con `CLOEXEC`), lanza `claude` como hijo que hereda el TTY, espera y sale con el código del hijo. Mientras espera ignora SIGINT y SIGQUIT y reenvía SIGHUP y SIGTERM. El pid del hijo se escribe en el archivo de candado y aparece en `doctor`, en `status` y en el mensaje «la sesión ya está abierta (pid N)». Cuesta un proceso extra de pocos MB por sesión abierta. En la Fase 2 el candado lo tendrá la app.

## D-04 · Qué notificaciones pasan a `waiting`

**Decisión.** `permission_prompt`, `elicitation_dialog` y `elicitation_url_dialog` → `waiting`. `idle_prompt` **no**.

**Porqué.** `idle_prompt` se dispara unos 60 s después de cualquier turno terminado sin que el usuario escriba. Tratarlo como espera dejaría todas las sesiones en `waiting`, y la regla de hibernación de la Fase 2 exige `idle`.

## D-05 · Orden de resolución de `claude` y filtro de envoltorios

**Contexto.** El plan resolvía con `$SHELL -lc 'command -v claude'`. Medido en la máquina de Jhon el 21-nov-2026: ese shell de login no interactivo no carga `~/.zshrc` y `path_helper` reordena el PATH, así que encuentra el cask de Homebrew (2.1.280, sin autoactualización) en vez del instalador nativo de `~/.local/bin` (2.1.330), que es el que Jhon usa. Además, dentro de cmux el PATH heredado trae un `claude` de `$TMPDIR/cmux-cli-shims/`.

**Decisión.**
1. El ajuste `claude_path`, si está definido y es ejecutable.
2. `$SHELL -lic 'command -v claude'`, con stdin en `/dev/null` y timeout de 3 s; la última línea que sea ruta absoluta y ejecutable.
3. `$SHELL -lc` como respaldo, con advertencia en `hook.log` y en la salida de `open`.

En 2 y 3 se descartan los candidatos bajo `$TMPDIR`, `/tmp`, `/private/tmp`, `/var/folders`, `/private/var/folders` o cuya ruta contenga `shims`. `claude_path` no pasa por ese filtro: es una elección explícita del usuario. Si `claude_path` está definido pero no es ejecutable, se sigue con el paso 2 y se advierte.

**Consecuencia.** Quien instale `claude` con un gestor de versiones basado en `shims` (asdf, mise) tendrá que fijar `claude_path`.

## D-06 · `lithemux config` con dos claves validadas

**Decisión.** `lithemux config get|set|unset <clave>` entra en la Fase 0. Claves: `claude_path` (ruta absoluta) y `context_language` (`es` o `en`). Cualquier otra se rechaza con la lista de las válidas.

**Porqué.** Sin este comando no había forma de fijar `claude_path` ni de pedir el contexto en español.

## D-07 · Slugs sin guion inicial ni final

**Decisión.** Además de `[a-z0-9-]`, un slug no puede empezar ni terminar en guion, y tiene un máximo de 48 caracteres.

**Porqué.** El slug termina en `claude -n <nombre>`; un `-foo` se leería como un flag.

## D-08 · `cat rm` conserva el tablero

**Decisión.** `lithemux cat rm` borra la categoría y sus sesiones de la base de datos, pero deja `board.md` en disco e imprime su ruta. Si la categoría se vuelve a crear, el tablero existente no se sobrescribe.

**Porqué.** El tablero es contenido escrito por la madre; borrarlo no se puede deshacer.

## D-09 · Telemetría y clientes HTTP: lista en `deny.toml` + guarda sobre `Cargo.lock`

**Contexto.** El plan pedía prohibir `sentry*`, `posthog*`, etc., pero `cargo-deny` no admite comodines en nombres de crate.

**Decisión.** `deny.toml` lista los crates concretos conocidos. Los prefijos se hacen cumplir con un test (`crates/core/tests/single_source_guards.rs`) que recorre `Cargo.lock` y falla si aparece un paquete con esos prefijos o un cliente HTTP. Reemplaza al script sobre `cargo tree` que se había acordado al principio: una sola fuente, y corre con `cargo test`.

**Pendiente para la Fase 1.** gpui trae clientes HTTP como dependencia. La guarda hoy mira todo el `Cargo.lock`; al entrar `crates/app` habrá que acotarla a `core`, `cli` y `hook`.

## D-10 · Capas como módulos privados; la puerta re-exporta nombres concretos

**Decisión.** En cada `modules/<dominio>/mod.rs` las capas se declaran privadas (`mod domain; mod application; mod ports; mod infrastructure;`). La puerta re-exporta, por nombre, la API de `application`, los tipos de `domain`, los traits de `ports` y los constructores de `infrastructure`.

**Porqué.** Con las capas privadas, el compilador ya impide que otro módulo o un binario importe el interior de un módulo. Los traits y los constructores tienen que salir por la puerta porque Rust no tiene contenedor de inyección: los binarios cablean a mano (`crates/cli/src/wiring.rs`) y los tests implementan puertos en memoria.

## D-11 · El tiempo entra a los casos de uso como parámetro

**Decisión.** Los casos de uso reciben `now: i64` en vez de depender de un puerto `Clock`. Los binarios pasan `platform::clock::system_clock::now_unix()`.

**Porqué.** Es la forma más simple de que el dominio y la aplicación sean deterministas en los tests, sin un trait más que cablear.

## D-12 · Toolchain de Rust instalado por Homebrew

**Contexto.** La máquina no tenía Rust.

**Decisión.** `brew install rustup hyperfine cargo-deny` y `rustup default stable`. No se tocó `~/.zshrc`. Como `rustup` de Homebrew es keg-only, `cargo` no queda en el PATH: hay que anteponer `/opt/homebrew/opt/rustup/bin`. El pre-commit lo busca ahí por su cuenta.

Versiones: rustup 1.29.3, rustc/cargo 1.100.0 (fijado en `rust-toolchain.toml`), cargo-deny 0.21.0, hyperfine 2.0.0.

## D-13 · Guarda: ningún test corre un shell real ni el `claude` real

**Contexto.** Durante la Fase 0 un test de la CLI usó `/bin/sh` como `SHELL` con un `PATH` mínimo. El `sh` de login reconstruyó el PATH desde `/etc/paths.d`, encontró el `claude` de Homebrew y lo ejecutó. Salió con error por falta de entrada, sin llamar a la API ni tocar nada fuera del directorio temporal, pero es justo lo que la regla «ningún test ejecuta el `claude` real» prohíbe. Se corrigió con `tests/fakes/fake-shell`.

**Decisión.** No va como regla `H-001` del lint de hallazgos, porque no llegó a un commit. Va como guarda de fuente única en `crates/core/tests/single_source_guards.rs` (lógica en `tests/guards/real_programs.rs`), que revisa todo el código de test (`tests/` y las carpetas `test/` de `src/`) y falla si:

| Regla | Qué marca |
|---|---|
| `real-shell` | cualquier mención de `/bin/sh`, `/bin/zsh` o `/bin/bash`, aunque sea como dato |
| `shell-env` | una línea que define o lee `"SHELL"` sin apuntar a un fake |
| `real-claude` | en un archivo que puede lanzar procesos (`Command`, `cargo_bin`, `LoginShellProbe`, `TerminalSessionRunner`, `.spawn(`, `.output(`), una ruta que termina en `/claude` o el programa `"claude"` |

Tiene fixtures `good` y `bad` en `tests/guard-fixtures/real-programs/`: el `bad` reproduce el caso original y cada archivo dispara exactamente la regla que lleva en el nombre.

**Consecuencia.** En los tests con puertos en memoria, las rutas de `claude` siguen permitidas como datos (no pueden lanzar nada); el shell de ejemplo pasó a ser `/fake/zsh`. Los únicos programas que un test puede ejecutar son `tests/fakes/fake-claude` y `tests/fakes/fake-shell`.
