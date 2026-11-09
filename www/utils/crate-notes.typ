// Hand-written, one-line prose per published crate, keyed by crate name.
// Everything a machine can state (the list, versions, direct-vs-transitive)
// comes from www/generated/crate-map.json. This dict has only what that file
// cannot: what the crate is for. `just docs::facts` fails naming any
// crate-map.json entry with no matching key here.
//
// Refs: truth/tasks/derive-the-drift-prone-doc-facts-from-their-sources.typ

#let crate-notes = (
  "mindtape": [The domain library, with the config, the task model, the
    filter language, config-driven validation and the project loader. Other
    tools (e.g. speconaut) depend on this, not on `mindtape-cli`.],
  "mindtape-cli": [The thin `mt` binary (clap) over `mindtape`.],
  "typst-world": [A configurable eval-only `typst::World` over a file-provider
    trait, plus a Typst-free value tree. Every eval-only tool in this list
    uses it, so none of them re-implements `World`.],
  "typst-harvest": [Evaluates a Typst file and collects the data it marked for
    export, treating Typst as a data format. Never lays out or renders a
    page.],
  "typst-cst": [Reads a file's structure (its heading tree, with source
    spans) by parsing alone, without evaluating it.],
  "typst-edit": [Rewrites a Typst file in place at known spans, applying a
    batch of edits as one change.],
  "typst-cleanup": [Retires a tracked file, refuses a dirty tree, writes a
    tombstone that records the file's SWHID, and rewrites every inbound link
    in one pass. It works without knowing what consumes those links.],
  "swhid-mint": [Mints and resolves Software Heritage IDs (SWHID, ISO/IEC
    18670:2025) for content-addressed artifact identity; a VCS-agnostic core
    with a git backend.],
  "artifact-gate": [A structural rule engine that works on any tree of nodes.
    It checks a tree against require/forbid/order/allow path rules and emits
    diagnostics. `mt check`'s own structural rules run on it.],
)
