#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Hint at a template only for the field a refusal names",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 2),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[a refused `mt add` points only at templates that would fix the refusal],
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
  status: done(2026, 10, 3),
)

== Problem

+ Filing the stem-hash tasks with \`mt add --template feat -f \<yaml>\`
+ 🔴 #raw("crates/mindtape-cli/src/add_hint.rs"): mt add hints at every template that pins any value the caller passed, even when the refusal is about another field, so a body word-cap refusal lists four sweep templates for assignee=agent.

+ Run `mt add "x" --template feat -f t.yaml` with `assignee: agent` and a TO-BE over 120 words.
+ 🔴 It prints `error: x: body.content.rules.section-word-cap: …`, then `hint: template sweep-coverage files assignee=agent` and three more like it.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/add_hint.rs").

== TO-BE

- 🟢 `mt add` hints at a template only when a refused breach names the field that template pins.


== Changes

- `crates/mindtape-cli/tests/add_template_hint_e2e.rs`: red test first: a body-rule refusal with `assignee` set prints no hint.
- `crates/mindtape-cli/src/add_hint.rs`: filter hints by the fields the breaches name.


== Rollback

Revert the commit; every refusal hints again.

== Out of scope

- The hint wording.


== Dogfood

Agents filing tasks here see a hint only when it fixes their refusal.

