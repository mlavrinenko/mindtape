#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Fail the clippy gate on a disallowed-methods path that resolves to nothing",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 2),
  tags: ("found", "gating"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a ban that names a missing path fails the gate instead of banning nothing],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[every clippy ban provably fires],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 13),
)

== Problem

+ Adding the `util::open_warned` and `Diag::did_you_mean` bans to `clippy.toml` for the snippet-extraction task
+ 🔴 #raw("clippy.toml"): clippy accepts a `disallowed-methods` entry whose path names no function, so a mistyped ban silently bans nothing

The agent's first path for `warn_errors` was wrong, and `just clippy` passed with it; only re-inlining a copy and seeing clippy stay green showed the ban was dead.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("Justfile").

== TO-BE

- 🟢 A `disallowed-methods` or `disallowed-macros` entry whose path resolves to nothing fails `just clippy`, naming the entry.

```toml
# clippy.toml, with a typo in the path
{ path = "mt::util::warn_eror", reason = "use util::open_warned" },
# just clippy: error: `mt::util::warn_eror` does not refer to an existing function
```


== Changes

- `Justfile` or `clippy.toml`: make an unresolved ban path an error, by clippy's own config lint if the pinned toolchain has one, else a check that each path resolves.


== Rollback

- Revert the commit. Bans keep working, unchecked.


== Out of scope

- The bans themselves.


== Dogfood

Every `just check` in this repo verifies the six current bans resolve.
