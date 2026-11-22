# H-001 · Un proceso lanzado sin el filtro de entorno

**Fase:** prueba manual de la Fase 1; corregido en la §2.0 (punto 19).

**Qué pasó.** La app se lanzó desde una sesión de Claude Code dentro de cmux. El `claude` que arrancó en su terminal heredó `CLAUDE_CODE_CHILD_SESSION`, se creyó anidado y **no guardó el transcript**: la conversación se perdió.

**Regla.** En el código de producción (`crates/*/src`) nadie construye un proceso con `Command::new` ni con `CommandBuilder::new`. Solo dos archivos pueden, y los dos aplican el filtro de `core::platform::process::session_markers`:

- `crates/core/src/platform/process/session_markers.rs`: `command_without_session_markers`, de donde salen todos los procesos de `core` (el lanzador, las sondas del shell y de `claude`, la prueba en seco del hook).
- `crates/app/src/terminal_view/pty.rs`: el proceso de cada terminal, cuyo entorno arma `child_env`.

**Cómo se arregla un hallazgo.** Usar `command_without_session_markers(programa)` en vez de `Command::new(programa)`. Si de verdad hace falta otro punto de lanzamiento, tiene que aplicar el mismo filtro y entrar en `exclude` con su justificación aquí.

**Esta regla es solo la primera línea.** Es un patrón de texto, rápido, para el pre-commit. La auditoría de la §2.0 lo esquivó de cinco maneras (un alias, `.map(Command::new)`, otros constructores de `CommandBuilder`, un espacio antes del paréntesis, `<std::process::Command>::new`). **La comprobación que manda es un test**, `production_code_names_a_process_builder_only_where_the_filter_is` en `crates/core/tests/single_source_guards.rs`. Lee el código como tokens y no deja ni nombrar `Command` o `CommandBuilder` fuera de tres archivos: los dos de arriba y `platform/process/detached.rs`, que recibe un comando para desligarlo de la terminal y no construye ninguno. Los cinco casos de la auditoría son sus fixtures `bad`.

**El constructor devuelve un envoltorio.** `command_without_session_markers` entrega un `CleanCommand`, sin `envs` ni `env_clear` y con un `env` que no acepta claves marcadoras, para que lo filtrado no vuelva con una llamada inocente.

**Límite.** La regla mira dónde se construye el proceso, no si el filtro se aplica dentro de los dos archivos permitidos. Eso lo cubren los tests `inherited_session_test.rs` (CLI) y `child_env_test.rs` (app).
