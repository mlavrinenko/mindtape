#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Print counts as 1 task or 12 tasks, never task(s)",
  kind: "fix",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("cli", "gating"),
  estimate: (src-files: 14),
  x-goals: (
    product: (
      readable-tasks: helps[`mt check` and `mt rm` say 1 task or 12 tasks, the way a person would],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[one helper owns the plural, and a gate refuses a new `(s)`],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 15),
)

== Problem

+ Run `mt init` in an empty git repo, then `mt add Fix the login`.
+ Run `mt check`.
+ 🔴 It prints `ok: 1 task(s) valid`. `mt rm` prints `rm will delete 1 task(s):`, and `mt mv` prints `rewrote 4 link(s) in 2 file(s)`.


== Environment

- `mt` built from `main` at the commit that filed this, on Linux.


== TO-BE

- 🟢 `mt check` prints `ok: 1 task` or `ok: 12 tasks`.
- 🟢 Every count reads `1 task`, `12 tasks`, `1 link`, `3 links`, through one helper in the `mindtape` crate.
- 🟢 `just plural-check` refuses `(s)` inside a string literal in non-test Rust code under `crates/`, and `just check` runs it.

```text
$ just plural-check
crates/mindtape-cli/src/cmd/check/mod.rs:146: (s) in a string; count with mindtape::count(n, "noun")
plural-check: say 1 task or 12 tasks with mindtape::count or mindtape::plural, not task(s)
```


== Changes

- `crates/mindtape/src/plural.rs`: `plural` and `count`, with their tests first.
- `crates/mindtape-cli/src/cmd/check/`: the `ok:` lines.
- `crates/mindtape-cli/src/cmd/rm/`, `cmd/mv.rs`, `cmd/link/mod.rs`: the counts in reports and prompts.
- `crates/mindtape/src/template/fill/mod.rs`, `validate/links.rs`, `body/gate.rs`: the library's messages.
- `.just/scripts/plural-check.sh`, `Justfile`, `.mmz/config.yaml`, `outdatty.yaml`: the gate, with a fixture it must refuse first.


== Out of scope

- Code comments that say `(s)`. They reach no user, except the two doc comments the config schema quotes, which change here.


== Dogfood

This repo's own `mt check` and `mt rm` print the new counts, and `just check` runs the gate on every commit.

