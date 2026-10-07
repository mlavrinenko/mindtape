#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep Typst's hints in typst-world's eval errors",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  estimate: (src-files: 2),
  tags: ("found", "substrate"),
  x-goals: (
    product: (
      readable-tasks: helps[a task that fails to load shows Typst's own suggestion for fixing it],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 7),
)

== Problem

+ Reading how \`mt\` renders Typst diagnostics for the error inventory
+ 🔴 #raw("Cargo.lock"): \`World::eval\_error\` in typst-world 0.4.0 copies each diagnostic's message, location and trace but not its hints, so \`mt\` never shows them

```
~/.cargo/registry/src/*/typst-world-0.4.0/src/diagnostic.rs
    pub fn eval_error(&self, diags: &[SourceDiagnostic]) -> EvalError {
        let diagnostics = diags
            .iter()
            .map(|diag| Diagnostic {
                message: diag.message.as_str().to_owned(),
                location: self.locate(diag.span),
                trace: diag
                    .trace
                    .iter()
                    .filter_map(|point| self.locate(point.span))
                    .collect(),
            })
            .collect();
```
A load error reaches the user as only the message:
```
warning: tasks/broken.typ:2: eval error: unexpected argument: titel
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("typst-world, the upstream crate").

== TO-BE

- 🟢 `typst-world`'s `Diagnostic` carries Typst's hints, and its `Display` keeps them reachable.
- 🟢 `mt` prints each hint under the load error it belongs to.


== Changes

- `typst-world` `src/diagnostic.rs`: add `hints` to `Diagnostic` and fill it from `diag.hints`.
- `crates/mindtape/src/loader/mod.rs`: keep the hints on `LoadError` after bumping typst-world.


== Rollback

- Pin the previous typst-world in `Cargo.toml` and revert the loader.


== Out of scope

- The `eval error:` prefix and the line format, which the error-format task decides.


== Dogfood

Broken tasks in this repo's `truth/tasks/` show Typst's fix next to the error.
