#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Add quiet and verbose flags to mt fix",
  kind: "feat",
  priority: ice(
    impact: 3,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("mt", "cli", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      config-over-code: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[a gate prints one line per problem and nothing when the tree is clean],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 21),
)

== AS-IS

+ Run `mt fix --check` in a project whose tasks pin a supported older prelude.
+ 🔴 It prints one `kept` row per file, 923 of them here, and no summary:

```
$ mt fix --check | wc -l
923
$ mt fix --check | tail -1
kept    truth/tasks/2026-11/x-goals-sweep.typ    0.3.0
```

== TO-BE

- 🟢 `mt fix --check` prints the summary and the rewrites, and no `kept` row.
- 🟢 `-v` adds the `kept` rows, and a reader who wants the scan record passes `-v`.
- 🟢 `-q` doesn't print anything on success, as `mt check -q` does. A CI gate then reads one line per problem.
- 🟢 `mt fix` in write mode keeps its rows.

```
$ mt fix --check -q
$ echo $?
0
```

== Changes

- `crates/mindtape-cli/src/cmd/fix/mod.rs`: `FixArgs` gains `-q, --quiet` and `-v, --verbose`. `check` prints the summary and the rewrites, adds the `kept` rows under `-v`, and doesn't print anything under `-q`.
- `crates/mindtape-cli/src/cmd/fix/report.rs`: `print` takes whether to list the kept rows, and always returns the tally.
- `crates/mindtape-cli/tests/it/fix_e2e.rs`: the default, `-v` and `-q` shapes, and `-q` still exits 1 on an unsupported pin.
- `www/content/cli/fix.typ`: the two flags, and the gate recipe `mt fix --all --check -q`.
- `CHANGELOG.md`: an entry under `[Unreleased]` `Added`.

== Alternatives

- `-q` alone, leaving 923 `kept` rows in the default: a reader who runs `--check` to see whether anything needs fixing reads a screen of rows saying nothing does.
- Drop the `kept` rows everywhere: write mode loses the record of what the fixer scanned and left alone.

== Out of scope

- The `kept` rows in write mode. They show what the fixer scanned and left alone, which is the point of a write.
- The exit code. `--check` still exits 1 while an unsupported import remains.

== Dogfood

This repo pins 923 tasks at `0.3.0`, so `mt fix --check` prints 923 rows here. With `-q` it doesn't print anything and exits 0, which is what a gate needs.
