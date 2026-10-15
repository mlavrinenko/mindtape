#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add a recipe for verb-first titles",
  kind: "docs",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("www", "docs"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: helps[a reader learns how to make every title in a list say what the task does],
      implementable-tasks: neutral,
      enforced-practice: helps[the recipe shows mt refusing a title that starts with no verb],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the page's refusal and config come from a live capture of examples/api-service],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 16),
)

== Problem

+ Open the docs site's Recipes nav group, or the recipe table in `README.md`.
+ 🔴 No recipe shows how to require a verb-first title, though the `mindtape-tasks` pack's `imperative-title` rule and the shipped verb dictionaries do exactly that.


== TO-BE

- 🟢 A Start titles with a verb recipe says why verb-first titles help, shows the few lines of config, the refusal `mt add` prints for a title like Login redirect broken, and how to add your own words.
- 🟢 Every command output on the page comes from a capture, and `README.md`'s recipe table lists the page.


== Source of truth

- `crates/mindtape/src/packs/mindtape-tasks.toml`: the `imperative-title` rule and the `verbs` dictionary.
- `crates/mindtape/src/dicts/verbs.txt` and `crates/mindtape/src/dicts/dev-verbs.txt`: the shipped words.
- `www/generate-api-service.sh`: the captured session.


== Changes

- `examples/api-service/.mindtape/config.toml`: a `start-titles-with-a-verb` slice holding the rule and a `[dicts.verbs]` with two words of the project's own.
- `www/generate-api-service.sh`: a session where `mt add` refuses a title with no verb and files two that start with one.
- `www/content/start-titles-with-a-verb.typ`, new: the recipe page.
- `www/utils/site.typ`: list the page in the Recipes nav group.


== Out of scope

- The other recipe pages. Their own polish is separate work.
- A command that lists a dictionary's words. The page points at the word files instead.


== Dogfood

Not applicable. This repo already extends the `mindtape-tasks` pack, which runs the rule.
