#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Polish the concepts page",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("www", "docs"),
  estimate: (src-files: 2),
  x-goals: (
    product: verdicts(
      (
        cheap-adoption: helps[a newcomer finds the hash, the file-change rule and the field checks in plain words],
      ),
      "readable-tasks",
      "implementable-tasks",
      "enforced-practice",
      "typst-over-markdown",
      "plain-files",
    ),
    process: verdicts(
      (
        docs-match-code: helps[the extension-field facts live on the config page that documents them],
      ),
      "sound-codebase",
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 16),
)

== AS-IS

+ Open `www/content/concepts.typ`, or the Concepts page it builds.
+ 🔴 Identity never mentions the short hash that `mt ls` shows, the extension-field section holds config details that belong to Configuration, and two sentences read stiffly.


== TO-BE

- 🟢 Identity says every command takes the short hash in place of the name.
- 🟢 Extension fields ends in one line that points at Configuration, which documents `[tasks.fields.x-<ns>]`.
- 🟢 The two stiff sentences use the wording the product owner approved.


== Source of truth

- `examples/api-service`: `mt ls` prints the short hash in its `SLUG#` column.
- `crates/mindtape/src/config/`: `[tasks.fields.<name>]` takes `values`, `required`, `values-when` and `multiple`.


== Changes

- `www/content/concepts.typ`: add the hash paragraph, reword the file-change and mistake sentences, cut the extension-field config paragraph to one line.
- `www/content/config.typ`: add an Extension fields section that holds the moved paragraph.


== Out of scope

- Other docs pages. They keep their own depth.


== Dogfood

Not applicable. The change is docs prose.
