#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "the file filter field and the file column name different paths",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 5,
  tags: ("cli",),
  estimate: (files: 5),
  assignee: "agent",
  status: done(2026, 9, 9),
  links: related(
    "csv-and-json-disagree-about-what-the-file-field-means.typ",
  )[the same split on the machine formats],
)

== Summary

\`file\` names two different values. The filter field is the project-root-relative
path (\`crates/mindtape/src/fields.rs\`, \`task.file\_key()\`); the \`file\` column
and \`--format path\` are relative to the working directory
(\`crates/mindtape-cli/src/render/file\_path.rs\`, \`cell\_text\`). Inside the
project root the two coincide, which is why this went unnoticed. Under
\`-w \<other-project>\` they diverge, and a filter built from what the listing
shows returns zero rows with no error.


== Scope

Measured against a dotfiles repo from a mindtape checkout: the column prints
\`../../../.nix/tasks/zed-writes-decrypted-api-keys-into-its-plaintext-log.typ\`,
\`mt ls 'contains("../../../.nix/tasks", file)'\` matches nothing, and
\`mt ls 'contains("tasks/zed-writes", file)'\` matches. Silently — an empty
listing is what a correct filter over an empty result set looks like too.

The DISPLAY is not the bug and should not change. Working-directory-relative
is the only one of the three candidates always usable from where the reader
is standing: root-relative is unopenable under \`-w\`, and absolute is longer
and leaks the home directory into shared output. \`--format path\` already
agrees with the column, which is what makes them a coherent pair.

Pick one of:

- rename the filter field to what it is (\`path\`? \`id\` already carries the
  stem, so the root-relative path may not need a filter field at all);
- make the filter field match either spelling, normalising the needle
  against both the root- and cwd-relative forms before comparing;
- keep both under one name and make the mismatch loud rather than empty.

The first is cleanest and cheapest pre-1.0. Whichever lands, the filter
docs (\`www/content/filters.typ\`) and the skill both currently spell the
distinction out in a parenthesis, which is the shape of a fact nobody reads
in time.

= Watch for

The \`file\` column now renders automatically in captured output, so this is
much easier to hit than when the column was opt-in: the first path a reader
ever sees is the one that does not work in a filter. Does any existing view
or \`\[\[ls.rules\]\]\` entry in the examples filter on \`file\` in a way that would
change meaning under a rename?

