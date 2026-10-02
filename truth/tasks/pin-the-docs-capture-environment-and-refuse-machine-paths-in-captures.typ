#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Pin the docs capture environment and refuse machine paths in captures",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("www", "tooling"),
  estimate: (src-files: 2),
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
      sound-codebase: neutral,
      docs-match-code: helps[a capture shows what any reader's `mt` prints, whoever builds the site],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 2),
  links: parent("make-the-docs-plain-problem-led-and-unable-to-drift.typ"),
)

== Problem

+ Run `rg -n /home/ www/generated/`.
+ 🔴 It prints `www/generated/config-origin.txt:2:xdg: /home/user/.config/mindtape/defaults.toml (exists)` and the same line from `rule-packs-origin.txt`. The /cli/ and /rule-packs/ pages publish both.
+ The cause: `www/generate.sh` pins `MINDTAPE_NOW` but runs `mt` with the builder's own user config directory. A builder with a `defaults.toml` also changes what every capture shows.


== TO-BE

- 🟢 Captures run with an empty user config layer, so every machine renders the same bytes.
- 🟢 `www/generate.sh` fails, naming file and line, when a capture contains `$HOME` or the absolute repo path.


== Changes

- `www/generate.sh`: first add the guard: after the captures, search `www/generated/` for `$HOME` and the repo root and fail on a hit. It fails on today's two files. Then export an empty `MINDTAPE_CONFIG_HOME` beside `MINDTAPE_NOW`, and the guard passes.
- `www/generate-rule-packs.sh`: the same export, if it runs `mt` outside `generate.sh`'s environment.


== Rollback

Revert the commit; the captures go back to reading the builder's config.


== Out of scope

- Relative paths such as `examples/learn-piano/...`: pages are meant to show them.


== Dogfood

`just check` builds the site, so the guard runs on every commit in this repo.

