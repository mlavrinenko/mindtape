#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Stop check-doc-prose passing all-caps emphasis that names a shell variable",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  estimate: (src-files: 3),
  tags: ("found", "gating"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[emphasis in all-caps cannot hide behind a shell variable's name],
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
  status: proposed(2026, 10, 5),
)

== Problem

+ Closing the check-doc-prose pending list and reading how it allows constants
+ 🔴 #raw(".just/scripts/check-doc-prose.awk"): a word the scanned files assign as a shell variable is allowed in all-caps anywhere in prose, and a single shouted capital inside an all-caps run is not flagged

```
$ grep -n 'consts_of' .just/scripts/check-doc-prose.sh
scan "$(consts_of "${typst_sources[@]}" "${comment_sources[@]}")"
$ # a comment "# NOT A GITLEAKS RULE" passes when a script assigns NOT= or A=
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".just/scripts/check-doc-prose.awk").

== TO-BE

- 🟢 An all-caps word passes only as the constant: `$NAME`, `${NAME}` or `NAME=`.
- 🟢 A single capital letter inside an all-caps run, as in "NOT A GITLEAKS RULE", is flagged.
```sh
# NOT A GITLEAKS RULE
check-doc-prose: f.sh:1: all-caps "A"
```


== Changes

- `.just/scripts/check-doc-prose.awk`: allow a constant only in its code forms; flag one-letter shouting.
- `.just/scripts/check-doc-prose.sh`: control cases for both; reword comments the gate newly flags.


== Rollback

- Revert the commit.


== Out of scope

- The acronym list in `ALLOWED`.


== Dogfood

This repo's `just check` runs the tighter gate over its own comments.
