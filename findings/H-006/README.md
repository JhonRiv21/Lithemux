# H-006 · Un atajo con Shift sobre una tecla que no es una letra

**Fase:** prueba con clics y teclas reales del bloque 3 (la planificadora); corregido en el bloque 4.

**Qué pasó.** Cambiar de pestaña con «cmd+shift+]» no hacía nada, en ningún teclado. gpui nombra cada tecla por el carácter que produce: con Shift sobre una tecla que no es una letra, macOS informa el carácter ya desplazado y gpui descarta el Shift. La pulsación llega como `cmd-}`, y un atajo escrito `cmd-shift-]` no coincide nunca.

**Regla.** Ningún `KeyBinding::new` combina `shift-` con una tecla de un solo carácter que no sea una letra, esté el `shift-` donde esté entre los modificadores (`cmd-shift-]`, `shift-cmd-]`, `cmd-shift-alt-[`). Con letras (`cmd-shift-d`) y con teclas con nombre (`cmd-shift-right`) el Shift sí se conserva.

**Cómo se arregla un hallazgo.** Escribir el carácter que resulta (`cmd-}`), y dar además un atajo que no dependa de la distribución del teclado (letras, dígitos o flechas): en un teclado latinoamericano ese carácter puede necesitar Option.

**Límite.** El patrón no sabe de distribuciones. Que cada acción tenga un atajo independiente del teclado lo comprueba `every_action_of_the_plan_has_a_shortcut_that_needs_no_layout` (`crates/app/src/workspace/test/shortcuts_test.rs`), con las pulsaciones construidas como las construye gpui.
