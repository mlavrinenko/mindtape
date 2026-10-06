#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Record a hash for each CLI command page outdatty lists by glob",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
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
      sound-codebase: neutral,
      docs-match-code: helps[an edit to a command page leaves a review record like every other dependent],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 7),
)

== Problem

+ Integrating the per-command CLI pages and the docs-terms pass, which edited those pages.
+ 🔴 #raw("outdatty.lock"): The \`cli-docs\` group lists \`www/content/cli/\*.typ\` as a glob dependent, and the lock recorded no hash for any file it matches, because nobody ran the update after the 21 pages were added. The glob itself works: outdatty 0.5.0 records one hash per matched file, as `outdatty update --group cli-docs --dependent www/content/cli/add.typ` shows.

`outdatty.yaml:150`: `- www/content/cli/*.typ`
`rg -c 'content/cli/' outdatty.lock` printed nothing, while `www/content/cli.typ` had its hash.

== TO-BE

- 🟢 `outdatty.lock` holds a hash for each of the 21 command pages, recorded after reviewing each page against the cli-docs sources, so an edit to one is caught.
- 🟢 A command page added later is recorded by `just outdatty-update cli-docs <path>`, with no change to `outdatty.yaml`.


== Changes

- `outdatty.lock`: the 21 hashes, each recorded per page, not as a whole-group update, which would also re-hash `www/content/recurring.typ` unreviewed.
- Review found no drift: the prose on `check`, `config`, `flip`, `graph`, `link`, `mv`, `recur`, `rm`, `views` and `ls` matches `mt --help`; the other pages only render the generated reference.


== Rollback

Revert the commit; the pages go back to unrecorded.

== Out of scope

- Other groups' dependents.
- A gate that names an unrecorded page: `outdatty check` does not flag a dependent with no hash, and that is outdatty's behavior.


== Dogfood

This repo's next edit to a command page records its review in `outdatty.lock`.
