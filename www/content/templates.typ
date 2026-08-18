#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, terminal
#import "../utils/site.typ": u

#let meta = (
  route: "/templates/",
  label: "Templates & slots",
  title: "Templates & slots",
  summary: "How mt add scaffolds a task from a Typst template whose values are slot() calls.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

`mt add` files a new task from a #strong[template] — a task whose authored values
are `slot(…)` calls. It discovers the slots on the parse plane (no evaluation),
fills them, injects the status and id, then writes and validates the result. The
written file holds plain literals; no `slot` call survives.

= A template

Templates live under `[templates] dirs` (default `.mindtape/templates`). A slot
declares a name, a kind, and an optional default, prompt, or environment
variable. This is the example project's template:

#raw(read("../generated/template.typ"), lang: "typ", block: true)

Slots nest: `discover_slots` walks into `ice(…)`, so each leaf becomes its own
fillable field. Structured fields have no CLI path syntax — expose each leaf as
a slot instead.

= Inspect the slots

`--describe` prints the schema (`--describe=json` for machine use):

#terminal(read("../generated/describe.txt"))

= Fill the slots

Values come from several sources, highest priority first: the `key OP value`
tokens, a YAML or JSON file (`-f`), the positional title, environment
variables, then the slot default. A slot names its variable with
`slot(…, env: "VAR")`; without one it reads `MINDTAPE_ADD_<SLOTNAME>`
instead. A missing or unreadable variable is skipped, not an error.

```sh
mt add "Practice arpeggios" -- impact=8 tags+=technique # tokens after --
mt add --set impact=8 --set tags+=technique            # the same tokens as flags
mt add "Practice arpeggios" -- summary@notes.txt       # value from a file
mt add cleanup kitchen                                 # positional → title slot
mt add fix login +backend +auth                        # +word → tags slot
mt add -f values.yaml                                  # values file
mt add --edit                                          # fill in your editor
```

- #strong[One grammar, two carriers]: a token is `key=value` (set),
  `key+=value` (append to a list slot, repeatable) or `key@path` (read the
  value out of a file). Pass it after `--set` or bare after `--` — they parse
  identically, so the shorthand can never mean something different from the
  flag.
- #strong[The first operator separates], `=` or `@`, whichever comes first —
  the rule that already made `summary=a=b` a value holding an `=`. So
  `summary=ping @ada` sets prose containing an `@`, and `summary@v=1.txt`
  reads a path containing one.
- #strong[A bare positional is title text], never a token: `mt add "fix a=b
  parsing"` files a task by that name and sets no slot. The `--` is what makes
  an argument an assignment.
- #strong[Nesting is a naming convention], not a path syntax. A slot's name is
  any string, so a template that writes `slot("estimate.files")` accepts
  `--set estimate.files=2`. MindTape never parses the dot.
- #strong[Positional words] join into the `title` slot: `add cleanup kitchen` →
  `cleanup-kitchen.typ`.
- #strong[`+word`] arguments are tag sugar: peeled into a `tags` slot if the
  template declares one, instead of joining the title.
- #strong[A bare `name=value` for a list slot errors] — one string is never
  silently a list. Append with `+=`, or pass an array through `-f`.
- #strong[Text slots are escaped] into Typst markup, whichever source the value
  came from. Prose containing `*`, `_`, `$`, `#`, `[`, `]`, `<` or `@` stays
  literal instead of opening emphasis or evaluating away. Numeric, choice and
  date slots are literals and are left alone.
- #strong[Raw slots are not escaped]: `kind: "raw"` splices its value in
  verbatim, so a fill value may carry code spans, emphasis or a list. The
  opt-out belongs to the template author, never to the caller — a template is
  trusted repo content, a `--set` value is not. The name says what it does:
  the value is pasted, not read, so nothing builds a value out of it and
  nothing type-checks one.
- #strong[What a raw slot checks is narrow.] A value that does not parse as
  Typst markup fails before anything is written, naming the slot — that covers
  a `#slot(…)` in the body, which is what the kind is for. A raw slot spliced
  into an argument position is a different plane: there the value is an
  expression, and markup reads parens and `*` as ordinary text, so a broken
  one reaches the file and `mt add` fails loading what it wrote. Use `raw` for
  body markup; reach for a typed kind anywhere a value has a shape.

= A value from a file with `@`

`key@path` puts a file where a command-line value would go. It serves the long
value: a `summary` or a `scope` that is uncomfortable to type as a shell
argument and does not justify writing a whole `-f` document.

```sh
mt add "Practice arpeggios" -- summary@notes.txt   # prose into a text slot
mt add "Split or re-cap ls.rs" -- remedy@remedy.typ # markup into a raw slot
```

The operator carries bytes; it converts nothing. What they become is decided
by #strong[the slot's kind], exactly as for a typed value:

- Into a #strong[`text`] slot the bytes are markup-escaped like any other
  caller value. The file contributes prose, not formatting — which is why the
  example is `.txt`. A `.md` file is a trap: nothing converts Markdown to
  Typst, so its headings and code spans land as the literal `#` and backticks
  they are.
- Into a #strong[`raw`] slot the bytes are Typst, spliced verbatim. This is
  the case the operator is worth building for — write the `.typ`, check it,
  then file the task, instead of making the markup survive a shell argument.
- Into any #strong[other kind] the file is read too, and coerced like a typed
  value. `weight@score.txt` fills an `int` slot; a file that does not fit
  fails naming the slot, the same error the same value typed inline would get.

Trailing newlines are stripped and nothing else is, so `@` is a drop-in for
the `--set "key=$(cat path)"` it replaces. There is no append form: a file is
one value, and a list goes through `-f` as an array. There is no stdin form
either — `-f -` is that channel, and a YAML block scalar carries a prose slot
verbatim.

= Structured values with `-f`

`-f` reads a whole set of values at once. It takes YAML or JSON: `.json` and
`.yaml`/`.yml` pick the parser by extension, while stdin (`-`) and any other
extension sniff the content — a leading `{`/`[` tries JSON first, else YAML.
YAML is a superset of JSON, so one channel accepts both.

Prefer YAML, and prefer it especially when you are generating the values
rather than typing them. There is no shell quoting to get right, and a block
scalar carries a prose slot verbatim — which matters, because a `summary` or
`scope` is exactly the value JSON makes you escape hardest.

```sh
mt add -f - <<'YAML'
title: Practice arpeggios
estimate.files: 2
tags: [technique, daily]
summary: |
  Work the 7th chords hands-separate, then together at 60bpm.
YAML
```

Sources merge left to right, and `-f` is repeatable: a later source's value
for a key replaces an earlier one, and an explicit `null`/`~` unsets it.

= Placement and preview

```sh
mt add -t bug --name oauth-bug --to auth   # named template, explicit stem, nested subdir
mt add --set title="…" --print             # preview without writing
mt add --into .                            # target the first root explicitly
```

`--to <subdir>` nests under a root (`--to shopping` → `<root>/shopping/`);
`--name` sets the stem directly.

= Required core fields always surface

If your config marks `priority`, `difficulty`, `kind`, or `estimate` as required
but the template omits it, `mt add` splices in a slot for it before discovery —
so it appears in the scaffold, in `--describe`, and in the written task, even
with the barebones built-in `task` template.

#callout("note")[
  Strict-framework priority (`framework` + `allow-manual = false`) has no scalar
  form, so the splice writes the framework call itself — one slot per input, in
  the generic `framework("name", …)` spelling. See
  #link(u("/priority/"))[Priority].
]

= Editor mode

`--edit` (or `[add] interactive`) opens an editor on a commented YAML scaffold of
the slots, then parses the saved buffer back into fill values. The editor resolves
`[add] editor` → `$VISUAL` → `$EDITOR` → `vi` (`notepad` on Windows), and only
opens with a TTY.
`[add] header` overrides the instruction block; `[add] comments = false` strips
every `#` comment (header and per-slot annotations), leaving a bare mapping.
