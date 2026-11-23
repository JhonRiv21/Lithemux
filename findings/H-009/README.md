# H-009 · Una transacción de SQLite que no es `IMMEDIATE`

**Fase:** bloque 3B de la fase 3, en la auditoría de la ronda de correcciones.

**Qué pasó.** La importación de cmux guardaba sus categorías y sesiones con `unchecked_transaction()`, que en rusqlite es `DEFERRED`: la transacción toma el lock de escritura solo al primer INSERT. Lo que había que decidir antes (qué conversaciones ya estaban, qué slugs estaban libres) se leía fuera de ella. Dos importaciones a la vez, o la app y la CLI, podían leer lo mismo y agregar dos veces la misma conversación.

**Regla.** En el código de `src/`, toda transacción de SQLite es `IMMEDIATE`: `transaction_with_behavior(TransactionBehavior::Immediate)`, `Transaction::new_unchecked(conn, TransactionBehavior::Immediate)` o `BEGIN IMMEDIATE`. Lo que la transacción decide se lee dentro de ella.

**Variantes que atrapa:**

- `unchecked_transaction()` y `transaction()` de rusqlite, que son `DEFERRED`;
- `TransactionBehavior::Deferred` escrito a mano;
- `BEGIN` o `BEGIN DEFERRED` en SQL, en mayúsculas o minúsculas, con o sin `;`.

**Por qué no hay excepciones.** Una transacción que solo escribe tampoco pierde nada por ser `IMMEDIATE`: espera el lock al empezar en vez de a la mitad. Una que solo lee no necesita transacción.

**Cómo se arregla un hallazgo.** Cambiar la transacción por una `IMMEDIATE` y mover adentro las lecturas que deciden lo que se escribe, como `SqliteImportStore::atomically`.
