#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Drop Mindtape's world builders and HVal converters that typst-world 0.5.7 covers",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  kind: "refactor",
  tags: ("substrate",),
  estimate: (src-files: 9),
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
      sound-codebase: helps[Mindtape stops keeping copies of a world constructor and HVal converters its substrate owns],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 28),
)

== Problem

+ Read `crates/mindtape/src/validate/candidate.rs`, `rules/typst.rs` and `model/canonical.rs`.
+ 🔴 Three sites build a disk world around an in-memory main source by hand, naming the id twice and `DiskProvider` explicitly, and each repeats the package-root and overrides policy that `loader::build_world` and `mt eval` repeat too.
+ 🔴 `hval_to_json` and `json_to_hval` convert `HVal` to and from JSON by hand, and `check_literal` writes Typst syntax by hand, though typst-world 0.5.7 has `serde` for `HVal` and `HVal::to_typst`.


== TO-BE

- 🟢 Every world with a generated main source comes from `World::with_main_source`, and one Mindtape helper applies the package root and overrides to every world Mindtape builds.
- 🟢 `HVal` goes to and from JSON through typst-world's `serde` feature, and the check probe's literal is `HVal::to_typst` except for dates, which Mindtape keeps writing as it chooses.


== Changes

- `crates/mindtape/Cargo.toml`: typst-world 0.5.7 with `serde`.
- `crates/mindtape/src/loader/mod.rs`: one helper for the package root and overrides; `build_world` uses it.
- `crates/mindtape/src/validate/candidate.rs`, `crates/mindtape/src/rules/typst.rs`: `with_main_source` and the helper.
- `crates/mindtape-cli/src/cmd/eval.rs`: the helper.
- `crates/mindtape/src/model/canonical.rs`, `loader/cache.rs`, `import_map/extras.rs`: serde in place of `hval_to_json` and `json_to_hval`.


== Out of scope

- `emit/format.rs`'s `hval_to_typ` and the check probe's date literal: Mindtape writes a date as it chooses, which is policy, not a converter typst-world owns.


== Dogfood

Every `mt` command in this repo loads its tasks through the cache's serde path and checks them through the probe worlds.

