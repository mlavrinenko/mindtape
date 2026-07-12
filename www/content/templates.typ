#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal, callout, pkg
#import "../utils/site.typ": u

#show: page.with(
  title: "Templates & slots",
  active: "/templates/",
  summary: "How mt add scaffolds a task from a Typst template whose values are slot() calls.",
)

`mt add` files a new task from a #strong[template] — a task whose authored values
are `slot(…)` calls. It discovers the slots on the parse plane (no evaluation),
fills them, injects the status and id, then writes and validates the result. The
written file holds plain literals; no `slot` call survives.

= A template

Templates live under `[templates] dirs` (default `.mindtape/templates`). A slot
declares a name, a kind, and an optional default or prompt. This is the example
project's template:

#raw(read("../generated/template.typ"), lang: "typ", block: true)

Slots nest: `discover_slots` walks into `ice(…)`, so each leaf becomes its own
fillable field. Structured fields have no CLI path syntax — expose each leaf as
a slot instead.

= Inspect the slots

`--describe` prints the schema (`--describe=json` for machine use):

#terminal(read("../generated/describe.txt"))

= Fill the slots

Values come from several sources, highest priority first: `--set`, a JSON file
(`-f`), the positional title, then the slot default.

```sh
mt add --set title="Wire OAuth" --set impact=8        # explicit values
mt add cleanup kitchen                                # positional → title slot
echo '{"title":"Wire OAuth","impact":8}' | mt add -f - # JSON on stdin
mt add fix login +backend +auth                       # +word → tags slot
mt add --edit                                         # fill in your editor
```

- #strong[Positional words] join into the `title` slot: `add cleanup kitchen` →
  `cleanup-kitchen.typ`.
- #strong[`+word`] arguments are tag sugar: peeled into a `tags` slot if the
  template declares one, instead of joining the title.
- #strong[List slots] need `--set name[]=value`, repeatable to append. A bare
  `--set name=value` for a list errors — one string is never silently a list.

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
so a required field always appears in the scaffold, in `--describe`, and in the
written task, even with the barebones built-in `task` template.

#callout("note")[
  Strict-framework priority (`framework` + `allow-manual = false`) has no scalar
  form and is not spliced. Declare it in a template as nested slots, as above. See
  #link(u("/priority/"))[Priority].
]

= Editor mode

`--edit` (or `[add] interactive`) opens an editor on a commented YAML scaffold of
the slots, then parses the saved buffer back into fill values. The editor resolves
`[add] editor` → `$VISUAL` → `$EDITOR` → `vi`, and only opens with a TTY.
`[add] header` overrides the instruction block.
