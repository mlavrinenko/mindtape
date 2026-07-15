#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "JSON schema for TOML config",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 4,
  ),
  tags: ("cli", "tooling",),
  status: proposed(2026, 7, 15),
)

== Summary

Bundle a JSON Schema for .mindtape/config.toml so TOML editors (tombi/Taplo) assist filling config values. mt install writes the schema next to the app-wide config (XDG); mt init scaffolds config.toml with a top schema-association comment directive pointing at the installed schema.

== Scope

Generate schema from the mindtape config structs as source of truth (schemars derive); ship a golden schema file with a drift gate; mt install writes it to the XDG config dir near defaults.toml (with --no-schema/--force); mt init prepends a tombi schema-association comment directive to the scaffold; tests validate shipped configs against the schema and cover install/init wiring.
