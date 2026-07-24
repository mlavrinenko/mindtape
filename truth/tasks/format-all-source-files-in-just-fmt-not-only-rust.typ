#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "format all source files in just fmt not only rust",
  priority: ice(
    impact: 5,
    confidence: 0.7,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("tooling", "gating"),
  estimate: (files: 4),
  status: done(2026, 7, 24),
)

== Summary

just fmt runs only cargo fmt, so TOML and Typst drift from what editors produce. Adopt Tombi as the canonical TOML formatter and typstyle for Typst, run both from just fmt/fmt-check, and gate them in just check so manual IDE saves stay in sync with CI.

== Scope

- Add `tombi` to the flake devShell packages (it is only a Zed extension today, not on the project PATH); `typstyle` is already present.
- `just fmt`: keep `cargo fmt --all`, add `tombi format` over the TOML set and `typstyle` over the Typst set (`lib/**`, `truth/tasks/**`, `examples/**`, `www/**`). Typst is near-noop — `mt` already typstyle-formats its writes and the `[format]` path — but it catches hand-edits.
- `just fmt-check`: mirror with `tombi format --check` (or `lint`) and `typstyle --check`; wire fmt-check into `just check` so drift fails CI. Today only `cargo fmt --check` runs.
- Decide the TOML glob deliberately. Tombi reorders tables and rewraps arrays (see the dogfood config reformat that reordered `[[tasks.limits]]` ahead of `[[on.flip]]`). Include `.mindtape/config.toml`, `examples/**/config.toml`. Weigh whether to include `Cargo.toml` manifests (cargo has its own conventions) and exclude `*.lock`. `schema/config.schema.json` is JSON — leave to a JSON formatter, not Tombi.
- This is the sync the request wants: editor save (Tombi/typstyle) and `just fmt` must produce byte-identical output, so a manual edit never fails fmt-check.
- Tests/gates: after adopting, `just fmt-check` is green on the whole tree; a deliberately mis-formatted TOML/Typst fixture fails it.
