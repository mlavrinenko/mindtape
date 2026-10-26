#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Resolve a declared pack from the cache and verify its hash",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.6,
    ease: 5,
  ),
  difficulty: 5,
  tags: ("mt", "gating"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[`extends = ["acme:commits"]` reads one shared copy per machine, offline],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 26),
  links: parent("share-config-between-projects-through-packs.typ")
    + depends-on(
      "fetch-a-locked-pack-source-into-a-cache-directory.typ",
      "resolve-a-packs-check-and-dict-files-from-its-directory.typ",
    ),
)

== Problem

+ Declare `[pack-sources.acme]`, lock it, and add `extends = ["acme:commits"]`.
+ 🔴 `mt check` refuses `acme:commits` as an unknown pack: only the `mindtape` source resolves.


== TO-BE

```text
<cache dir>/mindtape/packs/9c41…/commits.toml
<cache dir>/mindtape/packs/9c41…/team/pack.toml
```

- 🟢 `acme:<pack>` resolves to `<pack>.toml` or `<pack>/pack.toml` under the cache directory named by the lock's `sha256`.
- 🟢 The cache directory is `$MINDTAPE_CACHE_HOME`, else the platform cache directory, such as `~/.cache` on Linux.
- 🟢 `mt` hashes the cached tree on every load with `pack_sources::hash` and refuses a mismatch, naming the directory.
- 🟢 A source missing from the cache fails with `pack source not installed: 'acme'` and a hint to run `mt packs install`. No read command touches the network.
- 🟢 `mt config show --origin` prints `pack acme:commits`.
- 🟢 No file of the tree resolves outside it. `<pack>` takes letters, digits, `-` and `_`. A dictionary `path` stays in the tree, and any pack's `check` module in the pack's directory.


== Changes

- `crates/mindtape/src/config/packs_tests_cache.rs`: the red tests first, against a cache directory the test fills.
- `crates/mindtape/src/config/pack_cache.rs` (new): resolve a declared source's pack from its verified tree, as `PackOrigin::Cached`. An unknown pack lists the tree's packs. `pack_resolve.rs` routes the project layer's declared sources there.
- `crates/mindtape/src/config/pack_files.rs`, `crates/mindtape/src/rules/typst.rs`: the dictionary and check bounds.
- `crates/mindtape/src/config/layers.rs`: `load_config_layered_in` takes the cache directory. `load_config_layered_from` reads none, so it stays free of the machine.
- `crates/mindtape/src/config/pack_sources/mod.rs`: `cache_root()`, `<cache dir>/mindtape/packs`, for `mt packs install` too.
- `crates/mindtape/src/loader/app_dirs.rs`: `cache_home()` with the `MINDTAPE_CACHE_HOME` override.
- `crates/mindtape-cli/src/cmd/config.rs`: print the new origin.
- `www/content/packs.typ`, `www/content/config.typ`: the cache directory, its variable, and a `packs-not-installed` capture. `www/capture.sh` and the e2e `isolated_env` give `MINDTAPE_CACHE_HOME` a directory of their own.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- Trust the cache directory's name without hashing: a hand-edited cached pack would pass unseen, and a few KB hash in microseconds.
- The user's config directory: it holds files the user writes, and the cache holds what `mt` can fetch again, as Typst keeps `@preview` packages.
- Bound a dictionary to the pack's own directory: two packs of one source couldn't share a word list, and the hash covers the whole tree.


== Out of scope

- Writing the cache: `fetch-a-locked-pack-source-into-a-cache-directory.typ`.


== Dogfood

Not applicable: this repo adopts only the built-in `mindtape:` packs.

