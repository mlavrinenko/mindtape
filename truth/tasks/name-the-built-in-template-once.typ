#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Pin the built-in template name to one constant",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.95,
    ease: 7,
  ),
  difficulty: 2,
  links: (
    (
      task: "let-rules-and-templates-choose-where-new-tasks-land.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "tooling", "stateless"),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[one name decides the fallback template everywhere],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[a rename cannot leave one spelling pointing at nothing],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 11),
)

== AS-IS

+ Reviewing the template-name code added with the add hooks. Only `BUILTIN_TEMPLATE` is new; the other spellings predate the commit.
+ 🔴 #raw("crates/mindtape/src/config/add_hooks.rs"): The built-in template name `task` is spelled in four independent places.

```
crates/mindtape/src/config/add_hooks.rs: const BUILTIN_TEMPLATE: &str = "task";
crates/mindtape-cli/src/cmd/add/values.rs: if name == "task" {
crates/mindtape/src/config/templates.rs: default: "task".to_owned(),
crates/mindtape/src/config/defaults.toml: default = "task"
```

`template_names` inserts the constant, `resolve_template` compares the literal, `TemplatesConfig::default` sets the literal again, and the embedded floor is a fourth live spelling. A rename that edits only the three Rust spots leaves `defaults.toml` pointing at a template `resolve_template` then refuses.


== TO-BE

- 🟢 One public constant names the built-in template.
- 🟢 Listing, resolving and `TemplatesConfig::default` read that constant.
- 🟢 A test asserts the embedded floor's `templates.default` equals the constant, since TOML cannot reference Rust.

```
pub const BUILTIN_TEMPLATE: &str = "task";
```


== Changes

- `crates/mindtape/src/config/add_hooks.rs`: make `BUILTIN_TEMPLATE` public and document it.
- `crates/mindtape-cli/src/cmd/add/values.rs`: compare against the constant in `resolve_template`.
- `crates/mindtape/src/config/templates.rs`: build `TemplatesConfig::default` from the constant.
- `crates/mindtape/src/config/defaults.toml`: keep the literal, and pin it to the constant.
- `crates/mindtape/src/config/tests.rs`: assert the embedded floor and the `Default` impl both equal `BUILTIN_TEMPLATE`.


== Out of scope

- Renaming the built-in template or changing which template is the default.


== Dogfood

A future built-in template rename touches one line.

