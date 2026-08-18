#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "capture docs output with the debug binary, not release",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("docs", "tooling"),
  estimate: (files: 4),
  assignee: "agent",
  status: done(2026, 8, 18),
)

== Summary

`www/generate.sh` builds `mindtape-cli` with `--release` before capturing the
docs transcripts, so any Rust edit pays a fat-LTO single-threaded whole-program
relink (~4m30s) on every `just check` or `docs md` run. The captures are
`--help`, `_cli`, `config schema`, and `mt ls` / `mt check` transcripts, all
byte-identical between the debug and release binaries because none depends on
optimisation. Switch the capture to the debug binary (`cargo build` without
`--release`, then `target/debug/mt`), which the test suite already keeps warm.


== Scope

- `www/generate.sh`: point `MT` at `target/debug/mt` and drop `--release` from
  the build; update the pre-lock comment that argues the release-build
  placement.
- `docs/generate-md.sh`: update the header comment that says the standalone
  render pays a release build.
- `docs/src/contributing/generated-docs.typ`: update the same note in the
  source and regenerate `docs/contributing/generated-docs.md`.

