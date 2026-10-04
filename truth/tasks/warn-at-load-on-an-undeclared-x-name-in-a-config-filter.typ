#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Warn at load on an undeclared x- name in a config filter",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 2),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a misspelled x- name in a rule is reported at load instead of silently never matching],
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

+ Reviewing filter reads of x- fields in 4cd099ab
+ 🔴 #raw("crates/mindtape/src/filter.rs"): a config filter naming a misspelled \`x-\` field loads and matches nothing, while a misspelled core field fails the load

```
# [tasks.fields.x-machine] declared; rule when = 'x-machnie == "alpha"'
$ mt check
ok: 5 task(s) valid

# the same rule with when = 'kindd == "alpha"'
$ mt check
    invalid config: tasks.rules[0].when: unknown field in filter: kindd
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/filter.rs").

== TO-BE

- 🟢 When `[tasks.fields]` declares any `x-` name, a config filter naming an undeclared one prints a load warning with the nearest declared name.
- 🟢 Reading an undeclared `x-` name stays legal, in config and on the command line.


== Changes

- `crates/mindtape/src/config_tests.rs`: red test first — a rule naming `x-machnie` beside a declared `x-machine` warns.
- `crates/mindtape/src/filter.rs`: `expr_problem` reports undeclared `x-` names as a warning with `suggest::hint`.


== Rollback

- Revert the commit; an undeclared name loads silently again.


== Out of scope

- Refusing undeclared names: the filter design keeps them readable.
- `mt ls` arguments.


== Dogfood

Not applicable: this repo's filters name only `x-goals`, which it declares.
