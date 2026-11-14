#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Refuse reserved or ignored task destinations",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.85,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("mt", "cli", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the writer refuses a destination that every later command ignores.],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[the refusal states the path and the way out.],
    ),
    process: (
      sound-codebase: helps[the write path and the read path agree on what exists.],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  links: depends-on(
    "define-and-export-one-task-source-policy.typ",
    "discover-task-files-through-git.typ",
  ),
  status: done(2026, 11, 10),
)

== AS-IS

+ In a root-`.` project with an ignored `target/`, run `mt add --to target --name hidden --set title="Hidden task"` with a template that needs only a title.
+ 🔴 It writes a task that `mt ls` never shows. `mt import` and `mt recur --advance` can do the same:

```
$ mt add --to target --name hidden --set title="Hidden task"
wrote target/hidden.typ
$ mt ls
default view · 0 of 0 tasks
```


== TO-BE

- 🟢 `mt add`, `mt import`, and `mt recur` (spawn, mutate, skip, advance) refuse a destination `would_be_visible` rejects before creating anything.
- 🟢 The guard resolves symlinks and `..`, checks the final `--dedup` path, and `--no-verify` does not bypass it.
- 🟢 The refusal states the final path, why, and the mode's fix: another `--into`/`--to` for add/import, move or stop ignoring for recur.
- 🟢 Previews leave every file untouched. A destination outside every root keeps the warning.
- 🟢 The commands may mutate or skip an existing tracked-ignored task. They refuse a new sibling under the same ignored parent.

```
$ mt add --to target --name hidden --set title="Hidden task"
error: `target/hidden.typ` would be invisible; choose another `--into`/`--to`
$ test -e target/hidden.typ; echo $?
1
```


== Changes

- `crates/mindtape-cli/src/cmd/write_guard.rs`: one destination guard shared by every writer.
- `crates/mindtape-cli/src/cmd/add/target.rs`, `cmd/add/mod.rs`: call it before writing.
- `crates/mindtape-cli/src/cmd/import/write.rs`: call it for every record.
- `crates/mindtape-cli/src/cmd/recur/mod.rs`, `cmd/recur/advance.rs`: call it for spawn, mutate, skip and advance.
- `crates/mindtape-cli/tests/it/add_e2e.rs`, `crates/mindtape-cli/tests/it/import_e2e.rs`, `crates/mindtape-cli/tests/it/recur_e2e.rs`: the red tests first.


== Alternatives

- Warn only: the file is still written and still invisible. The trap then remains.
- Put ignored paths back into discovery: defeats the Git-visible contract.
- Guard every writer before the write: chosen, so there is no file to clean up.


== Out of scope

- Turning an outside-root destination into an error.
- A config escape hatch for deliberately ignored task locations.
- Content validation and `mt fix` scope.


== Dogfood

This project's `mt add` writes under `truth/tasks`, which is neither reserved nor ignored. The guard matters for root-`.` adopters.

