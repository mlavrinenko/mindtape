#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Apply the plain-prose rules to the Justfile and YAML config comments",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  estimate: (src-files: 7),
  tags: ("found", "tooling"),
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
      sound-codebase: neutral,
      docs-match-code: helps[a comment states the current behaviour; history stays in git log],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 2),
)

== Problem

+ Writing check-doc-prose, which scans only the doc sources.
+ 🔴 #raw("Justfile"): The comments in the Justfile, .just/docs.just, .mmz/config.yaml, outdatty.yaml and .linecop.yaml tell history and shout, and no check scans them.

`rg -c -i '#.*\b(used to|no longer|anymore|formerly)\b'` counts 12 lines across the Justfile, `.just/docs.just` and the three YAML configs, and 35 more across `.just/scripts/*.sh` and `www/*.{sh,jq}`. `Justfile:3` reads "now that this file no longer holds all"; `.mmz/config.yaml:319` reads "TEN OF THEM ARE NAMED BY NO RULE".


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".just/scripts/check-doc-prose.sh").

== TO-BE

- 🟢 Comments in the Justfile, `.just/docs.just` and the three YAML configs state the current behaviour and its reason, with no history words or all-caps emphasis.
- 🟢 `just check-doc-prose` scans those comments too, and the tree passes it.


== Changes

- `.just/scripts/check-doc-prose.sh`: scan `#` comments in the Justfile, `.just/docs.just`, `.mmz/config.yaml`, `outdatty.yaml` and `.linecop.yaml`.
- `docs/src/contributing/documentation.typ`: name the extra scope.
- `Justfile`, `.just/docs.just`, `.mmz/config.yaml`, `outdatty.yaml`, `.linecop.yaml`: rewrite each comment with a hit.


== Rollback

Revert the commit; the scan scope is one list in the script.

== Out of scope

- Comments in `.just/scripts/*.sh` and `www/*.{sh,jq}`: about 15 files, a pass of their own.
- Rust and TOML comments: the separate pass the docs task names.


== Dogfood

Every later gate and config comment in this repo is written and checked under these rules.
