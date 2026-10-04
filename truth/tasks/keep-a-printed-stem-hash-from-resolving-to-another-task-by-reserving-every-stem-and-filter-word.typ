#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep a printed stem hash from resolving to another task by reserving every stem and filter word",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 4),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[the hash an agent reads off a listing acts on that row's task],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[the filter-word list cannot drift from the dispatcher],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 4),
  links: related(
    "give-the-filter-functions-one-registry-that-dispatch-docs-coverage-and-mt-rms-filter-words-read.typ",
  )[the reserved filter words come from the registry],
)

== Problem

+ Reviewing the stem-hash commits 45d958bd to 9006d477
+ 🔴 #raw("crates/mindtape/src/handle.rs"): \`StemHashes\` mints hashes without knowing the stems, so a printed hash equal to another task's exact stem resolves to that stem's task, and one equal to a filter word reads as a filter in \`mt rm\`.

```
$ mt ls --columns 'slug#,title,file'
SLUG#  TITLE           FILE
hb3    Alpha           tasks/alpha.typ
6da    Something else  tasks/hb3.typ
$ mt wip hb3
flipped hb3 (Something else): todo → wip
```
`util.rs:160` tries an exact stem before a hash. `filter_dispatch_tests.rs` checks that each of the 17 `WORDS` (`filter_eval.rs:21`) dispatches, not that each dispatched word is listed.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/handle.rs").

== TO-BE

- 🟢 A printed stem hash never equals a loaded stem or a word `Filter::names_term` accepts; such a hash lengthens past it.
- 🟢 Every word `nullary` or `function` dispatches is in `WORDS`, and a test fails when one is missing.


== Changes

- `crates/mindtape-cli/tests/stem_hash_e2e.rs`: red test first: a stem spelling another task's hash.
- `crates/mindtape/src/handle.rs`: take a reserved set and lengthen any hash it contains.
- `crates/mindtape-cli/src/util.rs`: pass every stem and filter term as reserved.
- `crates/mindtape/src/filter_dispatch_tests.rs`: the reverse test, every dispatched word is listed.


== Rollback

- Revert the commit; hashes mint without a reserved set again.


== Out of scope

- The hash function itself, which its own finding mixes.
- Moving resolution into the library, which its own finding does.


== Dogfood

Agents here pass `mt ls` hashes to `mt wip` and `mt done` without a stem ever capturing one.
