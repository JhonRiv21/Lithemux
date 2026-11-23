# H-010 · Un estado para el usuario leído de la fase cruda del host

**Fase:** bloque 3B de la fase 3, en la prueba de la planificadora; se arregló al inicio del 3C.

**Qué pasó.** La tarjeta de la barra lateral calculaba su estado con `LiveSessions::shown_phase`, que suelta el «open elsewhere (pid N)» del host cuando ese proceso ya murió. La pestaña y el cuerpo del panel lo calculaban con `live.host.phase(...)`, la fase cruda. Cuando el proceso externo terminaba, la tarjeta volvía a «hibernada» y el panel seguía diciendo «open elsewhere (pid N)» hasta pulsar Abrir.

**Regla.** En `crates/app/src/`, lo que se muestra al usuario como estado de una sesión (`status_of`) se calcula con `shown_phase`, nunca con `host.phase`. `host.phase` sigue valiendo para decidir qué hace el host (arrancar, despertar), no para mostrar.

**Variantes que atrapa:** `status_of(host.phase(..))`, `status_of(live.host.phase(..))`, `status_of(self.host.phase(..))` y `status_of(&x.host.phase(..))`, con o sin espacios.

**Fuera de alcance.** Una llamada que rustfmt parta en varias líneas con `host.phase` en la siguiente, o la fase cruda guardada antes en una variable. Para lo primero, la regla de uso es una sola fuente: `LiveSessions::shown_phase`.

**Cómo se arregla un hallazgo.** Cambiar `host.phase(id)` por `shown_phase(id).as_ref()`.
