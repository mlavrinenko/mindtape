#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Fix the NegatedObject rule's truncated span and over-broad lookahead",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.95,
    ease: 8,
  ),
  difficulty: 2,
  links: (
    (
      task: "add-quiet-and-verbose-flags-to-mt-fix.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "tooling", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a prose rule catches the shape it names],
      config-over-code: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 21),
)

== AS-IS

+ Fixing the prose in eleven filed tasks so `just docs::vale-tasks` passes
+ 🔴 #raw("nix/dev/vale-styles.nix"): the `nothing` token reports a truncated span, and its lookahead drops the construction the rule exists to catch

```
$ vale --no-global --config=.config/vale.ini --output=line t.typ
t.typ:3:13:Prose.NegatedObject:AI stilted negation: 'finds no m'. ...
t.typ:5:13:Prose.NegatedObject:AI stilted negation: 'shows no way'. ...
```

Vale misses three lines of the same file: `prints nothing to stdout`, `writes nothing to disk` and `does nothing for the user`.

== TO-BE

- 🟢 The `finds?|found|sees?|saw` token ends in the noun tail the other tokens use.
- 🟢 The `nothing` token's lookahead keeps `but`, `more`, `less` and `else`, and drops `to`, `for`, `in`, `on` and `from`.
- 🟢 Every token shares one idiom list, so `shows no way` and `finds no way` agree.
- 🟢 The `nothing` token lists each verb once.

```
$ vale --no-global --config=.config/vale.ini --output=line t.typ
t.typ:3:13:Prose.NegatedObject:AI stilted negation: 'finds no match'. ...
t.typ:7:13:Prose.NegatedObject:AI stilted negation: 'prints nothing to stdout'. ...
```

== Changes

- `nix/dev/vale-styles.nix`: a `yq` edit beside the `FormalRegister` one, rewriting the two tokens and deduplicating the verb list.

== Out of scope

- The upstream rule. `tbhb/vale-ai-tells` owns it, and this repo pins a version.
- The other rules' exclusion lists. Each gets its own finding if it drifts.

== Dogfood

The next prose sweep stops writing around a rule that misses its own target shape.
