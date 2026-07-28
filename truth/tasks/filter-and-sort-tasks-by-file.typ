#import "@local/mindtape:0.2.0": *

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

This task was filed on a false premise, and the record should say so rather
than quietly shrink. It claimed `file` was absent from the filter language's
field dispatch and from `SORT_FIELDS`, so that `contains(file, "shopping/")`
and `--sort file` did not work. Both had in fact shipped months earlier
(`filter.rs` and `sort.rs`), and both worked. The filing grep asked about
`title`, `status`, and `due` and never about `file`.

One real defect survived the correction, and it is the reason this task still
had work in it. Both arms read `task.file.display()`, which prints the
platform-native separator. A filter or a `[views.*]` bundle written on a Unix
machine — `contains(file, "technique/")` — would silently stop matching on
Windows, where the same task's path renders with backslashes. Silent, because
nothing errors: the view just returns fewer tasks than it should.

== Decisions

- The filter value and the sort key read the root-relative path, not the
  working-directory-relative one the `file` column displays. A filter lives in
  `.mindtape/config.toml` or a `[views.*]` bundle; if it read the cwd-relative
  form, one view would match different tasks depending on which directory `mt`
  ran from. Display is localized, the stored value is not — the same split
  dates already have between the ISO value and `[ls] date-format`.
- The separator is `/` on every platform, so an expression written on one
  machine is portable to another.
- One implementation, not two. `Task::file_key()` on the model rebuilds the
  string from `Path::components()`; `filter.rs` and `sort.rs` both call it, so
  the filter and the sort key cannot drift apart.

== Outcome

- `Task::file_key()` added; both call sites moved onto it.
- Tests: `contains(file, "sub/")` against a nested task, full-path equality,
  mixed-set sort ordering, and the separator contract pinned through the same
  `Components` codepath Windows would take. The last one cannot fail on Linux
  by construction — it pins the contract, it does not reproduce the bug.
- Estimate revised from 4 files to the 6 the work actually touched.
