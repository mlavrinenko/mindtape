#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Organize each crate's source as one folder per area with a mod.rs",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 6,
  kind: "refactor",
  tags: ("decision", "tooling"),
  estimate: (src-files: 0),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a module path names its file, and an area's internals stay private to its folder],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 7)[agreed: mod.rs folders, placements A],
)

== Problem

+ Run `ls crates/mindtape/src | wc -l`.
+ 🔴 It prints 229: about 15 areas spelled as flat prefixed files (`filter_compare.rs`), with half of them `_tests.rs` siblings.
+ Run `rg -n '#\[path = "filter_eval.rs"\]' crates/mindtape/src`.
+ 🔴 `filter.rs:430` loads `crate::filter::eval` from `src/filter_eval.rs`: about 20 child modules are kept flat with `#[path]`, beside self-named folders (`emit.rs` + `emit/`) and `mod.rs` folders (`render/mod.rs`). Four layouts, so a module path does not name its file.

== TO-BE

- 🟢 A module path is its file path. An area is a folder whose `mod.rs` opens with a `//!` map of its files; `self_named_module_files = "deny"` in `[workspace.lints.clippy]` refuses `foo.rs` + `foo/`.
- 🟢 Tests stay ejectest siblings (`filter/eval.rs` + `filter/eval_tests.rs`); a `mod.rs` ejects to a plain `mod tests;` in `tests.rs` (ejectest `--mod-rs-tests tests`).
- 🟢 `#[path]` names only a `*_tests*.rs` file or the `rrule` feature stub.

== Changes

- No code: the moves, the lint and the `#[path]` check are their own tasks. They place each file as below.
- Library areas: `config/` (sections, layers, packs, schema, merge, name_cycle), `rules/` (`[[tasks.rules]]`: config_rules, eval, typst, sites), `filter/` (+ expr, condition, condition_env), `validate/` (+ candidate, violation, severity), `body/` (body_gate, headers, content, gate, content_gate, density), `commit/`, `git/`, `graph/` (+ ready, dep_rank, children), `model/` (+ marker, fields, field_descriptor, numeric, canonical, handle, naming, task_spans), `loader/` (+ cache, derive, app_dirs), `link/`, `framework/`, `recur/`, `template/` (+ add_slots), `emit/` (+ status_value), `date/`, `import_map/`; `format.rs`, `fs_atomic.rs`, `sort.rs`, `suggest.rs`, `worktrees.rs`, `test_support.rs` stay at the root.
- CLI areas: `cli/` (cli, check, dump, command_class, task_ref, completion/), `cmd/<name>/` or `cmd/<name>.rs` per command, `render/` (+ output, breach, stdout); `git.rs`, `pre_write.rs`, `task_edit.rs`, `util.rs`, `test_util.rs` stay at the root.

== Out of scope

- Splitting into more crates: the modules form cycles (`filter` ↔ `config` ↔ `commit_*`), which crates forbid.
- Renaming modules beyond dropping the folder's prefix.

== Dogfood

Every later task cites the new paths, and `ls crates/*/src` reads as the map of areas.
