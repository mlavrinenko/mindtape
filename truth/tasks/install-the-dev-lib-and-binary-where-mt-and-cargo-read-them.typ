#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Install the dev lib and binary where mt and cargo read them",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 4),
  tags: ("found", "tooling"),
  x-goals: (
    product: verdicts(
      (:),
      "readable-tasks",
      "implementable-tasks",
      "enforced-practice",
      "cheap-adoption",
      "typst-over-markdown",
      "plain-files",
    ),
    process: verdicts(
      (
        sound-codebase: helps[`just install` puts the lib where `mt` loads it],
      ),
      "docs-match-code",
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 5),
)

== AS-IS

+ Auditing the scripts and docs for paths that only hold on one machine
+ 🔴 #raw(".just/scripts/_typst-package.sh"): \`just install-lib\` links the lib under a fixed \`\$HOME/.local/share\`, while \`mt\` reads \`\$MINDTAPE\_DATA\_HOME\`, else \`\$XDG\_DATA\_HOME\`, and \`just install-bin\` ignores \`\$CARGO\_HOME\`

```
.just/scripts/_typst-package.sh:7:TARGET="$HOME/.local/share/typst/packages/local/mindtape"
.just/scripts/install-bin.sh:4:BIN="$HOME/.cargo/bin"
.just/scripts/install.sh:11:"$HOME/.cargo/bin/mt" install --no-lib --no-defaults --force
crates/mindtape/src/app_dirs.rs: resolve(std::env::var_os(DATA_HOME_ENV), dirs::data_local_dir)
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".just/scripts/_typst-package.sh").

== TO-BE

- 🟢 With `XDG_DATA_HOME` or `MINDTAPE_DATA_HOME` set, `just install-lib` links the lib under that directory, where `mt` loads `@local/mindtape`.
- 🟢 With `CARGO_HOME` set, `just install-bin` writes `$CARGO_HOME/bin/mt` and `just install` runs that binary.
- 🟢 With neither set, both land where they land today.


== Changes

- `.just/scripts/_typst-package.sh`: resolve the data directory as `mt` does on Linux.
- `.just/scripts/install-bin.sh`: install into `${CARGO_HOME:-$HOME/.cargo}/bin`.
- `.just/scripts/install.sh`: run the binary from the same directory.
- `Justfile`: name the install directory in the `install-bin` doc string.


== Rollback

- Revert the four files.


== Out of scope

- macOS and Windows data directories: contributor support there is deferred.
- The skill links under `~/.claude/skills` and `~/.config/opencode/skills`.


== Dogfood

Every `just install` in this repo uses the recipes.
