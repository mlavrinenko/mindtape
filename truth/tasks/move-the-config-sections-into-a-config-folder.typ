#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Move the config sections into a config folder",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  kind: "refactor",
  tags: ("tooling",),
  estimate: (src-files: 47),
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
      sound-codebase: helps[the module path names the file, and the area's internals can go private to its folder],
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

== Problem

+ Run `ls crates/*/src`.
+ 🔴 `config.rs`, every `config_*.rs` apart from the `[[tasks.rules]]` four, `merge.rs`, `name_cycle.rs`, `defaults.toml`, `packs/`, `dicts/` sit flat at the crate root, so the module path does not name the file and nothing marks the area's internals.

== TO-BE

- 🟢 `crates/mindtape/src/config/`: `mod.rs` (from `config.rs`) and one file per section with its prefix dropped (`config_ls.rs` → `config/ls.rs`).
- 🟢 Each moved file's tests move beside it (`x_tests.rs`); a `mod.rs` loads a plain `mod tests;` from `tests.rs`.
- 🟢 `mod.rs` opens with a `//!` map of the folder's files.
- 🟢 `just check` passes; the commit is moves plus path edits, so `git log --follow` traces every file.

== Changes

- The moved files: `git mv`, then `mod`/`use`/`#[path]`/doc-link paths. A cross-folder caller gets `pub(crate)`, never an alias at the old path.
- `outdatty.yaml`, `crap-baseline.json` (`just crap-baseline`) and every comment or doc that cites a moved path.
- `include_str!` paths for `defaults.toml`, `packs/` and `dicts/` move with them; `www/generate-facts.sh:143` and `.mmz/config.yaml:213` follow.

== Out of scope

- Renames beyond dropping the folder's prefix.
- Tightening visibility past what compiles; a later sweep owns that.
- Done tasks citing old paths: they are history.

== Dogfood

`ls` of the crate's `src/` shows the area as one folder.
