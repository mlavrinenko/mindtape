#import "@local/mindtape:0.3.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "Complete a status shortcut verb's task argument",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 4,
  ),
  difficulty: 3,
  estimate: (files: 4),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[the spelling the docs list first is the one that completes],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 9, 22),
)

== AS-IS

+ Wiring live shell completions. Every other value-taking argument got a source, and this one could not.
+ 🔴 #raw("crates/mindtape-cli/src/cli.rs"): A status shortcut verb takes its task argument through a clap `external_subcommand`, whose tail clap never parses and so cannot complete.

`crates/mindtape-cli/src/cli.rs:163` declares the verbs as
`#[command(external_subcommand)] Status(Vec<String>)`.
`crates/mindtape-cli/src/completion.rs:32` records the consequence: the
parser offers the verbs themselves, `mt flip done <task>` completes in full, and
`mt done <TAB>` returns an empty list. The `check-doc-facts` completion arm cannot
see the gap either, because the argument it would name does not exist in
`mt _cli --json`.


This fell outside the task that surfaced it, so I filed it here rather than
leave it in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cli.rs").

== TO-BE

- 🟢 `mt done <TAB>` offers the same task stems `mt flip done <TAB>` does.
- 🟢 The verbs stay generated from the legend, not declared one by one, so a
  config adding a status still gets both the verb and its completion.
- 🟢 Whichever way the decision goes, `completion.rs`'s module doc stops describing a
  limitation that does not hold.


== Changes

- `crates/mindtape-cli/src/cli.rs`: build the legend's verbs as real
  subcommands on the `Command` at runtime rather than catching them with
  `external_subcommand`. Each then carries a task argument clap can see. The
  dispatch stays one arm into `flip`.
- `crates/mindtape-cli/src/completion.rs`: give that argument the stem
  source the other task arguments use, and delete the paragraph about the
  gap.
- `.just/scripts/check-doc-facts.sh`: the completion arm sees these
  arguments, so confirm it demands a source for them rather than skipping.
- `crates/mindtape-cli/tests/completion_e2e.rs`: an arm asserting `mt done`
  and `mt flip done` offer the same candidates.


== Rollback

Revert the four files. The legend generates the verbs either way,
so no task file or config changes meaning.


== Out of scope

- The dispatch itself, which already works. Only the parse shape changes.
- Completing a filter expression, which the completions task left out on
  purpose.


== Dogfood

This repo's legend has six verbs and the docs teach `mt done` before
`mt flip done`. Tab after `mt done ` here is the test.

