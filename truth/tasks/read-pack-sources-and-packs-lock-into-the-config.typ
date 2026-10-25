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
  estimate: (src-files: 4),
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
+ 🔴 `mt check` refuses it as an unknown key, and no file can record the commit the project adopted.


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

- 🟢 `[pack-sources.<name>]` takes `git` and `ref`, where `ref` is a tag, a branch or a commit. `mindtape` is refused as a name.
- 🟢 `mt check` refuses a source the lock lacks, or whose `git` or `ref` differ from the lock's, with a hint to run `mt packs update`.
- 🟢 A pack refuses `[pack-sources]`, as it refuses `extends`.
- 🟢 Only the project layer may declare sources.


== Changes

- `crates/mindtape/src/config/pack_sources_tests.rs`: the red tests first, one per TO-BE bullet.
- `crates/mindtape/src/config/pack_sources.rs` (new): parse `[pack-sources]` and `.mindtape/packs.lock`, and compare them.
- `crates/mindtape/src/config/mod.rs`: list the file in the module header.
- `crates/mindtape/src/config/packs.rs`: read `[pack-sources]` from the project layer and refuse it in a pack or another layer.
- `schema/config.schema.json`: regenerate.
- `www/content/packs.typ`: a "Packs from a git source" section.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- Sources in the user's config directory: a clone or CI would lack them.
- Sources in any layer: a source the project doesn't name would make a clone resolve differently on another machine.


== Out of scope

- Resolving `acme:<pack>` from the cache: `resolve-a-declared-pack-from-the-cache-and-verify-its-hash.typ`.
- Fetching: `fetch-the-locked-pack-sources-with-mt-packs-install.typ`.


== Dogfood

Not applicable: this repo adopts only the built-in `mindtape:` packs.

