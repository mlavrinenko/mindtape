#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Support shell comments in the code prose gate",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 4,
  tags: ("stateless", "tooling", "gating"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a shell comment meets every prose rule a Rust doc comment meets],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 16)[gate green at tip],
)

== AS-IS

+ Run `grep -n '^# sh = r' .config/vale-code.ini`.
+ 🔴 The mapping sits behind a `#`, so `lint::vale-code` never reads a shell comment.
+ Write a passive line into a shell comment and run the gate over it.

```
$ printf '#!/usr/bin/env bash\n# The value is written by the loader.\n' > /tmp/demo.sh
$ vale --no-global --config=.config/vale-code.ini --output=line /tmp/demo.sh
$
```

== TO-BE

- 🟢 `lint::vale-code` reads a `#` comment in every tracked shell script, and holds it to the same 45 `Prose` rules a Rust doc comment meets.
- 🟢 `.config/vale-code.ini` carries `sh = r` and a `[*.sh]` section with every rule at `error`.
- 🟢 `grep -c '^Prose\..* = NO$' .config/vale-code.ini` prints `0`.

```
[formats]
rs = md
sh = r

[*.sh]
BasedOnStyles = Prose
Prose.History = error
Prose.Shouting = error
Prose.PassiveVoice = error
```

== Changes

- `.config/vale-code.ini`: turn on `sh = r` and the `[*.sh]` section, one rule at a time as its alerts clear.
- `.just/scripts/check-code-prose.sh`: add a shell fixture to the control, and pass every tracked `*.sh` to Vale.
- `.config/vale-plain.ini`: drop its `[*.sh]` section, so one config owns a shell comment.
- `docs/src/contributing/documentation.typ`: state that a shell comment meets the same set as a Rust doc comment.

== Alternatives

- Keeping the shell prose in `vale-plain.ini`: the `Plain` style extends three rules. A shell comment then meets a fraction of the set a Rust doc comment meets.
- A second `vale-code-sh.ini`: two configs to keep in step, and `lint::vale-levels` reads both.

== Out of scope

- A Justfile and a YAML comment, which `vale-plain.ini` reads through the `Comments` View.
- Inline `//` comments, which the `RustDoc` View leaves out.

== Dogfood

This repo's `just lint vale-code` holds every shell comment in `.just/scripts/` to the full `Prose` set.
