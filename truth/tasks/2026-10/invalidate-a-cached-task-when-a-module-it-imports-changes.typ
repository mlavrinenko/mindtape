#import "@local/mindtape:0.3.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "Invalidate a cached task when a module it imports changes",
  kind: "dev",
  priority: ice(
    impact: 8,
    confidence: 0.95,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 2),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a warm read returns what a cold eval returns],
      docs-match-code: helps[internals.typ's cache promise holds for tasks that import a module],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 8)[warm load re-hashes imported modules],
)

== AS-IS

+ Reviewing the docs before release, checking internals.typ's cache claim against a task that imports a shared module
+ 🔴 #raw("crates/mindtape/src/loader/cache.rs"): the cache key covers the task file's bytes and the build key (mt version, config, prelude) but never the modules the task imports. mt ls then shows a field computed from an edited module at its old value

```
$ # tasks/imp.typ: #import "../shared.typ": t, then title: t
$ mt ls                      # shared.typ says "Title one"
Title one
$ # edit shared.typ to "Title TWO"
$ mt ls
Title one
$ MINDTAPE_NO_CACHE=1 mt ls
Title TWO
```
`typst_harvest::Harvest` already returns `dependencies: Vec<PathBuf>` (typst-harvest 0.6.0, src/harvest.rs:36).


This fell outside the task that surfaced it, so I filed it here rather than
leave it in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/loader/cache.rs").

== TO-BE

- 🟢 Editing a module a task imports makes the next `mt ls` show the new value, with the cache on.
- 🟢 A task whose file and imports haven't changed still reads from the cache without a Typst eval.


== Changes

- `crates/mindtape/src/loader/cache.rs`: store each dependency's path and content hash in the entry, excluding the task file and the prelude the build key already covers. On a read, re-hash them and treat any mismatch or missing file as a miss. Bump the cache format version.
- `crates/mindtape/src/loader/` (the module that writes entries from a `Harvest`): pass `Harvest::dependencies` into the entry.
- Add a test beside the cache: a task imports a module, the test edits the module, and the warm load returns the new field.


== Rollback

- Revert the commit. The format bump sends old entries to a fresh directory either way.


== Out of scope

- Rule functions such as `check-goals` already re-run on every load.


== Dogfood

Every task in truth/tasks/ imports `../lib.typ`, so an edit to truth/lib.typ reaches `mt ls` here without clearing the cache.

