#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Polish what init and install print",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("cli",),
  estimate: (src-files: 6),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a first run reads as plain questions and ends with the next command to type],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 15),
)

== Problem

+ Run `mt init --no-input` in an empty directory.
+ 🔴 It ends with `run mt init --list-presets to see the other starter shapes` and no hint of what to do next.
+ Run `mt init` on a terminal.
+ 🔴 The questions read `Workflow — status legend and transitions:`, with choice blurbs like `raw ints still allowed` and `no raw fallback`, and it ends with `scaffolded a custom configuration`.
+ Run `mt install`.
+ 🔴 It ends with `source the one for your shell from its startup file, e.g. ...`, always the bash line.


== TO-BE

- 🟢 Every `mt init` path ends with `next: mt add "My first task"`; `--no-input` says `want something else? mt init --list-presets shows the other presets`.
- 🟢 The wizard asks plain questions, like `Which statuses do you want?`, with plain choice blurbs and no em dashes.
- 🟢 `mt install` ends with `For Tab completion, add this to your shell's startup file:` and the line for the shell `$SHELL` names, bash otherwise.


== Changes

- `crates/mindtape-cli/src/cmd/install/tests.rs`: red tests first for the per-shell source line.
- `crates/mindtape-cli/src/cmd/install/mod.rs`: print the hint for `$SHELL`'s shell.
- `crates/mindtape-cli/src/cmd/init/tests.rs`: red tests for the next-step line.
- `crates/mindtape-cli/src/cmd/init/mod.rs`: the next-step line on every path, the new preset hint.
- `crates/mindtape-cli/src/cmd/init/wizard/axes.rs`: plain question titles and blurbs.
- `crates/mindtape-cli/src/cmd/init/wizard/mod.rs`: the defaults gate and the bad-choice line.


== Alternatives

- Printing every shell's source line after `mt install`. Five lines where one does, and most users run one shell.


== Out of scope

- The preset descriptions `mt init --list-presets` prints. They are their own pass.
- The TOML comments the wizard writes, beyond em dashes.


== Dogfood

Not applicable. This repo is set up already, but the quickstart captures show the new output.

