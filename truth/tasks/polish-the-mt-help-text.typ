#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Polish the mt help text",
  kind: "docs",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("cli", "docs"),
  estimate: (src-files: 12),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a newcomer reads `mt --help` at a glance and finds the first commands on top],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the CLI reference pages, generated from the same doc comments, read the same way],
    ),
  ),
  assignee: "agent",
  status: wip(2026, 10, 15),
)

== Problem

+ Run `mt --help` in any project.
+ 🔴 The about line reads "Command-line interface for Mindtape, a file-based task tracker over Typst", which says nothing a newcomer can act on.
+ 🔴 `init` sits twelfth and `add` first, so the commands a newcomer needs first are scattered through the list.
+ 🔴 The listing never says each status is a command too, like `mt done <task>`.
+ 🔴 Short help lines run to 250 characters, such as `mt rm --help`'s `[TARGETS]` and `mt allow`'s line, and wrap into paragraphs in `-h`.
+ 🔴 `--color` is explained as "When to colour the output", two spellings side by side.


== TO-BE

- 🟢 The about reads "Mindtape keeps your tasks as Typst files and checks them against your rules."
- 🟢 The listing opens with init, add, ls, find, flip and check, and ends with help.
- 🟢 After the list, the help says every status is a command and points at `mt flip --list`.
- 🟢 Every command's and flag's short help is one plain sentence of about 70 characters. The detail moves to the long help, with no fact lost.
- 🟢 Help text spells color one way, as the `--color` flag does.


== Source of truth

- `mt --help`, `mt -h` and `mt <cmd> --help` as the built binary prints them.
- `mt _cli --json`, which the site's CLI reference renders.


== Changes

- `crates/mindtape-cli/src/cli/tree_json_tests.rs`: the red tests first, a cap on every short help line and the command order.
- `crates/mindtape-cli/src/cli/mod.rs`: the about, the command order and the status note after the list.
- `crates/mindtape-cli/src/cmd/*/`: split each long `Args` doc comment into a short first line and a long body.
- `crates/mindtape-cli/src/render/color.rs`: spell color as the flag does.


== Out of scope

- The crate's Cargo `description`, which crates.io shows to a different reader.
- Hand-written docs prose. It changes only where it quotes a line this task rewrites.


== Dogfood

Agents in this repo read `mt <cmd> -h` before they run a command, and get the one-line answer first.
