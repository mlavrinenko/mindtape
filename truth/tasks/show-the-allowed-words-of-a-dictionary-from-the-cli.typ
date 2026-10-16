#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Show the allowed words of a dictionary from the CLI",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli", "docs"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a refused title names the dictionary and the nearest allowed words, so the fix needs no guessing],
      cheap-adoption: helps[a project that extends the pack finds out what its verb rule accepts without reading the pack],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[SKILL.md points at a command, so no shipped word list goes stale],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 16),
)

== Problem

+ Run `mt add Banana the thing` in a project that extends `mindtape-tasks`.
+ 🔴 It prints `rules.imperative-title: start the title with an imperative verb`. The line names no dictionary and no allowed word.
+ Look for the vocabulary. `mt config show` prints `[dicts.verbs] include = ["mindtape-verbs", "mindtape-dev-verbs"]`, the names but no entries. `mt config show --origin` only says the rule comes from a pack. `mt views` lists views, and nothing lists dictionaries.
+ 🔴 The 500 + 318 built-in words sit in `crates/mindtape/src/dicts/*.txt`, and a project dictionary can add `values`, a `path` file and `include`. A user or an agent can only read source or guess. SKILL.md must not ship the list, because dictionaries are per project.


== TO-BE

- 🟢 A user or agent lists the dictionaries of the project and reads the entries of one, from the CLI alone.
- 🟢 A refusal caused by `in_dict(...)` says which dictionary missed and the closest allowed words.
- 🟢 SKILL.md names the command and carries no word list.


== Changes

- Owner decides between (a), (a) plus (c) and the rest of Alternatives. Recommended: (a) plus (c).
- `crates/mindtape-cli/src/cmd/dicts.rs`: new read command beside `views.rs`. `mt dicts` lists each dictionary with its entry count and case. `mt dicts <name>` lists its entries. Both take `--json`.
- `crates/mindtape/src/config/dicts.rs`: hand the compiled dictionaries to the CLI.
- `crates/mindtape/src/filter/strings.rs`: carry the missed dictionary and the nearest words on the violation, for the refusal hint.
- `crates/mindtape-cli/tests/it/`: red test first, a refused `mt add` naming the dictionary.
- `docs/src/skill.typ`: point at `mt dicts`; regenerate SKILL.md.


== Alternatives

- (a) `mt dicts [name]` with `--json`. One command, answers before the first refusal, works for every dictionary.
- (b) Fold entries into `mt config show`. About 800 more lines of context, the waste the owner wants gone.
- (c) A helpful refusal. The engine must learn which `in_dict` failed and a `message` is static. It helps at the moment of need but cannot list on request.
- (d) List the words in the docs. A copy that drifts from the code and from project overrides.


== Out of scope

- Editing dictionaries from the CLI. A dictionary stays config.
- Changing the contents of `mindtape-verbs` or `mindtape-dev-verbs`.
- Listing rules, which is its own task.


== Dogfood

This repo's `imperative-title` rule reads the `verbs` dictionary, so `mt dicts verbs` answers the first refused title here.
