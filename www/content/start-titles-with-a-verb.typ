#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal
#import "../utils/site.typ": u
#import "../utils/recipes.typ": own-config, pack-rule, recipe-config

#let meta = (
  route: "/start-titles-with-a-verb/",
  label: "Start titles with a verb",
  title: "Start titles with a verb",
  summary: "Refuse a task whose title doesn't start with a verb, so each line of a list says what the task does.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

A backlog reads "Login redirect broken", "Redis" and "Audit log pagination".
Each line names a thing, so you open the task to learn whether it's a bug, a
feature or a question. Start each title with a verb, like "Fix the login
redirect", "Add a Redis client" and "Paginate the audit log", and the list
tells you what to do before you open anything.

= Config

#recipe-config(
  "start-titles-with-a-verb",
  before: pack-rule(
    "imperative-title",
  ),
)

= Behavior

`sections.yaml` holds the Problem and the other sections the template asks
for, so only the title changes. `mt add` refuses a title that starts with no
verb, names the nearest verbs, and writes no file. It files the ones that
start with a verb:

#terminal("start-titles-with-a-verb")

"Backport" is in neither shipped list. The `values` line adds it.

= Deep dive

`word(title, 1)` is the title's first word, and `in_dict()` looks it up in
the `verbs` dictionary, ignoring case. A hyphenated word stays whole, so
`rate-limit` is one word. `!terminal` runs the rule on open tasks only, so an
old done task with a noun for a title doesn't fail `mt check`.
#link(u("/filters/"))[Filter language] covers both functions.

`include` pulls in the two word lists `mt` ships. `mindtape-verbs` holds the
500 most frequent English verbs, and `mindtape-dev-verbs` adds the ones
software work uses, like `refactor`, `deprecate` and `rate-limit`. Run
`mt dicts verbs` to see every word the rule accepts, yours included.

Your team says "dockerize" or "backport"? Put the word in `values`. A longer
list fits better in a file with one word per line, named by `path`.
#link(u("/named-sets/"))[Views & dictionaries] covers every dictionary key.

The example project extends the `mindtape-tasks` #link(u("/packs/"))[rule pack],
which ships the rule and a `verbs` dictionary, so it only declares
`[dicts.verbs]`. That replaces the pack's dictionary by name, so keep the
`include` line when you add words.

This repo gets the rule from the pack in #own-config.
