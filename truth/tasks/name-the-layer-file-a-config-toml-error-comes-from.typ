#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Name the layer file a config TOML error comes from",
  kind: "dev",
  priority: ice(
    impact: 3,
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
      enforced-practice: neutral,
      cheap-adoption: helps[a broken shared defaults.toml is found by the path the error names],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 6),
)

== Problem

+ Fixing the first-run errors, which locate an unknown table in the project config only
+ 🔴 #raw("crates/mindtape/src/config_packs.rs"): A TOML syntax error in an ancestor \`.mindtape/defaults.toml\` prints under \`load \<project>/.mindtape/config.toml\`, so its line and column point into the wrong file

```
$ cat ~/.mindtape/defaults.toml
oops =
$ mt ls      # in ~/p, whose .mindtape/config.toml is valid
Error: load /tmp/tmp.QoQ62sXNwK/p/.mindtape/config.toml

Caused by:
    parse error: TOML parse error at line 1, column 7
      |
    1 | oops =
      |       ^
    invalid string
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/config_packs.rs").

== TO-BE

- 🟢 A TOML error in a `defaults.toml` layer or an `extends` pack names that file, so its line and column point into the file the message names.

== Changes

- `crates/mindtape/src/config_packs.rs`: wrap a layer or pack parse error with the layer's path.
- `crates/mindtape/src/config.rs`: a `ConfigError` variant that carries the path beside the TOML error.

== Rollback

- Revert the two files.

== Out of scope

- The CLI's `load <config.toml>` context, which stays right for the project layer.

== Dogfood

Not applicable: this repo keeps no `defaults.toml` above its root, so its contributors never meet it.
