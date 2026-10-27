#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Drop the linecop workarounds that linecop 0.8.1 makes needless",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("tooling",),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the git hooks and just modules get a size cap like every other script],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[the size config carries no dummy base and no stale belief about linecop],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 28),
)

== Problem

+ Read `.config/linecop.yaml`: it leaves `.githooks/*` and `.just/*.just` uncapped, on the belief that linecop detects language by extension only and knows no Just.
+ 🔴 Both beliefs were wrong: tokei falls back to the shebang and knows `Just`. linecop 0.8.0 did ignore a `Bash` key, since it looked languages up by tokei's display name `BASH`, and it counted only languages named in `limits`, so the `docs/src/**` overrides needed a dummy `Typst: 1000` base.

== TO-BE

- 🟢 The dev shell carries linecop 0.8.1, which caps a language under any key spelling and lets an override's caps stand without a language base.
- 🟢 `.githooks/*` sit under a `Bash` cap and the Justfile and `.just/*.just` under a `Just` cap, both 300 like `Shell`.
- 🟢 `.config/linecop.yaml` has no `Typst` base, and its comments describe no workaround.

== Changes

- `flake.lock`: bump `qahq/linecop` to the v0.8.1 release commit.
- `.config/linecop.yaml`: add `Bash: 300` and `Just: 300`, drop `Typst: 1000`, and rewrite the comments that explained both workarounds.

== Rollback

- Revert the commit. The lock returns to linecop 0.8.0 and the config to its dummy base.

== Out of scope

- A real cap for every Typst file. `lib/prelude.typ` and the site sources lose the 1000-line ceiling that only existed to wake the overrides; picking a Typst cap is its own call.

== Dogfood

`just check` runs `lint::size`, which now holds the git hooks and just modules to a cap.
