#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Run e2e test scripts through one hermetic shell helper",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 4),
  tags: ("found", "tests"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a new script test cannot skip the hermetic shell],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[one door replaces three hand-rolled env strips],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 14),
)

== Problem

+ Running jscpd_baseline_merge_e2e on main, which fails in a tempdir
+ 🔴 #raw("crates/mindtape-cli/tests/it/jscpd_baseline_merge_e2e.rs"): the test sets BASH_ENV= only inside its script, but bash runs the host BASH_ENV hook (/etc/bash_env) at startup, which drops the dev shell PATH in a tempdir with no .envrc; outdatty_lock_merge_e2e.rs and wt_carry_lcov_e2e.rs each hand-roll env_remove("BASH_ENV"), so the bug recurs with every new script test

```
just: not on PATH
```
from `cargo nextest run -p mindtape-cli --test it jscpd_baseline`

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/tests/it/support/shell.rs").

== TO-BE

- 🟢 Every e2e script spawn goes through `support::shell`, which strips `BASH_ENV`, so `jscpd_baseline_merge_e2e` passes from a tempdir, and a test that spawns `bash` or `sh` directly fails.

== Changes

- `crates/mindtape-cli/tests/it/support/shell.rs`: the helper and a guard test.
- The three e2e files call it and drop their hand-rolled `BASH_ENV` handling.

== Rollback

- Revert the commit.

== Out of scope

- `git` spawns, which run no shell hook.

== Dogfood

The three script tests pass from `just test`.
