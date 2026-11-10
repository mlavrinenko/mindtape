#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Unify mt add and mt import destinations under --dir",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("mt", "cli", "stateless", "no-repro"),
  x-goals: (
    product: verdicts(
      (
        self-describing: helps[one project-root-relative flag is easier to explain and complete than two overlapping ones],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (:),
      ..("sound-codebase", "docs-match-code"),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 10),
)

== AS-IS

+ Run `mt add --to fixes "Remove the legacy import"` from a subdirectory.
+ 🔴 `--to` resolves under `--into`, which itself resolves against the current directory, so the same command writes to a different directory depending on where it runs.
+ `--into .` is a sentinel for the first root, and `mt import` carries the same two overlapping flags.


== TO-BE

- 🟢 `mt add --dir <PATH>` and `mt import --dir <PATH>` name one destination. A relative path resolves from the project root, and an absolute path is taken as-is.
- 🟢 Without `--dir`, `mt add` uses its configured destination and `mt import` keeps each record's `file` path, as today.
- 🟢 `--into` and `--to` are removed. The refusal names `--dir`, and every docs example uses it.

```
$ mt add --dir truth/tasks/fixes "Remove the legacy import"
wrote truth/tasks/fixes/remove-the-legacy-import.typ
```


== Changes

- `crates/mindtape-cli/src/cmd/add/target.rs`: resolve one project-root-relative destination, remove the base-plus-subfolder split, and update `cmd/check/templates.rs` and `write_guard::Mode::Add`'s hint to `--dir`.
- `crates/mindtape-cli/src/cmd/import/place.rs`: send the batch to `--dir` when given, else keep the record `file` path.
- `crates/mindtape-cli/src/cmd/add/mod.rs`: replace the two flags with `--dir`, with a hint on the old names.
- `crates/mindtape-cli/src/cmd/import/mod.rs`: replace `--into` and `--to` with `--dir`.
- `crates/mindtape-cli/tests/it/add_e2e.rs`, `crates/mindtape-cli/tests/it/add_guard_e2e.rs`, `crates/mindtape-cli/tests/it/import_e2e.rs`: a run from a subdirectory landing in the root-relative folder, and the old flags refused.
- `www/content/templates.typ`, `www/content/cli/add.typ`, `www/content/cli/import.typ`: one destination flag in every example.


== Alternatives

- Keep `--into` and `--to` and add `--dir` beside them: two ways to name one destination.
- Resolve `--dir` from the current directory: scripts keep the cwd-dependent bug this task removes.
- Add `--root` plus `--dir`: a root is already a path, so one flag covers both.


== Out of scope

- The default destination and per-template routing; this only changes how a caller names one.
- `mt import` with no `--dir`; it keeps the record's `file` path.
- A compatibility shim beyond a refusal naming `--dir`.


== Dogfood

This repo's `truth/tasks` is the first root, so `mt add` needs no flag. Every docs and skill example that names a directory switches to `--dir`.

