#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Let mt check --commits load each tree's own package version",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.95,
    ease: 4,
  ),
  difficulty: 5,
  estimate: (src-files: 2),
  tags: ("found", "mt", "stateless"),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[a commit range across a prelude bump passes without dropping the local-package pin],
      ),
      "readable-tasks",
      "implementable-tasks",
      "cheap-adoption",
      "typst-over-markdown",
      "plain-files",
    ),
    process: verdicts(
      (:),
      "sound-codebase",
      "docs-match-code",
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 2),
)

== AS-IS

+ Landing `prelude-verdicts` while `mt check --commits main..HEAD` judged the range
+ 🔴 #raw("crates/mindtape/src/loader/mod.rs"): a past commit's task files are evaluated with the tip's config, so a local-package pin holding a new version breaks that commit's older imports

```
$ mt check --commits main..HEAD
warning: ac00b735: truth/tasks/a-wrong-typed-required-field-is-reported-twice-the-second-time-as-missing.typ:1:9 as HEAD: package manifest contains mismatched version 0.3.0
error: ac00b735: commits.rules.flip-with-work: accept-a-folder-path-in-mt-rm: start or close a task in a commit that carries its work
error: 1 commit broke commit rules
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/loader/mod.rs").

== TO-BE

- 🟢 A range check loads each commit's task files against that tree's package version.
- 🟢 A local-package pin whose manifest version differs from the import falls through to the installed package.

```
$ mt check --commits main..HEAD
ok: commit rules pass on 2 commits
```


== Changes

- `crates/mindtape/src/commit/tasks.rs`: resolve an overlaid tree's package imports against its own version, or overlay its config.
- `crates/mindtape/src/loader/mod.rs`: let a version-mismatched local-package pin fall through.
- `crates/mindtape/src/loader/tests.rs`: the mixed-version range case.


== Out of scope

- The 0.3.0 bump itself. This finding fixes the seam that exposed it.
- The installed package layout. The installed 0.2.0 copy stays as it is.


== Dogfood

This repo's next prelude bump lands with a green `mt check --commits` and keeps its local-package pin.

