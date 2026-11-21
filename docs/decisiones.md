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
