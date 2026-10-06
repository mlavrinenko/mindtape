// The render mode, read from `sys.inputs` and nothing else, so a module that
// must not read www/generated/ (framework-notes.typ) can import it.
//
// www/generate-md.sh renders each page a second time with typlite and
// `--input markdown=1`, into `<route>index.md` beside the page's HTML. In that
// mode page.typ drops tola's wrapper, ui.typ's components render as plain
// Markdown, and site.typ's u() links to the other pages' Markdown copies.
#let MARKDOWN = sys.inputs.at("markdown", default: "") == "1"
