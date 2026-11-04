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

- 🟢 `mt explain` is the name. It absorbs `mt rules` and `mt tags`; `mt explain rules` keeps the derived-fields block.
- 🟢 A bare `mt explain` prints both listings under one header, and `--json` keeps each listing's current keys.
- 🟢 `--for add|flip|commit` keeps filtering rules and tags; `mt explain tags` keeps its reader line.
- 🟢 `mt views` and `mt dicts` stay separate named-set lookups, so the command covers policy, not every listing.

```text
$ mt explain rules
error  judgment-stays-small  split the decision from the mechanical fan-out
```


== Changes

- `truth/tasks/decide-on-one-command-for-the-projects-policy-listings.typ`: records this decision; no code changes.


== Alternatives

- `mt info`: too generic, and it names the input rather than the answer.
- `mt describe`: clear, but longer to type; `explain` is the same call in one word.
- `mt policy`: covers rules and tags, but not the derived fields a rule reads.
- `mt show`: collides with `mt config show`.
- Absorb `mt views` and `mt dicts` too: they are named-set lookups with names, not policy.
- Keep two commands: the friction is remembering which one holds the answer.


== Out of scope

- The implementation; a follow-up `fold-mt-rules-and-mt-tags-into-mt-explain.typ` moves them and keeps the JSON shapes.
- A richer entry with origin and readers; `mt config show --origin` stays the source for origins.


== Dogfood

After this lands, this repo runs `mt explain rules` and `mt explain tags` instead of two commands, and its docs point at one policy surface.
