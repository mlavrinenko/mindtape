#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Print what a pack update changes",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.7,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("mt", "gating", "cli"),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a pack update names every rule and `run` line it changes before the lock commits it],
      cheap-adoption: neutral,
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
    + depends-on("move-pack-sources-along-their-ref-with-mt-packs-update.typ"),
)

== Problem

+ Run `mt packs update acme` after a fix lands on `v1`.
+ 🔴 It prints the old and new commit only, and the lock's diff shows a new hash, so a reviewer can't see that the update changed a rule or added a `run` line.


== TO-BE

```console
$ mt packs update acme
acme v1: 3f2c1a9 → 8d01e44
  acme:commits  ~ rule commits.rules.changelog-with-product-change
                + on.flip close-gate: run "just check"
wrote .mindtape/packs.lock
```

- 🟢 Per pack, it prints each key, rule and `on.flip` entry added (`+`), changed (`~`) or removed (`-`), by name or dotted key.
- 🟢 Every new or changed `run` line prints in full, and still waits for `mt allow`.
- 🟢 A pack added to or dropped from the source prints as one `+` or `-` line.


== Changes

- `crates/mindtape/src/pack_sources/diff_tests.rs`: the red tests first, one per change kind.
- `crates/mindtape/src/pack_sources/diff.rs` (new): diff two resolved packs, comparing parsed values, not text.
- `crates/mindtape/src/pack_sources/mod.rs`: list the file in the header.
- `crates/mindtape-cli/src/cmd/packs/mod.rs`: print the diff under each moved source.
- `www/content/packs.typ`: the report.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- Print a text diff of the TOML: a reordered file would look like a change, while the parsed diff names what behaves differently.


== Out of scope

- Writing the diff into the commit message: the user copies what the review needs.
- Captured output in the docs: `capture-mt-packs-output-in-the-docs-from-a-local-git-fixture.typ`.


== Dogfood

Not applicable: this repo adopts only the built-in `mindtape:` packs.

