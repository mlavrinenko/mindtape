#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Fetch the locked pack sources with mt packs install",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.6,
    ease: 5,
  ),
  difficulty: 5,
  tags: ("mt", "gating", "cli"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[one command, in CI or a fresh clone, restores every pack the lock pins],
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
      "resolve-a-declared-pack-from-the-cache-and-verify-its-hash.typ",
    ),
)

== Problem

+ Clone a project whose `.mindtape/packs.lock` pins `acme`, on a machine whose cache lacks it.
+ 🔴 `mt check` fails with `pack source "acme" is not installed`, and no command fetches it.


== TO-BE

```console
$ mt packs install
fetched acme 3f2c1a9 into ~/.cache/mindtape/packs/9c41…
$ mt packs install
1 source up to date
```

- 🟢 For each locked source not in the cache, `mt` runs `git` to fetch the locked commit into a temporary directory, hashes it, and renames it into place only when the hash matches the lock.
- 🟢 It refuses a lock that disagrees with `[pack-sources]`, as `npm ci` does, and a hash mismatch, naming the source and both hashes.
- 🟢 Two installs at once, as from two worktrees, both succeed: the rename is atomic and the second one keeps the first's copy.
- 🟢 A missing `git` fails with a hint to install it.


== Changes

- `crates/mindtape-cli/tests/it/packs.rs`: the red tests first, fetching from a local bare repository by path, and `crates/mindtape-cli/tests/it/main.rs` declaring it.
- `crates/mindtape/src/config/pack_sources.rs`: fetch a source at a commit into the cache.
- `crates/mindtape-cli/src/cmd/packs/mod.rs` (new): `mt packs` and its `install` subcommand.
- `crates/mindtape-cli/src/cmd/mod.rs`: list the folder in the module header.
- `crates/mindtape-cli/src/cli/mod.rs`: wire `packs`.
- `www/content/cli/packs.typ` (new), `www/content/packs.typ`: the command and a CI example.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- Fold it into `mt install`: that command sets up the machine, while this one restores what one project pins.
- Fetch with an HTTP client: a TLS stack in `mt`, and private repositories would need a token scheme of their own.


== Out of scope

- Moving a source to a newer commit: `move-pack-sources-along-their-ref-with-mt-packs-update.typ`.
- Pruning old cache entries: the cache is safe to delete whole.


== Dogfood

Not applicable: this repo adopts only the built-in `mindtape:` packs. The tests fetch from a local repository, on Windows too.

