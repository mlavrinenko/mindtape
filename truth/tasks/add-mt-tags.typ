#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add an `mt tags` command that lists tags and the rules that read them",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("cli", "tooling", "stateless"),
  estimate: (src-files: 4),
  x-goals: (
    product: verdicts(
      (
        readable-tasks: helps[a task writer sees the waiver and the rule that names it in one read],
      ),
      "implementable-tasks",
      "enforced-practice",
      "cheap-adoption",
      "typst-over-markdown",
      "plain-files",
    ),
    process: verdicts(
      (:),
      "sound-codebase",
      "docs-match-code",
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 1),
)


== AS-IS

+ Run `mt rules --json` and read the `when` and `assert` strings to find the tags a rule reads.
+ 🔴 Nothing lists the tags a project allows beside the rules that read them, so an agent has to rebuild the map from the config and the rule text.
+ 🔴 The only description of an opt-out tag is a comment in `.mindtape/config.toml`, which an adopter's task writer never reads.

== TO-BE

- 🟢 `mt tags` prints each allowed tag with the rules that read it and each rule's message, and `mt tags --for add` narrows the list to the tags read at filing.
- 🟢 It derives the association by scanning each resolved rule's `when` and `assert` for `has_tag("<name>")`, so a new rule needs no registration.
- 🟢 `mt tags --json` returns the same map and a `schema_version`, and a tag that no rule reads prints as an area label.

```sh
$ mt tags --for add
no-repro             tasks.body.content.rules.as-is-has-a-repro
                     show the bug in one fenced block you can run, or tag ...
```

== Changes

- `crates/mindtape-cli/src/cmd/tags.rs`: the command, the tag scan, the `--for` filter and the `--json` envelope.
- `crates/mindtape-cli/src/cmd/mod.rs`: the module and the dispatch.
- `crates/mindtape-cli/src/cli/mod.rs`: the command registration and its help.
- `crates/mindtape-cli/src/cli/command_class.rs`: the read class entry, so the read-command tests and `mt _cli` see it.

== Alternatives

- Add `--tags` to `mt rules`: one flag, but an agent looking for a tag does not know to look under `rules`.
- A config comment: not machine-readable and absent from the pack an adopter reads.

== Out of scope

- The full rule list and its `--for` filter in `mt rules`, which stay as they are.

== Dogfood

`mt tags --for add` is the command the agents for this backlog run before filing.
