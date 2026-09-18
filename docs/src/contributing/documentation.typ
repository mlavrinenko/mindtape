// docs/src/contributing/documentation.typ — source for
// docs/contributing/documentation.md. Rendered by docs/generate-md.sh via
// typlite, same as every other docs/src source; see docs/src/lib.typ for the
// render path's constraints.
//
// Deep dive behind CONTRIBUTING.md's Documentation section. Kept out of
// CONTRIBUTING because that file is always loaded: the rule reads in five
// lines, and the arguments for it — why a cap is not a target, why one home
// per fact, why a checked example — are what this file carries.
#import "../lib.typ": doc-title, just

#metadata((
  output: "docs/contributing/documentation.md",
)) <mindtape-md>

#doc-title[Refactoring the documentation]

User-facing docs are code with a reader instead of a compiler. The gates
catch one kind of rot — `docs::check` a dead link, `check-doc-facts` a
missing entry for a derived fact, `check-doc-recipes` a hand-typed recipe
name. None of them catches duplicated prose, or a page that lost its reader.
Those are what the rules below are for.

= Three authored homes

`www/content/*.typ` is the docs site; `www/utils/*.typ` holds the notes
rendered behind its generated refs. `docs/src/readme.typ`,
`docs/src/agents.typ` and `docs/src/skill.typ` render to README.md, AGENTS.md
and `skills/mindtape/SKILL.md`. `docs/src/contributing.typ` renders
CONTRIBUTING.md. No render is ever edited; #just.docs-md() overwrites it.

= Refactor whole, never patch

A file you touch is rewritten against its reader job: who reads it, and what
they can do after. A line cap is a scope brake, not a target to shave a
paragraph down to. The failure the rule exists for is a paragraph kept
because deleting it would have taken one more line, which is how a file
passes every gate and still wastes the reader's attention. A file that hits
its cap is refactored whole, or split — deep arguments move to a
`docs/contributing/*.md` file and the always-loaded source keeps a pointer.

= One home per fact

Generated facts (`www/generated/*.json`, read back by both render paths)
cannot drift. Hand-written sentences can: two pages say the same thing
slightly differently, both validate, and nothing fails. So a fact already on
one page is linked from everywhere else, never retold. What is deliberately
shared, like README's and the site root's opening task example, is derived
once and rendered twice (`www/generate-facts.sh`'s `readme-task.json`), never
typed twice.

= A checked example beats a description

A code block in the docs comes from a real file under `examples/` that
`mt check` validates, sliced by a generator into `www/generated/`. The DSL
page's field table is `lib/prelude.typ`'s own signature; the CLI page's
transcripts are the real binary against `examples/learn-piano`; the
quickstart's written task is a capture. A hand-typed snippet is a future lie,
and the reader has no way to tell it was one.

= The audit

#just.dup-prose() reads the doc sources, strips markup and code, and reports
sentences shared between two files, worst pair first. It is advisory: the
threshold is a judgement call, and the report is an input to a human deciding
where a fact should live. Run it before and after a refactor; the algorithm,
the knobs and why it is jq rather than a language of its own are in
`contrib/dup-prose.jq`.
