#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Report the repeats arm's classes from their real copies",
  kind: "decision",
  priority: ice(
    impact: 6,
    confidence: 0.95,
    ease: 5,
  ),
  difficulty: 4,
  links: (
    (
      task: "dedupe-the-cmd-tests-outside-add.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "stateless", "tooling"),
  assignee: "agent",
  status: done(
    2026,
    11,
    24,
  )[The repeats gate replaced the arm, and the removal resolves this finding. The gate groups jscpd's duplicates pairs by fragment text, and jscpd 5.4.3 has no classes array to misread.],
)

== AS-IS

The repeats arm names files a snippet is not in. Its findings mislead.

+ Deduping the cmd tests outside add, after the repeats arm named 9 snippets there
+ 🔴 #raw("contrib/ask-refactoring/repeats.sh"): the arm reads jscpd's `classes` array, whose `fragments` list holds unrelated windows, so a finding's copy count and locations are wrong

```
$ jq -r '.classes[] | select(.fragments[0].name == "mindtape-cli/src/cmd/fix/scope_tests.rs" and .fragments[0].start == 51) | "copies=\(.fragments|length) tokens=\(.tokens)\n\(.fragment)"' /tmp/rep/jscpd-report.json
copies=59 tokens=30
fn project(run: &str) -> tempfile::TempDir {
    let dir = support::scaffold_project(&config(run));

$ jq -r '[.duplicates[] | select(.fragment | test("fn project\\(run: &str\\)")) | .firstFile, .secondFile] | unique_by(.name + ":" + (.start|tostring)) | .[] | "\(.name):\(.start)"' /tmp/rep/jscpd-report.json
mindtape-cli/tests/it/allow_e2e.rs:28
mindtape-cli/tests/it/estimate_queries_e2e.rs:209
mindtape-cli/tests/it/waits_e2e.rs:22
```

The class names 59 copies, and the `duplicates` array holds 3, all under `tests/it/`. The other 56 locations hold two-line windows of their own, such as `mindtape-cli/src/cmd/fix/scope_tests.rs:51-52`, which reads `fn git_project() -> tempfile::TempDir {`.

== TO-BE

- 🟢 A finding's copy count and locations are the copies of the fragment it prints.
- 🟢 `score_repeats` reads a report whose classes hold one snippet each, or rebuilds the classes from `duplicates` and the source at each range.
- 🟢 A snippet that appears once is never reported, whatever its neighbours.

```
$ just advisory ask-refactoring
crates/mindtape-cli/src/cmd/fix/scope_tests.rs  51  3  30  55  3
```

== Changes

- `contrib/ask-refactoring/repeats.sh`: build each class from the `duplicates` array and the source at its ranges, not from `classes[].fragments`.
- `docs/contributing/signal-sweep.md`: say which jscpd field the arm reads and why.

== Out of scope

- The `lint::dry` gate: it reads `--new-only` and the baseline, not the json report.
- The 15-token threshold and the gain formula.

== Dogfood

The next `just advisory ask-refactoring` files findings a maintainer can act on, instead of nine that name files the snippet is not in.

== Goals

+ #Helps(maintainer, to: keep-changes-safe)[a sweep whose files are wrong sends a maintainer to edit code the snippet is not in].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).
