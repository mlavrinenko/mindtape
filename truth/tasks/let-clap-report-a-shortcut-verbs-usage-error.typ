#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Let clap report a shortcut verb's usage error",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a script reads the same exit code from `mt done` as from `mt flip done`],
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

+ Reviewing the shortcut-verb help fix in \`crates/mindtape-cli/src/flip.rs\`
+ 🔴 #raw("crates/mindtape-cli/src/flip.rs"): a shortcut verb wraps clap's usage error in \`bail!\`, so it prints \`Error: error:\` and exits 1, where \`mt flip\` exits 2

```
$ mt done --bogus; echo $?
Error: error: unexpected argument '--bogus' found
1
$ mt flip done --bogus; echo $?
error: unexpected argument '--bogus' found
2
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/flip.rs").

== TO-BE

- 🟢 `mt done --bogus` prints clap's usage error once and exits 2, as `mt flip done --bogus` does.


== Changes

- `crates/mindtape-cli/tests/`: the red e2e test first, asserting exit 2 and no `Error: error:`.
- `crates/mindtape-cli/src/flip.rs`: hand the clap error to `err.exit()` in both arms.


== Rollback

- Revert the `flip.rs` arms.


== Out of scope

- Exit codes of other commands. A rule refusal's own exit code belongs to `require-a-refusal-capture-to-name-why-it-refused`.


== Dogfood

An agent here scripting `mt done` tells a typo from a refused flip by its exit code.
