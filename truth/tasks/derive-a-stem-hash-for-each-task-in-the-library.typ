#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Derive a stem hash for each task in the library",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("mt",),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[an agent addresses a task by a short code that every listing agrees on],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a pure module replaces ordinal bookkeeping spread over the CLI],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 3),
)

== Problem

+ Run `mt ls 'has_tag("found")' --columns '#,title'` and note a handle, such as `7i`.
+ Run `mt eval 7i`.
+ 🔴 It names another task, or refuses with `handle check mismatch`: a handle is an ordinal in the default view plus a check letter, so only bare `mt ls` prints handles other commands resolve.


== TO-BE

- 🟢 A `handle` module in `crates/mindtape` gives each stem a stem hash: the shortest prefix of its code no other stem shares, at least 3 characters.
- 🟢 The code is FNV-1a 64 of the stem in lowercase Crockford base32. Folders do not enter it.
- 🟢 Resolving input yields the stem, the stems it is ambiguous between, or no match. Input ignores case and reads `i`, `l` as `1` and `o` as `0`.
- 🟢 A new stem sharing a prefix lengthens only the hashes that share it, so an older short hash resolves as ambiguous, never to another task. Equal full codes stay ambiguous at every length.


== Changes

- `crates/mindtape/src/handle_tests.rs`: red tests first: one stem's code pinned to a literal so the hash stays stable, the 3-character floor, growth on a shared prefix, input normalisation, ambiguity, no match, and a full collision through an injected hash.
- `crates/mindtape/src/handle.rs`: the hash, the encoding, the prefixes and the resolver.
- `crates/mindtape/src/lib.rs`: re-export the module.


== Alternatives

- An ordinal per listing: stable only within one listing, which is the bug.
- A content hash: every write changes it, so `mt wip <h>` then `mt done <h>` fails.
- Base62: case-sensitive, and `l`, `1`, `I`, `O`, `0` blur when typed or read aloud.


== Out of scope

- Wiring the module into `mt` commands, which is the next task.
- Caching: hashing every stem costs microseconds per run.


== Dogfood

Not applicable until the next task wires it into `mt`.

