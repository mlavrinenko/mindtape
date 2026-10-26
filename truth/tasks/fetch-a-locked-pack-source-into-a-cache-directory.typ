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
  status: done(2026, 10, 26),
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

- 🟢 `pack_sources::fetch(git, ref, commit, expected: Option<&str>, cache_root) -> sha256` fetches `ref` into a temporary repository inside `cache_root`, writes `commit`'s files from git's objects with no checkout, hashes them as above, and renames them to `<cache_root>/<sha256>/`.
- 🟢 Given `expected`, the lock's hash for `mt packs install`, a different hash fails, naming both. `mt packs update` passes none and locks the hash it gets. A symlink, a submodule or a path some platform would write differently fails.
- 🟢 A target directory already present whose hash matches is success, so two fetches at once both pass.
- 🟢 A missing `git` fails as `git not found`, with a hint to install it.

== Changes

- `crates/mindtape/src/config/pack_sources/fetch_tests.rs`: the red tests first, against a local bare repository whose `.gitattributes` asks for CRLF, and a racing second fetch. `hash_tests.rs`: the framing and its order.
- `crates/mindtape/src/config/pack_sources/hash.rs` (new): the framed tree hash.
- `crates/mindtape/src/config/pack_sources/fetch.rs` (new): the fetch, the tree written from `git ls-tree` and `git cat-file --batch`, the rename, and `FetchError`.
- `crates/mindtape/src/config/pack_sources/listing.rs` (new): read the listing, refusing on every platform a path that is not ASCII, has an empty, `.` or `..` part, holds `\`, `:`, a control character or one of `<>"|?*`, ends in a dot or space, names a device (`CON`, `PRN`, `AUX`, `NUL`, `COM0`-`COM9`, `LPT0`-`LPT9`, any case, any extension), or lowercases to a path or directory another entry spells differently. `safe_join` builds each file's path from plain names only and checks it stays in the tree.
- `crates/mindtape/src/config/pack_sources/git.rs` (new): run git without the variables `git rev-parse --local-env-vars` names or `GIT_ALLOW_PROTOCOL`, with `transfer.fsckObjects=true` and `protocol.allow=never` but https, ssh, git and file, and stream `cat-file --batch`, checking each header's id, `blob` type, size and trailing newline.
- `crates/mindtape/src/config/pack_sources/scratch.rs` (new): the scratch directory, a sweep of `.fetch-*` directories over an hour old at the start of a fetch, since `Drop` doesn't run on Ctrl-C, and the rename. A 64-digit commit inits the scratch repository with `--object-format=sha256`.
- `crates/mindtape/src/config/pack_sources/mod.rs`: list the new files in the header.
- `crates/mindtape/Cargo.toml`, `Cargo.lock`: the `sha2` crate.
- `crates/mindtape/src/loader/cache.rs`: share its `hex` with the tree hash.


== Alternatives

- Fetch by bare commit: some servers refuse a fetch of an unadvertised object.
- Check out the commit with `-c core.autocrlf=false -c core.eol=lf`: the source's own `.gitattributes`, an LFS filter or a `post-checkout` hook still change the files, the last two per machine.
- Fold Unicode case and NFC for the case check: Windows and macOS fold beyond ASCII by different tables, and pack file names are ASCII, so refusing the rest is the one rule every platform agrees on.
- Hash with git's own tree id: it is SHA-1, and it hashes git objects, not the bytes `mt` reads.


== Out of scope

- The command that calls it: `fetch-the-locked-pack-sources-with-mt-packs-install.typ`.
- Pruning old cache entries: the cache is safe to delete whole.


== Dogfood

Not applicable: this repo adopts only the built-in `mindtape:` packs. The tests fetch from a local repository, on Windows too.

