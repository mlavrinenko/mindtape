#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "gate formatting of just module files",
  priority: ice(
    impact: 6,
    confidence: 0.95,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (files: 2),
  status: done(
    2026,
    8,
    14,
  )[JUST\_FMT\_PATHS shared by fmt and fmt-check; fmt-fixture-check gained a just arm asserting both the drift failure and the reformatted success, so an unstable-refusal cannot pass it],
)

== Summary

`just --fmt` does not descend into module files, so `docs.just` sits outside
`fmt-check` and has already drifted.

== Verified

On the dev shell's just 1.46.0, against a synthetic root justfile with `mod sub`
and a deliberately mis-formatted `sub.just`: `just --fmt` rewrote the root file
and left `sub.just` byte-identical. `--fmt --check` reported only the root file's
diff.

Live consequence, today, on this repo:

```
$ just --justfile docs.just --fmt --check
error: Formatted justfile differs from original.
```

The root `Justfile` is clean — a `--fmt` pass over a copy diffs empty. Only the
module file is drifted, and only because nothing looks at it.

== Why this blocks the layout work

The `.just/` restructure adds module and script files. Landing more files into a
surface the formatter gate cannot see makes the hole bigger. Close it first.

== Scope

- `fmt` and `fmt-check` both run `just --fmt` against the root only. Add the
  module file to both, as `just --justfile <module> --fmt`, so the fix and the
  check cover the same set — the reason `TOML_FMT_PATHS` and `TYPST_FMT_PATHS`
  exist as shared lists in the first place. Consider a `JUST_FMT_PATHS` variable
  in the same spirit, so a third module file cannot land in the fix but not the
  check.
- Reformat `docs.just` in the same commit; the drift is one missing blank line.
- A module file needs its own `set unstable := true` for a standalone `--fmt`
  invocation to be accepted. `docs.just` already has it — keep it, and note why,
  since it otherwise reads as dead config.
