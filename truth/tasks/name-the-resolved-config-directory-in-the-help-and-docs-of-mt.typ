#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Name the resolved config directory in the help and docs of mt",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 4),
  tags: ("found", "cli", "docs"),
  x-goals: (
    product: verdicts(
      (
        cheap-adoption: helps[a macOS or Windows user finds their presets and worktrees file],
      ),
      "readable-tasks",
      "implementable-tasks",
      "enforced-practice",
      "typst-over-markdown",
      "plain-files",
    ),
    process: verdicts(
      (
        docs-match-code: helps[the docs name the directory `mt` reads on every platform],
      ),
      "sound-codebase",
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 5),
)

== AS-IS

+ Auditing the scripts and docs for paths that only hold on one machine
+ 🔴 #raw("crates/mindtape-cli/src/init.rs"): \`mt\` names \`~/.config/mindtape\` as its config directory on every platform, while it reads \`MINDTAPE\_CONFIG\_HOME\`, else \`dirs::config\_dir()\`, which is \`~/Library/Application Support\` on macOS and \`%APPDATA%\` on Windows

```
crates/mindtape-cli/src/init.rs:51:    /// `~/.config/mindtape/presets`, or a path to a config or preset directory.
crates/mindtape-cli/src/init.rs:233:    println!("\nuser presets live in ~/.config/mindtape/presets and shadow these.");
crates/mindtape-cli/src/cli.rs:77:    /// List worktree aliases (`@name` sugar; `~/.config/mindtape/worktrees.toml`).
www/content/agents.typ:90:`~/.config/mindtape/worktrees.toml`. A `[[worktrees]]` entry with `name` and
www/content/rule-packs.typ:76:layer `~/.config/mindtape/defaults.toml` it is `~/.config`. `mt` refuses a missing file and
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/init.rs").

== TO-BE

- 🟢 `mt init --list-presets` prints the user preset directory `mt` resolves, such as `$MINDTAPE_CONFIG_HOME/mindtape/presets`, and omits the line when none resolves.
- 🟢 The `init --preset` and `worktrees` help and the two doc pages name the config directory, not `~/.config`.


== Changes

- `crates/mindtape-cli/src/init.rs`: print `preset::user_preset_dir()`, and reword the `--preset` help.
- `crates/mindtape-cli/src/cli.rs`: reword the `worktrees` help.
- `www/content/agents.typ`, `www/content/rule-packs.typ`: name the file in the config directory.
- `crates/mindtape-cli/tests/init_e2e.rs`: assert the printed directory.


== Rollback

- Revert the files.


== Out of scope

- Test fixtures and comments that use `/home/u` or `~/.config` as sample paths.
- `www/content/config.typ`, which already says "on Linux".


== Dogfood

Not applicable, this repo's maintainers run Linux, where the printed path is unchanged.
