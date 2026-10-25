#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Resolve a pack's check and dict files from its directory",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.6,
    ease: 4,
  ),
  difficulty: 5,
  tags: ("mt", "gating"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a pack rule's Typst check and its word lists come with the pack],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 26),
  links: parent("share-config-between-projects-through-packs.typ")
    + depends-on("read-a-directory-pack-and-ship-its-templates.typ"),
)

== Problem

+ Put `check.typ` beside `team/pack.toml`, with a rule `check = { typst = "check.typ", function = "check-x", field = "x-x" }` and a `[dicts.words] path = "words.txt"`.
+ Add `extends = ["./team/pack.toml"]` and run `mt check`.
+ 🔴 Both paths resolve from the project root: `check.typ` "does not exist", since `CheckSpec` in `crates/mindtape/src/rules/mod.rs` carries only the path string and `rules/typst.rs` joins it onto the root.


== TO-BE

```toml
# team/pack.toml
[[tasks.rules]]
name = "x-is-valid"
check = { typst = "check.typ", function = "check-x", field = "x-x" }
```

- 🟢 A pack's `check.typst` and `[dicts]` `path` resolve from the pack directory.
- 🟢 A pack's check file is evaluated in a Typst world rooted at the pack directory, so it imports files of the pack and `@local` packages, never the project's.
- 🟢 A layer's own paths resolve from the project root, as before.


== Changes

- `crates/mindtape/src/rules/typst_tests.rs`: the red tests first: a pack check file that imports a sibling, a pack dict file, a project check file unchanged.
- `crates/mindtape/src/rules/mod.rs`: a `#[serde(skip)]` root field on `CheckSpec`.
- `crates/mindtape/src/config/layers.rs`: stamp that root on each rule from its pack origin in `build_config`.
- `crates/mindtape/src/rules/eval.rs`, `crates/mindtape/src/rules/typst.rs`: evaluate a check from its own root, not `rules_root`.
- `crates/mindtape/src/config/packs.rs`: rebase a pack dict's `path` onto the pack directory as an absolute path during the fold, since a dict keeps no origin after the merge and `dicts.rs` already reads a path outside the root.
- `www/content/packs.typ`: the paths a pack resolves.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- Rewrite a pack's paths relative to the project root: a fetched pack sits outside the root, where Typst can't reach it.


== Out of scope

- A check file that reads the project's own files, such as `truth/lib.typ` reading `truth/goals.typ`: it stays in the project.


== Dogfood

Not applicable: this repo's `goals` check reads its own goal lists, so it stays local.

