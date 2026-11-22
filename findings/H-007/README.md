# H-007 · «madre» e «hijas» en vez de «sesión coordinadora» y «sesiones»

**Fase:** ronda de correcciones del bloque 4 (decisión de Jhon del 22-nov-2026).

**Qué pasó.** La sesión que coordina una categoría se llamaba «madre» y las demás, «hijas» (`mother`/`child` en el código, en la BD, en la CLI y en el contexto que recibe Claude). Se cambió a **sesión coordinadora** (`coordinator`) y **sesiones** de la categoría (`member`). Un cambio de vocabulario así vuelve a colarse con facilidad: en un texto nuevo, en un test, en una clave de `locales`.

**Regla.** En `crates/*/src`, `locales/` y el `README.md` no aparecen `mother`, `madre`, `hija`, `sister`, `hermana`, `Kind::Child`, `'child'`/`"child"` ni «child session».

**Excepción.** `platform/store/migrations.rs`: las migraciones 1 a 3 son el esquema tal como existió y la 4 convierte los valores viejos.

**Por qué no se prohíbe `child` a secas,** como pedía el encargo: es también la palabra para un proceso hijo (`std::process::Child`, `child.wait()`) y el método `.child(…)` con que gpui arma toda la interfaz. La regla atrapa las formas en que `child` nombra el tipo de sesión.

**Fuera de la regla, a propósito:** los reportes de las fases y las decisiones anteriores, que conservan el término de su momento (D-52), y los tests y fixtures de migración, que necesitan escribir los valores viejos.

**Cómo se arregla un hallazgo.** «coordinator»/«coordinadora» y «member»/«sesión»; para las demás sesiones de la categoría, «the other sessions»/«las demás sesiones».
