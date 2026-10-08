#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Report task references and project discovery in the error format",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  kind: "refactor",
  tags: ("cli",),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[every error says what failed and what to try, in one shape],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    10,
    8,
  )[suggest::hint and suggestion keep callers outside this task; filed as a finding],
  links: depends-on(
    "build-an-error-dsl-that-fixes-the-message-format-and-the-exit-code.typ",
  ),
)

== Problem

+ Run `mt eval nope.typ` in a project.
+ 🔴 It prints `Error: no such file: nope.typ`, unquoted and capitalised.
+ Run `mt ls '@nope'`.
+ 🔴 It prints `Error: unknown view: nope; known views: default, stuck, this-week`, the suggestion folded into the error line.

== TO-BE

- 🟢 Each message is built with `Diag` (`crates/mindtape-cli/src/diag/`), so it follows the format in that module's docs: one `error:` line, the typed value single-quoted, the next step on a `hint:` line.
- 🟢 A did-you-mean is its own `hint: did you mean x?` line, and a read or write failure reads `error: can't read: '<path>': no such file or directory`.

== Changes

- `crates/mindtape/src/suggest.rs`: return the nearest name only; drop the `; did you mean: x?` string helpers once no caller uses them.
- `crates/mindtape/src/model/handle.rs`: an empty reference gets its own message.
- `crates/mindtape-cli/src/util.rs`: the unknown view, the config warnings and one I/O helper (`can't read`/`can't write` with the OS reason) that the other fan-out tasks reuse. The load-error lines are `report-typst-errors-at-their-line-with-typsts-hints`'s, and `report-template-load-failures-and-install-hints-in-the-error-format` drops the `util` `allow` and its `diag/allowlist.rs` entry.

== Out of scope

- The commands that call the I/O helper: each fan-out task switches its own sites.

== Dogfood

Every `mt` error in this repo's sessions from the files above comes out in the new shape.
