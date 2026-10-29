#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Serve the embedded prelude when the package is not installed",
  kind: "fix",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("mt",),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a CI runner or a fresh clone runs `mt check` without `mt install`],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 29),
)

== Problem

+ On a machine that never ran `mt install`, such as a CI runner, run `mt init --no-input`, then `mt add "Try it"`.
+ 🔴 `mt add` fails with `did not load: file not found (searched at …/typst/packages/local/mindtape/0.2.0/typst.toml)`, and `mt check` warns `can't find the prelude package`, though the binary embeds that very prelude for `mt install`.


== Environment

`mt` 0.3.0 from the flake, in a dev shell with an empty `HOME`.


== TO-BE

- 🟢 Every world `mt` builds resolves `@local/mindtape` at the version the binary embeds to that embedded copy when no `[[typst.local-packages]]` entry and no installed package has it.
- 🟢 An installed copy still wins, so `mt` and an editor read the same file once `mt install` ran.
- 🟢 Neither `mt add` nor `mt check` reports a missing prelude it can serve.


== Changes

- `typst-world`, the sibling crate: `World::with_file_provider`, so a host can wrap the disk provider. Release it.
- `crates/mindtape/src/loader/embedded.rs`: the red test first, then a provider that falls back to the embedded package.
- `crates/mindtape/src/loader/mod.rs`: `with_packages` installs it.
- `crates/mindtape-cli/src/cmd/init/mod.rs`: read the prelude and its version from the library, which now holds the copy.
- `crates/mindtape-cli/src/cmd/check/`: no missing-prelude warning for the embedded version.
- `www/content/quickstart.typ`: `mt install` serves the editor.


== Out of scope

- An import of another prelude version, which still needs its installed copy and keeps today's hint.


== Dogfood

This repo pins `lib/` under `[[typst.local-packages]]` and keeps reading it. canibump's and mmz's CI run `mt check` with no install.

