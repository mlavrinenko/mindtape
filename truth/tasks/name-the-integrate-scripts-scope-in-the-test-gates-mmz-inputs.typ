#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Name the integrate script's scope in the test gate's mmz inputs",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
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
      sound-codebase: helps[an edit to integrate.sh re-runs the test that runs it],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 6),
)

== Problem

+ Wiring the outdatty.lock merge driver's e2e test into the \`just test\` mmz rule
+ 🔴 #raw(".mmz/config.yaml"): \`crates/mindtape-cli/tests/integrate\_refs\_e2e.rs\` runs \`.just/scripts/integrate.sh\`, but the \`just test\` and \`just cover\` rules do not list \`script-integrate\`, so an edit to integrate.sh alone leaves both served as fresh cache hits.

```
$ rg -n 'integrate.sh' crates/mindtape-cli/tests/integrate_refs_e2e.rs
52:    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.just/scripts/integrate.sh");
$ grep -n -A2 'name: just test' .mmz/config.yaml
  - name: just test
    tags: [gate]
    inputs: [rust, policy, examples, gates-meta, script-outdatty-lock-merge]
```
The scope list's own comment says integrate.sh is "named by no rule".


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".mmz/config.yaml").

== TO-BE

- 🟢 Editing `.just/scripts/integrate.sh` alone makes `just check` re-run `just test` and `just cover`.


== Changes

- `.mmz/config.yaml`: add `script-integrate` to the `just test` and `just cover` inputs, and drop integrate.sh from the "named by no rule" list in the scope comment.


== Rollback

- Revert the `.mmz/config.yaml` lines.


== Out of scope

- A gate that derives which scripts a Rust test reads. `script-scope-check` walks recipe bodies only, and a test reading a script is rare enough to list by hand.


== Dogfood

The next edit to integrate.sh re-runs the test that checks its refusal.
