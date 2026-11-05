#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep non-UTF-8 paths in the query file list",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.95,
    ease: 5,
  ),
  difficulty: 4,
  links: (
    (
      task: "add-ast-grep-queries-as-a-shared-query-engine.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "stateless"),
  x-goals: (
    product: verdicts(
      (
        plain-files: helps[a file the repository tracks is never silently dropped from a query],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[path identity survives the engine instead of collapsing in a lossy string],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 5),
)

== AS-IS

+ Reviewing both waves of the ast-grep milestone after it landed
+ 🔴 #raw("crates/mindtape/src/query/mod.rs"): A tracked file whose name is not valid UTF-8 is silently skipped by the query engine.

A git repo held `src/ok.rs` and a tracked `src/bad\xffname.rs`, both containing `warm(...)`. A collector with `glob = "src/**"` published only the UTF-8 name:

```
$ git ls-files | cat -v
"src/bad\377name.rs"
src/ok.rs

$ mt eval a --json | jq .derived
{"callers": ["src/ok.rs"]}
```

`crates/mindtape/src/git/state.rs` decodes git output with `String::from_utf8_lossy`, and `crates/mindtape/src/query/mod.rs` stores only `String` paths, so the file is never read.

== TO-BE

- 🟢 The file list keeps raw path bytes, so a tracked file with an invalid UTF-8 name is read and counted.
- 🟢 Only display and JSON use the lossy spelling; matching and counting use the path.
- 🟢 A Unix red test pins the invalid name.
- 🟢 A collector still publishes the path once, even when two names share a lossy spelling.

```
$ mt eval a --json | jq '.derived.callers | length'
2
```

== Changes

- `crates/mindtape/src/git/state.rs`: add `git_bytes` for `ls-files -z` and keep the existing lossy `git` wrapper.
- `crates/mindtape/src/query/mod.rs`: store `PathBuf` in `Tree`, the match cache and `matches`, and convert to a lossy string only at the collector edge.
- `crates/mindtape/src/git/state.rs`: add `PathFilter::admits_path`, with `admits` delegating to it.
- `crates/mindtape/src/query/tests.rs`: a Unix test writes an invalid UTF-8 name and sees it in the result.
- `crates/mindtape/src/estimate_queries.rs` and `crates/mindtape/src/collect/astgrep.rs`: adapt to the path result.

== Out of scope

- A public path type change beyond the query engine.
- Invalid names on Windows; the test is Unix-only.
- Reworking `count_paths` to read invalid names.

== Dogfood

No current file has an invalid name; the test pins the behavior before a real monorepo hits it.
