# H-011 · Un `claude_path` que no sirve, reemplazado por otro `claude`

**Fase:** bloque 3B de la fase 3, en la prueba de la planificadora; se arregló al inicio del 3C.

**Qué pasó.** Si el ajuste `claude_path` no era ejecutable (una ruta que ya no existe, o un valor guardado con comillas JSON, `"/ruta"`), el lanzador anotaba una advertencia que nadie veía y seguía con el `claude` del PATH. La app lanzaba un binario que el usuario no eligió, quizá de otra versión.

**Regla.** `claude_path` es una elección explícita: si está definido y no sirve, el lanzamiento falla con `LaunchError::ClaudePathUnusable`, que la tarjeta, el panel, `doctor` y `app.log` muestran. No se busca otro `claude`.

**Qué atrapa.** Un mensaje de `src/` que hable de `claude_path` y diga que se usa otra cosa en su lugar: «instead», «looking in the PATH», «falls back», «falling back».

**Fuera de alcance.** Un respaldo que no deje ningún mensaje: ese lo cubren los tests `a_claude_path_that_is_not_executable_is_an_error_and_no_other_claude_is_looked_for` (casos de uso) y `session_host_claude_path_test` (integración, con un shell que sí encuentra otro `claude`).

**Cómo se arregla un hallazgo.** Devolver `LaunchError::ClaudePathUnusable` en vez de seguir buscando.
