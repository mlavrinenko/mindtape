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
  estimate: (src-files: 3),
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
  status: proposed(2026, 10, 26),
  links: parent("share-config-between-projects-through-packs.typ")
    + depends-on("fetch-the-locked-pack-sources-with-mt-packs-install.typ"),
)

== Problem

+ Add `[pack-sources.acme]` with `ref = "v1"`, or push a fix to `v1` of an already locked source.
+ 🔴 Nothing writes the source into `.mindtape/packs.lock` or moves the lock to the new commit, and a reviewer of the lock's diff sees only a new hash.


== TO-BE

```console
$ mt packs update acme
acme v1: 3f2c1a9 → 8d01e44
  acme:commits  ~ rule commits.rules.changelog-with-product-change
                + on.flip close-gate: run "just check"
wrote .mindtape/packs.lock
```

- 🟢 `mt packs update [<source>…]` resolves each source's `ref` to a commit, fetches it into the cache, and rewrites its lock entry. With no source it updates them all.
- 🟢 It prints, per pack, each key, rule and `on.flip` entry added, changed or removed, and every new or changed `run` line.
- 🟢 A source whose `ref` still names the locked commit prints `acme v1: up to date` and leaves the lock alone.
- 🟢 A changed `run` line still waits for `mt allow`.


== Changes

- `crates/mindtape-cli/tests/it/packs.rs`: the red tests first: a first lock, a moved tag, an unchanged ref.
- `crates/mindtape/src/config/pack_sources.rs`: resolve a `ref` with `git ls-remote`, write the lock, and diff two resolved packs.
- `crates/mindtape-cli/src/cmd/packs/mod.rs`: the `update` subcommand and its report.
- `www/content/cli/packs.typ`, `www/content/packs.typ`: the command.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- Print a text diff of the TOML: a reordered file would look like a change, while the semantic diff names what behaves differently.
- Let `mt packs install` write a missing lock entry: then CI could pass on a lock nobody reviewed.


== Out of scope

- Writing the diff into the commit message: the user copies what the review needs.


== Dogfood

Not applicable: this repo adopts only the built-in `mindtape:` packs.

