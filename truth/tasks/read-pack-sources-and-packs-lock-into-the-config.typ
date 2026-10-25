#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Read pack sources and packs.lock into the config",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.7,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("mt", "gating"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a project declares a team's pack repository in three lines and `mt check` says when the lock lags],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 26),
  links: parent("share-config-between-projects-through-packs.typ")
    + depends-on(
      "fetch-packs-from-a-git-source-pinned-in-packs-lock.typ",
      "name-packs-as-source-colon-pack.typ",
      "merge-every-config-key-from-a-pack-below-the-layer-that-extends-it.typ",
    ),
)

== Problem

+ Add `[pack-sources.acme]` with `git` and `ref` to `.mindtape/config.toml`.
+ 🔴 `mt check` refuses `pack-sources` as an unknown field, and no file can record the commit the project adopted.


== TO-BE

```toml
# .mindtape/config.toml
[pack-sources.acme]
git = "https://github.com/acme/mindtape-packs"
ref = "v1"

# .mindtape/packs.lock, written by mt
[acme]
git = "https://github.com/acme/mindtape-packs"
ref = "v1"
commit = "3f2c1a9e0b7d4c55e1a2b3c4d5e6f708192a3b4c"
sha256 = "9c41…"
```

- 🟢 `[pack-sources.<name>]` takes `git` and `ref`, a tag, branch or commit. Only the project layer may declare one, `mindtape` is refused as a name, and a pack refuses the key.
- 🟢 Every config load refuses a source the lock lacks, or whose `git` or `ref` differ from the lock's, with `pack source lock is stale: 'acme'` and a hint to run `mt packs update`.
- 🟢 An `acme:<pack>` entry in an xdg or ancestor layer is refused, since that layer can't declare `acme`.
- 🟢 `pack_sources::read_raw` reads the sources and the lock without resolving `extends`, for `mt packs`.


== Changes

- `crates/mindtape/src/pack_sources/tests.rs`: the red tests first, one per TO-BE bullet.
- `crates/mindtape/src/pack_sources/mod.rs` (new area): parse `[pack-sources]` and `read_raw`, with a `//!` header listing the area's files.
- `crates/mindtape/src/pack_sources/lock.rs` (new): parse `.mindtape/packs.lock` and compare it with the sources.
- `crates/mindtape/src/lib.rs`: register the area.
- `crates/mindtape/src/config/packs.rs`: read `[pack-sources]` from the project layer, refuse it elsewhere, and run the staleness check.
- `schema/config.schema.json`: regenerate.
- `www/content/config.typ`: a `pack-sources` row. `www/content/packs.typ`: a "Packs from a git source" section.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- Sources in any layer: a source the project doesn't name would make a clone resolve differently on another machine.
- Check staleness in `mt check` only: `mt ls` would then read a pack the config no longer asks for.


== Out of scope

- Writing the cache: `fetch-a-locked-pack-source-into-a-cache-directory.typ`.
- Resolving `acme:<pack>`: `resolve-a-declared-pack-from-the-cache-and-verify-its-hash.typ`.


== Dogfood

Not applicable: this repo adopts only the built-in `mindtape:` packs.

