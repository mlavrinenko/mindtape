#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Emit project imports when a writer rebuilds a task file",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.85,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("mt", "tooling", "tests", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[a task rebuilt from JSON keeps the imports its body calls.],
      enforced-practice: neutral,
      config-over-code: helps[the imports are a project config list, not a special case in Rust.],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: helps[a task file rebuilt by a writer still evaluates on its own.],
      self-describing: neutral,
    ),
    process: (
      sound-codebase: helps[the emitter and the import path share one task-file contract.],
      docs-match-code: helps[the config page describes the emit list.],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 21),
  links: parent("rewire-goals-through-typst-markers.typ"),
)

== AS-IS

+ 🔴 `emit_task` writes the fixed prelude import and the body, and nothing before `task.with(…)` survives the model. A task body calling a project helper has no import in its rebuilt file.
+ Run `mt ls --json | mt import`. 🔴 A migrated task calls `/truth/prelude/2/lib.typ`, but the writer drops that import. `mt import` skips the record with `did not load: unknown variable: Helps`.
+ Put the import in the body. 🔴 A body-local `#import` binds, but it puts a code line in the prose body, and `mt import` would duplicate it on every round-trip.

== TO-BE

- 🟢 `[tasks] emit-imports = ["/truth/prelude/2/lib.typ"]` lists root-absolute Typst modules. The writer emits `#import "<path>": *` after the prelude import and before the body.
- 🟢 `emit_task_with(task, imports)` writes each entry once, in order. `emit_task` stays as the no-import wrapper. The public callers keep compiling.
- 🟢 The hardcoded `#import "@local/mindtape:0.4.0": *` line stays first, and the writer appends the configured list after it. A project with no `emit-imports` writes byte-for-byte the file it writes today.
- 🟢 A rebuilt task with a project helper evaluates, and `mt ls --json | mt import` round-trips a marker-bearing task.
- 🟢 The key is project policy, not a Rust special case. A project without it writes the same file it writes today.

```toml
# inside the existing [tasks] table
emit-imports = ["/truth/prelude/2/lib.typ"]
```

== Changes

- `crates/mindtape/src/config/tasks.rs`, `crates/mindtape/src/config/tasks_tests.rs`: the `emit-imports` list and a root-aware `validate_emit_imports` that refuses a non-root path, `..`, a non-`.typ` file or a missing module. `crates/mindtape/src/config/layers.rs` calls it from `build_typed`, because `Config::validate` is rootless. A missing module fails every command that builds the typed project config. `mt config show` merges raw config first.
- `crates/mindtape/src/emit/mod.rs`: add `emit_task_with`, emit `#import "<path>": *` for each import after the prelude and before the blank line, and fix the module doc that calls the emitter pure and config-blind.
- `crates/mindtape/src/lib.rs`: re-export `emit_task_with`.
- `crates/mindtape-cli/src/cmd/import/write_tests.rs`: the call site that uses the new wrapper.
- `crates/mindtape-cli/src/cmd/import/mod.rs`: call `emit_task_with` with the configured imports.
- `schema/config.schema.json`, `schema/output/config-show/1.schema.json`, `www/generated/**`: regenerate after the new key.
- `crates/mindtape/src/emit/tests.rs`, `crates/mindtape-cli/tests/it/import_e2e.rs`: a unit round-trip with a project helper, an empty list that writes today's file unchanged, and an `mt ls --json | mt import` case that keeps a fixture `/truth/prelude/2/lib.typ`, evaluates the helper and passes `mt check`.
- `www/content/config.typ`: document the key and the `: *` wildcard.
- `CHANGELOG.md`: an `Added` entry for `emit_task_with` and the config key.

== Alternatives

- Store the imports on `Task`. Rejected: the model then carries a source header it otherwise never edits.
- Hardcode the goal prelude path in the emitter. Rejected: that is the goal hardcoding this milestone exists to avoid.
- Keep a body-local import. Rejected: it binds, but it puts a code line in the prose body and the emitter would duplicate it.

== Out of scope

- Preserving arbitrary pre-`show` imports or comments. The list is explicit.
- Parsing a task file to discover its imports.
- The marker field and the goal DSL. The earlier leaves.

The migration leaf sets `.mindtape/config.toml` once `/truth/prelude/2/lib.typ` exists, because `mt import` validates the emitted file.

== Dogfood

`mt ls --json | mt import` rebuilds a task that calls `#Helps`, and the next `mt check` reads its markers.
