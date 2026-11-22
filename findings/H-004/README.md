# H-004 · La ruta del socket de la app, calculada en más de un sitio

**Fase:** auditoría del bloque 3 (§2.2–§2.3), reproducido por la planificadora.

**Qué pasó.** El socket donde la app escucha a los hooks era `<home>/app.sock`. La dirección de un socket Unix admite 103 bytes en macOS, y con un `LITHEMUX_HOME` largo (o un nombre de usuario largo) la app **no arrancaba**: «binding …/app.sock: path must be shorter than SUN_LEN».

**El arreglo.** `core::platform::paths::app_socket` decide dónde va: en el home si la ruta cabe y, si no, en `/tmp/lithemux-<uid>/<hash del home>.sock`, un directorio privado (0700, del usuario, no un enlace) que la app prepara antes de escuchar y que el hook comprueba antes de conectar.

**Regla.** El nombre `app.sock` solo aparece en `platform/paths/app_socket.rs`. La app y el hook son binarios distintos: si cada uno armara la ruta por su cuenta, con un home largo dejarían de encontrarse y los estados no se actualizarían, sin ningún error a la vista.

**Cómo se arregla un hallazgo.** Usar `Home::app_socket()` (o `app_socket_path()`).

**Límite.** Mira el nombre del archivo, no que quien conecte compruebe el directorio. Eso lo cubren `app_socket_test.rs` y los tests de home largo (`a_home_too_long_for_a_socket_still_hears_the_hooks`, `the_hook_reaches_an_app_whose_home_is_too_long_for_a_socket`, `the_app_starts_on_a_home_too_long_for_a_socket_address`).
