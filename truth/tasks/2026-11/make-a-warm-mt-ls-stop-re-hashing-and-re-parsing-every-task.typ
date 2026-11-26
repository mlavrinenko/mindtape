#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Make a warm mt ls stop re-hashing and re-parsing every task",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("mt", "tooling", "stateless"),
  assignee: "agent",
  status: done(2026, 11, 27),
)

== AS-IS

A warm `mt ls` spends four fifths of its instructions on work it already did.

+ Run `mt ls` in this repository, then `just bench::profile`.
+ 🔴 `TaskCache::content_hash` is 56% of the run: every task's entry names `lib/prelude.typ`, and the loader reads and SHA-1s that 30 KB file once per task, 1129 times.
+ 🔴 `assemble_task` is 24% more: every cache hit runs `TaskCallSpans::parse`, a full Typst parse of the file to find its `task.with(…)` call.
+ 🔴 The warm path runs on one thread while nineteen cores idle.


== TO-BE

- 🟢 A warm load reads and hashes each project-local dependency once, however many tasks import it.
- 🟢 A cache hit replays the spans the cold evaluation stored, so it never parses a task file through Typst.
- 🟢 The warm path runs on one thread per core, and the chunks merge in order. The listing comes out the same.
- 🟢 `load/warm/2000` retires 173 M instructions, down from 2.5 G on this backlog.

```
$ hyperfine --shell=none '/tmp/mt-before-bin ls' '/tmp/mt-after ls'
BEFORE real repo     230.3 ms ±  14.3 ms
AFTER  real repo      97.4 ms ±  9.8 ms
```


== Changes

- `crates/mindtape/src/loader/cache.rs`: a per-load memo of dependency content hashes, and an entry that lists each dependency once. The entry also holds the call spans, the format version moves to 5, and `hex` renders a digest with a digit lookup.
- `crates/mindtape/src/model/task_spans.rs`: `TaskCallSpans` serializes, so the cache can hold it.
- `crates/mindtape/src/loader/assemble.rs`: `assemble_task` takes the spans instead of parsing the source.
- `crates/mindtape/src/loader/mod.rs`: `split_hits` runs on one thread per core and merges its chunks in order. `record_miss` parses the spans once and stores them.
- `crates/mindtape/src/loader/many.rs`: `map_chunks` and `shards`, a bounded split beside `map_parallel`'s one-thread-per-item.


== Alternatives

- Validating a hit by `stat` instead of by content hash. It would skip reading the task file at all, but a restored mtime would then serve a stale task, and the cache promises it never does.
- One packed cache file per build key. It removes 1129 opens, most of the remaining 10 ms of harvest, but every write then rewrites the whole file and two `mt` processes racing lose each other's entries.
- Sharding the cache directory. It spreads the opens over 256 directories without changing the entry format, but the opens stay, and the filesystem is the floor either way.


== Out of scope

- The cold path. It is 13.8 s of blocked time, and `strace` puts it in `futex` and `sched_yield` rather than in `fsync`.
- The four `git` subprocesses, 15 ms of a warm run. Merging the two `rev-parse` calls and scoping `ls-files` to the root's pathspec is worth about 5 ms and is its own task.
- `collect::annotate`, 9 ms on this backlog. It is per-task work that could run on the same chunks.


== Dogfood

This repository's own `mt ls` is the case the numbers come from.


== Goals

+ #Helps(maintainer, to: keep-changes-safe)[because the cache still validates every byte it serves, and the speedup costs no correctness].
+ #Helps(adopter, to: keep-plain-files)[because the cache stays one file per entry, and a lost entry costs one re-evaluation].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, explain-itself, match-docs-to-code).

