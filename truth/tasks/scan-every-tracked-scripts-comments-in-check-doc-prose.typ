#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Scan every tracked script's comments in check-doc-prose",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 3),
  tags: ("found", "tooling"),
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
      sound-codebase: neutral,
      docs-match-code: helps[the comments that explain each gate are held to the same prose rules as the docs],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 4),
)

== Problem

+ Reviewing commit 0c371aae
+ 🔴 #raw(".just/scripts/check-doc-prose.sh"): the gate reads comments from a hand list of five config files, so 14 of the 69 shell scripts still tell history

```
$ grep -n 'sources+=(Justfile' .just/scripts/check-doc-prose.sh
225:sources+=(Justfile .just/*.just .mmz/config.yaml outdatty.yaml .linecop.yaml)
$ grep -nE '^\s*#.*\b(USED to|used to|no longer|anymore|formerly)\b' .just/scripts/*.sh www/*.sh | head -2
.just/scripts/arg-pattern-check.sh:21:# biting is the quoting guarantee, and positional-args-check.sh no longer
.just/scripts/arg-pattern-check.sh:107:# one, the same kind `[positional-arguments]` used to guard recipes against:
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".just/scripts/check-doc-prose.sh").

== TO-BE

- 🟢 check-doc-prose picks its comment sources by kind: every tracked `*.sh`, `*.just`, `*.yaml` and `Justfile`, minus a short exclude list with a reason per entry.
- 🟢 A new script is scanned the day it lands, with no edit to the gate.
- 🟢 A PENDING list names the scripts not yet rewritten; it only shrinks, as check-doc-blocks' does, and each rewrite is its own commit.


== Changes

- `.just/scripts/check-doc-prose.sh`: read `#` comments from every tracked file of those kinds; add the exclude and PENDING lists.
- `.mmz/config.yaml`: give the rule inputs that cover every tracked script.
- `docs/src/contributing/documentation.typ`: say which comments the gate reads.


== Rollback

- Revert the commit; the five-file list returns.


== Out of scope

- Rewriting the 14 scripts' comments: each leaves PENDING in its own commit.
- Rust doc comments and clap help text.


== Dogfood

Every gate script in this repo explains itself in present tense, and `just check` keeps it so.

