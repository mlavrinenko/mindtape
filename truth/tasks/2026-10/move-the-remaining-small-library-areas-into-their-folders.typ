#import "@local/mindtape:0.3.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "Move the remaining small library areas into their folders",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  kind: "refactor",
  tags: ("tooling",),
  estimate: (src-files: 35),
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
      sound-codebase: helps[the module path points at the file, and the area's internals can go private to its folder.],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 7),
  links: depends-on("move-the-filter-language-into-a-filter-folder.typ")
    + related(
      "organize-each-crates-source-as-one-folder-per-area-with-a-mod-rs.typ",
    )[carries out its layout],
)

== AS-IS

+ Run `ls crates/*/src`.
+ 🔴 These paths sit flat at the crate root. The pairs are `loader.rs` + `cache.rs`, `link.rs` + `link_scan.rs`, `framework*.rs` + `frameworks.toml`, `template*.rs` + `add_slots.rs`, and `emit.rs` + `status_value.rs`. The singles are `derive.rs`, `app_dirs.rs`, `recur*.rs` and `date*.rs`. The module path does not follow the file, and nothing marks the area's internals.

== TO-BE

- 🟢 `loader/`, `link/`, `framework/`, `recur/`, `template/`, `emit/`, `date/` under `crates/mindtape/src/`, each with a `mod.rs`. `emit/`, `recur/` and `import_map/` turn their self-named root into `mod.rs`.
- 🟢 Each moved file's tests move beside it (`x_tests.rs`). A `mod.rs` loads a plain `mod tests;` from `tests.rs`.
- 🟢 `mod.rs` opens with a `//!` map of the folder's files.
- 🟢 `just check` passes. The commit is moves plus path edits, so `git log --follow` traces every file.

== Changes

- The moved files: `git mv`, then `mod`/`use`/`#[path]`/doc-link paths. A cross-folder caller gets `pub(crate)`, never an alias at the old path.
- `outdatty.yaml`, `crap-baseline.json` (`just crap-baseline`) and every comment or doc that cites a moved path.
- `.cargo/mutants.toml`'s `**/recur_rrule_off.rs` becomes `**/recur/rrule_off.rs`. `lib.rs`'s `#[path]` for the `rrule` stub follows.

== Out of scope

- Renames beyond dropping the folder's prefix.
- Tightening visibility past what compiles. A later sweep owns that.
- Done tasks citing old paths: they are history.

== Dogfood

`ls` of the crate's `src/` shows the area as one folder.
