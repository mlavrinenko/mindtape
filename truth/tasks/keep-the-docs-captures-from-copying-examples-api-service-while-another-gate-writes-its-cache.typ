#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep the docs captures from copying examples/api-service while another gate writes its cache",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.6,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 1),
  tags: ("found", "www"),
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
      sound-codebase: helps[a gate that fails at random teaches people to rerun it instead of reading it],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    10,
    12,
  )[capture\_copy copies a fixture through tar without its .mindtape/cache/],
)

== Problem

+ Running `just check` in the allow worktree, after the docs captures changed
+ 🔴 #raw("www/capture.sh"): `capture_copy` runs `cp -r` over `examples/api-service`, cache included, and the copy fails when another process removes a cache temp file mid-copy, which failed `check-doc-facts` once and passed on the rerun.


```
cp: cannot stat '<worktree>/examples/api-service/.mindtape/cache/c441b79ad48a54e990ad356873acfa325765df19/.50d81f2fdc449d1e35007a52093aaa5990cdd766.json.43cf3df4-81c4-4e5b-87c9-7ec2c9da5cc2.tmp': No such file or directory
error: Recipe `gen` failed on line 85 with exit code 1
check: 1 of 29 gates failed: just check-doc-facts (2 not run)
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("www/capture.sh").

== TO-BE

- 🟢 A capture's copy of a fixture never reads the fixture's `.mindtape/cache/`, so no parallel `mt` run can fail the docs build.


== Changes

- `www/capture.sh`: copy the fixture without its `.mindtape/cache/`, for example with `rsync --exclude` or a `tar --exclude` pipe.


== Rollback

- Revert the `capture_copy` change.


== Out of scope

- Which parallel arm writes the fixture's cache. The copy should not depend on it.


== Dogfood

Every `just check` in this repo builds the docs captures, so it stops failing on this race.
