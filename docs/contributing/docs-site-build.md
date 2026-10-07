<!-- Generated from docs/src/contributing/docs-site-build.typ by `just docs md`. Do not edit; edit the source. -->

# The docs site build

Why `just docs build` and `just docs check` rebuild the site under `www/content/` from scratch every time, and how they write each page’s Markdown copy.

## Why the build is always clean

Both recipes pass `--clean` to `tola build`. tola doesn’t track a `json()` read as a page dependency, so an incremental build can keep a page’s old HTML after the fact it reads changes. Every page that reads a generated fact is exposed, and `just docs check` would then validate HTML that no current source produces. With `--clean`, every page recompiles on every build.

`just docs serve` keeps the incremental build. It isn’t a gate, and a full rebuild on each keystroke would make hot reload useless.

## The Markdown copies and llms.txt

After `tola build`, `www/generate-md.sh` renders every page a second time with typlite and `--input markdown=1`, into `index.md` beside the page’s `index.html`. An agent reads the copy at a fraction of the HTML’s bytes. The same script writes `llms.txt` at the site root, in the [llmstxt.org](https://llmstxt.org) format: each `NAV` group’s pages with their summaries, and a command page under the CLI entry. The Pages workflow deploys what `just docs check` builds, so the copies ship with the HTML.

`www/utils/mode.typ` reads the flag. In that mode `page.typ` renders the title, the summary and the body without tola’s wrapper, and `u()` links to the other pages’ copies by full URL. Each `ui.typ` component has a Markdown branch, because typlite drops a `div` or a `span` together with its content. A table whose cells hold paragraphs or lists has no Markdown form, so `cli-ref.typ` lists a command’s arguments instead.

The script fails the build on a page typlite can’t render, on an HTML tag typlite wrote outside a code block, and on a link to a copy that doesn’t exist. So if your page uses a new `html.elem`, give it a Markdown branch, or the build fails. The header’s Markdown link is a full URL too, because `tola validate` checks a site path against its own page list, which holds no `.md` file.
