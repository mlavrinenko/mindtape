#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Serve every docs page as Markdown and list the pages in llms.txt",
  kind: "docs",
  priority: ice(
    impact: 7,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 3,
  tags: ("www", "docs"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[an agent reads the docs at a sixth of the bytes and finds every page from one index],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the Markdown renders from the same Typst sources as the HTML, so the two cannot disagree],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 6),
)

== Problem

+ Fetch `https://mlavrinenko.github.io/mindtape/concepts/` the way an agent's fetch tool does.
+ 🔴 It gets 28.6 KB of HTML with the sidebar, styles and scripts around 4.9 KB of content. No Markdown copy exists.
+ 🔴 The site has no `/llms.txt`, so an agent finds pages only by crawling the sidebar.


== TO-BE

- 🟢 Every page at `/<route>/` has `/<route>/index.md`: its title, summary and body as Markdown, with code blocks keeping their language and links pointing at the other pages' `.md` copies.
- 🟢 `/llms.txt` follows llmstxt.org: the site title, a one-line summary, then every page under its `NAV` group as `[title](md url): summary`, with the summary from the page's `<page-meta>`.
- 🟢 Every page header shows a visible Markdown link to its copy, and its head carries `<link rel="alternate" type="text/markdown">`.
- 🟢 typlite renders the Markdown from `www/content/*.typ`, with the tola-only parts stubbed by a `sys.inputs` flag. A page typlite cannot render fails the docs build. Nothing converts HTML.


== Source of truth

- `www/content/*.typ`, the same sources tola builds.
- `NAV` and `PAGES` in `www/utils/site.typ`: the groups, titles and summaries.
- The llms.txt spec at llmstxt.org.
- A probe on 2026-10-06 with `utils/page.typ` swapped for a 4-line stub: typlite rendered `concepts`, `quickstart`, `split-oversized-tasks`, `comparison` and `cli` to clean Markdown. `index` failed on its inline SVG (`expected string, found bytes` in typlite's markdown library), and the `cli` tables fell back to HTML.


== Changes

- `www/utils/page.typ`: in Markdown mode, render the title, the summary and the body without tola's wrapper.
- `www/utils/ui.typ`: a Markdown branch for each `html.elem` component, such as `terminal`, `hero` and `cap`.
- `www/generate-md.sh`, new: run typlite per page into the built site and write `llms.txt`.
- `www/templates/layout.typ`: the header link and the `rel="alternate"` link.
- `.just/docs.just`: run it in `docs::build` and `docs::check`.


== Out of scope

- `llms-full.txt`, one file with every page. File it when an agent needs it.
- Content negotiation on `Accept: text/markdown`, which GitHub Pages cannot do.


== Dogfood

The mindtape skill and this repo's agents read the docs through `llms.txt` and the `.md` copies.

