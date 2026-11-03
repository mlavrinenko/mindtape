#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add mt fix to rewrite unsupported prelude imports",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.85,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("mt", "cli", "tooling", "stateless"),
  estimate: (src-files: 5),
  x-goals: (
    product: verdicts(
      (
        cheap-adoption: helps[updating a project is one command, not a hand edit over every task file],
        enforced-practice: helps[`mt fix --check` gates an unsupported prelude pin in CI],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[the rewrite parses Typst, changes only the version string, and check mode writes nothing],
        docs-match-code: helps[the task-files page names `mt fix` where it explains the import],
      ),
      ..(),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 3),
  links: depends-on(
    "declare-and-ship-the-prelude-versions-each-mt-supports.typ",
  ),
)

== AS-IS

+ Take a project whose task files import `@local/mindtape:0.1.0`, or this repo before the 0.3.0 bump.
+ 🔴 `mt` reports each file as a missing package, or silently reads a leftover global 0.1.0, and no command rewrites only the pin. The update is a hand `sed` over every task, template and helper.
+ The 0.3.0 bump rewrote 1007 `.typ` files this way.


== TO-BE

- 🟢 `mt fix` runs every applicable fixer; `--list` names them and `mt fix prelude-import` runs one.
- 🟢 `prelude-import` rewrites a `@local/mindtape:<version>` only when this mt does not support it; a supported old pin is kept.
- 🟢 `--dry-run` prints every edit and writes nothing; `--check` exits 1 while an unsupported pin remains.
- 🟢 Default scope is `tasks.roots` plus template dirs; `--all` scans the whole project tree.
- 🟢 Typst parsing changes only the version string. A newer-than-current pin is refused with an upgrade hint.

```
$ mt fix --dry-run
kept    old-supported.typ  0.2.0
rewrite old.typ            0.1.0 -> 0.3.0
would rewrite 1 import; 1 kept

$ mt fix --check
error: @local/mindtape:0.1.0 is not supported; first seen in old.typ
note: supported: 0.2.0, 0.3.0
hint: run `mt fix` to rewrite it to @local/mindtape:0.3.0
```


== Changes

- `crates/mindtape/src/prelude_import.rs` and `src/lib.rs`: the red test first, parse import spans and rewrite only an unsupported version.
- `crates/mindtape-cli/src/cmd/fix/mod.rs`: `mt fix`, `--list`, `--dry-run`, `--check`, `--all`, and the per-fixer report.
- `crates/mindtape-cli/src/cli/mod.rs`, `cli/command_class.rs`, `cmd/mod.rs`: wire `fix` as a write command.
- `crates/mindtape-cli/src/cli/completion/mod.rs`: complete the fixer name.
- `crates/mindtape-cli/tests/it/fix_e2e.rs` and `tests/it/main.rs`: kept, rewritten, newer refused, `--all`, `--check`.
- `crates/mindtape-cli/src/cmd/install/hint.rs`: name `mt fix`.
- `www/content/task-files.typ`, `www/content/config.typ`, `www/content/cli/fix.typ`, `CHANGELOG.md`: document the command.

Contract: `mt fix` reads config with `load_config`, never tasks. It matches only a string-literal `#import` of exactly `@local/mindtape`, never `#include`, a comment or another package, replacing only the version and preserving every other byte. Default scope is `tasks.roots` plus project-owned `templates.dirs` under the root, recursively; pack dirs outside the root are skipped. `--all` walks the project tree, skipping `.git`, `target`, `.mindtape/cache` and symlinked directories. Symlinked files with an edit are skipped and reported. An older unsupported pin is rewritten to `PRELUDE_VERSION`. A project-local pin is never edited; when it declares the imported unsupported version, that import is reported unfixable. A newer pin, malformed spec, unreadable file or skipped symlink is also reported. Any of these aborts the write before a file changes, and `--check` or `--dry-run` exits 1. A supported old pin is kept and reported; current is silent. `--check` prints one diagnostic per unsupported version, first file and set. A write ends with `mt check`.

== Alternatives

- A `sed` recipe in the changelog: no Typst parse, no Windows, no check mode.
- Fold it into `mt check --fix`: a validator run in CI would write.
- `mt install --fix`: a machine-scoped command would edit the project.
- Rewrite every older pin: changes supported pins that already work.


== Out of scope

- Semantic changes inside task bodies.
- Bumping `lib/typst.toml`, `PRELUDE_VERSION` or release snapshots.
- Fixers beyond `prelude-import`; the command leaves room.


== Dogfood

This repo runs `mt fix --all --check` before tagging, so its tasks, examples and templates import a supported prelude.

