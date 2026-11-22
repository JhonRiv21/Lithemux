# H-008 · Un INSERT con su propia lista de columnas

**Fase:** bloque 3A de la fase 3; el bug venía de la fase 2.

**Qué pasó.** `SqliteCategoryRepository::insert` escribía a mano sus columnas, y cuando la migración 4 agregó `collapsed` nadie la sumó al INSERT. La lectura usaba la constante `COLUMNS`, que sí la tenía. Una categoría creada como plegada se guardaba desplegada, y nada fallaba porque la app siempre crea categorías desplegadas. Apareció al escribir el contrato del repositorio para la columna nueva `notify` (migración 5): la categoría insertada con `notify = false` volvía con `true`.

**Regla.** En `infrastructure/sqlite_*_repository.rs`, un INSERT en `categories` o `sessions` que escribe sus columnas a mano se escribe con la constante: `INSERT INTO tabla({COLUMNS})`. Así leer y escribir no se separan.

**Variantes que atrapa.** La auditoría de la ronda de correcciones mostró que la primera versión se esquivaba sin querer. Ahora atrapa:

- con espacio antes del paréntesis: `categories (id, …`;
- en minúsculas o con mayúsculas mezcladas, incluido el nombre de la tabla (SQLite no distingue);
- `INSERT OR REPLACE` / `OR IGNORE`;
- la lista de columnas que empieza en la línea siguiente.

`grep -E` no tiene `(?i)`, así que la insensibilidad a mayúsculas va con clases de caracteres. Cada variante está en `bad.txt`.

**Por qué nombra las tablas.** Solo `categories` y `sessions` tienen `COLUMNS`. `settings(key, value)` y `workspace_layout(id, layout)` tienen dos columnas fijas, y `grep -E` no tiene forma de decir «cualquier tabla salvo estas». Si otra tabla gana su `COLUMNS`, se suma al patrón.

**Cómo se arregla un hallazgo.** `&format!("INSERT INTO tabla({COLUMNS}) VALUES (?1, …)")`, con los parámetros en el orden de `COLUMNS`.
