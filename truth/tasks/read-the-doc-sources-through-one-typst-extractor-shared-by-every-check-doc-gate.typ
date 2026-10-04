#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Read the doc sources through one Typst extractor shared by every check-doc gate",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 4,
  ),
  difficulty: 4,
  estimate: (src-files: 5),
  tags: ("found", "tooling"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a doc gate can no longer pass by misreading a fence],
      docs-match-code: helps[the span and recipe gates read every span a page renders],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 4),
)

== Problem

+ Reviewing commits e3a52225, d228fe5b and c0bd3b46
+ 🔴 #raw(".just/scripts/check-doc-spans.sh"): three doc gates tokenize Typst with their own awk, and a string holding three backticks blinds two of them for the rest of the file

````
$ cat a.typ
#let fence = "```"
Prose: `mt lsx` should be flagged.
$ spans a.typ | wc -l
0
$ typstyle --ast a.typ | grep -A2 Raw:
  Raw: 8 [
    RawDelim: "`",
    Text: "mt lsx",
$ time bash .just/scripts/check-doc-recipes.sh
real	0m52,546s
````


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".just/scripts/_doc-blocks-lib.sh").

== TO-BE

- 🟢 One extractor, built on the `typstyle --ast` reader `_doc-blocks-lib.sh` already has, prints each inline raw, block raw, comment and prose run of a doc source with its file and line.
- 🟢 check-doc-spans, check-doc-recipes and check-doc-prose filter that stream, so a fence inside a string hides nothing.
- 🟢 One source manifest names the files every doc gate reads, and a gate fails when it resolves to no file.
- 🟢 `just check-doc-recipes` runs in about a second instead of 52.


== Changes

- `.just/scripts/_doc-blocks-lib.sh`: lift the `typstyle --ast` reader into a shared extractor, and hold the source manifest.
- `.just/scripts/check-doc-spans.sh`: read spans from the extractor; drop `spans()`.
- `.just/scripts/check-doc-recipes.sh`: read spans and blocks from the extractor; drop the awk tagger and the per-line greps that cost the 52 s.
- `.just/scripts/check-doc-prose.sh`: read the prose runs of Typst sources from the extractor.
- `.mmz/config.yaml`: add the shared script to each doc rule's inputs.


== Rollback

- Revert the commit; each gate gets its own tokenizer back.


== Out of scope

- The Markdown and YAML sources check-doc-prose reads: they have no Typst parse tree.
- Validating `mt` spans with clap, filed separately.


== Dogfood

Every `just check` after a docs edit runs the doc gates on the parse tree this repo's pages render from.

