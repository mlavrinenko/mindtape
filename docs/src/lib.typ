// docs/src/lib.typ — shared helpers for the docs/src/*.typ sources that
// docs/generate-md.sh renders to Markdown with typlite (nixpkgs' tinymist,
// `$out/bin/typlite`; see flake.nix for the pin). This module is NOT itself a
// source: it carries no `<mindtape-md>` metadata block, so the generator's
// glob (docs/src/**/*.typ) skips it via the same zero-match check it uses for
// any future non-source helper under docs/src/.
//
// A docs/src source is a SEPARATE render path from the www/content/*.typ docs
// site pages: those open with `#show: page.with(...)` plus `@tola` imports
// and target HTML — neither typlite understands, and typlite errors on the
// `set document(...)` a `page` show rule issues (see below). A docs/src
// source is plain Typst with no page show rule and no tola import, so
// typlite can walk it on its own.
//
// Being a separate render path is also why a cross-link INTO the docs site is
// the one thing a source must never hand-type. Nothing in this repo resolves
// an absolute URL sitting in generated Markdown, so a renamed page ships a
// 404 that only a reader ever finds. `www-link` / `www-title` / `www-summary`
// below read the route out of the generated page manifest instead, and fail
// the render naming it when it is not there.
//
// Constraints typlite imposes on every source below, verified against the
// pinned tinymist 0.14.10 / typst 0.14.2 (this pin matches the dev shell's
// own `typst` 0.14.2 — the *Rust crate* `typst`, a separate eval plane
// mindtape links against, is 0.15.0 and irrelevant to this render path):
//
//   - `=`/`==` headings shift DOWN one level (`##`/`###`) — there is no way
//     back to a bare `#`: `set heading(offset: -1)` errors "number must be
//     at least zero". `doc-title` below is the one working path to `#`.
//   - `set document(...)` errors — typlite wraps the input in its own
//     container.
//   - `#footnote[...]` is a HARD ERROR inside typlite's own `md-link`
//     ("expected string, found location"). Footnotes are unusable here.
//   - `#html.elem("p"/"div"/"span")` is SILENTLY STRIPPED, attributes
//     included — nothing appears in the output and nothing errors. `h1`..
//     `h6`, `img`, `br`, `center`, `blockquote` ARE preserved as literal
//     HTML tags, attributes included.
//   - `#link("a/b.md")[x]` becomes `[x](a/b.md)`, or a raw `<a href="...">`
//     when nested inside an `html.elem`.
//   - `json(...)` and `raw(read(...))` both work — a source can read
//     `www/generated/*.json` (see `fact` below) or another file's bytes
//     directly.
//   - `#metadata(v) <label>` emits nothing into the rendered Markdown;
//     `typst query` reads it back out of the source. This is how the
//     `<mindtape-md>` block every source carries works.
//   - `\@foo` renders as the literal text `@foo`; an unescaped `@foo` risks
//     being read as a citation, so escape defensively rather than
//     per-instance.
//   - `->` renders escaped, as `-\>`; prefer backticks or `→` instead.
//   - `"straight quotes"` outside a raw/code span render as smart quotes.
//   - Tables, ordered/nested lists, fenced code, blockquotes, definition
//     lists, `**bold**` and `_em_` all round-trip clean.
//   - A backtick raw span must not be BROKEN ACROSS A SOURCE LINE. Typst
//     keeps the newline inside the span, and typlite emits it literally into
//     the Markdown — inside a table row that ends the row mid-cell and
//     corrupts the table. Hard-wrap the prose around a code span, never
//     through one. (Hit five times porting CONTRIBUTING.md and SKILL.md; it
//     is the most common trap in this list.)
//   - A `;` — and any other character Typst can read as continuing a code
//     expression — glued straight onto a `#link(...)[...]`'s closing `]` with
//     no space is SWALLOWED. Put a space before it.
//   - `doc-title` with a nested raw span (`` `mt` ``) renders the span as
//     `<code>`, so the emitted h1 is no longer attribute-free plain text and
//     docs/generate-md.sh's demotion (below) skips it — the title survives as
//     a raw `<h1>` tag. For literal inline backticks in a title, interpolate
//     the character instead: `#"`"`.
//   - A source line that OPENS with a number and a period — `15. A band
//     below 30 …`, the shape a hard-wrapped sentence lands in by accident —
//     is an enum item to Typst, not prose. The paragraph splits in three and
//     the Markdown grows a stray `15.` list marker; nothing errors (verified
//     both ways). Rewrap so the number never starts a line, or escape the
//     period: `15\.` renders as `15.` and rejoins the paragraph.
//   - A raw/code span containing a literal backtick (`raw("a`b")`, or
//     backtick markup with a backtick inside) does NOT get a wider Markdown
//     delimiter — typlite always wraps in a SINGLE backtick regardless of
//     content, so the emitted span is broken/ambiguous CommonMark (verified:
//     `raw("a`b`c")` → `` `a`b`c` `` literally, four backticks with no
//     escalation). Never put a literal backtick inside a raw span; rephrase
//     around it (e.g. two separate code spans) instead.
//
// One consequence worth stating plainly, because it is why the freshness
// gate and the linecop caps land where they do: typlite emits ONE LINE PER
// PARAGRAPH — it does not hard-wrap. A generated .md's line count is thus not
// a meaningful size proxy; .linecop.yaml caps the .typ SOURCE only (see that
// file's docs/src/** overrides) and excludes every path a source declares as
// its `output`.

// The one `#` (ATX h1) a document needs. typlite maps `=`/`==` down a level
// and has no working `offset: -1` escape hatch (see above), so the document
// title goes through the one HTML element typlite preserves verbatim at any
// position instead of through Typst's own heading syntax.
#let doc-title(body) = html.elem("h1")[#body]

// Spell a small derived count as an English word, so prose that used to read
// "Two crates" does not regress to "2 crates" the moment the number stops
// being hand-typed. Above the table, fall back to the numeral — that is where
// a numeral is what a reader wants anyway.
#let num-word(n) = {
  let words = (
    "zero",
    "one",
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "ten",
  )
  if n >= 0 and n < words.len() { words.at(n) } else { str(n) }
}

// Read one of www/generate-facts.sh's derived facts by name (e.g.
// "crate-map"), so every source reads the same root-relative path instead of
// retyping it. docs/generate-md.sh always runs `www/generate-facts.sh` before
// rendering any source, so this never depends on a prior `just docs::gen`.
#let fact(name) = json("/www/generated/" + name + ".json")

// The docs site's page manifest — the site URL, and per route the page's
// sidebar label, title and summary. Derived from www/utils/site.typ by
// www/generate-facts.sh, and read through `fact` above so it arrives on the
// same terms as every other derived fact.
#let _site = fact("site-pages")

// Resolve one page by route, or fail the render naming it.
//
// A docs/src source links INTO the site constantly, and a hand-typed absolute
// URL there is a dead link waiting to happen: typlite renders a `#link`
// without ever resolving it, so a renamed page leaves the generated Markdown
// pointing at a 404 that no build in this repo can see. Reading the route out
// of the manifest instead turns that into a `just docs md` failure. The
// message lists the known routes because the fix is always one of them —
// "unknown route /clix/" on its own only moves the search into another file.
#let _page(route) = {
  if type(route) == str and route in _site.pages {
    return _site.pages.at(route)
  }
  panic(
    "unknown docs-site route: "
      + repr(route)
      + " — known routes: "
      + _site.pages.keys().join(", "),
  )
}

// A page's own title and summary, for prose that NAMES a page (or lists every
// page, as README does) rather than linking one. Same panic on an unknown
// route, for the same reason.
#let www-title(route) = _page(route).title
#let www-summary(route) = _page(route).summary

// An absolute link to a docs-site page. `#www-link("/cli/")` renders the
// page's OWN title as the link text — the common case, and one less string to
// keep in sync with the site; pass a body to override it, as in
// `#www-link("/cli/")[the CLI reference]`.
//
// Emits a plain `#link`, which typlite renders as `[text](url)`, so the
// header's warning about the closing `]` applies verbatim: never glue a `;`
// — or anything else Typst can read as continuing the expression — onto it.
#let www-link(route, ..body) = {
  // Resolved unconditionally, never inside the branch below: an unknown route
  // has to fail even when the caller supplied its own link text.
  let entry = _page(route)
  let pos = body.pos()
  link(_site.url + route, if pos.len() > 0 { pos.at(0) } else { entry.title })
}

// Re-export www/generated/just.typ (www/generate-facts.sh) so a source
// writes `#import "lib.typ": fact, just` and then `#just.check()` instead of
// hand-typing a recipe name — a rename fails `just docs::md` naming the
// missing member instead of rotting silently. www/content/*.typ sources are
// a different render path (tola, not typlite) and import
// "/www/generated/just.typ" directly instead of through this file.
#import "/www/generated/just.typ" as just
