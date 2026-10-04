#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Pin the api-service facts the skill evals read, and give the fixture one clock",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.85,
    ease: 6,
  ),
  difficulty: 2,
  estimate: (src-files: 9),
  tags: ("found", "tests"),
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
      docs-match-code: helps[the evals' claims about api-service fail a test once the fixture drifts],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 4),
)

== Problem

+ Reviewing commit 3afcd40a
+ 🔴 #raw("skills/mindtape/evals/evals.json"): the evals state api-service's counts by hand, no outdatty group sources its tasks, and five generators restate its clock

```
$ grep -o "Agent lists all 12 tasks, including the 4 terminal ones" evals.json
Agent lists all 12 tasks, including the 4 terminal ones
$ grep -l '^export MINDTAPE_NOW=2026-10-01' www/*.sh
www/generate-cli.sh
www/generate-guides.sh
www/generate-import-export.sh
www/generate-reference.sh
www/generate-api-service.sh
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/tests/eval_fixture_e2e.rs").

== TO-BE

- 🟢 A test pins what evals 0-3 assert about api-service: 12 tasks, 4 done, 7 ready, `index-the-orders-by-customer` first by priority. Adding a task fails it and names the eval to update.
- 🟢 An edit under `examples/api-service/` flags the evals and the problem pages in `outdatty check`.
- 🟢 The fixture's clock is declared once, and every generator that runs api-service reads it.


== Changes

- `crates/mindtape-cli/tests/eval_fixture_e2e.rs`, new: assert the counts and the ready order under the fixture's clock.
- `outdatty.yaml`: make `examples/api-service/**` a source, with `skills/mindtape/evals/evals.json` and `skills/mindtape/evals/README.md` as dependents.
- `www/capture.sh`: declare the api-service clock once.
- `www/generate-cli.sh`, `www/generate-guides.sh`, `www/generate-import-export.sh`, `www/generate-api-service.sh`, `www/generate-reference.sh`: read that clock instead of exporting their own.


== Rollback

- Revert the commit; the facts and clocks are hand-kept again.


== Out of scope

- Running the evals in CI: they need a model.
- The other fixtures' clocks, which no eval reads.


== Dogfood

The next api-service task added for a capture fails the test until the evals are updated in the same commit.

