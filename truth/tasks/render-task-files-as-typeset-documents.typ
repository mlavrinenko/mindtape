#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Render task files as typeset documents",
  priority: ice(
    impact: 6,
    confidence: 0.6,
    ease: 4,
  ),
  tags: (),
  links: related("generate-docs-cli-reference-from-the-mt-binary.typ")[both build on the same typeset-render face],
  status: done(2026, 7, 12)[Shipped in `lib/prelude.typ`: the `task` show rule lays every set field out as a printed index card (status/priority/kind pills, metadata grid, tag chips, body), inert under `typst-harvest` eval. Docs-embed follow-on split out to embed-compiled-task-card-in-docs-hero.],
)

== Summary

Make the third face real. The docs promise "one file, three faces: source you edit, a row you query, a document you can typeset." The first two hold; the third does not. `typst compile truth/tasks/<stem>.typ` today emits only the `#metadata` marker plus the raw body — none of the structured fields (status, priority, estimate, tags, links, due), no chrome, no page setup. It renders sparse and boring, so the promise is aspirational rather than demonstrable.

Give the `task` show rule in `lib/prelude.typ` a proper paged render: a title heading, status/priority pills, a metadata block (due, estimate, tags, depends-on and the other links), then the authored body. Compiling any real task should produce the typeset card the landing page currently fakes as hand-built HTML (`www/utils/ui.typ` `task-card`).

== Constraints

- Eval-safe: MindTape harvests via `typst-harvest` eval-only, never layout. The `#metadata((marker: "mindtape.task", ...)) <mindtape>` element must stay exactly as-is and remain queryable; the visual chrome is additive and inert under eval. Verify `mt check` / `mt ls` still parse every task after the change.
- Paged output only: use native Typst content (headings, grid, box, `set page`), not `html.elem` (that path is the www SSG's). Page settings are inert under eval, so they cost harvesting nothing.
- Field-complete but graceful: render every core field that is set, skip cleanly the ones that are none/empty. Handle each status legend value, raw-int vs framework priority, the estimate unit dict, and task-to-task links by stem.
- Keep the prelude within its file-size budget; factor the render into a helper module if it pushes `lib/prelude.typ` past the limit.

== Scope

- Design the typeset layout (mirror the landing-page card face for visual continuity).
- Implement the render in the `task` show rule; keep the metadata marker untouched.
- Test: `typst compile` a representative task (multiple statuses, links, estimate, tags) and eyeball the PDF; assert `mt check` stays green.
- Optional follow-on: have the docs embed a genuinely compiled task instead of the hand-built HTML card, closing the loop on the promise.
