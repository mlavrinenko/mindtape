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
  status: done(2026, 10, 26),
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

- 🟢 A pack's `check.typst` and `[dicts]` `path` resolve from the pack directory: a directory pack's folder, or the folder holding a pack file.
- 🟢 A pack's check file is evaluated in a Typst world rooted at the pack directory, so it imports files of the pack and `@local` packages, never the project's.
- 🟢 A layer's own paths, and a layer rule that replaces a pack's, resolve from the project root, as before.
- 🟢 A built-in pack has no directory, so it names no files.


== Changes

- `crates/mindtape/src/config/packs_tests_files.rs` (new): the red tests first: a pack check file that imports a sibling and not the project's file, a pack dict file, a layer's own paths unchanged, for a pack file, a directory pack and one outside the project. `crates/mindtape-cli/tests/it/rules_e2e.rs` gets the `mt check` one.
- `crates/mindtape/src/rules/mod.rs`: a `#[serde(skip)]` root field on `CheckSpec`, and the doc of `typst`.
- `crates/mindtape/src/config/pack_files.rs` (new): `rebase_dict_paths`, run on each pack in the fold, and `stamp_check_roots`, run in `build_config` from the rule origins. `PackOrigin::root` gives the directory.
- `crates/mindtape/src/rules/typst.rs`: evaluate and validate a check from its own root, not `rules_root`. `eval.rs` needs no change, since `CompiledCheck::new` reads the spec's root.
- `crates/mindtape/src/config/packs.rs`: one call to `rebase_dict_paths`, before the pack's rule arrays come out.
- `crates/mindtape/src/config/dicts.rs`: the doc of `path`. The reader already takes an absolute path.
- `schema/config.schema.json`: regenerate.
- `www/content/packs.typ`, `www/content/named-sets.typ`: the paths a pack resolves.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- Rewrite a pack's paths relative to the project root: a fetched pack sits outside the root, where Typst can't reach it.


== Out of scope

- A check file that reads the project's own files, such as `truth/lib.typ` reading `truth/goals.typ`: it stays in the project.


== Dogfood

Not applicable: this repo's `goals` check reads its own goal lists, so it stays local.

