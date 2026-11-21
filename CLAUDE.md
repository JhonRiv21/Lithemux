# lithemux

Gestor ligero de sesiones de terminal y Claude Code, sin telemetría, con categorías madre/hijas. El plan vivo está en `docs/PLAN.md`; los reportes, en `docs/fases/`, y las decisiones, en `docs/decisiones.md`.

## Reglas

- **Nunca `git commit` ni `git push`.** Deja los cambios en stage; Jhon los ejecuta.
- **Una fase a la vez.** Al cerrar cada una: checklist de cierre de `docs/PLAN.md`, reporte y espera.
- Código, identificadores, comentarios, mensajes de la interfaz en `locales/en.toml` y README: **en inglés**. Los documentos de `docs/`, en español.
- **Comentarios mínimos:** solo donde el código no se explica solo.
- Sin `unwrap()`/`expect()` en rutas de producción. El binario de hook nunca falla hacia Claude: siempre `exit 0`.
- **Cero telemetría y cero red** en la app. Respeta la lista de `cargo-deny`.
- Solo interfaces documentadas de Claude Code. No leas archivos internos de `~/.claude/` (ver principio 4 del plan).
- No copies código de cmux (GPL-3.0) ni del `terminal_view` de Zed (GPL).
- Los tests se escriben desde la especificación del plan, no desde la implementación.
- Si el plan no cuadra con la realidad, documenta y reporta en vez de improvisar.

## Arquitectura y organización del código

Arquitectura «screaming», con la forma del backend de Control Tower (`~/Linktic/control-tower`, solo de referencia, no copiar código): el árbol cuenta de qué trata la app. Cada dominio es un módulo con capas y la infraestructura compartida vive aparte.

### `crates/core`

```
crates/core/src/
├── lib.rs                      solo `pub mod modules; pub mod platform;`
├── modules/
│   ├── category/
│   │   ├── mod.rs              puerta del módulo: declara capas y expone SOLO lo que otros pueden usar (la API de application y los tipos de domain)
│   │   ├── domain/             tipos del dominio (un archivo por entidad) y políticas PURAS en archivos aparte (slug_policy.rs, ...). Sin E/S, sin rusqlite, sin std::fs/process
│   │   ├── application/        casos de uso (create_category.rs, ...) que orquestan domain + ports
│   │   ├── ports/              traits que la aplicación necesita (CategoryRepository, Clock, ...). Solo firmas y tipos
│   │   ├── infrastructure/     implementaciones reales de los ports (sqlite_category_repository.rs, ...)
│   │   └── error.rs            errores del módulo (thiserror)
│   ├── session/                misma forma; mother_policy.rs vive en session/domain
│   ├── context/                domain: render PURO del contexto (plantillas es/en); application: build_session_context
│   ├── launch/                 domain: LaunchPlan y reglas de argumentos; ports: ClaudeBinary, TranscriptLocator, SessionLock; infrastructure: implementaciones de macOS
│   ├── hook_event/             domain: tipos del stdin documentado y la transición de estado; application: handle_hook_event
│   └── settings/
└── platform/                   infraestructura compartida SIN conocimiento de los módulos
    ├── store/                  conexión SQLite, pragmas, migraciones
    ├── paths/                  rutas (directories, LITHEMUX_HOME)
    ├── process/                señales, spawn
    └── clock/
```

### Reglas

1. **Tipos, lógica y utilidades nunca mezclados.**
   - `domain/` contiene tipos y políticas puras, en archivos separados.
   - `application/` orquesta y no hace SQL.
   - `infrastructure/` traduce y no tiene reglas.
   - Prohibido `utils.rs`, `helpers.rs`, `common.rs` o `misc.rs`.
2. **Fronteras:**
   - `domain` no importa `application`, `infrastructure`, `rusqlite`, `std::fs` ni `std::process`.
   - Un módulo nunca importa el interior de otro (`modules::x::infrastructure`, `::application::<archivo interno>`); solo lo que expone su `mod.rs`.
   - `platform` nunca importa `modules`.
   - Los binarios entran a un módulo solo por su `mod.rs`.
   - Nada fuera de una carpeta `test/` o de `tests/` importa código de test.
3. **Sin barrels:**
   - Nada de `pub use x::*` ni de `prelude`.
   - El `mod.rs` de un módulo puede re-exportar nombres concretos de su API pública; es su puerta, como el `*.module.ts` de Control Tower.
   - Los `mod.rs` de las capas solo declaran submódulos.
4. **Binarios delgados.** `cli` y `hook` solo parsean, llaman a un caso de uso y formatean.
   - La CLI organiza sus comandos por dominio en `cli/src/commands/` y el formateo de salida en `cli/src/output/`.
5. **Errores:** `thiserror` en `core`, uno por módulo; `anyhow` solo en los binarios. Clippy con `unwrap_used` y `expect_used` en deny fuera de los tests.
6. **Tamaño:** más de ~300 líneas por archivo o ~50 por función es señal de que hay que partir por responsabilidad.
7. **Fase 2:** la app se organiza por funcionalidad en `crates/app/src/<feature>/` (`sidebar/`, `workspace/`, `terminal_view/`, `session_host/`, `settings_window/`, `i18n/`).
   - Dentro, la vista, el estado y las acciones van separados.
   - La vista llama a casos de uso de `core`.

## Pruebas (forma de Control Tower)

No se persigue cobertura. Se prueba lo que puede fallar de verdad, **desde la especificación de `docs/PLAN.md`, no desde la implementación**.

1. **Unitarias junto a la capa que prueban,** en una carpeta `test/` dentro de cada capa (`modules/session/domain/test/mother_policy_test.rs`, declarada con `#[cfg(test)] mod test;`). Nunca mezcladas en el archivo de producción.
   - Nombres que se leen como la regla: `fn only_one_mother_per_category()`.
   - Un comentario de una línea con la sección del plan que respaldan (`// PLAN §0.2`).
2. **Casos de uso con puertos en memoria:** implementaciones `InMemory*` de cada port, que cumplen el mismo contrato que la de SQLite (comentario de una línea que lo diga). Viven en `crates/core/tests/support/` o en un módulo `#[cfg(test)]`, nunca en el código de producción.
3. **Integración real** en `crates/<crate>/tests/integration/`:
   - SQLite real en un `LITHEMUX_HOME` temporal (con `tempfile`);
   - el binario `lithemux-hook` ejecutado de verdad con stdin JSON de fixtures (`assert_cmd`);
   - la CLI de punta a punta.
4. **Fakes de servicios externos** en `tests/fakes/`:
   - un `fake-claude` (script) que registra los argumentos y el entorno con que lo llamaron, y responde a `--version`;
   - los tests del lanzador y de `open` lo usan vía `claude_path`.

   **Ningún test ejecuta el `claude` real.**
5. **Prueba de arquitectura** en `crates/core/tests/arch_boundaries.rs`: revisa los `use` del código contra las reglas de fronteras. Con fixtures `tests/arch-fixtures/good` y `bad` que demuestran que cada regla dispara: el `bad` debe fallar nombrando cada regla y el `good` debe pasar.
6. **Guardas de fuente única:**
   - un test que falla si alguna ruta interna de `~/.claude/` aparece fuera de `launch/infrastructure/transcript_locator.rs` (la única lectura permitida, PLAN principio 4);
   - otro que falla si aparece un cliente HTTP o un SDK de telemetría en `Cargo.lock`.
7. **Lint de hallazgos:** `scripts/lint-findings` (o un test `tests/findings_lint.rs`). Cada bug que llegue a un commit o a la revisión de una fase se vuelve una regla `H-00N`:
   - con la fase en que apareció, el porqué, un ejemplo `bad` que debe marcar y uno `good` que debe dejar pasar;
   - antes de revisar el repo, cada regla se prueba contra sus dos ejemplos;
   - se arranca con el mecanismo y cero reglas;
   - lo que Clippy ya cubre no va aquí.
8. **Pre-commit rápido** en `.githooks/pre-commit`, activado con `git config core.hooksPath .githooks`: `cargo fmt --check` y el lint de hallazgos sobre lo que está en stage. Tests y Clippy completo van en el cierre de fase y en CI.
9. **Las pruebas se corren después del ÚLTIMO cambio,** incluidos los scripts y los hooks.

## Documentación

- `docs/PLAN.md`: el plan.
- `docs/fases/F<N>-reporte.md`: un reporte por fase, que reemplaza a PROGRESS.md.
- `docs/decisiones.md`: decisiones tomadas durante la ejecución, con fecha y porqué.

## Roles

- **Planificadora** (Opus): diseña el plan, revisa cada fase y redacta los encargos. No escribe código del repo.
- **Ejecutora** (Fable): implementa las fases del plan.
- **Auditor** (agente Opus 5.5 que lanza la planificadora al cierre de cada fase): revisa la fase sin leer antes el reporte de la ejecutora. Sus hallazgos confirmados se corrigen antes del commit.
