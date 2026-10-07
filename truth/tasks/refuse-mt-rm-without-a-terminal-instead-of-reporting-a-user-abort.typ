#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Refuse mt rm without a terminal instead of reporting a user abort",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 1),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[an agent running mt rm learns to pass --yes instead of reading that it declined],
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
  status: proposed(2026, 10, 7),
)

== Problem

+ Capturing every error \`mt\` prints for the error inventory
+ 🔴 #raw("crates/mindtape-cli/src/cmd/rm/mod.rs"): with stdin not a terminal, \`mt rm\` prints its prompt, reads nothing and reports that the user aborted

```
$ mt rm add-redis-client </dev/null
rm will delete 1 task(s) and rewrite links in 1 file(s):
  - tasks/add-redis-client.typ
Continue? [y/N] Error: aborted by user
[exit 1]
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/rm/mod.rs").

== TO-BE

- 🟢 With no terminal on stdin, `mt rm` skips the prompt and fails, naming `--yes` as the way to confirm without one.
- 🟢 "aborted" is printed only when someone answered no.


== Changes

- `crates/mindtape-cli/src/cmd/rm/mod.rs`: check `stdin().is_terminal()` before prompting and refuse with `--yes` as the next step.


== Rollback

- Revert the file; the prompt runs on a closed stdin again.


== Out of scope

- `mt rm`'s gitless report prompt in `rm/gitless/report.rs`, unless it shares the same helper.


== Dogfood

Agents in this repo run `mt` without a terminal and get the right next step.
