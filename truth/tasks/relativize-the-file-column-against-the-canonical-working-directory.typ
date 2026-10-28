#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Relativize the file column against the canonical working directory",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.7,
    ease: 6,
  ),
  difficulty: 2,
  estimate: (src-files: 1),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a Windows user whose shell reports a short name gets the same short `file` column],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 28),
)

== Problem

+ Reading the Windows CI log of `ls_limit_e2e::a_worktree_flag_pages_with_the_flag_as_typed` while fixing path separators
+ 🔴 #raw("crates/mindtape-cli/src/render/file_path.rs"): `cwd()` reads `std::env::current_dir()` uncanonicalized, so a working directory spelled with an 8.3 short name such as `RUNNER~1` shares no folder with the canonical root and the `file` cell climbs to the drive

```
0b0    wip     normal    Alpha  ..\..\..\..\..\runneradmin\AppData\Local\Temp\.tmpGCvCyY\tasks\a.typ
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/render/file_path.rs").

== TO-BE

- 🟢 From a working directory spelled with a short name, the `file` column shows the same path as from its long name, such as `tasks\a.typ` at the root.


== Changes

- `crates/mindtape-cli/src/render/file_path.rs`: canonicalize the working directory once in `cwd()` through `mindtape::canonical`, falling back to it as read.


== Rollback

Revert the commit; the column goes back to relativizing the directory as the OS reports it.


== Out of scope

- The separator, which `display_path` settles.


== Dogfood

Not applicable: this repo is developed on Linux, where a directory has one spelling.

