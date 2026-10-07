#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Move validation and body gating into validate and body folders",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  kind: "refactor",
  tags: ("tooling",),
  estimate: (src-files: 32),
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
+ 🔴 `validate.rs`, `validate_*.rs`, `candidate.rs`, `violation.rs`, `severity.rs`, `body_*.rs`, `gate.rs`, `content_gate*.rs` sit flat at the crate root, so the module path does not name the file and nothing marks the area's internals.

== TO-BE

- 🟢 `crates/mindtape/src/validate/`: `mod.rs`, `links`, `priority`, `recur`, `candidate`, `violation`, `severity`; `crates/mindtape/src/body/`: `mod.rs` (from `body_gate.rs`), `headers`, `content`, `gate`, `content_gate`, `density`.
- 🟢 Each moved file's tests move beside it (`x_tests.rs`); a `mod.rs` loads a plain `mod tests;` from `tests.rs`.
- 🟢 `mod.rs` opens with a `//!` map of the folder's files.
- 🟢 `just check` passes; the commit is moves plus path edits, so `git log --follow` traces every file.

== Changes

- The moved files: `git mv`, then `mod`/`use`/`#[path]`/doc-link paths. A cross-folder caller gets `pub(crate)`, never an alias at the old path.
- `outdatty.yaml`, `crap-baseline.json` (`just crap-baseline`) and every comment or doc that cites a moved path.
- Public paths change (`mindtape::body_gate`, `mindtape::content_gate`, `mindtape::gate`, `mindtape::candidate`), so `CHANGELOG.md` names them.

== Out of scope

- Renames beyond dropping the folder's prefix.
- Tightening visibility past what compiles; a later sweep owns that.
- Done tasks citing old paths: they are history.

== Dogfood

`ls` of the crate's `src/` shows the area as one folder.
