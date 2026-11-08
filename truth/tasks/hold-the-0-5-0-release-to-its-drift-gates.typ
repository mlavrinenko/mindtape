#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Hold the 0.5.0 release to its drift gates",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tooling", "gating", "docs", "stateless"),
  x-goals: (
    product: verdicts(
      (:),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[a broken intra-doc link and an unpinned golden fail before release],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: wip(2026, 11, 8),
)

== AS-IS

+ Run the release preflight on the 0.5.0 tree.
+ 🔴 Four drifts reached it unobserved: a CRLF checkout can fail the schema goldens on Windows, `mt install` leaves a compatibly changed schema behind without `--force`, a broken rustdoc link fails no gate, and a yanked crate is only a `cargo publish` warning.


== TO-BE

- 🟢 A walk over every byte-compared golden holds it to `eol=lf`, so a new schema file cannot ship without the attribute.
- 🟢 Each install refreshes every generated schema, and a run that skips both config-home parts needs no config home.
- 🟢 `just check` runs `cargo doc` with warnings denied and fails on a broken or ambiguous intra-doc link.
- 🟢 The release hook refuses a `Cargo.lock` that pins a yanked crate, while leaving unrelated advisories to their own work.

```
$ just check
check: 36 gates passed
```


== Changes

- `.gitattributes` and `crates/mindtape-cli/tests/it/schema_e2e.rs`: the LF walk over schema and prelude goldens.
- `crates/mindtape-cli/src/cmd/install/mod.rs` and its tests: refresh generated schemas on every install, resolve the config home only when needed.
- `Cargo.toml`, `.just/lint.just`, `Justfile` and `.mmz/config.yaml`: the rustdoc lint levels and gate.
- `.just/scripts/release-hook.sh` and `nix/dev/shell.nix`: the yanked-crate release check.
- `CHANGELOG.md`, `www/`, `docs/src/contributing/` and `.config/outdatty.lock`: the docs and drift records.


== Out of scope

- Rewriting every schema `$id` or changing the schema versioning rule.
- Gating security advisories in `just check`; the five the tree carries are separate work.


== Dogfood

This repo's `just check` holds the new rustdoc and golden gates, and `just release 0.5.0` exercises the yanked check in its hook.

