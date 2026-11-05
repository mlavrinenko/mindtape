#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Decide on one command for the project's listings",
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
  status: done(2026, 11, 5),
  links: depends-on("add-derived-task-fields-and-a-regex-collector.typ"),
)

== AS-IS

+ Run `mt rules`, `mt tags`, `mt views` and `mt dicts` in a project.
+ 🔴 Each answers one slice of what the project enforces and why, so a reader runs four commands and still misses the collectors and fields.
+ Run `mt config show` to read the merged config.
+ 🔴 The raw config names every key but says nothing about the origin, the readers or the shape a rule sees.


== TO-BE

- 🟢 `mt explain` is the name. It is a verb, collides with no existing command, and is the umbrella for the project's listings.
- 🟢 It absorbs `mt rules`, `mt tags`, `mt views` and `mt dicts`; `mt explain rules` keeps the derived-fields block.
- 🟢 A bare `mt explain` may show an aggregate overview, and a subcommand reaches one listing. The exact split is a follow-up.
- 🟢 `--json` keeps each listing's current keys, and `--for add|flip|commit` keeps filtering rules and tags.

```text
$ mt explain rules
$ mt explain views
$ mt explain dicts
```


== Changes

- `truth/tasks/decide-on-one-command-for-the-projects-policy-listings.typ`: records this decision; no code changes.


== Alternatives

- `mt info`: too generic, and it names the input rather than the answer.
- `mt describe`: clear, but longer to type; `explain` is the same call in one word.
- `mt policy`: covers rules and tags, but not views and dicts.
- `mt show`: collides with `mt config show`.
- Keep four commands: the friction is remembering which one holds the answer.


== Out of scope

- The implementation, including the exact subcommand split and the bare overview; the follow-up `fold-the-project-listings-into-mt-explain.typ` settles them and keeps the JSON shapes.
- A richer entry with origin and readers; `mt config show --origin` stays the source for origins.


== Dogfood

After this lands, this repo runs `mt explain rules`, `mt tags`, `mt views` and `mt dicts` under one command, and its docs point at one listing surface.
