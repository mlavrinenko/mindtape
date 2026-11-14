#import "@local/mindtape:0.3.0": *

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

A handle's check character is `fnv1a(task.id) % 26`, one base-26 letter. A
wrong handle passes the guard roughly once in 26. That is thin for a token
whose entire job is to catch a listing that renumbered under the caller. It is
thinner still in the case that actually bites: a second `mt flip` on a task
the first flip re-sorted resolves a different task's ordinal. Only the check
letter stands between that and flipping the wrong task.

`handle.rs` already supports this. `check_chars(id, len)`, `handle_for(..,
check_len)` and `make_handles(.., check_len)` all take the length, and
`parse_handle` already accepts a multi-letter check. The module header even
states the gap. A future `[handle] check_len` config key can extend it without
changing this module's interface. Config has no reader, so three call sites
pass `DEFAULT_CHECK_LEN` and the key does not exist.


== Scope

- Add a `check_len` key under `[handle]` (a `config_handle.rs`, matching the
  `config_*.rs` pattern), defaulting to 1. `DEFAULT_CHECK_LEN` itself does not
  change. Every project that says nothing keeps today's handles
- Thread the configured length into the three call sites that hardcode it:
  `output.rs`, `render/table.rs`, and `resolve_by_handle` in `util.rs`
- Reject a short handle rather than accepting it as a prefix. Under
  `check_len = 2`, `2y` must not resolve `2yq`, because `2y` is a handle that
  can belong to a different task later. Error naming the expected length
- Cover the config surface the rest of the keys carry: schema, `mt config
  show`, the config reference page

