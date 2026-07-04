#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Slim `mt init` scaffold with `--full` opt-in, now that defaults are floored",
  status: done(2026, 7, 4),
  priority: ice(impact: 5, confidence: 0.75, ease: 6),
  tags: ("cli",),
  depends-on: (link("config-embedded-defaults.typ"),),
)

== Summary

With the embedded floor supplying legend / priority / difficulty / filters, `mt
init` no longer needs to dump ~190 lines of opinionated config into every repo.
Ship a minimal scaffold by default; keep the fully-expanded config behind
`--full` for users who want a concrete starting point to edit.

== Scope

- Slim default scaffold: `mt init` writes a short `.mindtape/config.toml` — the
  `[tasks] roots` line plus a header comment pointing at the layered defaults
  ("legend/priority/etc. come from the shipped floor and any
  `~/.config/mindtape/defaults.toml`; run `mt config show` to see the effective
  config; run `mt init --full` for a fully-expanded starter"). Keep it a handful
  of lines.
- `--full` flag on `InitArgs` (`init.rs:13`): write the current fully-expanded
  commented config (today's `default-config.toml` content) instead of the slim
  one. Two embedded strings: `SLIM_CONFIG` (new) and the existing `DEFAULT_CONFIG`
  renamed to `FULL_CONFIG`.
- `scaffold_config` (`init.rs:61`) takes which template to write; `run` selects on
  `args.full`.

== Tests

- `mt init` writes the slim scaffold; the written file parses + validates
  (through `load_config`, so the floor supplies the rest) and a subsequent `mt
  ls` / `mt check` on a task works.
- `mt init --full` writes the expanded config; it parses + validates and equals
  the prior behaviour.
- `--force` interplay unchanged for both.

== Notes

- Do NOT let the slim scaffold omit anything the floor does not supply and a
  project actually needs — `roots` is the one truly per-project key with no safe
  global default, so it stays in the slim scaffold.
- The full scaffold stays the canonical human-readable documentation of every
  section; keep its comments in sync when config shape changes (it is also what
  `mt install` seeds into XDG, per that task — source from one string if
  practical).
- Land after the foundation task so the slim config is actually sufficient.
