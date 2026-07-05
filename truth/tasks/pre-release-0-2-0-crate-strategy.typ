#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Pre-release 0.2.0: resolve the `crates/` strategy before tagging",
  status: proposed(2026, 7, 5)[
    Analysis done; awaiting a go/no-go on the recommended path (Option A).
  ],
  priority: ice(impact: 9, confidence: 0.7, ease: 4),
  tags: ("substrate", "tooling", "docs"),
)

== Summary

The 0.2.0 release is blocked by three unforced questions about `crates/`, not by
missing features. This task records the research and the recommended path so the
tag can go out without locking in a decision we would regret.

Findings (verified in-tree, not just by sub-agent):

- `typst-as-lib` (v0.16, typst 0.15) overlaps ONLY the World layer of
  `typst-harvest`. It cannot replace `harvest`/`typst-cst`/`typst-edit`/
  `typst-cleanup` — those have no counterpart there or anywhere else surveyed.
- `~/projects/home/jmav/crates/typst-world` is a direct FORK of our
  `typst-harvest` (identical `value.rs`; `harvest.rs` differs only by the typst
  0.14→0.15 API migration), extended with a `FileProvider` seam (wasm), render,
  and `sys.inputs`. This is the real duplication behind the hesitation.
- best-of-typst DOES surface one real overlap on the READ side: `typst-batch`
  (v0.2.8, tola-rs, MIT, typst 0.14.1 — version-aligned with us) exposes a
  `Scanner` reading `#metadata()` markers plus `.links()`/`.headings()` — i.e.
  our harvest + typst-cst + link-scan, in one young crate. It does NOT cover the
  WRITE side (typst-edit splicing, typst-cleanup, swhid-mint, artifact-gate).
  Other candidates: `tinymist-{world,query}` (v0.15, LSP-coupled, churny),
  `typstyle-core` (eval-free CST pretty-printer), `reflexo-typst` (web render).
- Names free on crates.io today: `typst-edit`, `typst-world`, `typst-cst`
  (404). Others unchecked.
- Version drift: Rust workspace is 0.4.0; the Typst lib is 0.2.0; `just release`
  ties the git tag to the Cargo version → `just release 0.2.0` ABORTS today.

== Scope

Decisions to ratify (each has a recommendation, not a mandate):

+ Do NOT publish the library crates for 0.2.0. Ship the `mt` binary + the
  `@local/mindtape` Typst package only; keep `crates/*` as path deps. `just
  release` already tags-and-pushes with no `cargo publish`, so this is the status
  quo — nothing to build, just a conscious "not yet".
+ Reconcile the version scheme before tagging. Pick one: (a) tag = Rust
  workspace version (0.4.0), Typst lib independently 0.2.0 — then the release is
  "v0.4.0", not "0.2.0"; or (b) realign the workspace to 0.2.0. Recommendation:
  (a) — the crates have genuinely moved on; document that "0.2.0" names the Typst
  package/product, "v0.4.0" the binary tag. Fix the `just release` help/README to
  say so.
+ Defer substrate convergence to a 0.3/post-0.15 milestone (separate task).

Not in scope for 0.2.0: the typst 0.15 bump (branch directive forbids it),
publishing to crates.io, extracting repos.

== Convergence plan (0.3 / post-0.15)

Target end-state: ONE shared Typst-substrate crate, consumed by mindtape, jmav,
and speconaut — no more fork.

- Unify `typst-harvest` ⟷ jmav `typst-world` into a single crate. jmav's is the
  superset (provider seam, render, wasm, 0.15); adopt it as the base, fold our
  eval-only path in, retire `typst-harvest`. Blocked on the workspace typst
  0.14→0.15 bump.
- Keep `typst-cst` and `typst-edit` as-is (they are already consumer-blind and
  unique); bump their `typst-syntax` to 0.15 in the same wave.
- typst-batch spike: DONE (see section below). Verdict: do not adopt for
  mindtape. It cleanly replaces only the World layer we already own a better-fit
  version of (jmav typst-world), while dragging openssl/TLS/network we avoid, and
  not covering value-dates, typst-cst spans, or typst-edit rewriting.

== Spike: typst-batch (evaluated 2026-07-05)

Built a throwaway (`~/projects/clone/tb-spike`) depending on typst-batch v0.2.8,
ran its `Scanner` against a real task file. Findings, all verified live:

- Resolves `@local/mindtape:0.2.0` from the platform dir with no custom resolver;
  `accessed_files`/`accessed_packages` cover our dependency-tracking need.
- `metadata("mindtape")` returns the full task dict as JSON; a ~30-line custom
  `Extractor` reproduces our all-markers-with-labels model. The `Extractor` trait
  is clean and the right shape.
- BLOCKER (dates): `serde_json::to_value` renders a Typst datetime as the repr
  string `"datetime(year: 2026, month: 7, day: 4)"` — unusable for `start <=
  now()`/overdue. Our `HVal`/`format_date` handles dates; typst-batch's JSON does
  not. So we'd keep our `value.rs` converter regardless (run inside a custom
  Extractor over `&meta.value`).
- Headings are eval-plane (resolved level, NO source line/byte-range) → does not
  replace typst-cst's parse-plane body gate. Links are content `LinkElem` (empty
  here — our links live in the metadata payload) with no byte range → does not
  touch typst-edit.
- Dependency weight: pulls typst-kit(fonts+packages) → ureq + native-tls →
  openssl-sys (+ rayon, owo-colors, serde_json). mindtape's typst-harvest pulls
  NONE of these (disk-only, no network). Not toggleable from typst-batch's public
  features. A system-openssl/pkg-config build requirement is a Nix packaging and
  supply-chain regression for a daemon-free disk tool.
- Maturity flags: v0.2.8, single ecosystem, 394 downloads; `scan` without `batch`
  fails to compile (rayon leak in `world/snapshot.rs`) — feature matrix untested.

Net: typst-batch replaces ~280 lines (our World) but only that, at the cost of a
heavy TLS/network dep tree and a young external dependency for our core read
path. The valuable hand-rolled code (typst-edit rewriting, typst-cst source
spans, HVal date handling) is exactly what it does NOT provide. Mine it for
patterns (VFS seam, cache strategies, Extractor trait) — jmav typst-world already
has the VFS seam — but do not take the dependency.

== Repo topology (when we publish)

Recommendation: TWO groupings, not repo-per-crate.

- One repo, one cargo workspace, for the Typst-coupled family — `typst-world`
  (was harvest), `typst-cst`, `typst-edit`, `typst-cleanup` — published as
  separate crates. Rationale: the dominant maintenance axis is "bump the pinned
  typst version", which touches all of them at once; a monorepo does that bump in
  one PR/one release wave. Repo-per-crate multiplies that cost by four for zero
  benefit (consumers still select crates individually).
- `swhid-mint` and `artifact-gate` are NOT typst-coupled (SWHID minting; a
  plane-agnostic rule engine). They evolve on their own axis → their own repo(s).

== Contributing to typst-as-lib (optional, low cost)

We cannot fully adopt `typst-as-lib` because: (1) it is compile/render-oriented
(no eval-only harvest path exposed — you would `world_builder().build()` then run
`typst_eval` yourself); (2) its `@local` resolution expects the standard
`<root>/local/<name>/<version>/` layout, whereas our dev/dogfood override maps a
package NAME straight to `lib/` (version-agnostic, edit-in-place); (3) there is NO
public API to supply a custom `Library` / merge native-Rust globals into the
global scope — only `sys.inputs`/module injection — which the speconaut
convergence needs (our `with_globals`).

Reason (3) is a clean, small contribution: PR a `TypstEngineBuilder` hook to
supply/customize the `Library` (e.g. `customize_library(impl FnOnce(&mut
Library))` or `with_library(LazyHash<Library>)`). If accepted, `typst-as-lib`
becomes a viable World base and we drop one hand-rolled layer. Track separately;
not a 0.2.0 blocker.

== Release checklist (0.2.0)

+ Decide version scheme (Scope item 2); update `just release` guard + help and
  the README/`www` version examples if the tag string changes.
+ Confirm `lib/typst.toml` = 0.2.0 and the `TYPST_PKG_VERSION` const +
  import examples agree (Version Management in CLAUDE.md).
+ `just check` green (fmt, clippy -D, tests, machete, linecop, jscpd, outdatty).
+ `just nix-build` green in the clean sandbox.
+ Tag + push via `just release <VERSION>`.
+ Remove the CLAUDE.md "Temporary — active only on branch" section when this
  branch merges to main (it says so itself).

== Notes

- Sub-agent findings (best-of-typst survey; jmav fork claim) were spot-checked
  against source: the harvest/value diff and the typst-as-lib `create_injected_
  library` limitation were read directly, not taken on trust.
- Keeping the crates unpublished for 0.2.0 costs nothing and preserves every
  option (rename, merge, extract) for the convergence milestone.
