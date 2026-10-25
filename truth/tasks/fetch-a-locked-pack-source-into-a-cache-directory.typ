#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Fetch a locked pack source into a cache directory",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.6,
    ease: 4,
  ),
  difficulty: 5,
  tags: ("mt", "gating"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a pinned source lands once per machine, byte for byte the same on Linux, macOS and Windows],
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
    + depends-on("read-pack-sources-and-packs-lock-into-the-config.typ"),
)

== Problem

+ Lock `acme` at a commit on Linux, then restore it on a Windows runner whose git has `core.autocrlf=true`.
+ 🔴 No code puts a source into a cache, and a naive checkout turns every LF into CRLF there, so any hash of the files would differ from the lock's.


== TO-BE

```text
sha256( for each file, sorted bytewise by its /-separated path:
        u64-LE(len(path)) path u64-LE(len(bytes)) bytes )
```

- 🟢 `pack_sources::fetch(git, ref, commit, expected: Option<&str>, cache_root) -> sha256` fetches `ref` with `git -c core.autocrlf=false -c core.eol=lf` into a temporary directory inside `cache_root`, checks out `commit`, hashes it as above without `.git/`, and renames it to `<cache_root>/<sha256>/`.
- 🟢 Given `expected`, the lock's hash for `mt packs install`, a different hash fails, naming both. `mt packs update` passes none and locks the hash it gets. A symlink fails.
- 🟢 A target directory already present whose hash matches is success, so two fetches at once both pass.
- 🟢 A missing `git` fails as `git not found`, with a hint to install it.


== Changes

- `crates/mindtape/src/config/pack_sources/fetch_tests.rs`: the red tests first, against a local bare repository with a CRLF-sensitive file and a racing second fetch.
- `crates/mindtape/src/config/pack_sources/hash.rs` (new): the framed tree hash.
- `crates/mindtape/src/config/pack_sources/fetch.rs` (new): the fetch, checkout and rename.
- `crates/mindtape/src/config/pack_sources/mod.rs`: list both files in the header.
- `crates/mindtape/Cargo.toml`, `Cargo.lock`: the `sha2` crate.


== Alternatives

- Fetch by bare commit: some servers refuse a fetch of an unadvertised object.
- Hash with git's own tree id: it is SHA-1, and it hashes git objects, not the bytes `mt` reads.


== Out of scope

- The command that calls it: `fetch-the-locked-pack-sources-with-mt-packs-install.typ`.
- Pruning old cache entries: the cache is safe to delete whole.


== Dogfood

Not applicable: this repo adopts only the built-in `mindtape:` packs. The tests fetch from a local repository, on Windows too.

