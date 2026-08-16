<!-- Generated from docs/src/contributing/generated-docs.typ by `just docs-md`. Do not edit; edit the source. -->

# Generating the root Markdown from Typst

Deep dive behind CONTRIBUTING.md’s pointer to this pipeline: how `docs/src/*.typ` becomes README.md, AGENTS.md, CONTRIBUTING.md, `skills/mindtape/SKILL.md` and `docs/contributing/*.md`, how a new source declares its own output, why the freshness gate diffs against a temp directory, and the typlite constraints every source has to design around.

### Why Typst sources for Markdown files

The root docs restate facts that already have a machine-readable source — the crate map, the DSL field list, the gate table, the linecop caps (`www/generate-facts.sh`, `www/generated/*.json`) — and a hand-written Markdown file cannot read JSON at build time. A Typst source can: `json("/www/generated/crate-map.json")` is a value like any other, so a fact gets stated once and every doc that needs it reads the same place. `docs/generate-md.sh` runs `www/generate-facts.sh` itself before rendering each source with typlite (nixpkgs’ `tinymist` package, `$out/bin/typlite`), so a source never depends on some other recipe having run first.

This is a separate render path from the docs site pages, `www/content/*.typ`: those open with `#show: page.with(...)` plus `@tola` imports and target HTML. A docs/src source is plain Typst with neither — a body typlite can walk on its own.

### Adding a source

Every source under `docs/src/` carries a `<mindtape-md>` metadata block naming its own output path, repo-root-relative:

```typ
#metadata((
  output: "AGENTS.md",
)) <mindtape-md>
```

`docs/generate-md.sh` globs `docs/src/**/*.typ`, skips anything with no `<mindtape-md>` block (so a helper module like `docs/src/lib.typ` is invisible to it without a separate exclude list), queries each block for its `output`, runs typlite, and writes the result there. Nothing outside the source file itself has to change to add one — no registry, no second list to keep in step.

`skills/mindtape/SKILL.md` needs YAML frontmatter (`name`, `description`) that typlite cannot emit, so its block carries a `frontmatter` dict alongside `output`:

```typ
#metadata((
  output: "skills/mindtape/SKILL.md",
  frontmatter: (
    name: "mindtape",
    description: "...",
  ),
)) <mindtape-md>
```

The generator prepends that frontmatter as a `---`-delimited YAML block, before the read-only banner every generated file opens with. The banner is an HTML comment naming the source, so it never leaks into the page GitHub renders:

```
<!-- Generated from docs/src/agents.typ by `just docs-md`. Do not edit; edit the source. -->
```

### The freshness gate diffs a temp directory, not the working tree

`just docs md-check` regenerates every source into a fresh temp directory and diffs each file there against the committed copy at its real path. It never regenerates in place and then runs `git status`/`git diff`.

That matters when the committed artifact itself is wrong — hand-edited, or left over from a stale render. Regenerating in place would heal it before the diff ever runs, so the gate would read clean on a file that was never correct. A temp render cannot self-heal: whatever is wrong on disk stays wrong while the comparison happens, so a hand-edit or a stale render fails the gate by name, every time.

### What typlite can and cannot do

Verified against the pinned tinymist (nixpkgs, this flake’s input) and the typst 0.14.2 it embeds — a separate typst plane from the `typst`/`typst-eval` **crates** mindtape links against (0.15.0; see `www/generated/crate-map.json`), so a nixpkgs bump that moves tinymist’s embedded typst means re-checking this table, not the crate versions.

| Construct | Result |
| --- | --- |
| `=` / `==` heading | Shifts down one level: `##` / `###`. No way back to a bare `#` — `set heading(offset: -1)` errors “number must be at least zero”. Use `doc-title` (docs/src/lib.typ) for the one `#` a document needs |
| `set document(...)` | Errors — typlite wraps the input in its own container |
| `#footnote[...]` | HARD ERROR inside typlite’s own `md-link` (“expected string, found location”). Footnotes are unusable here |
| `#html.elem("p"/"div"/"span")` | SILENTLY STRIPPED, attributes included — nothing appears, nothing errors |
| `#html.elem` with `h1`..`h6`, `img`, `br`, `center`, `blockquote` | Preserved as literal HTML tags, attributes included |
| `#link("a/b.md")[x]` | Becomes `[x](a/b.md)`, or a raw `<a href>` inside an `html.elem` |
| `json(...)`, `raw(read(...))` | Both work — a source reads `www/generated/*.json` or another file’s bytes directly |
| `#metadata(v) <label>` | Emits nothing into the Markdown; `typst query` reads it back — how `<mindtape-md>` works |
| `\@foo` | Literal `@foo`. An unescaped `@foo` risks being read as a citation |
| `->` | Escaped, as `-\>`. Prefer backticks or `→` |
| `"straight quotes"`, an apostrophe | Smart-quoted / smart-apostrophed outside a raw span |
| Tables, ordered/nested lists, fenced code, blockquotes, definition lists, `**bold**`, `_em_` | Round-trip clean |
| A raw span containing a literal backtick | NOT escalated to a wider delimiter — emitted as a single backtick regardless, so the span is broken/ambiguous CommonMark. Never put a backtick inside a raw span; rephrase into two spans instead |
| Bare URL in text | Auto-linked to `[url](url)` (Typst’s own markup feature, not typlite-specific) |

The footnote error and the silently-stripped `<p>`/`<div>`/`<span>` are the two sharpest edges: one fails loud, with a confusing message pointing at typlite’s own internals rather than your source; the other fails silent, the content simply missing with no diagnostic at all. Test a new construct against typlite directly (`typlite --root . <src> /tmp/out.md`, then read the output) before trusting it in a source.

### Two authoring traps when porting hand-written Markdown prose

Both bit the AGENTS.md port and are easy to reintroduce:

- Typst’s `*x*` is BOLD (`strong`), not italic — the opposite of Markdown’s single-asterisk convention. Ported prose that used `*word*` for italic emphasis silently becomes bold unless rewritten as Typst’s own `_word_` (which typlite renders back as Markdown `_word_`, not `*word*` — another representation change to expect, not a bug).
- A hand-wrapped line that ends mid-word with a hyphen (`wrong -\nedited`) does NOT get TeX-style hyphenation-continuation across the soft break: Typst joins it as `wrong- edited`, extra space and all, because typlite collapses the paragraph to one line without re-flowing it. Never end a wrapped source line on a hyphen that continues into the next line; move the whole hyphenated word to one line instead.

One more consequence, not a constraint: typlite emits ONE LINE PER PARAGRAPH, so a generated `.md`’s line count is not a meaningful size proxy. That is why `.linecop.yaml` caps the `.typ` SOURCE under `docs/src/**` and excludes every path a source declares as its `output`. The source itself still wraps like any other Typst file in this repo, and `typstyle` still formats it.
