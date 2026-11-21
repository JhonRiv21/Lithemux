# Fase 1 · Reporte del spike de interfaz nativa

**Fecha:** 2026-11-21 · **Ejecutora:** Fable 5.2 · **Estado:** spike construido, medido y corregido tras la auditoría independiente y tras el primer intento de prueba manual (la app no pintaba texto, ya arreglado); faltan las pruebas 1 a 6, que necesitan a Jhon. La decisión GO/NO-GO no se toma aquí.

## Resultado de las pruebas del spike

| # | Prueba | Resultado |
|---|---|---|
| 1 | Claude dentro del terminal: TUI completo, truecolor, sin parpadeo | **Pendiente de Jhon** |
| 2 | Teclado en español: teclas muertas, `ñ`, `¿`, `¡`, menú de acentos | **Pendiente de Jhon** |
| 3 | `shift+enter` inserta una línea nueva | **Pendiente de Jhon** |
| 4 | Copiar y pegar, pegado entre corchetes, selección con el ratón | **Pendiente de Jhon** |
| 5 | Redimensionar la ventana | **Pendiente de Jhon** |
| 6 | Dos paneles lado a lado y foco con el teclado | **Pendiente de Jhon** |
| 7 | Rendimiento | **Medido**: las cuatro metas se cumplen; el pico de footprint durante el volcado queda justo por debajo de la meta (detalle abajo) |

Los pasos de las pruebas 1 a 6 están en `docs/fases/F1-prueba-manual.md`.

**Lo que no pude comprobar yo.** No vi la ventana: esta sesión no tiene permiso de grabación de pantalla (`screencapture` falla) ni puede simular teclado. Sé que la app arranca, crea su ventana, lanza los dos shells, no escribe errores y responde a las mediciones; **no sé cómo se ve**. Todo lo visual (colores, alineación de columnas, bordes de caja, cursor) depende de la prueba de Jhon.

## Prueba 7 · Rendimiento

Binario release (`target/release/lithemux-app`, 6,4 MB), macOS 26.5.1, Apple M5. Medido después del último cambio de código, ya con las correcciones de la auditoría.

| Meta | Resultado | Medición |
|---|---|---|
| `cat` de 50 MB no congela la interfaz | **Cumple** | 52,5 MB en **cada uno de los dos paneles a la vez**: 0,96 a 1,01 s por panel en tres corridas. El hilo principal nunca estuvo más de 2 a 7 ms sin atender un temporizador de 16 ms |
| CPU en reposo con 2 terminales < 1 % | **Cumple** | **0,0 %**: 0,00 s de CPU en 60 s |
| Footprint con 2 terminales < 150 MB | **Cumple.** En reposo: 59 MB. Con los dos historiales llenos tras el volcado: 99 a 113 MB. Pico durante el volcado: **133, 148 y 133 MB** en tres corridas | `footprint <pid>` |
| Cero conexiones | **Cumple** | `lsof -nP -i` no lista nada; `nettop` tampoco |

El pico entra en la meta por poco: una de las tres corridas llegó a 148 MB. En una tanda anterior, con el mismo código salvo un reordenamiento de archivos, una corrida tuvo además un retraso aislado de 89 ms en el hilo principal durante el primer segundo, que incluye el arranque de la app; no se repitió.

Comandos:

```sh
# Reposo: lanzar, esperar 10 s y medir 60 s
./target/release/lithemux-app &
ps -o time= -p <pid>          # dos lecturas separadas 60 s: 0:00.21 y 0:00.21
footprint <pid>               # phys_footprint: 59 MB (pico de arranque: 97 MB)
lsof -nP -i -a -p <pid>       # sin salida
nettop -p <pid> -L 1 -x       # solo la cabecera

# Volcado: 640 000 líneas con color, 52 480 000 bytes, en ambos paneles
LITHEMUX_SPIKE_STALLS=1 ./target/release/lithemux-app <script que hace cat y cronometra>
```

`LITHEMUX_SPIKE_STALLS` es una ayuda de medición del spike: un temporizador de 16 ms en el hilo principal que imprime cada segundo su mayor retraso. Mide que el hilo de la interfaz sigue libre, no que cada cuadro se pinte a tiempo; eso se ve con los ojos (prueba 7 de la guía manual).

Sobre el footprint: el historial pasó de 10 000 a 5 000 líneas por terminal por indicación de la auditoría. Con 10 000, el pico era de 154 a 168 MB y quedaba en 118 a 133 MB después del volcado.

Al matar la app con los dos shells abiertos no quedó ningún proceso hijo vivo un segundo después.

## Qué se hizo

### 1.1 Base

- Crate `crates/app` (`lithemux-app`), con biblioteca (`terminal_view`) y binario (la ventana del spike).
- gpui fijado al tag `v1.29.3` de Zed con `runtime_shaders`, porque la máquina no tiene compilador de Metal (D-14). No hizo falta instalar Xcode.
- `alacritty_terminal` 0.26 y `portable-pty` 0.9 desde crates.io.
- `TerminalView` propio en `crates/app/src/terminal_view/` (D-18): celdas, colores de 16, 256 y 24 bits, negrita, cursiva, subrayado, tachado, vídeo inverso, caracteres anchos, cursor (bloque, barra, subrayado; hueco sin foco), selección con el ratón (carácter, palabra, línea) y scroll del historial con la rueda.
- Entrada: teclas con secuencias de xterm, texto por el método de entrada del sistema (teclas muertas, menú de acentos, IME), pegado entre corchetes, eventos de foco (D-17).
- `spike_window.rs`: una ventana con dos paneles, foco con `cmd+option+←/→`. Es lo desechable.

### 1.2 Pruebas

46 pruebas automáticas nuevas (45 en `crates/app` y una de fronteras), sin el `claude` real ni un shell real. Los fakes arrancan con el entorno vacío.

| Conjunto | Pruebas | Qué cubre |
|---|---|---|
| `terminal_view/test/keys_test.rs` | 9 | `shift+enter`, flechas y su modo, control, teclas de función; que el texto, `option` + carácter y `cmd` no pasan por la tabla |
| `terminal_view/test/composition_test.rs` | 6 | tecla muerta, `ñ ¿ ¡`, menú de acentos, composición abandonada, UTF-16 |
| `terminal_view/test/paste_test.rs` | 4 | pegado entre corchetes, UTF-8, texto que intenta cerrar el corchete o interrumpir, pegado sin corchetes |
| `terminal_view/test/grid_test.rs` | 7 | color de 24 bits exacto, cubo de 256, negrita, vídeo inverso, carácter ancho, cursor, cursor oculto con posición |
| `tests/integration/session_test.rs` | 16 | PTY real contra `tests/fakes/fake-program`: salida, entrada con acentos, `TERM`/`COLORTERM`, **el resize llega al programa**, pegado hostil de punta a punta, flechas según DECCKM, informes de foco, consulta de color, mínimo de 2 columnas, selección, scroll, dos sesiones independientes, fin del proceso |
| `tests/integration/lifecycle_test.rs` | 3 | actualización sincronizada sin cerrar, hijo que ignora `SIGHUP`, cero repintados en reposo durante 2 s |

Los valores esperados salen del plan, de la especificación de xterm y de la auditoría, no de ejecutar el código. Aun así, **estas pruebas se escribieron con la implementación en contexto**, porque el spike se construyó en la misma sesión: sirven de red contra regresiones, no como verificación independiente. La vista de gpui (pintado, ratón, foco) no tiene pruebas automáticas.

### 1.3 Decisión

Queda para Jhon y la planificadora. Lo que este reporte aporta:

- **A favor:** compila sin Xcode; cero red y cero telemetría; ningún crate GPL; reposo real (0,0 % de CPU, 59 MB); la salida masiva no bloquea la interfaz.
- **Sin evidencia todavía:** todo lo visual y el teclado, es decir, justo las pruebas 1, 2 y 6 que definen el NO-GO.

## Checklist de cierre

Ejecutado después del último cambio:

| Paso | Resultado |
|---|---|
| `cargo fmt --all --check` | sin diferencias |
| `cargo clippy --all-targets -- -D warnings` | sin avisos |
| `cargo test` | **233 pruebas, 0 fallas** (185 de la Fase 0 + 48 nuevas) |
| `cargo build --release` | ok; `lithemux-app --check-text` responde `text rendering: ok` |
| `cargo deny check` | advisories, bans, licenses y sources ok (D-15) |
| `scripts/lint-findings` | 0 reglas, sin hallazgos |
| Fronteras de arquitectura | pasa, con la regla de puertas ampliada y anclada a `crates/app` (D-19) |
| Guardas de fuente única | pasan; la de `Cargo.lock` con más crates prohibidos y la de programas reales con la regla `shell-by-name` (D-13, D-15) |

## Correcciones de la auditoría

La planificadora lanzó una auditoría independiente sobre el spike. Sus puntos, uno por uno:

| # | Punto | Arreglo | Test |
|---|---|---|---|
| 1 | Una actualización sincronizada sin cerrar congelaba la pantalla | Un hilo espera el plazo de 150 ms y muestra lo retenido. No se usó `poll`, porque exige `unsafe` (D-18) | `a_synchronized_update_left_open_is_shown_after_its_timeout`. **Falló antes del arreglo** |
| 2 | Un hijo que ignora `SIGHUP` sobrevivía al cerrar la vista | `SIGHUP` al grupo, 200 ms de gracia, `SIGKILL` al grupo; los hilos terminan por fin de archivo | `a_program_that_ignores_hangup_dies_with_its_session`. **Falló antes del arreglo** |
| 3 | `SIGHUP` a un pid reciclado | Marca de «recogido» que el cierre respeta | Sin test propio: la carrera no se puede provocar de forma fiable. Va con el 2 |
| 4 | El menú de acentos salía en (0,0) con el cursor oculto | La foto guarda siempre la posición y aparte `visible` | `a_hidden_cursor_still_reports_where_it_is`, que sustituye al test que daba el bug por bueno |
| 5 | Resize a 1 columna con un carácter ancho | Mínimo de 2 columnas en `TerminalSession`, también al crearla | `a_session_is_never_narrower_than_two_columns`. No reproduje el cuelgue, como se pidió |
| 6 | El pegado solo quitaba `ESC` | Quita `ESC`, U+009B y `ctrl+c` | `pasted_text_cannot_close_the_bracket_or_interrupt` y, por PTY, `a_hostile_paste_stays_inside_its_brackets` |
| 7 | La regla D-19 aceptaba `tauri-app/src/...` y un `app/src/` anidado | La ruta tiene que empezar por `app/src/` | `a_feature_door_exists_only_in_the_app_crate`, con dos fixtures `bad`. Comprobé que falla con la regla anterior |
| 8 | La guarda D-13 no veía shells por nombre | Regla `shell-by-name` y entorno vacío en los tests de la app | Tres fixtures `bad` (`zsh`, `/usr/bin/env bash -l`, `/opt/homebrew/bin/fish`) y uno `good` con las palabras como datos |
| 9 | OSC 4/10/11 respondía con la paleta por defecto | Responde con el color que fijó el programa | `a_color_query_returns_the_color_the_program_set` |
| 10 | Listas de bloqueo cortas | `telemetry`, `telemetry_events`, `reqwest_client`, `client`, `rustls`, `native-tls` y `tungstenite`, en `deny.toml` y en la guarda; comentario corregido a D-15; fecha de revisión en cada aviso ignorado | El test de la propia guarda cubre los nombres nuevos y que `http_client` no salta |
| 11 | `push_cell` cuadrático y clon doble al pintar | Marca `one_per_cell` en cada tramo; el texto se copia una vez | Sin test: es rendimiento. La prueba 7 se volvió a medir |
| 12 | Historial | 5 000 líneas por terminal | Medido arriba: el pico entra en los 150 MB |

Tests añadidos a petición de la auditoría: DECCKM de punta a punta, cero repintados en reposo e informes de foco. Tests retirados: el de agrupación en tramos (pasó a comprobar la negrita celda por celda) y el que probaba `cat -v` en lugar del pegado.

Los tests de los puntos 1 y 2 se escribieron y se vieron fallar antes de tocar el código. Los de los puntos 4, 5, 6 y 9 necesitaban la API nueva y se escribieron junto con el arreglo. El del punto 7 se comprobó volviendo a poner la regla anterior.

Para probar de punta a punta, la sesión ganó tres operaciones (`paste`, `press`, `report_focus`) y `session.rs` se partió en `session.rs`, `emulator.rs` y `pump.rs` para no pasar de 300 líneas.

Documentación: D-17 ahora dice lo que cuesta cada decisión de teclado; la guía manual tiene el degradado de 24 bits, la prueba 2 dentro de Claude, el pegado hostil, la comprobación de huérfanos y los comandos para medir la prueba 7.

## Corrección tras la prueba manual: la app no pintaba texto

**Qué vio Jhon.** La ventana abría con los dos paneles, el borde de foco y un cursor, pero sin prompt ni texto. La entrada sí llegaba a los shells.

**Causa.** Un error mío en D-14: declaré `gpui_platform` con `default-features = false` y sin el feature `font-kit`, creyendo que era opcional. Sin él, gpui en macOS usa un sistema de texto vacío (`NoopTextSystem`): ninguna fuente y glifos de cero bytes. gpui lo avisa por `log`, pero la app no tiene registrador. Además `paint.rs` descartaba el resultado de pintar el texto.

**Por qué no lo detecté.** No podía ver la ventana y ningún test pasaba por el sistema de texto de la plataforma. Di por buena la app porque arrancaba y no escribía errores.

| Cambio | Dónde |
|---|---|
| Feature `font-kit` activado | `Cargo.toml`; D-14 |
| Fuente git del fork de font-kit permitida por nombre | `deny.toml`; D-15 |
| Sonda sin ventana: la fuente existe y un glifo se rasteriza a píxeles | `terminal_view/font_probe.rs`, `lithemux-app --check-text` |
| Errores de pintado a stderr (el primero de cada proceso) y aviso al arrancar si no hay fuentes | `terminal_view/paint.rs`, `spike_window.rs` |

**Tests.**

- `the_app_binary_can_rasterize_the_terminal_font` (integración): ejecuta el binario con `--check-text`. **Lo vi fallar** sin el feature («font Menlo not found among 0 fonts») y pasar con él.
- `a_text_system_that_draws_nothing_fails_the_probe` (unitario): la sonda rechaza `NoopTextSystem`.

Estos tests prueban que el texto se convierte en píxeles, no que llegue a la pantalla: el atlas de glifos y el shader de sprites solo se ejercitan con una ventana, y gpui no ofrece en este `rev` una forma de leer el cuadro pintado sin su `test-support` (que usa una plataforma de prueba, no Metal). Esa última parte sigue dependiendo de los ojos de Jhon.

**No se añadió regla al lint de hallazgos.** El lint busca un patrón presente en una línea, y este bug es la ausencia de un feature; el test de integración lo cubre mejor.

**Mediciones repetidas con el binario corregido** (ahora sí rasteriza texto): reposo 65 MB (antes 59); pico durante el volcado de 52,5 MB de 141, 139 y 140 MB; después del volcado 103 MB; `cat` en 0,98 a 1,31 s por panel; paradas del hilo principal de 2 a 5 ms; cero conexiones (`lsof -nP -i`). La meta de 150 MB se sigue cumpliendo con poco margen.

**Sigo sin ver la ventana.** No tengo permiso de grabación de pantalla, así que no he comprobado con mis ojos que el texto aparece.

## Desviaciones del plan

| Desviación | Motivo |
|---|---|
| `cargo deny` evalúa solo los objetivos de macOS, permite la fuente git de Zed, dos licencias BSD e ignora tres avisos de «sin mantenimiento» | gpui no pasa sin eso; cada excepción es nominal (D-15) |
| La prueba de fronteras acepta `app/src/<feature>/mod.rs` como puerta | CLAUDE.md organiza la app por funcionalidad; la regla solo conocía `modules/` (D-19) |
| Protocolo de teclado de kitty apagado; `shift+enter` envía `ESC`+`CR` | alcance del spike (D-17) |
| La prueba de `cat` se hizo en los dos paneles a la vez | es el caso más duro y no costaba más |

## Qué quedó fuera

- Parpadeo del cursor, título de ventana por sesión, campana, OSC 52 (portapapeles desde el programa) e hipervínculos.
- Reporte de ratón al programa.
- Protocolo de teclado de kitty.
- Dibujo propio de los caracteres de caja: se pintan con la fuente (Menlo), así que puede haber rendijas entre líneas. Es lo primero que conviene mirar en la prueba 1.
- Fuente y tamaño configurables: fijos en Menlo 13.
- Integración con `core` (sesiones, hooks, categorías): es de la Fase 2.

## Riesgos detectados

1. **Sin verificación visual propia.** Explicado arriba.
2. **Tiempo de compilación.** Unos 5 minutos en limpio y 620 paquetes en `Cargo.lock`. El CI de GitHub tardará bastante más que en la Fase 0; no se ha ejecutado allí.
3. **gpui cambia rápido.** Entre el crate de crates.io y este `rev`, gpui se partió en `gpui` y `gpui_platform`. Subir de `rev` puede romper la app; hay que hacerlo a propósito y con la prueba manual.
4. **Tres crates sin mantenimiento** en el árbol de gpui (`ttf-parser`, `rustybuzz`, `chrono`), sin vulnerabilidades conocidas hoy.
5. **Cierre con pid reciclado.** Queda una ventana de microsegundos entre que el hijo es recogido y se anota la marca; en ese intervalo el cierre podría mandar la señal a un grupo ajeno.
6. **Costes de las decisiones de teclado** (D-17): `option` como texto anula los atajos meta de Claude (`option+p`, `option+t`); `shift+enter` como `ESC`+`CR` sale del modo inserción en vim; el borrado del menú de acentos puede llevarse un carácter real.
7. **Footprint en volcados grandes:** entra en la meta con poco margen (148 MB en una de tres corridas).
8. **`http_client` de Zed** está en el árbol como interfaz sin transporte. Si un `rev` futuro le pone un cliente por defecto, la guarda de `Cargo.lock` saltará (D-16).
