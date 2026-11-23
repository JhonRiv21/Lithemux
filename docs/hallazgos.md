# Lint de hallazgos

Cada bug que llega a un commit o a la revisión de una fase se convierte en una regla, para que no pueda volver sin que alguien lo note. Lo que Clippy ya cubre no va aquí.

## Cómo se corre

```sh
scripts/lint-findings            # todos los archivos del repo
scripts/lint-findings --staged   # solo lo que está en stage (lo usa el pre-commit)
scripts/lint-findings ARCHIVO... # archivos concretos
```

Códigos de salida: `0` limpio, `1` hay hallazgos, `2` una regla está rota o no había nada que revisar.

Sin argumentos, el lint lista los archivos con git. Si la lista sale vacía (no hay repositorio, por ejemplo) **falla**: no revisar nada no es lo mismo que no encontrar nada. La salida dice cuántos archivos revisó.

## Cómo es una regla

Una carpeta `findings/H-00N/` con:

| Archivo | Contenido |
|---|---|
| `pattern` | una expresión regular extendida (`grep -E`) |
| `paths` | opcional: regex que debe cumplir la ruta del archivo; por defecto `\.rs$` |
| `exclude` | opcional: regex de las rutas que la regla deja en paz (el único lugar donde el patrón es legítimo) |
| `bad.txt` | un ejemplo que la regla **debe** marcar |
| `good.txt` | un ejemplo que la regla **debe** dejar pasar |
| `README.md` | la fase en que apareció y el porqué |

Antes de revisar el repo, cada regla se prueba contra sus dos ejemplos. Si no marca su `bad.txt`, o marca su `good.txt`, o le falta un archivo, el lint se detiene con código `2`: una regla que no dispara es peor que no tenerla.

## Estado

| Regla | Fase | Qué impide |
|---|---|---|
| `H-001` | F1 (prueba manual), corregido en §2.0 | Lanzar un proceso sin quitarle los marcadores de sesión de Claude Code y de cmux |
| `H-002` | §2.0 | `unsafe` fuera de `platform/process/detached.rs`, su única excepción |
| `H-003` | §2.1 (verificación de la planificadora), corregido en §2.2 | Arrancar gpui fuera de `app_shell/run.rs`, es decir, una ventana sin el gestor de sesiones ni el candado de instancia única |
| `H-004` | auditoría del bloque 3 | Armar la ruta de `app.sock` fuera de `platform/paths/app_socket.rs` (un home largo dejaba a la app sin arrancar) |
| `H-005` | auditoría del bloque 3 | Mostrar al usuario el `to_string()` de un error en vez de un texto de `locales` |
| `H-007` | ronda de correcciones del bloque 4 | El vocabulario «madre/hijas» (`mother`, `child` como tipo de sesión, `sister`…) en el código, los textos o el README |
| `H-006` | prueba con teclas reales del bloque 3, corregido en el bloque 4 | Un atajo con `shift-` y una tecla que no es letra (`cmd-shift-]`): gpui lo recibe como `cmd-}` y no coincide nunca |
| `H-008` | contrato del repositorio en el bloque 3A de la fase 3 (bug de la fase 2) | Un INSERT de `categories` o `sessions` con su propia lista de columnas en vez de `COLUMNS`: la migración 4 sumó `collapsed` a la lectura y no a la escritura |
| `H-009` | auditoría de la ronda de correcciones del bloque 3B de la fase 3 | Una transacción de SQLite `DEFERRED` (`unchecked_transaction()`, `transaction()`, `BEGIN`): la importación leía fuera del lock lo que decidía qué escribir, y dos a la vez podían duplicar una conversación |
| `H-010` | prueba de la planificadora en el bloque 3B, corregido al inicio del 3C | Un estado de sesión para el usuario calculado con `host.phase` en vez de `shown_phase`: el panel seguía en «open elsewhere (pid N)» cuando la tarjeta ya había vuelto a hibernada |
| `H-011` | prueba de la planificadora en el bloque 3B, corregido al inicio del 3C | Un mensaje que dice que se usa otro `claude` en lugar de un `claude_path` que no sirve: la app lanzaba un binario que el usuario no eligió |

El funcionamiento del mecanismo se prueba en `crates/core/tests/findings_lint.rs` con una regla de muestra en un directorio temporal.
