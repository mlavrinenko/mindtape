#import "@local/mindtape:0.2.0": *

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

just check re-runs every sub-gate unconditionally; only the whole gate is memoized (via mmz just check). Wrap each sub-command (fmt-check, clippy, test, machete, file-size, copy-paste, outdatty, examples, docs) as its own mmz rule with a narrow input scope, so an unrelated change reruns only the affected gate and passes carry across invocations.

== Scope

- Today `.mmz/config.yaml` declares one rule, `just check`, scoped to everything; `just check` itself runs each sub-gate unconditionally through `parallel + chronic` (never touching the cache). Only `mmz just check` memoizes, and it is all-or-nothing.
- Restructure: give each sub-gate its own mmz rule with the narrowest input scope that is actually correct — fmt-check (rust sources + `rustfmt.toml`), clippy, test, machete (manifests), check-file-size (sources + `.linecop.yaml`), check-dry (sources), outdatty-check (`outdatty.lock` + tracked sources), check-examples (`examples/`), docs-check (`www/` + mirrored docs). The scope work is the hard part; too broad = no reuse, too narrow = false freshness.
- `just check` fans the sub-gates each wrapped in mmz (keep `parallel`); resolve the `chronic`/mmz-notice interplay so a cache hit stays quiet but a failure still surfaces the failing recipe.
- Payoff: an unrelated edit reruns only the affected gate, and a sub-gate passed earlier in the session (or by a direct `cargo test`) is reused. `mmz --is-fresh -- just check` then means every sub-rule is fresh.
- Verify cache correctness: mutate one input in each scope and confirm only that gate reruns; confirm concurrent mmz writes under `parallel -j 0` are safe.
- Couples to the sibling task on running `just fmt`/tombi/typstyle from `just check` — the fmt-check scope must include the TOML/Typst sets once those are gated.

== Correctness fixes (review follow-up)

The first per-rule scoping (ebd6344) was too narrow in five places — each a false-fresh where `mmz --is-fresh --tag gate` reports pass though the real gate would fail. Closed in a follow-up to `.mmz/config.yaml`:

- `rust` was `crates/**/*.rs`, dropping the `include_str!`'d assets (`defaults.toml`, `frameworks.toml`, the preset/axes `config.toml`, the embedded template `.typ`) the test suite asserts against. Widened to `crates/**`.
- `manifests` held only the workspace root, not the per-crate `Cargo.toml`s cargo-machete actually scans. Added `crates/**/Cargo.toml`.
- `flake.nix` was in no scope, so a dev-tool bump — or an outdatty `dev-docs` source edit — busted nothing. Moved `flake.nix`/`flake.lock` into `gates-meta`, shared by every rule.
- `typ-fmt` listed `truth/tasks/**`, coupling `fmt-check` to `mt flip`'s own writes, so a multi-target or sequential close went false-stale. Removed; `just check` still typstyle-checks the tree, it just is not memoized.
- `docs` omitted `.github/workflows/release.yml`, read by `www/generate.sh`. Added.

Verified each via `mmz --status=json`: every embedded asset now lands in nine gate rules, `flake.nix` in all ten, machete sees both crate manifests, `fmt-check` no longer references `truth/tasks/`, and `crates/**` resolves to exactly the git-tracked file set (no build junk).
