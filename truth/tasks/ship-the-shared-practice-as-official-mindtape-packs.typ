#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Ship the shared practice as official mindtape packs",
  kind: "feat",
  priority: ice(
    impact: 8,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 5,
  tags: ("mt", "gating", "decision"),
  estimate: (src-files: 0),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the WIP caps, budgets and commit rules run the same way in every project that adopts them],
      cheap-adoption: helps[the practice nine projects copy by hand becomes `extends` lines that receive every later fix],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    10,
    26,
  )[agreed: mindtape:\<pack> built into mt, mindtape-tasks renamed mindtape:tasks with no alias, and eight official packs],
  links: parent("share-config-between-projects-through-packs.typ"),
)

== Problem

+ Read the `.mindtape/config.toml` of the nine projects that adopt `mindtape-tasks`.
+ 🔴 Each copies what the pack lacks: `milestone-wip-cap` and `leaf-wip-cap` word for word in nine, `tweak-lands-done` and `refs-task-is-started` in eight, the Dogfood rules in six, the ICE priority in fourteen and `[tasks.comments]` in fifteen.
+ 🔴 A built-in pack is named by the reserved `mindtape-` prefix, a form with no room for a pack from another source.


== TO-BE

```toml
extends = [
  "mindtape:tasks", "mindtape:flow", "mindtape:ice", "mindtape:wip-caps",
  { pack = "mindtape:budget", with = { max-files = 8 } },
  "mindtape:tweaks", "mindtape:commits", "mindtape:dogfood",
]
```

- 🟢 `<source>:<pack>` names a pack. The `mindtape` source is built into `mt` and needs no declaration or fetch. An entry with a `/` is still a path. `mindtape-tasks` becomes `mindtape:tasks`, with no alias.
- 🟢 `tasks` adds `[tasks.fields.kind]` and `[tasks.comments]`. `flow` gives the floor's six statuses their transitions and a stale-WIP gate, `ice` puts the ICE framework on the floor's scale, `wip-caps` caps open milestones and leaves, `budget` holds `judgment-stays-small` and `src-budget`, `tweaks` the two tweak rules, `commits` `refs-task-is-started` and `changelog-with-product-change`, `dogfood` the four Dogfood rules.


== Changes

- `truth/tasks/*.typ`: the leaves that build it, linked with `parent` to `share-config-between-projects-through-packs.typ`. This decision changes no file itself.


== Alternatives

- Official packs in their own git repository: each would need a minimum `mt` version, while a built-in pack moves with the filter language it uses and this repo's CI tests it.
- Grow `mindtape-tasks` into one big pack: a project can't take the caps without the six sections.
- Keep the `mindtape-` prefix beside `acme:`: two spellings for one idea.


== Out of scope

- The `mmz` close gate: it runs a tool other projects may lack.
- Built-in dictionary names such as `mindtape-verbs`: `in_dict` reads them in their own namespace.
- Status names as parameters: the packs name `wip` and `done`, which the floor and `mindtape:flow` share. A project with a legend of its own keeps those two names, or overrides the rules that read them by name.


== Dogfood

This repo's `.mindtape/config.toml` swaps its hand-copied rules, legend, priority and stale gate for the `extends` line above, keeping only `goals`, its `[dicts]` and its close gate.

