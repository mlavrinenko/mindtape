#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Fix the release workflow binary name",
  priority: ice(
    impact: 9,
    confidence: 0.95,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tooling", "gating"),
  estimate: (files: 1),
  status: proposed(2026, 7, 24),
)

== Summary

The release workflow copies target/\<triple>/release/mindtape, but the CLI crate declares its binary as mt, so the rename step fails on every target and a tagged release would attach no binaries at all. Derive the binary name from Cargo.toml so it cannot break silently again.

== Scope

- Derive the binary name from cargo metadata rather than hardcoding it
- Fail loudly if the name cannot be resolved
- Verify the derivation locally against the real workspace
