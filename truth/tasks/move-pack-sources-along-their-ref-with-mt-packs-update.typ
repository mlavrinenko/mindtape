#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Move pack sources along their ref with mt packs update",
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
      enforced-practice: helps[a pack update prints every rule and `run` line it changes before the lock commits it],
      cheap-adoption: helps[taking a pack's next fix is one command whose output says what the fix changes],
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
    + depends-on("fetch-the-locked-pack-sources-with-mt-packs-install.typ"),
)

== Problem

+ Add `[pack-sources.acme]` with `ref = "v1"`, or push a fix to `v1` of an already locked source.
+ 🔴 Nothing writes the source into `.mindtape/packs.lock` or moves the lock to the new commit, so the project stays on the stale lock that `mt` refuses.


== TO-BE

```console
$ mt packs update acme
acme v1: 3f2c1a9 → 8d01e44
wrote .mindtape/packs.lock
```

- 🟢 `mt packs update [<source>…]` resolves each `ref` to a commit, fetches it with `pack_sources::fetch`, and rewrites the lock entry once every fetch passed. With no source it updates them all, and it drops lock entries no source declares.
- 🟢 A 40-hex `ref` is the commit itself. Otherwise `git ls-remote` answers, and for an annotated tag the peeled `^{}` line wins.
- 🟢 A source whose `ref` still names the locked commit prints `acme v1: up to date` and leaves the lock alone. A new source prints `acme v1: locked 3f2c1a9`, and a dropped entry `dropped acme`, with or without source names.
- 🟢 It reads the sources with `pack_sources::read_raw`, and counts as a write command.


== Changes

- `crates/mindtape-cli/tests/it/packs/update.rs`: the red tests first: a first lock, a moved annotated tag, a commit `ref`, an unchanged ref, a dropped source.
- `crates/mindtape/src/config/pack_sources/remote.rs` (new), `remote_tests.rs`: `resolve`, a `ref` to a commit, through the fetch's `Git`, and `FetchError::UnknownRef`.
- `crates/mindtape/src/config/pack_sources/lock.rs`: write the lock.
- `crates/mindtape-cli/src/cmd/packs/update.rs` (new): the `update` subcommand, with `install` moved to `install.rs` beside it.
- `crates/mindtape-cli/tests/it/packs/`: `packs.rs` becomes a folder, `install.rs` and `update.rs`.
- `crates/mindtape-cli/src/cli/command_class.rs`: classify `packs update` as a write.
- `www/content/packs.typ`: the command.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- Let `mt packs install` write a missing lock entry: then CI could pass on a lock nobody reviewed.


== Out of scope

- Reporting what the new commit changes: `print-what-a-pack-update-changes.typ`.


== Dogfood

Not applicable: this repo adopts only the built-in `mindtape:` packs.

