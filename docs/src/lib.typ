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
// `set document(...)` a `page` show rule issues. A docs/src
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
// Every constraint typlite imposes on a source is listed once, in
// docs/contributing/generated-docs.md. The `docs::md-check` gate fails on the
// ones it can detect in the rendered Markdown.

// The one `#` (ATX h1) a document needs. typlite maps `=`/`==` down a level
// and has no working `offset: -1` escape hatch, so the document
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
// route, for the same reason. A summary is plain text with `code` spans;
// typlite escapes backticks in text, so each span is rebuilt as raw.
#let www-title(route) = _page(route).title
#let www-summary(route) = (
  _page(route)
    .summary
    .split("`")
    .enumerate()
    .map(((i, part)) => if calc.odd(i) { raw(part) } else { part })
    .join()
)

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
