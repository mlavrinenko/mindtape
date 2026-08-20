#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "configure the handle check length per project",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("cli",),
  estimate: (files: 6),
  links: related(
    "print-the-refreshed-handle-after-a-flip.typ",
  )[both are the same
    stale-ordinal problem, from the guard side and the announcement side],
  assignee: "max",
  status: done(2026, 8, 20),
)

== Summary

A handle's check character is \`fnv1a(task.id) % 26\` — one base-26 letter, so
a wrong handle passes the guard roughly once in 26. That is thin for a token
whose entire job is to catch a listing that renumbered under the caller, and
it is thin in the case that actually bites: a second \`mt flip\` on a task the
first flip re-sorted resolves a different task's ordinal, and only the check
letter stands between that and flipping the wrong task.

\`handle.rs\` was written for this. \`check\_chars(id, len)\`, \`handle\_for(..,
check\_len)\` and \`make\_handles(.., check\_len)\` all take the length, and
\`parse\_handle\` already accepts a multi-letter check; the module header even
names the missing piece: "a future \`\[handle\] check\_len\` config key can extend
it without changing this module's interface". Nothing reads config, so three
call sites pass \`DEFAULT\_CHECK\_LEN\` and the key does not exist.


== Scope

- Add a \`\[handle\] check\_len\` config key (a \`config\_handle.rs\`, matching the
  \`config\_\*.rs\` pattern), defaulting to 1 — \`DEFAULT\_CHECK\_LEN\` itself does
  not change, so every project that says nothing keeps today's handles
- Thread the configured length into the three call sites that hardcode it:
  \`output.rs\`, \`render/table.rs\`, and \`resolve\_by\_handle\` in \`util.rs\`
- Reject a short handle rather than accepting it as a prefix: under
  \`check\_len = 2\`, \`2y\` must not resolve \`2yq\`, because \`2y\` is a handle that
  can belong to a different task later. Error naming the expected length
- Cover the config surface the rest of the keys carry: schema, \`mt config
  show\`, the config reference page

