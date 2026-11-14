#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "memoize just check sub-commands with mmz",
  priority: ice(
    impact: 6,
    confidence: 0.6,
    ease: 5,
  ),
  difficulty: 5,
  tags: ("tooling", "gating"),
  estimate: (files: 5),
  status: done(2026, 7, 24),
)

== Summary

`just check` re-runs every sub-gate unconditionally. Only the whole gate gets memoization, via `mmz just check`. Wrap each sub-command (fmt-check, clippy, test, machete, file-size, copy-paste, outdatty, examples, docs) as its own mmz rule with a narrow input scope. An unrelated change then reruns only the affected gate, and passes carry across invocations.

== Scope

- Today `.mmz/config.yaml` declares one rule, `just check`, scoped to everything. `just check` itself runs each sub-gate unconditionally through `parallel + chronic` (never touching the cache). Only `mmz just check` memoizes, and it is all-or-nothing.
- Restructure: give each sub-gate its own mmz rule with the narrowest input scope that is actually correct. That covers fmt-check (rust sources + `rustfmt.toml`), clippy, test, machete (manifests), check-file-size (sources + `.linecop.yaml`), check-dry (sources), outdatty-check (`outdatty.lock` + tracked sources), check-examples (`examples/`), and docs-check (`www/`). The scope work is the hard part. Too broad means no reuse, and too narrow means false freshness.
- `just check` fans the sub-gates each wrapped in mmz (keep `parallel`). Resolve how the `chronic` notices and mmz notices work together. A cache hit should stay quiet, while a failure still surfaces the failing recipe.
- Payoff: an unrelated edit reruns only the affected gate, and a sub-gate passed earlier in the session (or by a direct `cargo test`) gets reused. `mmz --is-fresh -- just check` then means every sub-rule is fresh.
- Verify cache correctness: mutate one input in each scope and confirm only that gate reruns. Confirm concurrent mmz writes under `parallel -j 0` are safe.
- Couples to the sibling task on running `just fmt`/tombi/typstyle from `just check`. The fmt-check scope must include the TOML and Typst sets once those also pass through the gate.

== Correctness fixes (review follow-up)

The first per-rule scoping (ebd6344) was too narrow in five places: each a false-fresh where `mmz --is-fresh --tag gate` reports pass though the real gate would fail. Closed in a follow-up to `.mmz/config.yaml`:

- `rust` was `crates/**/*.rs`, dropping the `include_str!`'d assets (`defaults.toml`, `frameworks.toml`, the preset/axes `config.toml`, the embedded template `.typ`) the test suite asserts against. Widened to `crates/**`.
- `manifests` held only the workspace root, not the per-crate `Cargo.toml`s cargo-machete actually scans. Added `crates/**/Cargo.toml`.
- `flake.nix` was in no scope, so a dev-tool bump (or an outdatty `dev-docs` source edit) busted nothing. Moved `flake.nix`/`flake.lock` into `gates-meta`, shared by every rule.
- `typ-fmt` listed `truth/tasks/**`, coupling `fmt-check` to `mt flip`'s own writes, so a multi-target or sequential close went false-stale. Removed. `just check` still typstyle-checks the tree, just without memoization.
- `docs` omitted `.github/workflows/release.yml`, read by `www/generate.sh`. Added.

We verified each via `mmz --status=json`. Every embedded asset lands in nine gate rules, and `flake.nix` in all ten. machete sees both crate manifests. `fmt-check` leaves `truth/tasks/` out of its scope, and `crates/**` resolves to exactly the git-tracked file set (with no build junk).
