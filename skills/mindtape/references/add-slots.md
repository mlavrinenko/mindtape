# `mt add` slot semantics

Reference for the mindtape skill. Read this when filling slots on a real
template — the three things that surprise callers, none of which you can
infer from `--describe` output alone.

## `--set` keys are slot names, and unknown ones are a hard error

A `--set`/`-f` key must name a slot of the active template. An unrecognised
key fails the command: `mt add fix bug --set priority=3` errors with
`unknown --set/-f key(s): priority; known slots: title, ...` rather than
writing anything. Run `mt add --describe` (plus `-t <template>` for a
non-default one) to see the real slot names, and `--print` to check the
result before it's written.

Slot names need not match field names. A framework priority is authored as
`priority: ice(impact: slot("impact"), …)`, so the slots are its leaves —
`--set impact=8 --set confidence=0.9 --set ease=4`, never `--set priority=8`.
Under a strict framework (`framework` + `allow-manual = false` in
`[tasks.priority]`) a scalar priority is invalid by construction, so even a
template that omits priority gets the framework call spliced in and the leaves
are still the slots. Same for any field the template nests inside a call.

Tags: `+word` positionals are sugar for the `tags` slot (`mt add fix login
+backend +auth`); or repeat `--set tags[]=backend --set tags[]=auth`. A bare
`--set tags=backend,auth` is a hard error — a `list` slot needs `[]=`.

## `text` slots are escaped, not markup

Every `text`-kind slot value (`-f`/`--set`/`--edit`/positional/environment,
JSON or YAML — the source makes no difference, escaping runs after parsing) is
markup-escaped before it lands in the file: a literal `` ` `` / `*` / `_` /
`#` / `[` / `]` / `@` / `<` you type becomes `` \` ``, `\*`, etc. — safe, but
never live Typst. So don't author code spans or emphasis inside slot content
(a `summary`/`scope` fill) expecting them to render — write plain prose there.
If a task genuinely needs formatted body text (a code span, a link), fill the
slot with a placeholder and hand-edit the body afterward — sanctioned per
"Authoring or editing a task file by hand" in SKILL.md, since only the section
*headers* are config-enforced, not the prose under them. Heuristic: after
`mt add`, search the written task for a literal backslash-backtick pair
(`` rg -F '\`' <file> ``) — a hit is almost always markup you meant literally,
not prose you meant escaped.

## `typst` slots opt out of escaping — a template's choice, never a caller's

A template can declare `kind: "typst"` instead of `kind: "text"` to skip the
escaping above: a filled value splices into the body as live markup, code
spans and all — `` `mt add` `` stays a code span, not `` \`mt add\` ``. This is
strictly a *template author*'s decision: only the checked-in template file
declares the kind, so a `--set`/`-f`/environment value never escapes escaping
on its own just by looking like markup — the same value fills a `text` slot
escaped and a `typst` slot raw, depending only on which the template
declared. Prefer `typst` when a slot's whole point is formatted body content
(a `summary`/`body`); a value that fails to parse as Typst markup still fails
`mt add` outright, naming the slot, before anything is written — no half-typed
file to clean up.

