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
| D-14 | 2026-11-21 | 1 | gpui: `rev` fijado, `runtime_shaders` y `font-kit` |
| D-15 | 2026-11-21 | 1 | `cargo deny` con gpui: objetivos, fuente git y licencias |
| D-16 | 2026-11-21 | 1 | Cero red con gpui en el árbol |
| D-17 | 2026-11-21 | 1 | Codificación de teclas del terminal |
| D-18 | 2026-11-21 | 1 | Forma del `TerminalView` |
| D-19 | 2026-11-21 | 1 | La prueba de fronteras reconoce la puerta de una funcionalidad de la app |
| D-20 | 2026-11-21 | 2.0 | Eventos sexto y séptimo del hook: `PostToolUse` y `PostToolUseFailure` |
| D-21 | 2026-11-21 | 2.0 | `SessionStart` con `source=compact` no cambia el estado |
| D-22 | 2026-11-21 | 2.0 | Marcadores de sesión que no se heredan |
| D-23 | 2026-11-21 | 2.0 | Captura de la salida de un comando con plazo |
| D-24 | 2026-11-21 | 2.0 | Guardas más estrictas, sobre tokens y `syn` |
| D-25 | 2026-11-21 | 2.0 | Contexto de `SessionStart`: una línea por texto del usuario y tope duro |
| D-26 | 2026-11-21 | 2.0 | La comprobación del directorio de datos vive en `platform` |
| D-27 | 2026-11-21 | 2.0 | La sonda del shell corre sin terminal: el único `unsafe` del workspace |
| D-28 | 2026-11-21 | 2.0 | El último `claude` bueno se recuerda y va antes que el shell no interactivo |
| D-29 | 2026-11-21 | 2.0 | Un presupuesto de espera compartido para las consultas al shell |
| D-30 | 2026-11-21 | 2.1 | Las piezas del lanzamiento que comparten la CLI y la app |
| D-31 | 2026-11-21 | 2.1 | Forma del `SessionHost` |
| D-32 | 2026-11-21 | 2.1 | Una sola instancia y el socket de eventos |
| D-33 | 2026-11-21 | 2.1 | El host no espera: abrir y cerrar en segundo plano |
| D-34 | 2026-11-21 | 2.1 | Las salidas de la app y el fin del grupo de procesos |

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

**Añadido en la §2.0 (2026-11-21).**

- **Riesgo conocido: doble reanudación si el padre muere por `SIGKILL`.** `SIGKILL` no se puede atrapar ni reenviar. Si `lithemux open` muere así, el kernel suelta el candado y el `claude` hijo queda huérfano y vivo. Un segundo `open` de la misma sesión arranca entonces otro `claude` sobre la misma conversación, que es justo lo que el candado quería impedir. La CLI no lo evita. En la Fase 2 lo cierra el SessionHost, que es dueño del PTY y mata al hijo al cerrar (anotado en la §2.1 del plan).
- El test que antes mataba al padre con `SIGKILL` y celebraba poder reabrir ahora termina al padre con `SIGTERM`, comprueba que el hijo murió y solo entonces reabre: eso es lo que de verdad se garantiza.
- **El candado se toma antes de decidir el plan.** `open` toma el candado y después mira si existe el transcript. Antes era al revés, y dos `open` simultáneos podían concluir los dos que la conversación era nueva.
- **Qué hay en el archivo de candado.** Al tomarlo se escribe el pid del propio `lithemux` (lo que hubiera de un proceso muerto se reemplaza al instante); cuando arranca el hijo, el suyo; al soltarlo, queda vacío. `holder()` ya no toma el `flock` para preguntar: lee el pid y comprueba que el proceso vive, así que consultar el estado nunca hace fallar un `open`. Costo: un pid reciclado tras una caída puede hacer que `status` diga «abierta» por error; `acquire` sigue siendo lo único que decide.

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

**Ampliación (Fase 1, auditoría).** La guarda no veía un shell nombrado sin ruta. Regla nueva `shell-by-name`: en un archivo de test que puede lanzar procesos (ahora también cuentan `ShellCommand` y `TerminalSession`), marca un literal cuyo programa sea `zsh`, `bash`, `sh` o `fish`, con ruta o sin ella, o que los lance con `env` (`/usr/bin/env bash -l`). No marca nada que lleve `fake` en la ruta, ni la palabra dentro de una frase. Además, el soporte de los tests de `crates/app` arranca los fakes con el entorno vacío y un `PATH` que solo tiene la carpeta de fakes, `/usr/bin` y `/bin`.

**Consecuencia.** En los tests con puertos en memoria, las rutas de `claude` siguen permitidas como datos (no pueden lanzar nada); el shell de ejemplo pasó a ser `/fake/zsh`. Los únicos programas que un test puede ejecutar son `tests/fakes/fake-claude` y `tests/fakes/fake-shell`.

## D-14 · gpui: `rev` fijado y `runtime_shaders`

**Fecha:** 2026-11-21 (Fase 1).

**Contexto.** El plan pide gpui como dependencia git fijada a un `rev` del repo de Zed. Esta máquina solo tiene CommandLineTools: `xcrun -f metal` falla, así que los shaders no se pueden compilar al construir.

**Decisión.**

- `rev = d01b134cddf5e4f71964deb2d6f81cdb38fcf1e7`, que es el tag **`v1.29.3`** de Zed (2026-11-20), la última versión estable publicada ese día. Se eligió un tag estable y no la punta de `main` porque es código que ya salió a usuarios y porque un tag se puede volver a encontrar.
- En ese `rev` gpui está partido en dos crates que hay que declarar: `gpui` (0.2.2) y `gpui_platform`, que trae la plataforma (`gpui_platform::application()`).
- `default-features = false` en ambos, para no arrastrar Wayland ni X11.
- Feature **`font-kit`** en `gpui_platform`: es el que le da a macOS su sistema de texto real (ver la corrección de abajo).
- Feature **`runtime_shaders`** en `gpui_platform`: compila los shaders de Metal al arrancar la app, con lo que no hace falta Xcode. Comprobado: la app abre su ventana en esta máquina.
- `alacritty_terminal` 0.26 y `portable-pty` 0.9 desde crates.io, como dice el plan.

**Corrección tras la prueba manual (2026-11-21).** La primera versión dejó fuera `font-kit` a propósito, creyendo que era una dependencia opcional. No lo es: sin ese feature `gpui_macos` usa `NoopTextSystem`, que no conoce ninguna fuente y rasteriza cada glifo a cero bytes. La app pintaba fondos, bordes y cursor, y ningún texto. gpui lo avisa con `log::warn!`, pero la app no instala un registrador y el aviso se perdió. Jhon lo vio en la prueba 1.

- Se activó `font-kit`. Trae `zed-font-kit` (el fork de Zed, MIT/Apache-2.0) desde git, que entra en `allow-git` de `deny.toml` (D-15). No añade crates de red: `Cargo.lock` no cambió, porque cargo ya lo tenía bloqueado.
- `lithemux-app --check-text` comprueba sin ventana que la fuente del terminal existe y que un glifo se rasteriza a píxeles no vacíos. Un test de integración ejecuta el binario con esa opción, y uno unitario comprueba que la sonda rechaza `NoopTextSystem`.
- Los errores de pintado de texto ya no se descartan: el primero de cada proceso sale por stderr. Al arrancar, la app avisa por stderr si el sistema de texto no tiene fuentes.

**Consecuencia.** La primera compilación en limpio tarda unos 5 minutos en debug. `Cargo.lock` pasa a 620 paquetes, porque cargo bloquea también los de Linux, Windows y web; en macOS se compilan 380.

## D-15 · `cargo deny` con gpui: objetivos, fuente git y licencias

**Contexto.** Con gpui en el árbol, `cargo deny check` fallaba en tres frentes: fuentes git, licencias y avisos de crates sin mantenimiento.

**Decisión.**

- **Objetivos.** `deny.toml` evalúa solo `aarch64-apple-darwin` y `x86_64-apple-darwin`, que es lo que lithemux compila hoy. Sin esto fallaba por crates de Linux (`zbus`, `oo7`, `wgpu`) que aquí no se compilan. Cuando llegue Windows (Fase 4) hay que añadir su objetivo y revisar de nuevo.
- **Fuente git.** Se permite una sola, nominal: `https://github.com/zed-industries/zed`. Las otras fuentes git del `Cargo.lock` (`wgpu`, `font-kit`, `oo7`) quedan fuera del grafo de macOS y siguen prohibidas.
- **Licencias.** Se añaden `BSD-2-Clause` (`arrayref`) y `BSD-3-Clause` (`exr`, `lebe`, `tiny-skia`, `tiny-skia-path`), permisivas. **No hay ningún crate GPL ni AGPL en el árbol**; los crates de Zed que se usan (`gpui`, `gpui_platform`, `gpui_macos`, `gpui_apple`, `http_client`, `collections`, `sum_tree`...) son Apache-2.0.
- **Avisos.** Se ignoran tres, uno por uno y con motivo en `deny.toml`. Son avisos de «sin mantenimiento», no vulnerabilidades, de crates que gpui usa en ese `rev`: `ttf-parser` (RUSTSEC-2026-0192), `rustybuzz` (RUSTSEC-2026-0206) y `chrono` (RUSTSEC-2026-0543).

Cada aviso ignorado lleva fecha de revisión en `deny.toml` (2027-02-21) y se revisa antes si cambia el `rev` de gpui.

**Consecuencia.** La lista de `bans` sigue pasando. Tras la auditoría se amplió, en `deny.toml` y en la guarda de `Cargo.lock`, con `tungstenite`, `rustls`, `native-tls` y los crates de telemetría y red de Zed que quedan a una dependencia git de distancia: `telemetry`, `telemetry_events`, `reqwest_client` y `client`.

**Añadido tras la prueba manual.** `allow-git` incluye también `https://github.com/zed-industries/font-kit`, el fork de font-kit que usa `gpui_apple` con el feature `font-kit` (D-14). Es el único crate de esa fuente.

## D-16 · Cero red con gpui en el árbol

**Contexto.** Se esperaba que gpui trajera clientes HTTP de forma transitiva.

**Lo que se encontró.** La guarda de `Cargo.lock` (D-09) **pasa sin cambios**: no entra `reqwest`, `hyper`, `ureq`, `h2` ni ningún SDK de telemetría. En el grafo que se compila para macOS, lo relacionado con red es:

| Crate | Por qué entra | Qué es |
|---|---|---|
| `http`, `http-body`, `bytes`, `url` | `gpui` → `http_client` (crate de Zed) | Solo tipos y un trait de cliente; no trae transporte. lithemux no registra ningún cliente |
| `polling` | `alacritty_terminal` | `kqueue` para su propio bucle de PTY, que no se usa (el PTY es de `portable-pty`) |

`Cargo.lock` nombra además `tokio`, `socket2`, `async-net`, `zbus` y parecidos: pertenecen a otras plataformas o a features opcionales de gpui (por ejemplo `hot-reload`) que no están activas, y no se compilan.

**Decisión.** No hizo falta acotar la guarda ni lista de permitidos. Se añade una comprobación en tiempo de ejecución al cierre de cada fase con interfaz: con la app abierta y dos terminales, `lsof -nP -i -a -p <pid>` no debe listar nada. En la Fase 1 no lista nada.

**Riesgo.** La guarda sigue siendo global sobre `Cargo.lock`. Si una versión futura de gpui activa un cliente HTTP por defecto, saltará, y ahí sí habrá que decidir crate por crate.

## D-17 · Codificación de teclas del terminal

**Decisión.**

- Secuencias clásicas de xterm. El protocolo de teclado de kitty queda apagado (`kitty_keyboard: false`): implementarlo entero no cabe en el spike.
- **`shift+enter` envía `ESC` + `CR`**, lo mismo que `option+enter`, que Claude Code interpreta como línea nueva en cualquier terminal.
- **`option` + tecla de carácter es texto, no prefijo meta.** En el teclado español `option` escribe `@`, `#`, `[`, `]`, `{`, `}` y `~`; tratarlo como meta los rompería. Solo `option` con flechas, borrar y enter tiene secuencia propia.
- El texto nunca pasa por la tabla de teclas: llega por el método de entrada del sistema (`NSTextInputClient`, vía `EntityInputHandler` de gpui). Así funcionan las teclas muertas, el menú de acentos al mantener una vocal y los IME.
- El menú de acentos sustituye una letra que ya se envió al programa: se manda un borrado (`DEL`) por cada carácter reemplazado y luego el texto nuevo.
- `cmd+c` y `cmd+v` son copiar y pegar; `cmd+←`, `cmd+→` y `cmd+borrar` se traducen a inicio de línea, fin de línea y borrar línea.

**Lo que cuesta.**

- **`option` como texto anula los atajos meta de Claude Code**, como `option+p` y `option+t`: la tecla escribe el carácter de la distribución (`π`, `†`) en vez de mandar `ESC` + letra. Queda como ajuste para la Fase 2 («option como meta»), igual que en Terminal.app e iTerm2.
- **`shift+enter` como `ESC` + `CR` sale del modo inserción en vim** y en cualquier programa donde `ESC` signifique algo: ahí se comporta como `esc` seguido de enter. La salida limpia es el protocolo de teclado de kitty, que queda fuera del spike.
- **El `DEL` del menú de acentos borra un carácter real si el programa no insertó la letra.** lithemux no sabe qué hizo el programa con la `a`; si la ignoró o la usó como comando (por ejemplo en el modo normal de vim), el borrado se lleva lo que hubiera antes del cursor.
- En el pegado entre corchetes se quitan del texto `ESC`, el CSI de un byte (U+009B) y `ctrl+c`. Un texto que los contenga a propósito llega sin ellos.

## D-18 · Forma del `TerminalView`

**Decisión.** `crates/app/src/terminal_view/`, con la vista, el estado y la entrada en archivos separados:

| Archivo | Responsabilidad |
|---|---|
| `view.rs` | entidad de gpui: foco, teclado, ratón, rueda, portapapeles |
| `ime.rs` | la vista como cliente de entrada de texto |
| `paint.rs` | dibuja la grilla con primitivas de gpui |
| `session.rs` | la sesión: lo que la vista le pide a un terminal en marcha |
| `emulator.rs` | grilla (`alacritty_terminal::Term`), analizador de secuencias y respuestas al programa |
| `pump.rs` | hilos que mueven bytes entre el PTY y el emulador |
| `grid.rs` | foto de la grilla ya resuelta a colores y agrupada en tramos; sin tipos de gpui |
| `pty.rs` | proceso y pseudo-terminal (`portable-pty`) |
| `keys.rs`, `paste.rs`, `composition.rs` | teclas, pegado y composición a bytes; puros |
| `palette.rs` | colores del terminal a RGB |

- La vista recibe una sesión ya arrancada, así que crearla no puede fallar y la sesión se prueba sin gpui.
- **El programa siempre arranca en un directorio que existe** (añadido en la §2.0): el de la sesión si existe; si no, `HOME` si existe; si no, `/`. `portable-pty`, dejado a su aire, usa `HOME` sin comprobarlo, y un `HOME` inexistente impedía arrancar el terminal con un error que nombraba al programa.
- Un hilo lee el PTY y alimenta el emulador; otro escribe. La vista nunca bloquea en E/S. El candado del emulador es el `FairMutex` de alacritty, para que el hilo lector no deje sin turno al de la interfaz cuando hay mucha salida.
- No hay repintado periódico: la vista solo se redibuja cuando llega salida o entrada. El cursor no parpadea.
- No se leyó el `terminal_view` ni el crate `terminal` de Zed (GPL); solo los ejemplos y el código de `gpui`, que son Apache-2.0.

**Cambios tras la auditoría de la Fase 1.**

- **Actualización sincronizada sin cerrar.** Un tercer hilo espera el plazo (150 ms) y muestra lo retenido si el programa se queda callado o muere. La auditoría proponía leer el PTY con `poll`; no se hizo así porque `poll` sobre el descriptor de `portable-pty` exige `unsafe`, y el workspace lo prohíbe. El efecto es el mismo y el analizador pasó a vivir bajo el mismo candado que la grilla.
- **Cierre.** Al soltar la sesión se manda `SIGHUP` al grupo de procesos del hijo y, si a los 200 ms no terminó, `SIGKILL` al grupo. Si el hijo ya fue recogido no se manda nada, porque su pid puede ser de otro proceso. Queda una ventana de microsegundos entre que `wait` devuelve y se anota la marca.
- **Historial:** 5.000 líneas por terminal (antes 10.000, el valor por defecto). Será configurable en la Fase 2.
- **Mínimo de 2 columnas** en `TerminalSession`: con 1 columna y un carácter ancho, la reacomodación de alacritty no termina y consume memoria sin límite.
- **Cursor oculto:** la foto guarda siempre la posición y aparte si se ve, para que el menú de acentos y el texto en composición salgan en su sitio.
- **Consultas de color** (OSC 4, 10, 11): se responden con el color que el programa haya fijado y, si no fijó ninguno, con la paleta.
- La sesión expone `paste`, `press` y `report_focus`, que aplican el modo que pidió el programa, para que se puedan probar de punta a punta sin gpui.

## D-19 · La prueba de fronteras reconoce la puerta de una funcionalidad de la app

**Contexto.** La regla `no-barrels` solo dejaba re-exportar nombres en `modules/<módulo>/mod.rs`. CLAUDE.md dice que la app se organiza por funcionalidad en `crates/app/src/<feature>/`, y el `mod.rs` de `terminal_view` necesita exponer `TerminalView`, `TerminalSession` y sus tipos.

**Decisión.** `app/src/<feature>/mod.rs` (un solo nivel bajo `app/src/`) cuenta como puerta y puede re-exportar nombres concretos. Los `mod.rs` más adentro siguen pudiendo solo declarar submódulos, y los `pub use x::*` siguen prohibidos en todas partes. Fixtures nuevos: `good/app/src/terminal_view/` y `bad/app/src/terminal_view/no_barrels_inner/`.

**Corrección tras la auditoría.** La primera versión buscaba `app/src/` en cualquier parte de la ruta, así que aceptaba `crates/tauri-app/src/...` y un `app/src/` anidado dentro de otro crate. Ahora la ruta, relativa a `crates/`, tiene que empezar por `app/src/`. Hay un fixture `bad` para cada caso y un test que los nombra.

## D-20 · Sexto evento del hook: `PostToolUse`

**Fecha:** 2026-11-21 (§2.0, bug 4).

**Contexto.** Una sesión que pedía un permiso pasaba a `waiting` y se quedaba ahí hasta el `Stop`, aunque el usuario ya hubiera aprobado y Claude siguiera trabajando.

**Decisión.** Se agrega `PostToolUse` → `lithemux-hook post-tool-use`, con timeout de 2 s. Tiene una sola transición: `waiting` → `working`. Con cualquier otro estado no escribe nada, ni siquiera la actividad.

- **Síncrono, no `async`.** Con `async` el hook de una herramienta podría terminar después de una notificación posterior y pisar un `waiting` legítimo. Síncrono, el orden de los eventos es el de Claude Code. No comprobé si `async` está documentado en la v2.1.330, porque no se iba a usar.
- **Costo medido** (hyperfine, 500 corridas, binario de release, carga de 9 en la máquina): `post-tool-use` p95 = 9,6 ms; los otros cinco eventos, entre 9,3 y 10,5 ms. La meta es 20 ms. Se paga en cada herramienta que Claude ejecuta.
- **Límites.**
  - Si el usuario **niega** el permiso a mano no hay ningún evento: la documentación dice que `PermissionDenied` solo se dispara en modo automático. La sesión sigue en `waiting` hasta el `Stop`.
  - Con herramientas en paralelo, una que termina puede pasar la sesión a `working` mientras otra todavía espera permiso.

**Añadido tras la auditoría: séptimo evento, `PostToolUseFailure`.** `PostToolUse` solo se dispara si la herramienta termina bien. Si falla después de aprobada, la sesión se quedaba en `waiting`. `PostToolUseFailure` está documentado en code.claude.com/docs/en/hooks («Runs when a tool that started executing fails») y entra con la misma transición y el mismo timeout. La página no da versión mínima. De paso verifiqué que `async` sí está documentado para hooks de tipo `command`; se mantiene síncrono por lo dicho arriba.

## D-21 · `SessionStart` con `source=compact` no cambia el estado

**Fecha:** 2026-11-21 (§2.0, bug 3).

**Decisión.** Una compactación ocurre en mitad de un turno. Con `compact` el hook solo actualiza `last_activity_at`, sigue la conversación si cambió el id y devuelve el contexto. No toca `state`, así que ni `working` ni `waiting` se pierden.

## D-22 · Marcadores de sesión que no se heredan

**Fecha:** 2026-11-21 (§2.0, punto 19).

**Contexto.** La app se lanzó desde una sesión de Claude Code dentro de cmux. El `claude` de adentro heredó `CLAUDE_CODE_CHILD_SESSION`, se creyó anidado y no guardó el transcript. La documentación lo describe: un `claude` interactivo iniciado así queda excluido de `--resume` y del historial.

**Decisión.** Una sola lista, en `core::platform::process::session_markers`, que usan el lanzador de `core`, sus sondas y la app. Todo proceso que lithemux arranca hereda el entorno menos estos nombres:

| Origen | Variables |
|---|---|
| Documentadas como «las pone Claude Code» (code.claude.com/docs/en/env-vars) | `CLAUDECODE`, `CLAUDE_CODE_CHILD_SESSION`, `CLAUDE_CODE_SESSION_ID`, `CLAUDE_CODE_BRIDGE_SESSION_ID`, `CLAUDE_CODE_MESSAGING_SOCKET`, `CLAUDE_CODE_MESSAGING_TOKEN`, `CLAUDE_PID`, `CLAUDE_EFFORT`, `CLAUDE_JOB_DIR` |
| Vistas en el entorno real de una sesión de la v2.1.330, sin documentar | `CLAUDE_CODE_ENTRYPOINT`, `CLAUDE_CODE_EXECPATH`, `CLAUDE_CODE_SESSION_ATTENDED`, `CLAUDE_CODE_CHILD_NOTES_DIR` |
| Nombrada en el plan; no la vi en esta máquina ni en la documentación | `CLAUDE_CODE_SSE_PORT` |
| Vista en el entorno real; la encontró la auditoría | `AI_AGENT` |
| cmux | todo lo que empiece por `CMUX_` (vi 28 variables distintas en una pestaña) |

**Añadido tras la auditoría: lo que no es un simple nombre.** La lista dejó de ser un filtro de nombres y pasó a ser una limpieza del entorno (`without_session_markers`), siempre en el mismo archivo:

| Caso | Qué se hace |
|---|---|
| `GIT_EDITOR`, `COREPACK_ENABLE_AUTO_PIN`, `NoDefaultCurrentDirectoryInExePath` | Se quitan **solo si `CLAUDECODE` venía en el entorno del padre**. Claude Code las pisa para sus herramientas; fuera de él son del usuario. Con `GIT_EDITOR=true` heredado, un `git commit` sin `-m` en una sesión shell aborta |
| `NODE_OPTIONS` | Se le quita solo la opción que carga el script de cmux (`--require=<ruta>`, `--require <ruta>` o `-r <ruta>` con `cmux-claude-node-options`). El resto del valor se conserva; si no queda nada, se quita la variable |
| `PATH` | Se le quitan las entradas que tengan una carpeta llamada `cmux-cli-shims` y la carpeta `bin` del bundle de cmux, identificada por su ruta (`…/cmux.app/Contents/Resources/bin`) y no por lo que contiene. Esa carpeta trae un `open` que taparía al de macOS. El resto conserva su orden |

Lo de cmux estaba previsto para la §2.1 y se adelantó para dejar la lista completa en un solo cambio.

**Lo que no se quita.**

- La configuración que el usuario fija a propósito: `CLAUDE_CONFIG_DIR`, `CLAUDE_CODE_MAX_OUTPUT_TOKENS`, `CLAUDE_CODE_SUBAGENT_MODEL`, `ANTHROPIC_*` y el resto. La lista es de nombres exactos, no del prefijo `CLAUDE_`, justamente para no llevarse estas.
- `CLAUDE_ENV_FILE`, `CLAUDE_CODE_REMOTE*`, `TRACEPARENT` y `CLAUDE_CODE_SAFE_MODE`: la documentación las describe como configuración o como propias de sesiones en la nube, no como marca de anidamiento.
- `GHOSTTY_*` y `TERMINFO`, que deja el terminal desde el que se lanzó la app. No rompen el transcript. `TERM`, `COLORTERM` y `TERM_PROGRAM` sí los fija la app. Queda anotado como posible ajuste si la integración de shell de Ghostty se activa donde no debe.

**Cómo se sostiene.**

- **El constructor filtrado devuelve un envoltorio** (`CleanCommand`), no un `Command`. No tiene `envs` ni `env_clear`, y su `env(clave, valor)` descarta en silencio las claves marcadoras. Antes, `command_without_session_markers(x).envs(std::env::vars_os())` devolvía todos los marcadores sin nombrar nada prohibido.

- La documentación no ofrece una forma de limpiar el marcador heredado; su remedio es `CLAUDE_CODE_FORCE_SESSION_PERSISTENCE=1`. No se usa: quitar la causa cubre también las demás marcas (mensajería, pid, id de sesión) y no depende de una variable más.
- Regla `H-001` del lint de hallazgos: en `crates/*/src` solo dos archivos pueden construir un proceso, y los dos aplican el filtro.
- Una versión nueva de Claude Code puede añadir marcadores. La lista se revisa al subir de versión.

## D-23 · Captura de la salida de un comando con plazo

**Fecha:** 2026-11-21 (§2.0, bug 1).

**Decisión.** `stdout_within` lanza el comando en su propia sesión y grupo de procesos, lee su salida sin bloquear y mata el grupo entero cuando el comando termina o cuando vence el plazo. (Estado final; las dos correcciones de abajo cuentan cómo se llegó.)

**Desviación del plan.** El plan pedía devolver `None` siempre que algo retuviera la salida. Se devuelve `None` solo si el comando no termina a tiempo o falla. Si el comando **ya terminó bien**, su salida está completa aunque un proceso que dejó en segundo plano siga con la tubería abierta. Motivo: un `.zshrc` que arranca algo con `&` es común, y con `None` `lithemux` no encontraría `claude` nunca en esas máquinas.

**Corrección del mismo día (TTY).** La primera versión ponía al comando en su propio grupo con `process_group(0)`. Con una terminal de verdad eso lo rompía todo: ver D-27.

**Corrección tras la auditoría (M1).** La primera versión leía en un hilo y, al salir el comando, esperaba 200 ms por si la tubería cerraba. En esa ventana un trabajo en segundo plano del perfil pudo escribir `/bin/ls`, y `lithemux` lo tomó por `claude`. Ahora no hay hilo ni espera: la tubería se lee sin bloquear mientras el comando corre y una última vez cuando sale. Lo que llegue después no se lee. Con eso desaparece también el hilo lector que quedaba bloqueado.

**Corrección tras la auditoría del delta: la respuesta va entre marcas.** Leer solo hasta que el shell sale no bastaba: un trabajo del perfil podía escribir una ruta justo mientras el shell respondía, y la auditoría vio a `doctor` elegir `/bin/ls` 11 veces de 30. Ahora la sonda envuelve el script entre dos líneas con una marca aleatoria (`printf '\n<marca>\n'; <script>; printf '\n<marca>\n'`) y solo cuenta lo que hay entre ellas. Lo que el perfil imprime antes, lo que un gancho de salida imprime después y casi todo lo que escribe un trabajo quedan fuera. Queda una ventana de microsegundos, la de la ejecución del propio script; para eso está la regla del nombre de D-28.

**Límite.** Un shell con control de trabajos pone cada trabajo en segundo plano en su propio grupo, así que `killpg` no lo alcanza. Ese proceso sigue vivo: es del usuario, y su perfil lo arrancó a propósito.

## D-24 · Guardas más estrictas

**Fecha:** 2026-11-21 (§2.0, puntos 11 y 12).

**Reescritas tras la auditoría.** Las tres guardas nuevas del bloque eran de texto y la auditoría las esquivó con casos realistas: una macro, un alias, un `super::super::`, un espacio. Por decisión de la planificadora leen ahora el código de verdad:

- **Tokens** (`proc-macro2`, en `tests/guards/tokens.rs`): el código como lista plana de identificadores, literales y `::`. Ni los espacios, ni los comentarios, ni el contenido de un string engañan, y se ve el interior de las macros, que `syn` no analiza.
- **`syn`** donde hay que entender los `use`: alias, grupos anidados, globs.

`syn` y `proc-macro2` son dependencias de desarrollo de `core`. Ya estaban en `Cargo.lock` (las traen las macros de otros crates), así que la guarda de `Cargo.lock` y `cargo deny` no cambian.

| Guarda | Regla ahora |
|---|---|
| `real_programs` · `workspace-binary` | Los binarios del workspace solo se obtienen en archivos llamados `sandbox.rs` (uno por crate de tests). Fuera de ellos no puede aparecer un identificador que empiece por `cargo_bin` (función o macro), ni un literal con `CARGO_BIN_EXE_`, ni una ruta a `target/debug` o `target/release` |
| `real_programs` · `inherited-env` | Dentro de un `sandbox.rs`, cada binario necesita su propio `env_clear()` |
| `real_programs` · `machine-env` | Ningún test lee el entorno real de la máquina con `env::vars()` ni `var("PATH")`, `var("HOME")` o `var("TMPDIR")`: es la forma de devolverle la máquina a un entorno limpio |
| `arch_boundaries` · `domain-purity` | En un `domain/`, toda ruta hacia `crate::modules`, escrita como sea (import directo, del módulo entero, con alias, con `super::`, dentro de una macro), tiene que acabar en un tipo que la puerta exponga con `pub use domain::…`. Vale también para la puerta del **propio** módulo. Un archivo de `domain/` no puede re-exportar nada de fuera de su propio dominio |
| Procesos (`H-001`) | Fuera de tres archivos, el código de producción no puede ni nombrar `Command` ni `CommandBuilder`. Para que la regla sea así de simple, el enum de la CLI pasó a llamarse `CliCommand` |

Los fixtures `bad` nuevos son los casos de la auditoría, uno por archivo. La regla de dominio ya no pierde los `pub use domain::{a::A, b::B}` anidados, que antes solo registraban el último.

**Ajustes tras la auditoría del delta, con el criterio de `CLAUDE.md`** (las guardas protegen contra errores honestos, no contra la evasión deliberada):

- `machine-env` marca también `env!` y `option_env!` de una variable que no sea de cargo ni de lithemux (el `PATH` de quien compiló es un error natural), cualquier `vars()` sea cual sea el alias de `std::env`, y `var(…)` con cualquier argumento que no sea un literal `LITHEMUX_…` o `CARGO_…`.
- `workspace-binary` marca `current_exe` fuera de `sandbox.rs`.
- `inherited-env`: el `env_clear` tiene que estar en el mismo bloque (el cuerpo de la función) que obtiene el binario, después de él. Uno por binario.
- La regla de dominio marca `use crate::modules::*;`.
- Todas normalizan `r#nombre`. La de procesos y `H-002` revisan todos los archivos bajo `src/`, no solo los `.rs`; lo que no es Rust se revisa por palabras.

**Fuera de alcance, a propósito.** Solo existen para esquivar, y nadie los escribe de buena fe:

- `extern crate self as otro` para llegar a `crate::modules` por otro nombre;
- `concat!` para armar `CARGO_BIN_EXE_` o un nombre de variable por trozos;
- una macro `#[macro_export]` definida en otra capa que construya la ruta prohibida;
- `include!` o `#[path]` hacia un archivo que no sea `.rs`, salvo en la guarda de procesos y en `H-002`, donde revisar todo `src/` era barato.

**Límites que quedan.**

- La regla de dominio resuelve alias dentro de un archivo. Un nombre que llegue re-exportado desde *otro* archivo que no sea una puerta no se sigue.
- `machine-env` mira tres variables concretas y `vars()`. Otra variable real leída a mano no se detecta.

## D-25 · Contexto de `SessionStart`: texto del usuario en una línea y tope duro

**Fecha:** 2026-11-21 (§2.0, bug 8).

**Decisión.** Todo texto que viene del usuario se aplana a una línea antes de entrar al contexto: el nombre de la categoría (máximo 80 caracteres), la carpeta raíz y la ruta del tablero (240) y el rol (160, como antes). **Corrección tras la auditoría (M6).** La primera versión añadía un corte final como red. Con los topes por campo esa rama era inalcanzable, así que se quitó. En su lugar hay un test con todos los campos en su máximo y más sesiones de las que caben, que demuestra que el texto entra.

## D-26 · La comprobación del directorio de datos vive en `platform`

**Fecha:** 2026-11-21 (§2.0, punto 15).

**Desviación del plan.** El plan pedía mover la E/S de `doctor` a casos de uso de `core`. La prueba en seco del hook es ahora el caso de uso `inspect_hook` (módulo `launch`, con su puerto `HookProbe`), y la resolución de carpetas es `locate_directory` (módulo `category`, puerto `Directories`). La prueba de escritura del directorio de datos quedó como `Home::check_writable` en `platform/paths`: no tiene regla de negocio ni pertenece a ninguno de los seis módulos, y crear un módulo para una sola comprobación no cuenta nada del dominio. La CLI ya no hace E/S propia en ninguno de los tres casos.

## D-27 · La sonda del shell corre sin terminal: el único `unsafe` del workspace

**Fecha:** 2026-11-21 (§2.0, regresión del bug 1 encontrada por la planificadora).

**Qué pasó.** El arreglo del bug 1 puso a la sonda `$SHELL -lic` en su propio grupo de procesos para poder matarla con lo que dejara atrás. Sin terminal funciona. Con una terminal de verdad, que es el uso real, ese grupo queda en segundo plano: el shell interactivo intenta tomar la terminal, el kernel lo detiene y la sonda agota sus 3 s. `lithemux` caía al shell no interactivo y abría `/opt/homebrew/bin/claude` 2.1.280 en vez de `~/.local/bin/claude` 2.1.330. Ningún test lo vio porque todos corrían sin terminal. **Fue un error mío.**

**Decisión.** La sonda arranca en una sesión nueva (`setsid`), sin terminal que la controle. No tiene nada que tomar, así que no se detiene, y sigue liderando su grupo, de modo que `killpg` funciona igual. Solo afecta a los comandos cortos que se capturan (`stdout_within`); el `claude` interactivo sigue heredando la terminal.

**El `unsafe`.** En Rust estable, hacer algo en el hijo entre `fork` y `exec` solo se puede con `CommandExt::pre_exec`, que es `unsafe`. `CommandExt::setsid` existe pero es inestable (rust-lang/rust#105376).

- Vive en una función de cinco líneas, `detach_from_terminal`, en `crates/core/src/platform/process/detached.rs`. El cierre hace una sola llamada al sistema, `setsid`, que es segura entre `fork` y `exec`; no reserva memoria ni toma candados.
- `unsafe_code = "forbid"` no admite excepciones, así que `core` pasó a `deny` con un `#[allow(unsafe_code)]` en esa función. `cli`, `hook` y `app` siguen en `forbid`.
- Regla `H-002`: la palabra `unsafe` no puede aparecer en `crates/*/src` fuera de ese archivo.
- **Cargo no deja heredar los lints del workspace y sobrescribir uno solo:** `[lints] workspace = true` es todo o nada. Por eso `core` repite la tabla. Para que la copia no se desvíe hay un test, `core_has_the_lints_of_the_workspace_except_for_unsafe_code`, que exige que los lints de Clippy de `core` sean exactamente los del workspace, que `unsafe_code` sea la única diferencia y que `cli`, `hook` y `app` sigan heredando.

**Alternativas descartadas.**

- *Un trampolín sin `unsafe`* (relanzar el propio binario, que llama a `rustix::process::setsid()` y hace `exec`). No tiene `unsafe`, pero la sonda vive en `core`, que es una biblioteca: dependería de qué binario la aloja, y bajo los tests ese binario es el arnés de pruebas. Más piezas y más formas de fallar que cinco líneas.
- *Ignorar `SIGTTOU`/`SIGTTIN`.* No sirve: un shell interactivo se detiene a sí mismo a propósito hasta estar en primer plano.
- *Darle a la sonda una PTY propia.* Mezcla stdout con stderr y añade secuencias de terminal a una salida que hay que analizar.
- *Depender de `perl` o `script` del sistema* para hacer el `setsid`. Frágil.

**Tests.** `on_a_terminal_test.rs` corre `doctor` y `open` dentro de una PTY real, con un shell falso (`fake-shell-interactive`) que cambia los ajustes de su terminal, como hace un shell interactivo. Antes del arreglo tardaban 9,0 s y 6,0 s; ahora menos de 1 s.

## D-28 · El último `claude` bueno se recuerda y va antes que el shell no interactivo

**Fecha:** 2026-11-21 (§2.0). Amplía D-05.

**Contexto.** Cuando la sonda interactiva fallaba, el respaldo (`$SHELL -lc`) encontraba un `claude` más viejo y lo abría con solo un aviso en el log. En la máquina de Jhon eso es pasar de la 2.1.330 a la 2.1.280.

**Decisión.** Orden nuevo:

1. `claude_path`, si es ejecutable.
2. El shell interactivo. Cuando acierta, la ruta se guarda en el ajuste interno `last_claude_path` (solo si cambió).
3. **Nuevo:** si el shell interactivo no responde, la ruta recordada, siempre que siga siendo ejecutable y no sea un envoltorio. Con aviso.
4. El shell no interactivo, con aviso.

- Se recuerda la **ruta**, no la versión: el instalador nativo se actualiza en su sitio, así que la misma ruta es siempre la versión al día. Comparar versiones obligaría a ejecutar `--version` en cada candidato durante el lanzamiento.
- `last_claude_path` no se puede cambiar con `lithemux config`: lo escribe el lanzador. Un `claude_path` fijado por el usuario no lo modifica.
- Efecto lateral: `doctor` también lo escribe, porque resuelve `claude` igual que un lanzamiento.
- Queda sin cubrir la primera vez: si la sonda interactiva falla y no hay nada recordado, se cae al paso 4 con su aviso.

**Añadido tras la auditoría del delta.**

- **Solo se acepta y se recuerda un archivo que se llame `claude`.** Vale para lo que encuentre cualquier shell y para la ruta recordada. No vale para `claude_path`, que es elección explícita del usuario. Una ruta envenenada como `/bin/ls` ya no puede elegirse ni quedar guardada.
- **`doctor` muestra lo recordado** en la línea `claude remembered`, y se borra con `lithemux config unset last_claude_path`. `config get` lo lee; `config set` lo rechaza: lo escribe el lanzador.
- **Recordar es de mejor esfuerzo.** Si la escritura falla (la base ocupada), el lanzamiento sigue.

## D-29 · Un presupuesto de espera compartido para las consultas al shell

**Fecha:** 2026-11-21 (auditoría de la §2.0, M2).

**Contexto.** Cada consulta al shell del usuario tenía su propio plazo de 3 s. Con un perfil que se cuelga, `open` tardaba 6 s (dos consultas) y `doctor` 9 s (tres).

**Decisión.** Un lanzamiento, o un `doctor`, dispone de **4 s en total** para esperar al shell (`ShellBudget`). Cada consulta sigue teniendo su tope de 3 s, pero nunca más de lo que queda. Sin saldo, el shell ya no se consulta.

**Corrección tras la auditoría del delta: se descuenta el tiempo transcurrido.** La primera versión descontaba lo concedido cuando no había respuesta, sin mirar el reloj. Un shell que fallaba en milisegundos le quitaba 3 s al respaldo, y `open` llegó a decir «could not find claude» con un perfil de 1,5 s. Además, una consulta lenta que sí respondía no descontaba nada, y `doctor` tardó 5,81 s.

- Ahora cada consulta, con respuesta o sin ella, descuenta `min(transcurrido, concedido)`.
- El reloj es del puerto (`ShellProbe::now`): el adaptador real usa el del sistema y el de los tests lo inventa, así que los casos de uso siguen sin depender del tiempo real.
- Con un perfil colgado: la consulta interactiva gasta 3 s y el respaldo recibe 1 s. `open` y `doctor` tardan 4 s.
- No cubre `claude --version`, que no pasa por el perfil del usuario y conserva sus plazos.

## D-30 · Las piezas del lanzamiento que comparten la CLI y la app

**Fecha:** 2026-11-21 (§2.1).

**Decisión.** `core::launch` expone el lanzamiento en piezas, y las dos lo usan igual:

- **`prepare_launch`** hace todo menos arrancar el proceso: busca la sesión, **toma el candado** y después decide el comando (`--resume` o `--session-id`, mirando el transcript ya con el candado puesto). Devuelve un `PreparedLaunch` con el plan y el candado dentro.
- **`PreparedLaunch`** suelta el candado cuando se destruye. Quien arranca el proceso lo conserva mientras el proceso viva, y le anota el pid con `started(pid)`.
- **El comando limpio.** La CLI usa `CleanCommand`; la app, que necesita un PTY, usa `child_env`, que limpia el entorno con la misma función (`without_session_markers`). Es una sola lista (D-22).
- **`open_session`** queda como lo que es: `prepare_launch` más el supervisor de la CLI (spawn y espera). El `runner` salió de `LaunchPorts` y es un parámetro suyo, porque la app no tiene ninguno.

La sesión shell se prepara sin candado, como antes.

**Comprobado.** Los tests de `open` no cambiaron de resultado: los de PTY, los de señales y los de D-03 siguen en verde.

**Lanzada desde el Dock** (lo normal desde la §2.2), la app recibe un entorno mínimo: `PATH=/usr/bin:/bin:/usr/sbin:/sbin`. Comprobado con `env -i` y ese entorno: la resolución de `claude` no depende de él, porque pregunta al shell de login interactivo, y encontró `~/.local/bin/claude` 2.1.330; un `zsh -lic` desde ahí reconstruye el `PATH` completo del usuario, así que las sesiones shell (`$SHELL -l` en un PTY) también. Sin `SHELL` en el entorno se usa `/bin/zsh`. No hace falta compensar nada.

## D-31 · Forma del `SessionHost`

**Fecha:** 2026-11-21 (§2.1).

**Decisión.** `crates/app/src/session_host/` no depende de gpui y se prueba sin ventana.

| Archivo | Qué contiene |
|---|---|
| `host.rs` | `SessionHost`: abre, cierra, sabe el estado y lee su buzón |
| `hosted.rs` | una sesión abierta aquí: su terminal y su `PreparedLaunch` |
| `mailbox.rs` | lo que los hilos de fondo le dejan dicho al host |
| `event_socket.rs` | `app.sock` |
| `instance_lock.rs` | `app.lock` |
| `config.rs`, `error.rs` | de dónde saca su entorno; sus errores |

- **Un solo hilo con buzón.** El host vive en el hilo de la interfaz. Los hilos de fondo (el del socket y el que espera a cada hijo) no tocan nada suyo: dejan un mensaje en un canal y avisan por otro. La interfaz espera ese aviso y llama a `process_pending()`. Así no hay estado compartido ni candados propios, y la base de datos se usa desde una sola conexión.
- **El terminal es compartido, no cedido.** El host guarda cada `TerminalSession` en un `Rc`; la vista recibe otro. Una sesión sin vista sigue viva y su emulador sigue leyendo; nadie consume sus avisos de repintado, así que no cuesta pintarla. `resize` pasó a tomar `&self`.
- **El estado no viaja.** Un mensaje del socket solo nombra la sesión. El host relee esa fila de la base de datos.
- **Fin del hijo.** El hilo que espera al hijo avisa. El host suelta el candado y, si es una sesión de Claude, escribe `ended` (respetando `hibernated`): normalmente ya lo hizo el hook `SessionEnd`, y esto cubre a un `claude` que murió sin avisar. El terminal se conserva. Volver a abrir pasa otra vez por `prepare_launch`.
- **Cierre.** `shut_down()`, que también corre al destruir el host, cuelga a todos los grupos, espera hasta 1,5 s a que desaparezcan (SIGHUP, 200 ms de gracia y SIGKILL, como en la Fase 1) y solo entonces suelta los candados.
- **«Abierta en otra terminal».** `open` devuelve `OpenElsewhere { pid }` y no lanza nada.

**Cambios tras la auditoría de la §2.1.** El host ya no espera nada salvo al salir (D-33), el candado lo suelta el hilo que espera al hijo, y las señales cierran en orden (D-34). Lo que sigue describe el estado final.

**Límites conocidos.**

- **`SIGKILL` a la app.** No se puede atrapar. El kernel cuelga a los hijos al cerrarse sus PTY y los candados se sueltan solos, pero un hijo que **ignore** SIGHUP sobrevive, y los estados de la base de datos quedan como estaban. Reconciliar al arrancar es de la §2.5.
- **Procesos que se van a su propia sesión** (`setsid`): salen del grupo y quedan fuera de alcance.
- **Reabrir justo después de cerrar.** La sesión sigue con candado hasta que su programa desaparece (hasta 200 ms, más si hay que matarlo). Si se pide abrirla en ese rato, la fase es «abierta en otra terminal», con el pid del propio hijo que se está yendo. La barra lateral de la §2.2 tendrá que tratarlo.
- **`status` de una sesión shell:** no tiene hooks ni candado, así que la CLI no puede saber que la app la tiene abierta.
- **Estado guardado viejo en la etiqueta.** Una sesión que se relanza muestra su último estado guardado (`ended`, por ejemplo) hasta que el hook `SessionStart` lo cambia. Con el `claude` falso, que no dispara hooks, se queda así.
- El `Rc` ata el host al hilo de la interfaz. Es deliberado.

## D-32 · Una sola instancia y el socket de eventos

**Fecha:** 2026-11-21 (§2.1).

- **`<home>/app.lock`** con `flock`, tomado antes de tocar el socket. Una segunda instancia termina con «lithemux is already running (pid N)» y código 1. El kernel lo suelta si la app muere.
- **`<home>/app.sock`**:
  - Si el archivo existe y alguien responde, la app no arranca (con el candado ya tomado solo puede ser otro programa). Si nadie responde, es resto de una caída y se borra.
  - Permisos 0600, puestos justo después de crearlo. Queda una ventana mínima entre el `bind` y el `chmod`; el home del usuario ya es privado.
  - Un hilo acepta conexiones y da a cada una un hilo corto, con **200 ms en total** para mandar su línea (hasta 1 KB), la reparta como la reparta. Así un cliente callado o lento no frena a los demás: con cinco callados y uno que goteaba un byte cada 150 ms, un evento tardaba 3,3 s; ahora menos de 150 ms. Lo malformado y lo que nombra una sesión desconocida se ignora.
  - Si `accept` falla (sin descriptores, por ejemplo) espera 50 ms antes de reintentar, en vez de girar en vacío.
  - Al cerrar, el archivo se borra.
- **Bug encontrado por los tests.** El hook escribe y cuelga al instante, a menudo antes de que la app acepte la conexión. En macOS no se puede poner un tiempo de espera de lectura en una conexión así, y la primera versión abandonaba el mensaje al fallar eso. Se perdían eventos. Ahora el fallo se ignora: lo enviado ya está esperando.
- **Límite del sistema.** La ruta de un socket Unix no puede pasar de unos 104 bytes. Con el home por defecto (`~/Library/Application Support/lithemux`) sobra; con un `LITHEMUX_HOME` muy largo la app no arranca y lo dice («path must be shorter than SUN_LEN»). El hook, en ese caso, simplemente no avisa.

## D-33 · El host no espera: abrir y cerrar en segundo plano

**Fecha:** 2026-11-21 (auditoría de la §2.1, I4 y punto 6).

**Contexto.** En la primera versión, `open` preparaba el lanzamiento en el hilo de la interfaz (hasta 3 s de sonda del shell y 2 s de `--version`), y `close` esperaba hasta 1,5 s. Con la barra lateral eso congelaría la ventana.

**Decisión.**

- **Abrir.** `open` solo busca la sesión en la base de datos y devuelve al instante. La sesión queda en fase `Opening` mientras un hilo, con su propia conexión, toma el candado y decide el comando. El resultado llega por el buzón; el host arranca entonces el PTY, que es rápido. Las fases son `Opening`, `Running`, `Ended`, `OpenElsewhere { pid }` y `Failed(motivo)`.
- **Cerrar.** `close` deja de mostrar la sesión, cuelga a su programa y vuelve. No espera.
- **El candado lo suelta el hilo que espera al hijo,** no el host: en cuanto no queda nada del grupo del programa. Antes se soltaba cuando la interfaz procesaba el aviso, o a los 1,5 s aunque el programa siguiera vivo.
- **Cada arranque tiene un número.** El aviso «terminó» lo lleva, y el host ignora el de un arranque anterior de la misma sesión.
- **`shut_down()` es lo único que espera,** hasta 3 s, y solo al salir de la app. También cierra el socket y borra su archivo, porque la app puede salir sin llegar a destruir el host.
- **El entorno de las sesiones sale de la configuración** (`HostConfig.environment`), no de `std::env` directamente. La app pasa el suyo; los tests, uno mínimo.

`PreparedLaunch` es de solo lectura (su plan no se puede cambiar una vez decidido con el candado) y `HeldSessionLock` es `Send`, para poder dárselo a ese hilo.

## D-34 · Las salidas de la app y el fin del grupo de procesos

**Fecha:** 2026-11-21 (auditoría de la §2.1, I1 e I2).

**Qué pasaba.** SIGTERM, SIGINT y SIGHUP mataban a la app sin cerrar nada. Un hijo que ignorase SIGHUP seguía vivo con el candado libre, y un `lithemux open` lanzaba un segundo `claude` sobre la misma conversación. SIGHUP es el caso cotidiano: cerrar la terminal desde la que se lanzó la app.

**Salidas de la app.**

| Cómo sale | Qué cierra las sesiones |
|---|---|
| SIGTERM, SIGINT, SIGHUP | `signal_hook` avisa al hilo de la interfaz, que llama a `shut_down()` y después a `quit` |
| cmd+q, cerrar la ventana, la petición de salida de macOS (la del Dock) | `on_app_quit` de gpui, que llama a `shut_down()` |
| SIGKILL, una caída | Nada. Es el límite de D-31 |

Las señales cierran por su cuenta y no a través de `on_app_quit` porque **gpui sin ventana no instala el delegado de la aplicación** y esos avisos no corren. Los tests del binario usan ese modo (`--no-window`), así que la ruta de las señales tenía que valer sin ventana.

**Fin del grupo de procesos** (en el PTY, vale para todo terminal):

- Al colgar: SIGHUP al grupo y, si el programa no ha muerto tras 200 ms, SIGKILL al grupo.
- **Cuando el programa termina, por lo que sea,** el hilo que lo esperaba mira si queda alguien en su grupo. Si queda: SIGHUP, 200 ms y SIGKILL, y espera hasta 1 s a que el grupo quede vacío. Antes, un hijo terco de un `claude` que moría (o que salía solo) se quedaba vivo.
- Una sesión está «ida» solo cuando su grupo está vacío, y es entonces cuando se suelta el candado.

**Costo aceptado.** Lo que un programa deje en su grupo muere con él. Un shell interactivo pone sus trabajos en segundo plano en grupos propios, así que esos no se tocan.

**Comprobado a mano con ventana y dos hijos que ignoran SIGHUP:** con SIGHUP a la app y con la petición de salida del sistema, la app se va, los hijos mueren, el socket se borra y `status` dice `closed`.

**Sobre `osascript … "lithemux-app" to quit`.** No cierra la app, y no es un fallo suyo: el binario no es un bundle y AppleScript no lo encuentra por ese nombre (no da error, simplemente no llega). La misma petición enviada por id de proceso (`NSRunningApplication.terminate()`, que es lo que hace el Dock) sí la cierra en orden. Cuando la app sea un `.app` (Fase 3), `osascript` funcionará por nombre.

## D-35 · La app siempre arranca con el gestor de sesiones; `--open` solo queda sin ventana

*2026-11-22, bloque §2.2–§2.3.*

`lithemux-app` sin argumentos abre la ventana con la barra lateral y el área de trabajo, y **antes de mostrar nada** arranca el `SessionHost` (candado de instancia única y socket de eventos). El modo spike desapareció: `lithemux-app <programa>` ya no se entiende y sale con código 2.

`--open <cat>/<sesión>` **se retiró de la ventana**: con ventana, las sesiones se abren desde la barra lateral, y pasarlo sale con código 2 y un mensaje que lo dice. **Se conserva solo junto a `--no-window`**, el modo sin interfaz que usan los tests de la salida de la app (señales, segunda instancia), porque un test no puede hacer clic en una barra lateral. Es una desviación menor de «`--open` se retira»: la alternativa era dejar sin pruebas la salida del binario real.

Regla `H-003`: solo `app_shell/run.rs` pone en marcha gpui.

## D-36 · Puertos en escucha: `listeners` y `sysinfo` en vez del crate `libproc`

*2026-11-22.*

El plan pide detectar los puertos «con `libproc`, sin red». Probé primero el crate `libproc` y lo descarté por dos razones:

- Para leer el puerto hay que leer una `union` (`soi_proto.pri_tcp`), que en Rust es `unsafe`. Sería una segunda excepción a `H-002`, y esa regla dice que cualquier otra se decide antes con la planificadora.
- Trae `bindgen` como dependencia de compilación (necesita `libclang` para compilar) y, por esa cadena, `libloading`, de licencia ISC, que `cargo deny` rechaza.

Lo que quedó, en `core::platform::process::listening_ports`:

- **`listeners` 0.6 (MIT, sin dependencias en macOS):** recorre los descriptores de cada proceso con la API de `libproc` del sistema (`proc_pidinfo`, `proc_pidfdinfo`) y devuelve los sockets TCP en `LISTEN` con su pid.
- **`sysinfo` 0.40 (MIT), solo con la característica `system`:** da el padre de cada proceso, para armar el árbol de la sesión.

Ninguno abre sockets ni hace red: leen las tablas de procesos del kernel. Nuestro código no tiene `unsafe` nuevo. El escaneo se hace cada 3 s en un hilo de fondo, **solo si hay sesiones en marcha**.

**Costo, tras la auditoría del bloque 3.** La primera versión refrescaba todos los procesos de la máquina con `sysinfo` para armar el árbol. Ahora se parte de quién escucha (pocos procesos) y se sube por sus padres, uno a uno y con memoria, hasta dar con la raíz de una sesión: `sysinfo` solo mira esos pids. Medido en release con ~680 procesos, 60 corridas: **de 8,2 ms a 3,7 ms de mediana** (p95 de 9,9 a 4,7 ms). Cada 3 s eso es un 0,12 % de CPU.

**Lo que no se pudo: consultar solo los pids de la sesión.** Ninguna versión de `listeners` lo permite (la 0.3 tenía `get_ports_by_pid`, que por dentro recorre toda la máquina y filtra). Los 3,7 ms que quedan son ese recorrido. Bajarlo de ahí exige `proc_pidfdinfo` por pid, es decir, el crate `libproc` con su lectura de `union` (`unsafe`) y su licencia ISC: lo mismo que se descartó arriba. Queda a decisión de la planificadora.

Abrir `http://localhost:<puerto>` lo hace `/usr/bin/open` a través de `command_without_session_markers` (`platform::process::local_page`), así que `H-001` sigue cumpliéndose y la app no abre ninguna conexión.

**Si la planificadora prefiere el crate `libproc`:** hace falta aceptar la lectura de la `union` como segunda excepción de `unsafe` y la licencia ISC en `deny.toml`.

**Límite conocido.** El árbol se arma por padre. Un proceso que se desliga del todo (doble `fork`, queda colgando de `launchd`) ya no es descendiente y su puerto no aparece.

## D-37 · Vista previa y no leídos: solo campos documentados de los hooks

*2026-11-22. Fuente: <https://code.claude.com/docs/en/hooks>, consultada ese día (Claude Code instalado: 2.1.330).*

| Evento | Campo documentado | Qué hace en la tarjeta |
|---|---|---|
| `UserPromptSubmit` | `prompt` | Vista previa. No cuenta como no leído |
| `Notification` que pide algo al usuario (`permission_prompt`, `elicitation_dialog`, `elicitation_url_dialog`) | `message` | Vista previa y +1 no leído |
| Cualquier otra `Notification` | — | Nada |
| `Stop` | `last_assistant_message` | Vista previa (si viene) y +1 no leído |

- **`Stop` sí trae el último mensaje.** La referencia documenta `last_assistant_message` («the text content of Claude's final response») y recomienda usarlo «instead of reading the transcript». La página no dice desde qué versión existe, así que el campo se trata como opcional: si no viene, la vista previa anterior se queda y el turno cuenta igual.
- **Solo cuentan las notificaciones que bloquean a Claude esperando al usuario** (corrección de la auditoría del bloque 3). Son las mismas tres que ponen la sesión en `waiting`. Las demás informan (`auth_success`, `quota_*`, `agent_completed`, `plugin_notification`…) o repiten un turno ya contado (`idle_prompt`, a los 60 s de reposo): no suman ni tapan lo último que dijo Claude. Un tipo que la app no conozca tampoco cuenta.
- La vista previa se guarda en una sola línea y con tope de 200 caracteres. **Nunca se lee el transcript.**
- **El hook escribe y la app marca como leído.** El hook suma no leídos aunque la app esté cerrada; la app los pone en cero cuando el usuario la está mirando: la pestaña mostrada de un panel **y la ventana activa**. Con la ventana en segundo plano no se marca nada; al volver a ella, sí. La decisión es una función pura (`workspace/read_policy.rs`).

Costo en el hook: una escritura más, unos 3 ms (media de 16 ms con `hyperfine`, 300 corridas).

## D-38 · Atajos de teclado del área de trabajo

*2026-11-22.* Son los de iTerm y cmux, que Jhon ya usa:

| Atajo | Qué hace |
|---|---|
| `cmd+t` | Nueva terminal en el panel con foco |
| `cmd+d` | Dividir a la derecha |
| `cmd+shift+d` | Dividir hacia abajo |
| `cmd+w` | Cerrar la pestaña (termina el programa de la sesión) |
| `cmd+shift+w` | Cerrar el panel con todas sus pestañas |
| `cmd+alt+→` / `cmd+alt+←` | Foco al panel siguiente / anterior |
| `cmd+shift+→` / `cmd+shift+←` | Pestaña siguiente / anterior |
| `cmd+1` … `cmd+9` | La pestaña de ese lugar en el panel con foco |
| `cmd+]` / `cmd+[` | Panel siguiente / anterior (teclado US) |
| `cmd+shift+]` / `cmd+shift+[` | Pestaña siguiente / anterior (teclado US) |
| `cmd+,` | Ajustes |
| `cmd+m` | Minimizar |
| `cmd+q` | Salir |

**Por qué los corchetes no servían (bloque 4, parte 0).** gpui nombra cada tecla por el carácter que produce. Con Shift sobre una tecla que no es una letra, macOS informa el carácter ya desplazado y gpui descarta el Shift: «cmd+shift+]» llega como `cmd-}`. El atajo estaba escrito como `cmd-shift-]`, que no coincide nunca, ni siquiera en un teclado US. Ahora está como `cmd-}` / `cmd-{`. En un teclado latinoamericano esa tecla da `´` y `¨`, y los corchetes piden Option, así que ahí los corchetes no son prácticos: **cada acción tiene al menos un atajo que no depende de la distribución** (letras, dígitos y flechas). Un test construye las pulsaciones como las construye gpui en macOS y lo comprueba.

Solo valen con el foco en el área de trabajo, para que no interfieran con un formulario de la barra lateral.

**Sin `ctrl+tab`** (corrección de la auditoría del bloque 3): gpui resuelve los atajos antes de que la tecla llegue al terminal, así que se los quitaba al shell y a los programas de dentro. Todos los atajos llevan `cmd`, que un terminal no usa.

## D-39 · Renombrar cambia el título, nunca el slug

*2026-11-22.* El slug de una sesión es su nombre para Claude (`-n`, `--resume <nombre>`, la dirección de `SendMessage`) y el de la categoría forma parte de esa dirección. Cambiarlos rompería la mensajería entre sesiones. Por eso:

- Renombrar una sesión guarda un **título** (`sessions.title`); vacío vuelve al slug.
- Renombrar una categoría cambia su `name`; el slug queda.
- Al crear desde la barra lateral se escribe un nombre libre y el slug se deriva (`slug_from_name`: «API de pagos» → `api-de-pagos`). Si el nombre no era ya un slug, queda como título. Una categoría con slug repetido recibe `-2`, `-3`…

## D-40 · «Nueva terminal» crea una sesión de shell en una categoría

*2026-11-22.* El plan dice que las sesiones de shell tienen tarjeta y que el contenido de los paneles se guarda, y el modelo exige que toda sesión pertenezca a una categoría. Así que el botón (y `cmd+t`) crea una sesión de perfil `shell` llamada `terminal`, `terminal-2`…:

- en la categoría de la sesión que muestra ese panel;
- si el panel está vacío, en la primera categoría;
- **si no hay ninguna categoría, no crea nada** y dice que primero hay que crear una.

## D-41 · Qué arranca y qué termina un programa

*2026-11-22.*

- **Al restaurar** el layout: en el bloque 3 ninguna pestaña arrancaba su programa. **Desde el bloque 4 lo visible se reanuda solo (D-50).**
- **Clic en una tarjeta:** muestra la sesión en el panel con foco (o va a la pestaña que ya tiene) y arranca su programa si la app no lo tiene. Si terminó aquí, se ve su última pantalla con el botón «Reabrir».
- **Cerrar una pestaña** (o el panel) termina el programa de la sesión. Ocultar sin terminar llegará con la hibernación (§2.4).
- **Una sesión, una pestaña:** un terminal tiene un solo canal de repintado, así que no puede verse en dos sitios.
- **Cerrar el último panel** no existe: el único panel se vacía. Un panel de un split que se queda sin pestañas desaparece y su vecino ocupa el sitio.

## D-42 · El layout se guarda como JSON en una tabla de una fila

*2026-11-22.* Migración 2 (`user_version = 2`):

- `sessions`: `title`, `pin_sort` (NULL = no fijada), `preview`, `unread`.
- `categories`: `collapsed`.
- `workspace_layout(id = 1, layout TEXT)`: el árbol de splits, los paneles con sus pestañas y el foco, en JSON.

El árbol es un dominio nuevo de `core` (`modules/workspace`), puro y con sus tests. Lo que vuelve de la BD **se valida** (hasta 4 paneles, ids únicos, una sesión en una sola pestaña, foco existente); si no cuadra o no es JSON, la app arranca con un panel vacío en vez de fallar. Las pestañas de sesiones borradas se quitan al restaurar. Cada cambio se guarda al momento; el arrastre de un divisor, 400 ms después de que se detiene (un antirrebote: cada movimiento descarta el guardado pendiente del anterior) y al salir.

Una base de la versión 1 se migra conservando sus datos (test con el esquema del commit de la Fase 0).

## D-43 · Estado de la tarjeta y fase «cerrando»

*2026-11-22.*

- **La interfaz pide la reapertura** (corrección de la auditoría del bloque 3). El host sabía reabrir, pero la ventana no se lo pedía: trataba «cerrando» como «ya la tengo» y había que hacer clic dos veces. La decisión de pedir o no es ahora una función pura con su test (`workspace/start_policy.rs`): se pide siempre, salvo que la sesión esté abriéndose, en marcha o terminada con su pantalla a la vista.
- **`Phase::Closing`.** Al cerrar una sesión, el host la deja en «cerrando» hasta que no queda nada suyo: ni el programa (y su grupo) ni un lanzamiento que se estuviera preparando, que también tiene el candado. Si se pide abrirla mientras tanto, el host lo apunta y **la abre él solo** en cuanto el candado queda libre (`Opened::WhenClosed`). Ya no se pide el candado mientras lo tiene nuestro propio hijo, así que no puede salir `OpenElsewhere` con su pid.
- **Etiqueta.** `Opening` → «abriendo», sea cual sea el estado guardado. Y con el programa ya en marcha pero sin que Claude haya avisado (el estado guardado sigue siendo `ended`, `hibernated` o `new`, de la corrida anterior), también «abriendo».
- **Una sesión que la app no tiene** muestra el estado de la BD: puede estar abierta con `lithemux open` en otra terminal. Tras una caída ese estado puede ser viejo; lo corrige la §2.5.

## D-44 · La interfaz no se prueba con el soporte de tests de gpui

*2026-11-22.* Lo intenté (`gpui` con `test-support`, ventana simulada, atajos y comandos reales). No sirve aquí: su planificador exige que todo ocurra en el hilo del test y aborta con «Your test is not deterministic» en cuanto un hilo de verdad despierta una tarea, que es justo lo que hacen el PTY y el buzón del host. Además traía ~40 crates de desarrollo. Lo revertí.

Lo que sí se prueba sin gpui: el árbol de paneles y su guardado, el árbol de la barra lateral, los estados, los menús, el orden, las acciones sobre SQLite real, el campo de texto, el host con PTY reales y el binario real en sus salidas. **Lo que no tiene test automático es el cableado de gpui** (clics, arrastres, foco, el menú flotante, escribir en un campo): se verifica a mano (`docs/fases/F2-prueba-manual.md`, pasos G a K).

## D-45 · El socket de la app cuando la ruta del home no cabe

*2026-11-22, corrección de la auditoría del bloque 3.*

La dirección de un socket Unix admite 103 bytes en macOS. Con un `LITHEMUX_HOME` largo o un nombre de usuario largo, `<home>/app.sock` no cabía y la app no arrancaba.

`core::platform::paths::app_socket` es el único sitio que decide la ruta (regla `H-004`), y lo usan la app y el hook:

- Si `<home>/app.sock` cabe, va ahí, como antes.
- Si no, va en `/tmp/lithemux-<uid>/<hash>.sock`. El hash (FNV-1a de 64 bits, escrito a mano para que dé lo mismo en los dos binarios) sale de la ruta del home, así que cada home tiene su socket.
- **`/tmp` es de todos.** La app crea el directorio con modo 0700 y se niega a arrancar si no es un directorio de verdad (no un enlace), del usuario y cerrado a los demás. El hook hace la misma comprobación antes de conectar: si no se cumple, no avisa a nadie (y, como siempre, termina con 0).

Se usa `/tmp` y no `$TMPDIR` porque en macOS este último ya es largo.

## D-46 · Borrar una sesión que está abierta

*2026-11-22, corrección de la auditoría del bloque 3.*

`lithemux rm` de una sesión que la app tenía en marcha dejaba a su `claude` vivo y sin tarjeta. Se cierran las dos puertas:

- **La CLI se niega** a borrar una sesión abierta, esté en la app o en otra terminal: «the session is already open (pid N)». `lithemux cat rm` se niega si alguna de las sesiones de la categoría lo está, y dice cuál. Lo comprueba con el candado de la sesión (`ensure_closed`).
- **La app cierra lo que ya no existe.** Cada vez que relee el tablero, cierra en el host las sesiones que la base de datos ya no tiene. Hace falta porque las sesiones de shell no toman candado (la CLI no puede saber que están abiertas) y por si la base cambia por otra vía.

Borrar desde la barra lateral no cambia: pide confirmación, cierra el programa y borra.

## D-47 · Ajustes a la tarjeta y a las pestañas tras la prueba con clics del bloque 3

*2026-11-22, bloque 4, parte 0.*

- **Puertos: solo los de lo que la sesión arrancó.** Claude Code 2.1.330 escucha él mismo en un puerto local por sesión, que no sirve al usuario. El proceso raíz de una sesión (el `claude`, o el shell) nunca aporta puertos; sus descendientes, sí.
- **La tarjeta nombra hasta 3 puertos** y cuenta el resto («+2»). El texto de estado es el que cede, con puntos suspensivos; los puertos no se cortan.
- **En un panel estrecho mandan las pestañas.** Los botones del panel ceden primero su sitio (queda al menos el de nueva terminal); después las pestañas se encogen hasta unas letras con puntos suspensivos, y solo entonces la fila se desplaza.
- **La vista previa no muestra markdown en crudo.** Al guardarla se quitan `**`, `__`, las comillas invertidas y el `#` de los títulos. Un `*` o un `_` sueltos se dejan: suelen ser un producto o un nombre.

## D-48 · Lo que Claude pide se guarda aparte de lo último que dijo

*2026-11-22, bloque 4, parte 0. Fuente: <https://code.claude.com/docs/en/hooks>, consultada ese día.*

**El problema.** Si Claude pedía un permiso y el usuario pulsaba Esc, la tarjeta seguía en «waiting / Claude needs your permission» hasta el siguiente mensaje, y el aviso quedaba como vista previa incluso con la sesión terminada.

**Qué dice la documentación.** No hay ningún evento para eso:

- `Stop` «does not run if the stoppage occurred due to a user interrupt».
- `PermissionDenied` «only fires in auto mode: it doesn't run when you manually deny a permission dialog».
- `PostToolUseFailure` «doesn't fire for tool calls rejected before execution… or a permission denial».
- Ningún tipo de `Notification` avisa de que el permiso ya no está pendiente.

**Lo que se hizo.**

- Migración 3: columna `sessions.notice`. Lo que Claude pide (`Notification` de los tres tipos que bloquean) va ahí, **no** a la vista previa. La tarjeta muestra el aviso mientras existe y, si no, lo último que se dijo.
- El aviso se borra en cuanto la sesión deja de esperar, por cualquier vía: `PostToolUse`, `Stop`, un mensaje nuevo, `SessionEnd`, el cierre desde la app o la hibernación. La regla vive en los casos de uso `apply_state` y `move_state`.
- **`idle_prompt` mueve `waiting → idle`, si llega.** Se dejó porque no hace daño y solo toca una sesión que esté en `waiting`.

**Lo que midió la planificadora con el `claude` real (ronda de correcciones del bloque 4).** La mitigación anterior **no funciona** como única salida: tras rechazar un permiso con Esc, la sesión siguió en `waiting` con «Claude needs your permission» **más de 3 minutos, sin que llegara ningún `idle_prompt`**. Y con un permiso de verdad pendiente la sesión se mantuvo en `waiting` 100 s sin que un `idle_prompt` la bajara (la sospecha contraria de la auditoría era falsa).

**Qué se hace entonces (PLAN §2.4, ajuste del 22-nov).** No hay señal documentada, así que no se intenta adivinar que el permiso se rechazó. La consecuencia que importaba era que una sesión así no hibernaría nunca, y eso se resuelve en la regla de hibernación: **una sesión en `waiting` también se hiberna** cuando lleva más de `hibernate_after_hours` sin ningún evento (D-49).

**Limitación conocida, tal como queda.** Tras rechazar un permiso con Esc, la tarjeta sigue diciendo que Claude espera, con el aviso del permiso, hasta que pase algo en la sesión: un mensaje nuevo, una herramienta, el fin de la sesión o su hibernación.

## D-49 · Hibernación y válvula de memoria

*2026-11-22, bloque 4, §2.4.*

- **Las decisiones son funciones puras** de `core` (`session/domain/hibernation_policy.rs`), con la hora como parámetro: nadie espera 48 h para probarlas.
  - Hibernar: en reposo (`idle`) **o esperando al usuario (`waiting`)**, fuera de la vista, sin `keep_alive` y **más de** `hibernate_after_hours` sin actividad. Trabajando (`working`), nunca.
  - Válvula: solo sesiones `idle`, fuera de la vista y sin `keep_alive`; **nunca una `waiting`**. La que lleva más tiempo quieta, una por minuto como máximo. Con presión **crítica**, sin mínimo; con presión de **aviso** (`warning`), solo si lleva al menos 30 min quieta. En macOS el nivel crítico casi nunca llega (jetsam actúa antes) y esta Mac pasa el día en aviso.
- **Hibernar es:** escribir `hibernated` en la BD **primero** (para que lo que el programa avise al salir no lo deshaga), SIGHUP al grupo y SIGKILL a los 5 s si sigue vivo. El candado se libera cuando no queda nada del grupo.
- **Despertar:** una sesión hibernada cuya pestaña pasa a estar a la vista se relanza con `core::launch`, que reanuda con `--resume`. La tarjeta dice «resuming»/«reanudando» hasta que Claude avisa. Si se la mira mientras su programa todavía se está yendo (los 5 s de gracia), se pide igual y el host la abre sola cuando el candado queda libre.
- **Lo que el programa avise mientras se va no deshace la hibernación:** `Stop`, `UserPromptSubmit`, `Notification` y `SessionEnd` respetan el estado `hibernated`. Solo `SessionStart`, el arranque de la siguiente corrida, saca a una sesión de ahí.
- **Una sesión que termina sola** queda en `ended` con el botón «Resume»: no se relanza por mirarla.
- **Cada 30 s** el área de trabajo, que es quien sabe qué está a la vista, repasa las sesiones.
- **Nunca se pierde una conversación:** la app no borra transcripts, y relanzar pasa siempre por `plan_launch`, que usa `--resume` si el transcript existe.

**Desviación: la presión de memoria se consulta, no se escucha.** El plan pide `DISPATCH_SOURCE_TYPE_MEMORYPRESSURE`. Suscribirse a esa fuente desde Rust exige `unsafe` (FFI con libdispatch), y la regla `H-002` deja el `unsafe` en un solo sitio. Se lee el mismo nivel del kernel, `kern.memorystatus_vm_pressure_level` (1 normal, 2 aviso, 4 crítico), con `/usr/sbin/sysctl` cada 30 s, a través de `command_without_session_markers`. Cuesta unos 4 ms cada vez y, como la válvula actúa una vez por minuto como máximo, no se pierde nada frente al aviso inmediato. **Aceptada por la planificadora.** `sysctl` se lanza con un plazo de 2 s, para que un cuelgue no detenga el ciclo.

## D-50 · Arranque y restauración: lo visible se reanuda

*2026-11-22, bloque 4, §2.5. Sustituye lo que D-41 decía del arranque.*

- **Cerrar la app es hibernar** (principio 3 del plan): al salir, las sesiones de Claude que estaban en marcha quedan `hibernated`, escrito antes de avisar a los programas. Las que el usuario había cerrado siguen `ended`. **Con la misma gracia que una hibernación:** todos los programas reciben el SIGHUP a la vez y tienen 5 s para salir; a lo que siga vivo se le manda SIGKILL desde el propio cierre, porque los hilos que lo harían mueren con la app. Con el `claude` real el cierre tarda menos de 1 s.
- **Al arrancar, antes de nada,** el host hiberna toda sesión de Claude que la BD todavía tenga como `working`, `idle` o `waiting` y que **nadie tenga**: es lo que deja una caída, un `kill -9` o un reinicio del Mac. Lo decide **el candado, no el pid** escrito en su archivo: se lee el pid, se intenta tomar el candado y solo si se consigue y ese pid no está vivo se escribe `hibernated`, con el candado tomado. Una sesión abierta con `lithemux open` en otra terminal, o un `claude` huérfano de una app que murió, no se tocan.
- **Con la ventana arriba:** las sesiones de Claude hibernadas que están a la vista (la pestaña mostrada de cada panel) se reanudan solas; las demás siguen hibernadas hasta que se las mire. Las sesiones de shell que tienen pestaña se relanzan en su carpeta.
- Una sesión `ended` a la vista no se relanza: muestra el botón.

## D-51 · Ajustes, idioma, tema y menú

*2026-11-22, bloque 4, §2.6.*

- **Los ajustes son los mismos en la ventana y en `lithemux config`,** con la misma validación (`settings::checked_value`): `language`, `context_language`, `theme`, `font_family`, `font_size`, `scrollback`, `hibernate_after_hours`, `memory_valve` y `claude_path`. El test del §0.6 que dejaba `language` fuera de la CLI se cambió citando el §2.6.
- **Todo se aplica sin reiniciar.** La app relee los ajustes cada 3 s junto con el tablero, así que también recoge lo que se cambie con `lithemux config` desde una terminal. El idioma cambia los textos, la barra de menús y la ventana de Ajustes al momento.
- **`scrollback` vale para las terminales que se abran desde entonces;** una ya abierta conserva el suyo.
- **Una fuente que el Mac no tiene** no se usa: se dibuja con Menlo. La lista de fuentes del sistema es lenta de obtener, así que solo se consulta cuando cambia la fuente pedida (consultarla cada 3 s llevaba la CPU en reposo al 1,17 %).
- **Tema:** claro, oscuro o el del sistema; con «sistema» la app sigue a macOS cuando cambia. Cada tema tiene sus colores por defecto del terminal. Un test comprueba el contraste del texto sobre cada fondo en los dos temas.
- **Barra de título:** gpui no deja fijar la apariencia de la ventana, y la barra del sistema no seguía el tema. Las ventanas la ocultan (`appears_transparent`) y pintan una franja propia bajo los botones de macOS, que arrastra la ventana y la amplía con doble clic.
- **Menú nativo:** el de la app (Acerca de, Ajustes…, Salir), Archivo, Edición, Visualización, Ventana y Ayuda. Copiar y pegar son acciones, para que el menú y el teclado pidan lo mismo. «Ayuda» solo tiene «Abrir la carpeta de datos»: la app no tiene documentación en línea que abrir ni debe conectarse a nada.

## D-52 · «Madre» e «hijas» pasan a ser «sesión coordinadora» y «sesiones»

*2026-11-22, ronda de correcciones del bloque 4. Decisión de Jhon.*

| Antes | Ahora |
|---|---|
| madre / `mother` | sesión coordinadora / `coordinator` |
| hija / `child` | sesión de la categoría / `member` |
| hermanas / `sisters` | las demás sesiones / `the other sessions` |

- **Código:** `Kind::Coordinator` y `Kind::Member`, `coordinator_policy.rs`, `find_coordinator`, `SessionError::CoordinatorExists`.
- **BD, migración 4:** los valores de `sessions.kind` y el índice `one_coordinator_per_category`. Los valores viejos estaban en un `CHECK` de la tabla, así que la tabla se rehace dentro de la migración. Probada desde las versiones 1, 2 y 3, con datos, y desde el esquema de una base v3 hecha por la propia app.
- **CLI:** `--coordinator` en `new` y `set`. `--mother` deja de existir, sin alias: la app no se ha publicado.
- **Contexto que recibe Claude** (es/en): «sesión coordinadora», «Coordinadora de la categoría», «Las demás sesiones».
- **Interfaz:** etiqueta «coordinator»/«coordinadora»; «Make coordinator»/«Hacer coordinadora»; «Remove as coordinator»/«Quitar como coordinadora»; «already has a coordinator session».
- **Quedan con el término viejo, por ser historia:** los reportes de las fases 0, 1 y 2 y las decisiones anteriores a esta. En `git log`, los commits anteriores.
- **Regla `H-007`:** el vocabulario viejo no vuelve a `crates/*/src`, `locales/` ni al `README.md`. No prohíbe `child` a secas, como pedía el encargo, porque es también «proceso hijo» y el método `.child(…)` de gpui; atrapa las formas en que nombraba el tipo de sesión.
