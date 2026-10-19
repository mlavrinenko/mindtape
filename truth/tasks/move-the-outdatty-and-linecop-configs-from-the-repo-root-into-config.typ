#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Move the outdatty and linecop configs from the repo root into .config/",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 45),
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
      sound-codebase: helps[each moved config is proven read at its new path, and the lock keeps its merge driver],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: wip(2026, 10, 19),
)

== Problem

+ Run `git ls-tree --name-only HEAD` in the repo root.
+ 🔴 It lists 35 entries. Three are one tool's config each: `outdatty.yaml`, `outdatty.lock` and `.linecop.yaml`. outdatty 0.9.0 and linecop 0.8.0 read a config from `.config/` with a `root:` that points back at the repo, but the dev shell pins outdatty 0.7.0 and linecop 0.7.0.


== TO-BE

- 🟢 The dev shell runs outdatty 0.9.0, linecop 0.8.0 and canibump 0.2.0, and `just check` passes on them.
- 🟢 The three files live in `.config/`, so the root lists 32 entries. A probe whose output depends on each config prints the same before and after the move, and prints something else without `root: ..`.
- 🟢 Each moved file is an input of the gate that reads it, the lock keeps its merge driver, and the commit-msg hook still asks for a `Docs-Unaffected:` footer.


== Changes

- `flake.lock`: qahq 0.8.0.
- `.config/outdatty.yaml`, `.config/outdatty.lock`, `.config/linecop.yaml`: the moved files, each config with `root: ..`.
- `.just/outdatty.just`, `Justfile`, `.just/scripts/*.sh`: the manifest and lock paths the scripts and the merge driver read.
- `.gitattributes`, `.mmz/config.yaml`, `.mindtape/config.toml`, `flake.nix`, `.githooks/commit-msg`: the new paths.
- `docs/src/contributing.typ`, `docs/src/contributing/*.typ`, `contrib/ask-refactoring*`, `.mindtape/templates/sweep-size.typ`, `www/*.sh`: the new paths, with the rendered docs.
- `crates/mindtape-cli/tests/it/outdatty_lock_merge_e2e.rs`: the lock and manifest at their new paths.


== Rollback

- Revert the move commit, then the bump. Each tool still reads a root config, so the old layout works again once the files are back.


== Out of scope

- Every other root file, `clippy.toml` and `rustfmt.toml` included. Each tool finds its config its own way and needs its own check.
- Old paths named in closed tasks, which record what was true when they closed.


== Dogfood

This repo's `just check` reads all three files from `.config/`, and the root lists three fewer entries.

