#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Decide on one command for the project's policy listings",
  kind: "decision",
  priority: ice(
    impact: 5,
    confidence: 0.7,
    ease: 8,
  ),
  difficulty: 3,
  tags: ("mt", "tooling", "gating", "stateless"),
  estimate: (src-files: 0),
  x-goals: (
    product: verdicts(
      (
        readable-tasks: helps[one command answers what the project enforces and where each entry comes from],
      ),
      ..(
        "implementable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        docs-match-code: helps[one listing keeps the CLI reference and the policy docs in one place],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 4),
  links: depends-on("add-derived-task-fields-and-a-regex-collector.typ"),
)

== AS-IS

+ Run `mt rules`, `mt tags`, `mt views` and `mt dicts` in a project.
+ 🔴 Each answers one slice of what the project enforces and why, so a reader runs four commands and still misses the collectors and fields.
+ Run `mt config show` to read the merged config.
+ 🔴 The raw config names every key but says nothing about the origin, the readers or the shape a rule sees.


== TO-BE

- 🟢 `mt explain` is the one command, with `rules`, `tags`, `collectors`, `views` and `dicts` as subcommands.
- 🟢 A bare `mt explain` prints the project's policy overview, and `--json` carries the same entries.
- 🟢 `--for add|flip|commit` still filters the rules a step enforces.
- 🟢 Each entry names its kind, origin and readers, so one command answers what the old four answered in pieces.

```text
$ mt explain rules
error  judgment-stays-small  split the decision from the mechanical fan-out
```


== Changes

- `truth/tasks/decide-on-one-command-for-the-projects-policy-listings.typ`: records this decision; no code changes.


== Alternatives

- `mt info`: too generic, and it names the input rather than the answer.
- `mt describe`: clear, but longer to type and less direct than `explain`.
- `mt policy`: covers rules and tags, but not views and dictionaries.
- `mt show`: collides with `mt config show` and `mt views`.
- Keep four commands: the friction is remembering which one holds the answer.


== Out of scope

- The implementation; a follow-up task moves `mt rules` and `mt tags` under the chosen name and keeps their JSON shapes.
- A plugin system for new listing kinds.


== Dogfood

This repo runs `mt explain rules` and `mt explain tags` instead of two commands, and its docs point at one policy surface.

