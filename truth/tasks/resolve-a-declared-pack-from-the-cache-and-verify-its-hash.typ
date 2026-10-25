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
  estimate: (src-files: 5),
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
  status: proposed(2026, 10, 26),
  links: parent("share-config-between-projects-through-packs.typ")
    + depends-on(
      "read-pack-sources-and-packs-lock-into-the-config.typ",
      "let-a-pack-directory-ship-templates-and-typst-files.typ",
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
- 🟢 `mt` hashes the cached tree on every load and refuses a mismatch, naming the directory.
- 🟢 A source missing from the cache fails with `pack source "acme" is not installed` and a hint to run `mt packs install`. No read command touches the network.
- 🟢 `mt config show --origin` prints `pack acme:commits`.


== Changes

- `crates/mindtape/src/config/pack_sources_tests.rs`: the red tests first, against a cache directory the test fills.
- `crates/mindtape/src/config/pack_sources.rs`: hash a tree as the sha256 of each file's relative path and bytes in path order, `.git/` excluded.
- `crates/mindtape/src/config/pack_resolve.rs`: resolve a declared source's pack from the cache.
- `crates/mindtape/src/loader/app_dirs.rs`: `cache_home()` with the `MINDTAPE_CACHE_HOME` override.
- `crates/mindtape/Cargo.toml`, `Cargo.lock`: the `sha2` crate.
- `www/content/packs.typ`, `www/content/config.typ`: the cache directory and its variable.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- Trust the cache directory's name without hashing: a hand-edited cached pack would pass unseen, and a few KB hash in microseconds.
- The user's config directory: it holds files the user writes, and the cache holds what `mt` can fetch again, as Typst keeps `@preview` packages.


== Out of scope

- Writing the cache: `fetch-the-locked-pack-sources-with-mt-packs-install.typ`.


== Dogfood

Not applicable: this repo adopts only the built-in `mindtape:` packs.

