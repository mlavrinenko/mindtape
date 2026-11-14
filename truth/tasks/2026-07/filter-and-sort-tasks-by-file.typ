#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Filter and sort tasks by file",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("cli",),
  estimate: (files: 6),
  status: done(2026, 7, 28),
)

== Summary

I filed this task on a false premise, and the record should say so rather
than quietly shrink. It claimed `file` was absent from the filter language's
field dispatch and from `SORT_FIELDS`, so that `contains(file, "shopping/")`
and `--sort file` did not work. Both had in fact landed months earlier
(`filter.rs` and `sort.rs`), and both worked. The filing grep asked about
`title`, `status`, and `due` and never about `file`.

One real defect survived the correction, and it is the reason this task still
had work in it. Both arms read `task.file.display()`, which prints the
platform-native separator. A filter or a `[views.*]` bundle written on a Unix
machine, such as `contains(file, "technique/")`, would silently stop matching on
Windows, where the same task's path renders with backslashes. Silent, because
nothing errors: the view just returns fewer tasks than it should.

== Decisions

- The filter value and the sort key read the root-relative path, not the
  working-directory-relative one the `file` column displays. A filter lives in
  `.mindtape/config.toml` or a `[views.*]` bundle. If it read the cwd-relative
  form, one view would match different tasks depending on which directory `mt`
  ran from. Display localizes the path, and the stored value stays raw: dates
  already split the same way between the ISO value and `[ls] date-format`.
- The separator is `/` on every platform, so an expression written on one
  machine is portable to another.
- One implementation, not two. `Task::file_key()` on the model rebuilds the
  string from `Path::components()`. `filter.rs` and `sort.rs` both call it, so
  the filter and the sort key cannot drift apart.

== Outcome

- `Task::file_key()` added, and both call sites moved onto it.
- Tests: `contains(file, "sub/")` against a nested task, full-path equality,
  mixed-set sort ordering, and the separator contract pinned through the same
  `Components` code path Windows would take. The last one cannot fail on Linux
  by construction: it pins the contract, it does not reproduce the bug.
- Estimate revised from 4 files to the 6 the work actually touched.
