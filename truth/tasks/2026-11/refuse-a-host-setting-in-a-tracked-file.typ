#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Refuse a host setting in a tracked file",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("tooling", "gating", "stateless"),
  assignee: "agent",
  status: done(2026, 11, 25),
)

== AS-IS

A doc that spells out a kernel setting reads as a requirement on the reader's machine.

+ Open `www/content/internals.typ`.
+ 🔴 It said the benchmark recipes don't need a kernel setting, which tells a reader the kernel matters.
+ Search the tree for the setting the sampler needed.
+ 🔴 Four files carried it, and each one told a reader to reconfigure a kernel before a benchmark would run.

== TO-BE

- 🟢 `just lint::host-details` fails on a tracked file that spells out a host setting.
- 🟢 The message reports the file, the line and the token, and says what to write instead: the tool's property, never the host's setting.

```
$ just lint::host-details
error: www/content/internals.typ:108 names a host setting `<token>`
check-host-details: a tracked file names a setting on the reader's machine
  state the tool's property instead, such as "both run unprivileged"
```

== Changes

- `.just/scripts/check-host-details.sh`: the scan, one `git ls-files -z` pass and one `awk` over the files it lists.
- `.just/scripts/_host-details-control.sh`: the control, which drives the scan over synthetic repos so a scan that never opens a file cannot pass.
- `.just/lint.just` and `Justfile`: the recipe, and its place in the fast stage.
- `.mmz/config.yaml`: the scope and the rule.
- `www/content/internals.typ`, `CHANGELOG.md`, `.just/bench.just`, `nix/dev/shell.nix` and the benchmark task: the settings removed.

== Out of scope

- A CPU model or a memory size. Both read like ordinary prose, so no scan catches them. The rule covers the settings a reader would have to change.

== Dogfood

The benchmark work put the setting in five files. The gate refuses the sixth.

== Goals

+ #Helps(maintainer, to: keep-changes-safe)[a doc that spells out a host setting fails a gate instead of landing].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).
