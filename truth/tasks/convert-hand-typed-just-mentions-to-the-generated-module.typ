#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "convert hand-typed just mentions to the generated module",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("docs", "tooling"),
  estimate: (files: 14),
  links: depends-on(
    "render-just-recipe-names-through-a-generated-typst-module.typ",
  ),
  status: proposed(2026, 8, 14),
)

== Summary

Rewrite every hand-typed recipe mention in the 12 doc sources that carry one to go through the generated module, then turn on the gate that keeps it that way.

== Scope

- `docs/src/contributing.typ`, `docs/src/agents.typ`, `docs/src/readme.typ`, and all six sources under `docs/src/contributing/`.
- `www/content/internals.typ` and `www/content/dsl.typ`.
- `docs/src/lib.typ`'s own header prose mentions recipes too — it is a helper module, not a source, so its mentions are comments; leave them, and say why in the header.
- Deliberately *not* converted: `www/content/config.typ`'s `run = "just check"` is example config content, a string a user would type into their own config, not a reference to this repo's gate. The gate must still validate the name resolves.
- The rendered Markdown must come out byte-identical except where a mention was already wrong. Verify by rendering before and after and diffing — a helper that silently changes spacing inside a code span is a regression, not a refactor.
- Turn the `check-doc-recipes` arm on only once the tree is clean, in the same commit.
