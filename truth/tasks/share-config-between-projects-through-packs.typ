#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Share config between projects through packs",
  priority: ice(
    impact: 8,
    confidence: 0.7,
    ease: 3,
  ),
  difficulty: 3,
  kind: "milestone",
  tags: ("mt", "gating"),
  estimate: (src-files: 0),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a project takes the shared practice in one `extends` line and gets its fixes without copying],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 26),
)

== Problem

+ Open the `.mindtape/config.toml` of nine projects that adopt `extends = ["mindtape-tasks"]`.
+ 🔴 Each still copies the status legend, the ICE bands, the WIP caps, the commit rules and the `on.flip` gates by hand, and the copies drift: `judgment-stays-small` caps at 5 files in one project and 8 in another.
+ 🔴 A pack holds only rule arrays and `[dicts]`, takes no parameters, and comes only from the binary or a path inside the project.


== TO-BE

```toml
extends = [
  "mindtape:tasks",
  { pack = "mindtape:budget", with = { max-files = 8 } },
  "acme:commits",
]

[pack-sources.acme]
git = "https://github.com/acme/mindtape-packs"
ref = "v1"
```

- 🟢 A project adopts the shared practice in one `extends` line, sets what differs through `with`, and overrides the rest by name.
- 🟢 Official packs ship inside `mt` as `mindtape:<pack>`. A team's own packs come from a git source pinned in `.mindtape/packs.lock`.


== Changes

- `truth/tasks/*.typ`: the four decisions and their leaves, linked with `parent` to this one. This milestone changes no file itself.


== Out of scope

- Moving other projects onto the packs: each project files its own task once the packs ship.
- The `mt init` presets: once the packs ship, a preset can shrink to an `extends` line, which is its own task.


== Dogfood

This repo's `.mindtape/config.toml` adopts the official packs and drops the rules it copies by hand.

