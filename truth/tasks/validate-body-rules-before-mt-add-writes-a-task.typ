#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Validate body rules before mt add writes a task",
  kind: "feat",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("mt", "gating"),
  estimate: (files: 4),
  assignee: "agent",
  status: done(2026, 9, 16),
)

== Problem

+ Run `mt add -t task` with a TO-BE bullet that does not open with 🟢.
+ 🔴 It writes the file. Only a later `mt check`, or the pre-commit hook, reports that every TO-BE bullet must open with 🟢.
+ Run it again with every goal `neutral`: this one is refused before writing, because a Typst panic fails the candidate's evaluation.


== Goals

#goals(
  product: (
    readable-tasks: helps[a malformed task never lands for a reader to skim],
    implementable-tasks: neutral,
    enforced-practice: helps[a task breaking a section rule is refused where it is written, not a commit later],
    cheap-adoption: neutral,
    typst-over-markdown: neutral,
    plain-files: neutral,
  ),
  process: (
    sound-codebase: neutral,
    docs-match-code: neutral,
  ),
)

== TO-BE

- 🟢 `mt add`, like every writer behind the pre-write seam, refuses a task whose body breaks a header or content rule, names each violation, and writes nothing:

```sh
$ mt add -t task …
Error: the task as filled breaks the body rules:
  TO-BE: open every TO-BE bullet with 🟢
```


== Changes

- `crates/mindtape/src/candidate_tests.rs`: red tests first, the Problem's task among them.
- `crates/mindtape/src/candidate.rs`: run `crate::gate::check_headers` and the content gate over each candidate's source beside `validate::validate`, keeping the violations that name the candidate.
- `crates/mindtape/src/content_gate.rs`: expose the content check for one source if it only runs over loaded files today.
- `crates/mindtape-cli/tests/add_e2e.rs`: `mt add` refuses the Problem's task and leaves no file.


== Alternatives

- Leave it to `mt check` and the pre-commit hook: the malformed task still lands and waits a commit to be caught.
- Run a whole `mt check` inside `mt add`: it would refuse a new task for violations in unrelated files.


== Out of scope

- Rules that span tasks: `check_candidates` already validates those.
- Which rules exist, or their messages.


== Dogfood

Built in: this repo's section rules refuse a malformed task at `mt add`.

