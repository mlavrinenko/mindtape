#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "unify mt add input on a key OP value token grammar",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 5,
  tags: ("cli", "docs"),
  estimate: (files: 9),
  status: done(
    2026,
    8,
    17,
  )[token grammar landed: `=` / `+=` across `--set` and `--`, \[\]= retired, dot-named slots documented. just check green],
  links: related("rename-the-typst-slot-kind-to-raw.typ")[
    the same milestone: this fixes the shape of an input, that one fixes what
    an input may be
  ]
    + related("showcase-yaml-as-the-structured-input-shape.typ")[
      the structured half of the same interface
    ],
)

== Summary

`mt add` has three input spellings for what is one idea. A scalar is
`--set k=v`, a list append is `--set 'k[]=v'`, and anything structured is
`-f`. The bracket form is the odd one: it encodes a verb in the key's suffix,
where nothing else does, and it needs shell quoting to survive globbing.

Replace the suffix with an operator, and accept the same token in two places:

```sh
mt add "Practice arpeggios" --set estimate.files=2 --set tags+=docs
mt add "Practice arpeggios" -- estimate.files=2 tags+=docs
```

One grammar, `key OP value`, with two carriers. `=` sets, `+=` appends. The
`--` form is the shorthand. The flag form stays for scripts that build `argv`.

Nesting takes care of itself. A slot's name is an arbitrary string
(`template.rs:150` takes positional 0 verbatim) and `collect_values`
(`add_values.rs:116`) looks it up flat, so a template with a slot called
`"estimate.files"` already accepts `--set estimate.files=2` today. Verified
end-to-end against an unmodified binary. The dot is a naming convention for
template authors, not a path parser, and nothing in Rust needs to learn it.

== Scope

- `crates/mindtape-cli/src/add_values.rs`: `collect_values` splits a token
  into `key`, operator, value. `+=` appends via the existing
  `append_list_value`. `=` sets. Keep `[]=` accepted as a deprecated alias
  through the redesign, or drop it outright. We have not released, so dropping
  is on the table and is the tidier end state.
- `crates/mindtape-cli/src/add.rs`: a trailing-args capture after `--`,
  feeding the same token parser as `--set`. The tokens are identical in both
  positions. There is no second grammar to test.
- A bare positional with no `--` stays title text. `mt add "fix a=b parsing"`
  must not assign a slot named `a`. That ambiguity is why `--` exists, and it
  needs a test.
- `+word` tag sugar stays. It carries no `=`, so it cannot collide with a
  token, and it is the shortest form for the commonest case.
- Docs: `www/content/templates.typ`, `docs/src/skill.typ` and
  `skills/mindtape/references/add-slots.md` all state the `[]=` rule as the
  only way to fill a list. All three are `cli-surface` dependents. The group
  then needs re-recording with `just outdatty-update cli-surface`.
- The skill's evals (`skills/mindtape/evals/evals.json`) assert the `[]=`
  spelling. They move with it.

== Open question

Whether `[]=` survives as an alias. Keeping it means two spellings for append
forever, which is the thing this task exists to remove. Dropping it is a
breaking change we are free to make pre-release. Prefer dropping.
