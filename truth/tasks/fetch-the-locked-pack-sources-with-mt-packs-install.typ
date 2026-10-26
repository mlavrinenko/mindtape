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
  difficulty: 4,
  tags: ("mt", "gating", "cli"),
  estimate: (src-files: 5),
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
  status: wip(2026, 10, 26),
  links: parent("share-config-between-projects-through-packs.typ")
    + depends-on(
      "resolve-a-declared-pack-from-the-cache-and-verify-its-hash.typ",
    ),
)

== Problem

+ Clone a project whose `.mindtape/packs.lock` pins `acme`, on a machine whose cache lacks it.
+ 🔴 `mt check` fails with `pack source not installed: 'acme'`, and no command fetches it.


== TO-BE

```console
$ mt packs install
fetched acme 3f2c1a9 into ~/.cache/mindtape/packs/9c41…
$ mt packs install
1 source up to date
```

- 🟢 `mt packs install` reads the sources and the lock with `pack_sources::read_raw`, without loading the config it repairs, and calls `pack_sources::fetch` for each locked source the cache lacks.
- 🟢 It refuses a lock that disagrees with `[pack-sources]`, as `npm ci` does, with a hint to run `mt packs update`.
- 🟢 It counts as a read command, since it changes no authored file.


== Changes

- `crates/mindtape-cli/tests/it/packs.rs`: the red tests first, fetching from a local bare repository by path, and `crates/mindtape-cli/tests/it/main.rs` declaring it. `support::isolated_env` points `MINDTAPE_CACHE_HOME` at `support::CACHE_HOME` under the test's directory.
- `crates/mindtape-cli/src/cmd/packs/mod.rs` (new): `mt packs` and its `install` subcommand. It passes `pack_sources::cache_root()`, `<cache dir>/mindtape/packs`, as `cache_root`, the directory the load reads, and a relative `git` path joined to the project root, since `fetch` runs git in the caller's directory.
- `crates/mindtape-cli/src/diag/from_lib.rs`: convert `pack_sources::FetchError`, matching every variant, with `FetchError::hint` as the hint.
- `crates/mindtape-cli/src/cmd/mod.rs`: list the folder in the module header.
- `crates/mindtape-cli/src/cli/mod.rs`: wire `packs`.
- `crates/mindtape-cli/src/cli/command_class.rs`: classify `packs install` as a read.
- `www/utils/site.typ`: `packs` in `CLI_GROUPS`.
- `www/content/cli/packs.typ` (new): the command page, which a `CLI_GROUPS` entry requires. `www/content/packs.typ`: a CI example.
- `docs/src/skill.typ`: tell an agent that meets `pack source not installed` to run `mt packs install`. It moved here from `resolve-a-declared-pack-from-the-cache-and-verify-its-hash.typ`, since `just docs spans` refuses a span naming a command that doesn't exist yet. `skills/mindtape/SKILL.md` is near its 6600-byte cap, so tighten other prose to fit.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- Fold it into `mt install`: that command sets up the machine, while this one restores what one project pins.
- Load the config first, as other commands do: the load fails on the very source this command installs.


== Out of scope

- Moving a source to a newer commit: `move-pack-sources-along-their-ref-with-mt-packs-update.typ`.


== Dogfood

Not applicable: this repo adopts only the built-in `mindtape:` packs.

