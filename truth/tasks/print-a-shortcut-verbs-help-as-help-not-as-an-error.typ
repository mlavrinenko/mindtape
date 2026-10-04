#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Print a shortcut verb's help as help, not as an error",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.95,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 1),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[an agent learns a verb's flags from its own --help instead of guessing],
      enforced-practice: neutral,
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
  status: proposed(2026, 10, 4),
)

== Problem

+ Flipping a task with \`mt done\`, I ran \`mt done --help\` to learn how to pass the note.
+ 🔴 #raw("crates/mindtape-cli/src/flip.rs"): \`mt \<status> --help\` exits 1, prefixes the help with \`Error:\`, opens it with the rustdoc of the internal \`Shortcut\` parser and names the binary \`mindtape-cli\`.

```
$ mt done --help; echo $?
Error: Parser wrapper for the shortcut-verb path: the raw external argv (`<status> <task>...`, plus any `-m`) re-parsed into [`FlipArgs`]

Usage: mindtape-cli [OPTIONS] [STATUS] [TASKS]...
1
```

`run_external` maps every `Shortcut::try_parse_from` error, the help request included, through `anyhow!("{err}")` (`crates/mindtape-cli/src/flip.rs:104`).


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/flip.rs").

== TO-BE

- 🟢 `mt done --help` prints the help for `mt done` and exits 0, like `mt flip --help`.
- 🟢 The help names `mt done`, describes the verb, and shows no internal doc comment.


== Changes

- `crates/mindtape-cli/tests/flip_e2e.rs`: red test first — `mt done --help` exits 0 and names `mt done`.
- `crates/mindtape-cli/src/flip.rs`: give `Shortcut` a user-facing `about` and the verb as its name, and let a help or version request print and exit instead of becoming an error.


== Rollback

Revert the commit; the shortcut verbs parse their arguments as before.

== Out of scope

- The `mt flip` help, which already works.


== Dogfood

Agents in this repo run `mt done --help` to find the note flag, as this finding's author did.
