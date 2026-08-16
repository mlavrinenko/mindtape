#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, pkg, terminal
#import "../utils/site.typ": u

#show: page.with(active: "/templates/")

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

Values come from several sources, highest priority first: `--set`, a JSON or
YAML file (`-f`), the positional title, environment variables, then the slot
default. A slot names its variable with `slot(…, env: "VAR")`; without one it
reads `MINDTAPE_ADD_<SLOTNAME>` instead. A missing or unreadable variable is
skipped, not an error. `-f` picks the parser from the extension (`.json`,
`.yaml`/`.yml`); stdin (`-`) and any other extension sniff the content — a
leading `{`/`[` tries JSON first, else YAML.

```sh
mt add --set title="Practice arpeggios" --set impact=8 # explicit values
mt add cleanup kitchen                                # positional → title slot
echo '{"title":"Practice arpeggios","impact":8}' | mt add -f - # JSON on stdin
mt add -f values.yaml                                 # YAML values file
mt add fix login +backend +auth                       # +word → tags slot
mt add --edit                                         # fill in your editor
```

- #strong[Positional words] join into the `title` slot: `add cleanup kitchen` →
  `cleanup-kitchen.typ`.
- #strong[`+word`] arguments are tag sugar: peeled into a `tags` slot if the
  template declares one, instead of joining the title.
- #strong[List slots] need `--set name[]=value`, repeatable to append. A bare
  `--set name=value` for a list errors — one string is never silently a list.
- #strong[Text slots are escaped] into Typst markup, whichever source the value
  came from. Prose containing `*`, `_`, `$`, `#`, `[`, `]`, `<` or `@` stays
  literal instead of opening emphasis or evaluating away. Numeric, choice and
  date slots are literals and are left alone.
- #strong[Typst slots are not escaped]: `kind: "typst"` splices its value as
  real markup, so a fill value may carry code spans, emphasis or a list. The
  opt-out belongs to the template author, never to the caller — a template is
  trusted repo content, a `--set` value is not. A `typst` slot whose value does
  not parse fails before anything is written, naming the slot.

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
