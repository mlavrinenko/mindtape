#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Use American English in user-facing text",
  kind: "fix",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("cli", "docs"),
  estimate: (src-files: 40),
  x-goals: (
    product: (
      readable-tasks: helps[the docs, help and schema use one spelling, the one `--color` and `NO_COLOR` already use],
      implementable-tasks: neutral,
      enforced-practice: neutral,
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
  status: proposed(2026, 10, 15),
)

== Problem

+ Run `mt init --help`.
+ 🔴 It lists `--organisation`, while the same binary spells `--color` and reads `NO_COLOR`.
+ Open the `Colour` column header on the statuses page, or the `colour` descriptions in `schema/config.schema.json`.
+ 🔴 They use British spelling too, so one product mixes two.


== Environment

- `mt` built from `main` at the commit that filed this, on Linux.


== TO-BE

- 🟢 `mt init --organization` replaces `--organisation`, with its value enum and wizard prompt spelled the same way.
- 🟢 Help, the config schema descriptions, the docs site, the docs sources, the skill and the prelude say color, behavior, organize, normalize, serialize and the like.
- 🟢 A grep for the British forms finds nothing a user reads.


== Changes

- `crates/mindtape-cli/tests/it/init_e2e.rs`: the red test first, `--organization` accepted and `--organisation` refused.
- `crates/mindtape-cli/src/cmd/init/`: rename the flag, its enum and the wizard text.
- `crates/mindtape/src/config/`, `crates/mindtape/src/model/numeric.rs`, `crates/mindtape/src/framework/mod.rs`: the doc comments the schema is generated from.
- `schema/config.schema.json`: regenerate.
- `www/content/`, `docs/src/`, `skills/mindtape/`, `lib/prelude.typ`: the prose.


== Out of scope

- `CHANGELOG.md`, which another change rewrites.
- The `cancelled` status value. It is a shipped status name that projects write into task files, so renaming it is its own decision.
- Rust comments that no user reads, and CSS class names.


== Dogfood

Not applicable. The change is spelling.

