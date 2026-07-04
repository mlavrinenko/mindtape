#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "`mt config show` — print the effective merged config and its layer sources",
  status: proposed(2026, 7, 4),
  priority: ice(impact: 5, confidence: 0.8, ease: 6),
  tags: ("cli",),
  depends-on: (
    link("config-embedded-defaults.typ"),
    link("config-defaults-discovery.typ"),
  ),
)

== Summary

Once defaults are layered (embedded floor + XDG + directory `defaults.toml` +
project `config.toml`), the effective config is no longer any single file on disk.
Users need a way to see the merged result and where each layer came from — so
shipped defaults are discoverable without being copy-pasted into every repo.

== Scope

- New `mt config` subcommand (clap `Config(ConfigArgs)` in
  `crates/mindtape-cli/src/cli.rs:31`) with a `show` action (the default when no
  action is given). Dispatch to a new `crates/mindtape-cli/src/config_cmd.rs`.
- `mt config show` — print the effective merged config as valid, re-parseable
  TOML (serialise the loaded `Config`, or re-emit the merged `toml::Table` from
  `merge_layers`; prefer the merged table so comments-free but faithful).
- `mt config show --origin` (alias `--sources`) — list the contributing layers in
  precedence order (`LayerSource` from the discovery task): embedded, XDG path,
  each directory `defaults.toml`, project `config.toml`. Mark which exist.
- `mt config show --defaults` — print just the embedded floor (`EMBEDDED_DEFAULTS`),
  so a user seeding an XDG or project override sees the baseline verbatim.
- Optional `--json` for the effective config (reuse the crate's serde/json path).

== Tests

- `mt config show` output re-parses through `parse_config` to an equal `Config`
  (round-trip).
- `--origin` lists exactly the layers `discover_layers` returned, in order.
- In a bare project (no `config.toml`), `show` reflects the floor (e.g. the legend
  and its terminal set are present).

== Notes

- Read-only command; no writes. Pairs with `mt install` (which seeds the XDG
  defaults `show --defaults` prints) and slim `mt init`.
- Keep the emitted TOML deterministic (stable key order) so snapshot tests and
  `diff`-based user workflows are stable; `toml::to_string` over a `BTreeMap`-
  backed value already sorts keys.
