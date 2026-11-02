#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Teach the skill and changelog the stem hash",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("docs",),
  estimate: (src-files: 1),
  x-goals: (
    product: verdicts(
      (
        implementable-tasks: helps[the skill tells agents to act on the stem hash any listing prints],
      ),
      "readable-tasks",
      "enforced-practice",
      "cheap-adoption",
      "typst-over-markdown",
      "plain-files",
    ),
    process: verdicts(
      (
        docs-match-code: helps[the skill and changelog match the stem hash],
      ),
      "sound-codebase",
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 3),
  links: depends-on("address-tasks-by-stem-hash-in-every-mt-command.typ"),
)

== AS-IS

+ Read the handle advice in `skills/mindtape/SKILL.md`.
+ 🔴 It says a handle resolves against bare `mt ls` and tells agents to pass the stem, which the stem hash makes wrong.


== TO-BE

- 🟢 The skill says a stem hash names one task in every listing, changes only with `mt mv` to a new stem, and is refused when ambiguous.
- 🟢 The skill's evals that type a handle use the stem hash and pass.
- 🟢 `CHANGELOG.md` `[Unreleased]` records that handles became stem hashes and `[handle] check-len` is gone.


== Source of truth

- `crates/mindtape/src/handle.rs` and `mt ls` output after the previous task.


== Changes

- `docs/src/skill.typ` and `skills/mindtape/SKILL.md` through `just docs md`: the handle advice.
- `skills/mindtape/references/` and `skills/mindtape/evals/`: every mention of ordinal handles.
- `CHANGELOG.md`: the entry.


== Out of scope

- `www/content/`, which the previous task changes with the code.


== Dogfood

Agents in this repo load the skill and type stem hashes.

