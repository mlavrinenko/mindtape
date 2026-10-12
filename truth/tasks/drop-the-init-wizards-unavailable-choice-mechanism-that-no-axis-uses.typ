#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Drop the init wizard's unavailable-choice mechanism that no axis uses",
  kind: "dev",
  priority: ice(
    impact: 2,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 2),
  tags: ("found", "cli"),
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
      sound-codebase: helps[the wizard keeps only code a real axis runs],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 12),
)

== Problem

+ Building the rrule engine into every `mt` binary, which removed the one `unavailable` block a real axis declared.
+ 🔴 #raw("crates/mindtape-cli/src/cmd/init/wizard/mod.rs"): No real axis declares an `unavailable` block, so `Axis::unavailable`, `Axis::offered`, the flag refusal in `choose` and the one-choice skip in `ask_axis` run only for the test axes `Probe` and `Lone`.

```
$ rg -n '^unavailable \{|\$\( unavailable' crates/mindtape-cli/src
crates/mindtape-cli/src/cmd/init/wizard/tests.rs:188:unavailable {
crates/mindtape-cli/src/cmd/init/wizard/tests.rs:245:unavailable {
crates/mindtape-cli/src/cmd/init/wizard/mod.rs:78:        $( unavailable { $( $off:ident when $cond:expr => $why:literal ),+ $(,)? } )?
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/init/wizard/mod.rs").

== TO-BE

- 🟢 Every `mt init` axis lists each declared choice, and the wizard has no build-dependent branch.
- 🟢 The test suite has no probe axes, because there is no hiding left to probe.

== Changes

- `crates/mindtape-cli/src/cmd/init/wizard/mod.rs`: drop `Axis::unavailable`, `Axis::offered`, the `unavailable` macro arm, the refusal in `choose` and the one-choice skip in `ask_axis`.
- `crates/mindtape-cli/src/cmd/init/wizard/tests.rs`: drop the `Probe` and `Lone` axes and their tests.

== Rollback

- Revert the commit. The mechanism comes back whole, with its probe tests.

== Out of scope

- The axes and their fragments. Each keeps its choices.

== Dogfood

Not applicable: this repo runs `mt init` only in tests.
