#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "resolve the Typst package from the checkout, not a global symlink",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("tooling",),
  estimate: (files: 2),
  links: related(
    "add-a-just-fork-recipe-that-stands-up-a-ready-worktree.typ",
  )[the recipe whose first real run surfaced this],
  status: proposed(2026, 8, 17),
)

== Summary

`@local/mindtape` resolves through one machine-global symlink —
`~/.local/share/typst/packages/local/mindtape/<version>`, pointed at whichever
checkout last ran `just install-lib`. Every worktree and every second clone
therefore reads the MAIN tree's `lib/prelude.typ` rather than the one beside it,
silently. A fork that edits the DSL has its `mt ls` / `mt check` — and the
`check-examples` gate — validate task files against the prelude it did not
change, and running `just install-lib` in the fork fixes it there by breaking it
everywhere else: one link, every checkout.

Found by running `just fork` for the first time and reading where the prelude
actually came from. That recipe now prints the resolved path as a note, which
makes the behaviour visible without making it correct.

== Scope

- Point the dogfood project at its own tree with a `[[typst.local-packages]]`
  entry in `.mindtape/config.toml` — `crates/mindtape/src/loader.rs` already
  resolves those, and an entry wins on a name clash — so every checkout reads
  the `lib/` beside it and AGENTS.md's claim that `mt` runs against the in-tree
  `lib/` becomes true by construction rather than by whoever ran `install-lib`
  last.
- Decide what `examples/learn-piano` does, which is the harder half. Its whole
  value is being a realistic user project, and a `../../lib` override is a
  fixture that stops resembling what it documents — but without one,
  `just check-examples` in a worktree keeps reading the main tree's prelude, and
  a prelude change is exactly when that gate is worth trusting.
- Check what the override does to `mt install` and the install hint
  (`crates/mindtape-cli/src/install_hint.rs`), both of which point at the global
  path today, and to a contributor who has never run `just install-lib` at all.
- Drop the note from `.just/scripts/fork.sh` once resolution is per-checkout.
  It exists only to make today's behaviour visible.
