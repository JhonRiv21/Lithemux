# Lint de hallazgos

Cada bug que llega a un commit o a la revisión de una fase se convierte en una regla, para que no pueda volver sin que alguien lo note. Lo que Clippy ya cubre no va aquí.

## Cómo se corre

```sh
scripts/lint-findings            # todos los archivos del repo
scripts/lint-findings --staged   # solo lo que está en stage (lo usa el pre-commit)
scripts/lint-findings ARCHIVO... # archivos concretos
```

Códigos de salida: `0` limpio, `1` hay hallazgos, `2` una regla está rota.

## Cómo es una regla

Una carpeta `findings/H-00N/` con:

| Archivo | Contenido |
|---|---|
| `pattern` | una expresión regular extendida (`grep -E`) |
| `paths` | opcional: regex que debe cumplir la ruta del archivo; por defecto `\.rs$` |
| `bad.txt` | un ejemplo que la regla **debe** marcar |
| `good.txt` | un ejemplo que la regla **debe** dejar pasar |
| `README.md` | la fase en que apareció y el porqué |

Antes de revisar el repo, cada regla se prueba contra sus dos ejemplos. Si no marca su `bad.txt`, o marca su `good.txt`, o le falta un archivo, el lint se detiene con código `2`: una regla que no dispara es peor que no tenerla.

## Estado

El mecanismo arranca con **cero reglas**. Su funcionamiento se prueba en `crates/core/tests/findings_lint.rs` con una regla de muestra en un directorio temporal.
