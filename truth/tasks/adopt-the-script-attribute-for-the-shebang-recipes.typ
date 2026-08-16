#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "adopt the script attribute for the shebang recipes",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("tooling",),
  estimate: (files: 4),
  links: related(
    "silence-the-mmz-wrapper-exit-message.typ",
  )[the other modern-just attribute worth adopting]
    + related(
      "lock-the-script-attribute-against-a-shebang-regression.typ",
    )[the gate that would keep this conversion from unwinding],
  status: done(2026, 8, 16),
)

== Summary

Five recipes open with `#!/usr/bin/env bash`, and only two of them set strict
mode. `[script]` plus `set script-interpreter := ["bash", "-eo", "pipefail"]`
moves the interpreter and its flags into one setting, so strict mode arrives by
construction instead of by remembering to type it, and the five shebang lines go
away. `count-tests` is the live bug that makes this more than tidying.

== The bug

`count-tests` is, verbatim:

```
count-tests:
    #!/usr/bin/env bash
    cargo test --workspace 2>&1 | grep "test result:" | awk '{sum += $4} END {print sum " tests"}'
```

No `set -eo pipefail`, so the pipeline exits with awk's status. A failing suite
prints a partial count and exits 0 — a wrong number, silently, which is the
failure mode this repo files tasks about everywhere else.

== The five recipes

`check-dry` and `install-hooks` already set `-eo pipefail` and change behaviour
not at all. `count-tests`, `file-sizes` and `uninstall-lib` do not, and are where
the work is: adding strict mode to a body written without it is a behaviour
change, not a refactor. Each one needs its post-change behaviour asserted, not
assumed.

- `count-tests` starts failing loudly instead of printing a wrong number. Under
  `pipefail` a red suite aborts the recipe, so it prints nothing at all. Decide
  whether that is the wanted shape for an advisory recipe or whether the count
  should survive a red suite by other means — and write the decision down.
- `file-sizes` ends `| sort -rn | head -20`. `head` exits at line 20 and `sort`
  takes SIGPIPE, so under `pipefail` the pipeline can report 141 and the recipe
  fails on a perfectly good run. This is the same trap the agent rules already
  name for gate commands ending in `| head`/`| tail`. It must be handled, and a
  run of `just file-sizes` on this repo must exit 0.
- `uninstall-lib` ends `[ -L "$TARGET" ] && rm ... && echo ... || echo ...`,
  which survives `-e` because the `||` consumes the failure — verify that the
  no-symlink path still exits 0 rather than reasoning about it.

== Scope

- `set script-interpreter` and `[script]` are both unstable. `set unstable :=
  true` is already declared in the root Justfile AND in `.just/docs.just`, since
  `just --fmt` reads the setting from the file it is pointed at. Measure whether
  a module inherits `script-interpreter` from the root or needs its own — do not
  assume either way.
- `[extension('.sh')]` pairs with `[script]`: it names the temp file just writes
  the body into, so a shell-aware tool or an error message sees `.sh`. Verified
  accepted by just 1.46 on a shebang recipe.
- The bodies already extracted into `.just/scripts/*.sh` are unaffected — each
  carries its own `set -eo pipefail` and stays a real file on disk. This task
  does not touch them.
- Prove the strict mode is actually in effect. A throwaway recipe whose body is
  `false | true` must fail. Without that assertion this ships a setting nobody
  demonstrated applies.

== The rendered-docs coupling

`www/gates.jq`'s `gate_command` reads `.shebang` from `just --dump --dump-format
json` and reports `null` for any shebang recipe, which is what makes
docs/src/contributing.typ fall back to `[doc(...)]` for the Command column. A
`[script]` recipe may well report `shebang: false`, which would make a
single-line body quotable where it was not before.

`check-dry` is the only gate in the five, and its body is multi-line, so the
column should not move. Check that against the regenerated artifact before
touching anything else — a rendered CONTRIBUTING.md diff with no doc source
edited is the outcome to catch early, not to discover at the gate.

`just check` must stay green, and the Justfile is a `dev-docs` outdatty source:
reconcile that group's dependents and re-record with `just outdatty-update
dev-docs`.
