# H-003 · Una ventana que arranca sin el gestor de sesiones

**Fase:** §2.1 (encontrado por la planificadora al verificar el bloque con el `claude` real); corregido en la §2.2.

**Qué pasó.** La app tenía dos entradas. Con `--open` arrancaba el gestor de sesiones (`SessionHost`), que toma `app.lock` y escucha en `app.sock`. Sin argumentos abría la ventana vieja del spike, que lanzaba sus terminales por su cuenta: sin candado de instancia única (se podían abrir dos apps sobre el mismo home), sin candado por sesión y sin recibir los hooks.

**Regla.** En el código de producción solo `crates/app/src/app_shell/run.rs` pone en marcha gpui (`gpui_platform::application()` o `gpui_platform::headless()`). Esa función arranca primero el `SessionHost` y, si no puede (otra instancia ya corre), sale con código 1 antes de mostrar nada.

**Cómo se arregla un hallazgo.** No abrir otra aplicación gpui: lo nuevo se muestra desde `app_shell` (una ventana más, un modo más de `run`), que ya tiene el gestor.

**Límite.** Es un patrón de texto: mira dónde se arranca gpui, no que `run` siga arrancando el gestor antes. Eso lo cubren los tests `a_second_app_with_a_window_leaves_too` y `a_second_app_on_the_same_home_leaves_and_disturbs_nothing` (`crates/app/tests/integration/app_exit_test.rs`), que ejecutan el binario real. `--check-text` usa `gpui_platform::current_platform`, que no abre ninguna aplicación.
