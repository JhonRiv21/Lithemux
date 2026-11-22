# lithemux

A lightweight manager for terminal and [Claude Code](https://code.claude.com) sessions, organized in categories with a **coordinator session** and the other sessions.

- **Light.** Idle `claude` processes are meant to be hibernated and resumed, not kept alive. The hook is a tiny binary that never blocks.
- **No telemetry.** lithemux makes no network calls at all: no analytics, no crash reports, no update checks. CI fails if such a dependency sneaks in.
- **Reliable.** State lives in SQLite (WAL). Conversations persist through `claude --resume`, not through processes that stay open.
- **Not a wrapper.** It launches *your* `claude`, resolved from your shell on every start, and only uses documented interfaces: CLI flags and hooks.

macOS first; Windows later. MIT licensed.

> **Status:** phase 0 — the core and the command line. The native app comes next (see `docs/PLAN.md`, in Spanish).

## How it works

A **category** groups the sessions of one project and has a root folder. Each **session** has a role. At most one session per category is the **coordinator**: it coordinates and keeps the category's board. Every session is started with `claude -n <category>-<session>`, so sessions can find and message each other by name, and a `SessionStart` hook tells each one who it is, who the coordinator session is and which are the other sessions.

## Build

```sh
cargo build --release
# target/release/lithemux and target/release/lithemux-hook must stay side by side
```

Requires the Rust toolchain pinned in `rust-toolchain.toml`.

## Usage

```sh
lithemux cat add demo --name "Demo" --root ~/code/demo
lithemux new demo/planner  --role "Plans, reviews and writes prompts" --coordinator
lithemux new demo/executor --role "Implements what the planner defines"

lithemux open demo/planner     # in one terminal tab
lithemux open demo/executor    # in another

lithemux status                # coordinator -> other sessions, state and last activity
lithemux context demo/executor # what that session is told at startup
lithemux doctor                # which claude would run, database, hook
```

| Command | What it does |
|---|---|
| `cat add <slug> --name <Name> --root <dir> [--color <c>]` | Create a category and its board |
| `cat list [--json]` · `cat rm <slug> [--yes]` | List or remove categories (the board file is kept) |
| `new <cat>/<slug> --role "<text>" [--coordinator] [--profile claude\|shell] [--cwd <dir>] [--model <m>]` | Create a session |
| `open <cat>/<slug>` | Run the session in this terminal; reopening resumes the same conversation |
| `status [<cat>] [--json]` | Tree of sessions with state and last activity |
| `context <cat>/<slug>` | Print the context the session receives |
| `set <cat>/<slug> [--role] [--coordinator [bool]] [--keep-alive [bool]] [--model <m> \| --default-model]` | Change a session |
| `rm <cat>/<slug> [--yes]` | Remove a session (its Claude transcript is never touched) |
| `config get\|set\|unset <key>` | `claude_path`, `context_language` (`en`\|`es`) |
| `doctor` | Check the installation |

Data lives in `~/Library/Application Support/lithemux/` (override with `LITHEMUX_HOME`).

## Development

```sh
git config core.hooksPath .githooks   # fast pre-commit: fmt + findings lint
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo deny check
```

No test runs the real `claude`: they use `crates/core/tests/fakes/fake-claude`.

---

## En español

**lithemux** es un gestor ligero de sesiones de terminal y de Claude Code, organizado en categorías con una **sesión coordinadora** y las demás sesiones.

- **Ligero:** las sesiones que no se usan se hibernan y se reanudan con `claude --resume`.
- **Sin telemetría:** no hace ninguna llamada de red.
- **Confiable:** el estado vive en SQLite; un reinicio no pierde conversaciones.
- **No envuelve a Claude Code:** lanza tu propio `claude` y solo usa interfaces documentadas.

Uso básico:

```sh
lithemux cat add demo --name "Demo" --root ~/codigo/demo
lithemux new demo/planificadora --role "Planifica, revisa y redacta prompts" --coordinator
lithemux new demo/ejecutora --role "Implementa lo que define la planificadora"
lithemux config set context_language es
lithemux open demo/planificadora
```

El plan y las decisiones del proyecto están en `docs/` (en español).
