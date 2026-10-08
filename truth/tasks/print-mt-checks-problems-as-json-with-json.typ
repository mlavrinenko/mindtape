#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Print mt check's problems as JSON with --json",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 3),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[an agent reads each problem's file, line and rule without parsing text],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[agents.typ's every-read-command-prints-JSON claim holds for mt check],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 8),
)

== Problem

+ Reviewing the docs before release, checking agents.typ against mt check --help
+ 🔴 #raw("crates/mindtape-cli/src/cmd/check/"): mt check has no --json flag, while www/content/agents.typ:80 and comparison.typ:172 promise JSON from every read command, so an agent following them gets a usage error

```
ls: format,json   find: format,json   views: json   worktrees: json
config show: json   eval: json   graph: format   check: (none)   preset ls: (none)
```
(`--json`/`--format` per command, from `mt _cli`)


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/check/").

== TO-BE

- 🟢 `mt check --json` prints every problem as an object with its severity, file, line, rule name, message and hints, in the envelope `mt ls --json` uses, and exits with the same code as plain `mt check`.
- 🟢 A clean project prints an empty problem list.


== Changes

- `crates/mindtape-cli/src/cmd/check/`: add the `--json` flag and a JSON renderer beside the text one, fed from the same problem list.
- `crates/mindtape-cli/tests/`: an e2e test over a fixture with a warning and an error, asserting the JSON fields and the exit code.
- `CHANGELOG.md`: an `[Unreleased]` entry.


== Rollback

- Revert the commit.


== Out of scope

- `mt graph` and `mt preset ls`, which get `--json` in their own task.
- Changing the text output.


== Dogfood

The commit hook and agents in this repo can read `mt check --json` instead of matching `error:` lines.

