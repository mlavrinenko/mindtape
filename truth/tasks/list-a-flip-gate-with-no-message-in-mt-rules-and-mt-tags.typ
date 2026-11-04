#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "List a flip gate with no message in mt rules and mt tags",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.95,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 2),
  tags: ("found", "cli", "stateless"),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[a gate a flip enforces is visible in mt rules and mt tags],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (:),
      ..("sound-codebase", "docs-match-code"),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 4),
)

== AS-IS

+ Explaining the out-of-scope note in `read-has-tags-and-spaced-has-tag-calls-in-mt-tags.typ` after fixing `mt tags`.

+ 🔴 #raw("crates/mindtape/src/config/active_rules.rs"): `active_rules` skips every `[[on.flip]]` action with no `message`, so a gate that refuses a flip is absent from `mt rules --for flip` and `mt tags --for flip`.


With `ship` read by a run-less gate with no message and `cli` by one with a message, only the second lists:

```
$ mt rules --for flip
error  on.flip[1].actions[0]  cli gate
$ mt tags --for flip
cli  on.flip[1].actions[0]
     cli gate
```

The first still refuses at flip time, through `runless_gate` with `DEFAULT_ADVICE`:

```
active_rules.rs:187: let Some(message) = &action.message else {
active_rules.rs:188:     continue;
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/config/active_rules.rs").

== TO-BE

- 🟢 `mt rules --for flip` lists every action with `gate = true`, message or not, as `(no message set)` when it has none.
- 🟢 `mt tags --for flip` names the tags that gate's `when` reads.

```
$ mt rules --for flip
error  on.flip[0].actions[0]  (no message set)
error  on.flip[1].actions[0]  cli gate
$ mt tags --for flip
ship  on.flip[0].actions[0]
      (no message set)
cli   on.flip[1].actions[0]
      cli gate
```


== Changes

- `crates/mindtape/src/config/active_rules.rs`: list an action when `gate = true` or it carries a `message`, not only when it carries a `message`.
- `crates/mindtape/src/config/active_rules_tests.rs`: pin a message-less gate in the listing and keep a message-less show or non-gating run out.


== Out of scope

- Which message a gate with none shows. It reuses `(no message set)`, as a body rule without one already does.
- The `[[on.flip]]` execution in `cmd/flip/hooks.rs`; it already refuses a message-less gate with `DEFAULT_ADVICE`.


== Dogfood

Any project that writes `gate = true` without a `message` sees it in `mt rules --for flip` and `mt tags --for flip`. This repo's shipped flow gate carries a message, so its own listing does not change.

