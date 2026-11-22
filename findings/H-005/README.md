# H-005 · Un error mostrado al usuario con el texto de `thiserror`

**Fase:** auditoría del bloque 3 (§2.2–§2.3).

**Qué pasó.** Los avisos de la barra lateral y del área de trabajo se armaban con `error.to_string()`. Ese texto está escrito en el código, en inglés, para la CLI y los logs: con la interfaz en español salía en inglés, contra «los textos van en archivos de recursos y nunca en el código» (PLAN §2.6).

**El arreglo.** `crates/app/src/i18n/problems.rs` convierte cada error en un `Message` (clave de `locales/*.toml` más valores) y `Texts::say` lo pone en el idioma elegido. Lo que no tiene texto propio (un fallo de la base de datos, por ejemplo) sale como «Algo salió mal: {detalle}».

**Regla.** En `crates/app/src` no se asigna a un `notice` el resultado de un `to_string()`.

**Cómo se arregla un hallazgo.** `self.texts.say(&(&error).into())`, y si el error es nuevo, darle su caso en `problems.rs` y su clave en los dos `locales`.

**Límite.** Solo ve esa forma. Que cada error tenga texto en los dos idiomas lo comprueba `every_problem_has_its_text_in_both_languages`; que no falte ninguna traducción, `every_text_is_translated`.
