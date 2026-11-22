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
