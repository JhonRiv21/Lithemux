# H-002 · `unsafe` fuera de su único lugar

**Fase:** §2.0 (regresión del punto 1, encontrada por la planificadora al probar con una TTY real).

**Qué pasó.** Para poder matar a la sonda `$SHELL -lic` con todo lo que dejara atrás, se la puso en su propio grupo de procesos. Con una terminal de verdad ese grupo queda en segundo plano; el shell interactivo intenta tomar la terminal, el kernel lo detiene y la sonda agota su plazo. `lithemux` caía entonces al shell no interactivo y abría un `claude` más viejo. Los tests no lo vieron porque corrían sin terminal.

**El arreglo** exige que el hijo llame a `setsid` entre `fork` y `exec`, y en Rust estable eso solo se puede con `CommandExt::pre_exec`, que es `unsafe`. Es la única excepción del workspace, en `crates/core/src/platform/process/detached.rs` (D-27).

**Regla.** La palabra `unsafe` no aparece en ningún archivo bajo `crates/*/src`, sea `.rs` o no (el código se puede traer desde un archivo con otra extensión), fuera de ese archivo. `cli`, `hook` y `app` siguen además con `unsafe_code = "forbid"`; `core` lo tiene en `deny` para poder permitir esa función.

**Cómo se arregla un hallazgo.** No usar `unsafe`. Si hiciera falta otra excepción, se decide con la planificadora y se documenta como decisión antes de tocar `exclude`.
