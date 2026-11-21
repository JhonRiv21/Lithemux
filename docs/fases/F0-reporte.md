# Fase 0 · Reporte

**Fecha:** 2026-11-21 · **Ejecutora:** Fable 5.2 · **Estado:** lista para commit; falta la prueba manual en cmux (criterio 2), que necesita a Jhon.

## Criterios de aceptación

| # | Criterio | Resultado |
|---|---|---|
| 1 | CI en verde en local: fmt, clippy `-D warnings`, test, build `--release`, `cargo deny check` | **PASA** |
| 2 | Prueba manual en cmux | **PENDIENTE** — preparada más abajo; la versión automatizada contra `fake-claude` pasa |
| 3 | Tiempo del hook, p95 < 20 ms | **PASA** — p95 = 11,2 ms |
| 4 | Detenerse y reportar | **PASA** — no se empezó la Fase 1 |

### 1 · CI en local

Ejecutado después del último cambio, con `rustc 1.100.0` en macOS 26.5.1 (Apple M5):

| Paso | Resultado |
|---|---|
| `cargo fmt --all --check` | sin diferencias |
| `cargo clippy --all-targets -- -D warnings` | sin avisos |
| `cargo test` | **185 pruebas, 0 fallas** |
| `cargo build --release` | ok (`lithemux` 2,9 MB, `lithemux-hook` 2,3 MB) |
| `cargo deny check` | advisories, bans, licenses y sources ok |
| `scripts/lint-findings` | 0 reglas, sin hallazgos |

El workflow de GitHub Actions (`.github/workflows/ci.yml`) corre los mismos pasos sobre `macos-latest`. **No se ha ejecutado en GitHub**: no hay push todavía.

Desglose de las 185 pruebas:

| Conjunto | Pruebas | Qué cubre |
|---|---|---|
| Unitarias de dominio (`src/modules/*/domain/test/`) | 53 | slugs, nombre de mensajería, política de madre, argumentos exactos, JSON de hooks, transiciones de estado, render del contexto (6 snapshots), claves de `config` |
| Casos de uso con puertos en memoria (`tests/use_cases/`) | 61 | categorías, sesiones, contexto, eventos de hook, resolución de `claude`, lanzador |
| Integración de `core` (`tests/integration/`) | 22 | SQLite real, contratos de los repositorios, `flock`, localizador de transcript con HOME temporal, rotación del log, `open` contra `fake-claude` |
| Integración del hook (`crates/hook/tests/`) | 10 | el binario real con stdin de fixtures |
| CLI de punta a punta (`crates/cli/tests/`) | 21 | todos los comandos contra `fake-claude` |
| Fronteras de arquitectura | 4 | fixture `good` pasa, `bad` dispara cada regla, el repo cumple |
| Guardas de fuente única | 7 | rutas internas de `~/.claude`, paquetes prohibidos en `Cargo.lock` y ningún test con un shell real o el `claude` real (D-13) |
| Lint de hallazgos | 7 | el mecanismo, con una regla de muestra |

Los siete puntos de PLAN §0.7 están cubiertos: unicidad de la madre, validación de slugs y nombres, `--session-id` vs `--resume` con HOME temporal, construcción exacta de los argumentos, render del contexto para madre/hija/sin madre, hook sin `LITHEMUX_SESSION_ID`, y hook con la BD bloqueada.

**Sobre la regla «tests desde la especificación»:** los tests se escribieron leyendo `PLAN.md` y `decisiones.md`, y cada uno cita la sección que respalda. Pero la misma sesión escribió el código y los tests, con la implementación en contexto, así que la independencia no es total. Para compensar se inyectaron cinco bugs a mano y se comprobó que la suite los detecta:

| Bug inyectado | Pruebas que fallaron |
|---|---|
| `--resume` y `--session-id` invertidos | 3 |
| `session-end` pisa `hibernated` | 2 |
| se permite una segunda madre | 2 |
| filtro de envoltorios apagado | 4 |
| el hook sale con código 1 | 4 |

### 3 · Tiempo del hook

`hyperfine -N --warmup 20 --runs 500` sobre el binario de release, con una base de datos real (categoría `demo`, 3 sesiones) y el stdin de los fixtures. Máquina con ~23 sesiones de `claude` abiertas y el swap casi lleno, es decir, el peor caso habitual.

| Evento | media | p50 | p95 | p99 | máx | RSS pico |
|---|---|---|---|---|---|---|
| `stop` (escribe en la BD) | 9,5 ms | 9,4 ms | **11,2 ms** | 16,8 ms | 43,8 ms | 3,2 MB |
| `session-start` (escribe y renderiza el contexto) | 9,3 ms | 9,3 ms | **11,1 ms** | 19,0 ms | 33,0 ms | 3,4 MB |
| `stop` sin `LITHEMUX_SESSION_ID` (no hace nada) | 4,7 ms | — | — | — | 8,6 ms | — |

Los 500 de cada serie salieron con código 0. La primera ejecución tras recompilar tarda ~400 ms: es la verificación de firma de macOS sobre un binario nuevo, ocurre una vez.

### 2 · Prueba manual en cmux (pendiente, necesita a Jhon)

Usa la base de datos real (`~/Library/Application Support/lithemux/`). Los binarios ya están compilados.

**Preparación** (una vez, en cualquier terminal):

```sh
alias lithemux=~/Personal/lithemux/target/release/lithemux
mkdir -p ~/Personal/lithemux-demo
lithemux doctor
lithemux cat add demo --name "Demo" --root ~/Personal/lithemux-demo
lithemux new demo/planificadora --role "Planifica, revisa y redacta prompts" --mother
lithemux new demo/ejecutora --role "Implementa las fases que define la planificadora"
lithemux new demo/documental --role "Levanta requerimientos y documenta"
lithemux config set context_language es
lithemux status
```

`doctor` debe decir que usa `~/.local/bin/claude` (2.1.330) y advertir que hay varias instalaciones.

**a) Abrir las tres**, cada una en una pestaña de cmux (repite el `alias` en cada pestaña o usa la ruta completa):

```sh
~/Personal/lithemux/target/release/lithemux open demo/planificadora
~/Personal/lithemux/target/release/lithemux open demo/ejecutora
~/Personal/lithemux/target/release/lithemux open demo/documental
```

**b) En la ejecutora:** `/list-agents` → deben aparecer `demo-planificadora` y `demo-documental` con esos nombres.

**c) En la ejecutora, pregunta:** «¿Quién es tu madre y quiénes tus hermanas?» → debe responder `demo-planificadora` y `demo-documental` sin que se lo hayas dicho. Dile además algo para recordar: «Recuerda la palabra clave: mandarina».

**d) Cierra las tres** con `/exit` y vuelve a abrirlas con los mismos tres comandos. En la ejecutora pregunta «¿Cuál era la palabra clave?» → debe responder «mandarina», y `/list-agents` debe seguir mostrando los mismos nombres.

**e) Candado:** con la ejecutora abierta, en otra pestaña:

```sh
~/Personal/lithemux/target/release/lithemux open demo/ejecutora
# esperado: lithemux: the session is already open (pid N)
~/Personal/lithemux/target/release/lithemux status demo
# esperado: demo-ejecutora ... open (pid N)
```

**Qué mirar de más** (riesgos que solo se ven con el `claude` real):
- que `ctrl+c` dentro de Claude interrumpa el turno y no mate la pestaña;
- que al cerrar la pestaña de cmux la sesión quede `ended` en `lithemux status` y el candado libre;
- que `tail ~/Library/Application\ Support/lithemux/logs/hook.log` no muestre errores.

**Limpieza:** `lithemux cat rm demo --yes` (las conversaciones de Claude no se borran).

Pega aquí las capturas de texto de b), c), d) y e) al terminar.

## Qué se hizo, por subfase

**0.1 Andamiaje.** `git init -b main`; workspace con `crates/core`, `crates/cli` (bin `lithemux`) y `crates/hook` (bin `lithemux-hook`); edición 2024; `rust-toolchain.toml` fijado a 1.100.0; `LICENSE` MIT; `README.md` en inglés con sección en español; `.gitignore`; CI en `.github/workflows/ci.yml`; `deny.toml`; `docs/fases/` y `docs/decisiones.md`. Andamiaje de pruebas completo: carpetas `test/` por capa, `tests/support` con puertos en memoria y contratos compartidos, `tests/fakes/fake-claude` y `fake-shell`, `tests/arch-fixtures`, test de fronteras, guardas de fuente única, lint de hallazgos sin reglas y `.githooks/pre-commit`.

**0.2 Modelo y almacenamiento.** SQLite en `~/Library/Application Support/lithemux/lithemux.db` (o `LITHEMUX_HOME`), `rusqlite` con `bundled`, WAL, `busy_timeout` de 250 ms, claves foráneas activas y migraciones por `PRAGMA user_version`. Esquema literal del plan, con el índice `one_mother_per_category`. El nombre de mensajería se calcula y se valida como único en todo el sistema al crear.

**0.3 Lanzador.** `plan_launch` construye el comando sin ejecutarlo. Resolución de `claude` según D-05. `--resume` si existe el transcript, `--session-id` si no. `-n`, `--settings` con los cinco hooks y `--model` cuando aplica. Entorno `LITHEMUX_SESSION_ID` y `LITHEMUX_HOME`. Candado `flock` con spawn + wait (D-03). Perfil `shell` con `$SHELL -l`.

**0.4 Hook.** Siempre sale con 0, nunca escribe a stderr (ni siquiera ante un pánico), no hace nada sin `LITHEMUX_SESSION_ID`, nunca crea la base de datos y registra los errores en `logs/hook.log` con rotación a 1 MB. Los cinco eventos de la tabla, más el seguimiento del `session_id` (D-01) y el aviso por `app.sock`.

**0.5 Contexto.** Render puro en `context/domain`, con plantillas en inglés y español, para madre, hija y categoría sin madre. Se arma desde la BD en cada `SessionStart`. Tope de 2.000 caracteres: si no cabe, lista menos sesiones y añade «… y N más». Tablero en `<home>/categories/<slug>/board.md`.

**0.6 CLI.** `cat add|list|rm`, `new`, `open`, `status`, `context`, `set`, `rm`, `config get|set|unset` y `doctor`, con `--json` en `cat list` y `status`.

**0.7 Pruebas.** Ver el desglose de arriba.

## Arquitectura

Forma hexagonal de `CLAUDE.md`: `modules/<dominio>/{domain,application,ports,infrastructure}` más `platform/{store,paths,process,clock}`. Las capas son módulos privados y cada `mod.rs` de módulo es la única puerta (D-10). La CLI cablea los adaptadores reales en `crates/cli/src/wiring.rs`.

Ningún archivo pasa de 300 líneas (el mayor de producción tiene 209). Tres funciones rondan o pasan las 50 líneas: `compose` (render del contexto), `handle_hook_event` y el `check_file` del test de fronteras.

Árbol final de `crates/` (sin `target/` ni snapshots):

```
crates/
├── cli/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── args.rs
│   │   ├── commands/
│   │   │   ├── category.rs
│   │   │   ├── config.rs
│   │   │   ├── doctor.rs
│   │   │   ├── launch.rs
│   │   │   ├── mod.rs
│   │   │   ├── session.rs
│   │   │   └── status.rs
│   │   ├── main.rs
│   │   ├── output/
│   │   │   ├── category.rs
│   │   │   ├── config.rs
│   │   │   ├── doctor.rs
│   │   │   ├── elapsed.rs
│   │   │   ├── mod.rs
│   │   │   ├── notices.rs
│   │   │   ├── session.rs
│   │   │   └── status.rs
│   │   ├── prompt.rs
│   │   └── wiring.rs
│   └── tests/
│       └── integration/
│           ├── categories_and_sessions_test.rs
│           ├── config_test.rs
│           ├── main.rs
│           ├── open_test.rs
│           └── sandbox.rs
├── core/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── modules/
│   │   │   ├── category/
│   │   │   │   ├── application/
│   │   │   │   │   ├── create_category.rs
│   │   │   │   │   ├── find_category.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── remove_category.rs
│   │   │   │   ├── domain/
│   │   │   │   │   ├── category.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── slug.rs
│   │   │   │   │   ├── slug_policy.rs
│   │   │   │   │   └── test/
│   │   │   │   │       ├── mod.rs
│   │   │   │   │       └── slug_policy_test.rs
│   │   │   │   ├── error.rs
│   │   │   │   ├── infrastructure/
│   │   │   │   │   ├── fs_board_store.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── sqlite_category_repository.rs
│   │   │   │   ├── mod.rs
│   │   │   │   └── ports/
│   │   │   │       ├── board_store.rs
│   │   │   │       ├── category_repository.rs
│   │   │   │       └── mod.rs
│   │   │   ├── context/
│   │   │   │   ├── application/
│   │   │   │   │   ├── build_session_context.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── resolve_language.rs
│   │   │   │   ├── domain/
│   │   │   │   │   ├── context_input.rs
│   │   │   │   │   ├── lang.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── render_context.rs
│   │   │   │   │   ├── role_text.rs
│   │   │   │   │   ├── templates/
│   │   │   │   │   │   ├── en.rs
│   │   │   │   │   │   ├── es.rs
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── phrases.rs
│   │   │   │   │   └── test/
│   │   │   │   │       ├── fixtures.rs
│   │   │   │   │       ├── mod.rs
│   │   │   │   │       ├── render_context_test.rs
│   │   │   │   │       └── role_text_test.rs
│   │   │   │   ├── error.rs
│   │   │   │   └── mod.rs
│   │   │   ├── hook_event/
│   │   │   │   ├── application/
│   │   │   │   │   ├── handle_hook_event.rs
│   │   │   │   │   ├── hook_ports.rs
│   │   │   │   │   └── mod.rs
│   │   │   │   ├── domain/
│   │   │   │   │   ├── hook_event.rs
│   │   │   │   │   ├── hook_input.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── session_start_output.rs
│   │   │   │   │   ├── test/
│   │   │   │   │   │   ├── hook_input_test.rs
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   ├── session_start_output_test.rs
│   │   │   │   │   │   └── transition_test.rs
│   │   │   │   │   └── transition.rs
│   │   │   │   ├── infrastructure/
│   │   │   │   │   ├── file_hook_log.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── socket_app_notifier.rs
│   │   │   │   ├── mod.rs
│   │   │   │   └── ports/
│   │   │   │       ├── app_notifier.rs
│   │   │   │       ├── hook_log.rs
│   │   │   │       └── mod.rs
│   │   │   ├── launch/
│   │   │   │   ├── application/
│   │   │   │   │   ├── inspect_claude.rs
│   │   │   │   │   ├── launch_ports.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── open_session.rs
│   │   │   │   │   ├── plan_launch.rs
│   │   │   │   │   └── resolve_claude.rs
│   │   │   │   ├── domain/
│   │   │   │   │   ├── child_env.rs
│   │   │   │   │   ├── claude_candidates.rs
│   │   │   │   │   ├── claude_plan.rs
│   │   │   │   │   ├── claude_version.rs
│   │   │   │   │   ├── hooks_settings.rs
│   │   │   │   │   ├── launch_environment.rs
│   │   │   │   │   ├── launch_plan.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── shell_plan.rs
│   │   │   │   │   └── test/
│   │   │   │   │       ├── claude_plan_test.rs
│   │   │   │   │       ├── claude_version_test.rs
│   │   │   │   │       ├── fixtures.rs
│   │   │   │   │       ├── hooks_settings_test.rs
│   │   │   │   │       ├── mod.rs
│   │   │   │   │       └── shell_plan_test.rs
│   │   │   │   ├── error.rs
│   │   │   │   ├── infrastructure/
│   │   │   │   │   ├── flock_session_locks.rs
│   │   │   │   │   ├── login_shell_probe.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── sibling_hook_binary.rs
│   │   │   │   │   ├── terminal_session_runner.rs
│   │   │   │   │   └── transcript_locator.rs
│   │   │   │   ├── mod.rs
│   │   │   │   └── ports/
│   │   │   │       ├── hook_binary.rs
│   │   │   │       ├── launch_notices.rs
│   │   │   │       ├── mod.rs
│   │   │   │       ├── session_locks.rs
│   │   │   │       ├── session_runner.rs
│   │   │   │       ├── shell_probe.rs
│   │   │   │       └── transcript_locator.rs
│   │   │   ├── mod.rs
│   │   │   ├── session/
│   │   │   │   ├── application/
│   │   │   │   │   ├── create_session.rs
│   │   │   │   │   ├── find_session.rs
│   │   │   │   │   ├── follow_claude_session.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── record_activity.rs
│   │   │   │   │   ├── remove_session.rs
│   │   │   │   │   └── update_session.rs
│   │   │   │   ├── domain/
│   │   │   │   │   ├── kind.rs
│   │   │   │   │   ├── messaging_name.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── mother_policy.rs
│   │   │   │   │   ├── profile.rs
│   │   │   │   │   ├── session.rs
│   │   │   │   │   ├── session_ref.rs
│   │   │   │   │   ├── state.rs
│   │   │   │   │   └── test/
│   │   │   │   │       ├── messaging_name_test.rs
│   │   │   │   │       ├── mod.rs
│   │   │   │   │       ├── mother_policy_test.rs
│   │   │   │   │       └── session_ref_test.rs
│   │   │   │   ├── error.rs
│   │   │   │   ├── infrastructure/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── sqlite_session_repository.rs
│   │   │   │   ├── mod.rs
│   │   │   │   └── ports/
│   │   │   │       ├── mod.rs
│   │   │   │       └── session_repository.rs
│   │   │   └── settings/
│   │   │       ├── application/
│   │   │       │   ├── configure.rs
│   │   │       │   ├── mod.rs
│   │   │       │   ├── read_setting.rs
│   │   │       │   └── write_setting.rs
│   │   │       ├── domain/
│   │   │       │   ├── config_policy.rs
│   │   │       │   ├── mod.rs
│   │   │       │   ├── setting_key.rs
│   │   │       │   └── test/
│   │   │       │       ├── config_policy_test.rs
│   │   │       │       └── mod.rs
│   │   │       ├── error.rs
│   │   │       ├── infrastructure/
│   │   │       │   ├── mod.rs
│   │   │       │   └── sqlite_settings_repository.rs
│   │   │       ├── mod.rs
│   │   │       └── ports/
│   │   │           ├── mod.rs
│   │   │           └── settings_repository.rs
│   │   └── platform/
│   │       ├── clock/
│   │       │   ├── mod.rs
│   │       │   └── system_clock.rs
│   │       ├── mod.rs
│   │       ├── paths/
│   │       │   ├── error.rs
│   │       │   ├── fs_error.rs
│   │       │   ├── home.rs
│   │       │   └── mod.rs
│   │       ├── process/
│   │       │   ├── capture.rs
│   │       │   ├── mod.rs
│   │       │   ├── shell.rs
│   │       │   ├── signals.rs
│   │       │   └── supervise.rs
│   │       └── store/
│   │           ├── connection.rs
│   │           ├── error.rs
│   │           ├── migrations.rs
│   │           └── mod.rs
│   └── tests/
│       ├── arch/
│       │   └── rules.rs
│       ├── arch-fixtures/  (good/ y bad/: 44 archivos .rs de ejemplo, no se compilan)
│       ├── arch_boundaries.rs
│       ├── fakes/
│       │   ├── fake-claude
│       │   └── fake-shell
│       ├── findings_lint.rs
│       ├── guard-fixtures/  (real-programs/good y bad: 8 archivos .rs de ejemplo, no se compilan)
│       ├── guards/
│       │   └── real_programs.rs
│       ├── integration/
│       │   ├── board_store_test.rs
│       │   ├── fake_claude_test.rs
│       │   ├── hook_log_test.rs
│       │   ├── main.rs
│       │   ├── session_locks_test.rs
│       │   ├── sqlite_contracts.rs
│       │   ├── store_test.rs
│       │   ├── temp_home.rs
│       │   └── transcript_locator_test.rs
│       ├── single_source_guards.rs
│       ├── support/
│       │   ├── contracts/
│       │   │   ├── category_repository_contract.rs
│       │   │   ├── mod.rs
│       │   │   ├── session_repository_contract.rs
│       │   │   └── settings_repository_contract.rs
│       │   ├── fake_shell_probe.rs
│       │   ├── in_memory_launch_ports.rs
│       │   ├── in_memory_store.rs
│       │   ├── mod.rs
│       │   ├── recording_ports.rs
│       │   └── sample_data.rs
│       └── use_cases/
│           ├── category_use_cases.rs
│           ├── context_use_cases.rs
│           ├── hook_event_use_cases.rs
│           ├── in_memory_contracts.rs
│           ├── launch_use_cases.rs
│           ├── main.rs
│           ├── resolve_claude_use_cases.rs
│           ├── session_use_cases.rs
│           └── settings_use_cases.rs
└── hook/
    ├── Cargo.toml
    ├── src/
    │   └── main.rs
    └── tests/
        ├── fixtures/
        │   ├── notification_permission.json
        │   ├── session_end.json
        │   ├── session_start_clear.json
        │   ├── session_start_startup.json
        │   ├── stop.json
        │   └── user_prompt_submit.json
        └── integration/
            ├── hook_events_test.rs
            ├── main.rs
            ├── never_in_the_way_test.rs
            └── seeded_home.rs
```

## Qué quedó fuera y por qué

- **CI en GitHub:** el workflow está escrito pero no se ha ejecutado; no hay push.
- **Prueba manual en cmux:** necesita a Jhon.
- **`claude_version` desde el hook:** el stdin de los hooks no trae la versión; la guarda `open` (D-02).
- **Empaquetado e instalación:** el plan no lo pide. Hoy se usa el binario desde `target/release/`, con `lithemux-hook` al lado.

## Desviaciones del plan

Todas aprobadas por la planificadora y detalladas en `docs/decisiones.md`: D-01 (seguir el `session_id`), D-02 (versión en `open`), D-03 (spawn + wait en vez de `exec`), D-04 (`idle_prompt` no es `waiting`), D-05 (`-lic` y filtro de envoltorios), D-06 (`lithemux config`), D-07 (slugs), D-08 (`cat rm` conserva el tablero), D-09 (guarda sobre `Cargo.lock`), D-10 (capas privadas), D-11 (el tiempo como parámetro) y D-13 (guarda contra programas reales en los tests). PLAN §0.3 y §0.6 ya reflejan D-03, D-05 y D-06.

Añadidos menores no pedidos, por si se quieren quitar:
- `lithemux status` muestra si la sesión está abierta y el pid (sale gratis del candado);
- `lithemux set --default-model` para volver al modelo por defecto;
- `CLAUDE_CONFIG_DIR` se respeta al buscar el transcript, porque es la variable documentada que mueve `~/.claude`.

## Hallazgos durante la ejecución

1. **`$SHELL -lc` resolvía un `claude` viejo** (2.1.280 del cask de Homebrew en vez del 2.1.330 nativo). Dio origen a D-05. `doctor` ahora lista las tres instalaciones de la máquina y advierte la diferencia.
2. **Un test llegó a ejecutar el `claude` real.** `open_explains_when_claude_cannot_be_found` usaba `/bin/sh` como shell del usuario con `PATH=/usr/bin:/bin`; el `sh` de login reconstruyó el PATH desde `/etc/paths.d` y encontró el `claude` de Homebrew, que se lanzó sin TTY y salió con error por falta de entrada. No hubo llamada a la API ni se tocó nada fuera del directorio temporal. Corregido con `tests/fakes/fake-shell`, que nunca ve el PATH real. Por decisión de la planificadora quedó además como guarda de fuente única (D-13): un test falla si el código de test menciona un shell real, define `SHELL` sin un fake o apunta al `claude` real desde un archivo que puede lanzar procesos. Se comprobó reintroduciendo el bug original: la guarda lo marca.
3. **Artefactos viejos tras la prueba de mutación.** Los cinco bugs se inyectaron copiando el archivo, editándolo y restaurando la copia con `mv`. La copia restaurada quedó con fecha anterior a la del artefacto compilado, cargo la dio por buena y la siguiente corrida de `cargo test` usó la biblioteca compilada **con el bug inyectado**: 3 pruebas fallaron con los fuentes ya correctos. Se resolvió con `touch` sobre todos los fuentes y una recompilación completa; el checklist final es posterior. Para repetir pruebas de mutación: restaurar con `git checkout` o hacer `touch` del archivo.
4. **Homebrew tiene un cask `claude-code@latest` desactualizado** en la máquina (2.1.280). No estorba a lithemux, pero conviene desinstalarlo: es lo que encuentra cualquier herramienta que use un shell de login no interactivo.

## Riesgos detectados

- **Sin probar con el `claude` real.** Por regla, ningún test lo ejecuta. Los flags (`--session-id`, `--resume`, `-n`, `--settings`, `--model`) se verificaron contra `claude --help` de 2.1.330, pero que el JSON de hooks se cargue, que el contexto llegue y que `/list-agents` muestre los nombres solo lo confirma la prueba manual.
- **`$SHELL -lic` ejecuta el `.zshrc` del usuario** en cada `open`. Bajo un pty tardó 0,6 s en total para `doctor`. Un `.zshrc` lento o que pida entrada agota los 3 s y cae al respaldo `-lc`, con advertencia. No se observó que zsh interactivo robe el control de la terminal, pero solo se probó en esta máquina.
- **`waiting` no se limpia hasta el `Stop`.** Sin `PreToolUse`/`PostToolUse` no hay evento cuando el usuario aprueba un permiso: la sesión figura `waiting` mientras sigue trabajando. Afecta al indicador de la Fase 2, no a la hibernación (que exige `idle`).
- **El estado puede quedar viejo.** Si la terminal muere sin `SessionEnd` (un `kill -9`, un cierre brusco), la BD sigue diciendo `idle` o `working`. Por eso `status` muestra aparte si la sesión está abierta, a partir del candado. La Fase 2 debería reconciliar ambos.
- **Dos sesiones pueden apuntar a la misma conversación** tras un `/resume` cruzado (consecuencia de D-01); el candado no cubre ese caso.
- **El filtro `shims` descarta instalaciones legítimas** de asdf o mise; hay que fijar `claude_path`.
- **Validaciones sin transacción.** La unicidad de slug y de madre tiene respaldo en índices de SQLite, pero la del nombre de mensajería entre categorías solo se comprueba en el caso de uso. Dos `lithemux new` simultáneos con nombres que colisionan (`a-b/c` y `a/b-c`) podrían pasar ambos.
- **`cargo` no está en el PATH** (D-12). Para compilar a mano: `export PATH=/opt/homebrew/opt/rustup/bin:$PATH`. El pre-commit lo resuelve solo.
- **La guarda de `Cargo.lock` es global.** En la Fase 1, gpui traerá clientes HTTP y habrá que acotarla a `core`, `cli` y `hook`.
- **El campo `repository` de `Cargo.toml`** apunta a `github.com/JhonRiv21/lithemux`, que es una suposición: el repo remoto todavía no existe.

## Para el commit

Todo está en stage con `git add -A`; no se hizo commit ni push. Se activó `core.hooksPath=.githooks` en la configuración local del repo: al hacer commit correrá `cargo fmt --check` y el lint de hallazgos.
