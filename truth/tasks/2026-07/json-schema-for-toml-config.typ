#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "JSON schema for TOML config",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 4,
  ),
  tags: ("cli", "tooling"),
  status: done(
    2026,
    7,
    15,
  )[done: `schemars`-derived schema, mt install writes it, mt init adds \#:schema directive. Drift + validation gates],
)

== Summary

Bundle a JSON Schema for .mindtape/config.toml so TOML editors (tombi/Taplo) assist filling config values. mt install writes the schema next to the app-wide config (XDG). mt init scaffolds config.toml with a top schema-association comment directive pointing at the installed schema.

== Scope

Generate the schema from the mindtape config structs as the source of truth (`schemars` derive). Keep a golden schema file with a drift gate. mt install writes it to the XDG config dir near defaults.toml, with `--no-schema`/`--force`. mt init prepends a tombi schema-association comment directive to the scaffold. Tests validate existing configs against the schema and cover install/init wiring.
