#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Generate www/generated once per tree instead of once per docs gate",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("tooling", "docs"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a full `just check` after an edit costs a minute less, so it runs more often],
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
  status: proposed(2026, 10, 6),
)

== Problem

+ Edit any file under `crates/`.
+ Run `just check-doc-facts`, `just docs::check`, `just docs::md-check` and `just site-pages-fixture-check` in parallel, as `just check` does.
+ 🔴 They take 99 s. Each runs the whole `www/generate.sh`, 23 s warm, and `target/www-generated.lock` makes them run it one after another.


== TO-BE

- 🟢 The first of those gates regenerates `www/generated/`. The others find it fresh and skip the 23 s.
- 🟢 A `cargo clean` or a deleted `www/generated/` makes the next gate regenerate, never read a missing file.


== Changes

- `.mmz/config.yaml`: a `just docs::gen` rule over everything `www/generate.sh` reads, with its key artifacts as `outputs`.
- `.just/scripts/check-doc-facts.sh`, `.just/scripts/site-pages-fixture-check.sh`, `docs/generate-md.sh`: run `mmz just docs::gen` under the lock they already hold, instead of `www/generate.sh`.
- `.just/docs.just`: `check` takes the lock and runs `mmz just docs::gen` instead of depending on `gen`.


== Rollback

Revert the commit; each gate regenerates on its own again.

== Out of scope

- Staging `just check`. Its own task.
- Making any one capture faster.


== Dogfood

This repo's `just check` runs the generator once after an edit.
