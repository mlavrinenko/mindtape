#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "cover the preset command entry point",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("tests",),
  estimate: (files: 1),
  status: done(
    2026,
    8,
    1,
  )[Already fixed as a side effect of fc842c1 (`dev: fix cargo crap warning`), which predates this task's filing time on the clock but landed after it in history: that commit added run-dispatch tests to `preset_cmd_tests.rs` that re-exec the test binary and call `run()` directly, exercising `resolve_root` and the user config dir lookup. Verified locally: `preset_cmd.rs:53 run` is now 100% line-covered, CRAP score 6.0, and `just crap` reports no function above threshold. No separate `preset_e2e.rs` needed.],
)

== Summary

`preset_cmd::run` is the only function above the CRAP threshold on a fresh lcov: score 42, cyclomatic complexity 6, 0% covered. `crates/mindtape-cli/tests/` has no `preset_e2e.rs` — the unit tests in `preset_cmd_tests.rs` call `save()` and `render_ls()` directly, so the command entry point that resolves the root and the user config dir is never executed. `preset_cmd.rs` has one commit (1238df1) and no preset e2e has ever existed on any branch, so the function has been uncovered since it landed. `just crap` fails on a fresh lcov for this one function alone (verified locally); the CI `coverage` job runs `just cover` then `just crap` in the same job with no `continue-on-error`, so that step fails wherever it runs — not verified against actual run history.

== Scope

- add `crates/mindtape-cli/tests/preset_e2e.rs` driving `mt preset save` and `mt preset ls` as subprocesses; `support::mt` already isolates `MINDTAPE_CONFIG_HOME`/`XDG_CONFIG_HOME`, which is what makes the user-preset dir testable
- llvm-cov merges child profraw, so a subprocess e2e credits `run` directly
- out of scope: lowering the threshold or `--allow`-ing the function — the gap is real, not a metric artifact
