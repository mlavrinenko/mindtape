// Hand-written, one-line prose per published crate, keyed by crate name.
// Everything a machine can state (the list, versions, direct-vs-transitive)
// comes from www/generated/crate-map.json; this dict carries only what it
// cannot — what the crate is for. `just check-doc-facts` fails naming any
// crate-map.json entry with no matching key here.
//
// Refs: truth/tasks/derive-the-drift-prone-doc-facts-from-their-sources.typ

#let crate-notes = (
  "mindtape": [The domain library — config, the task model, the filter
    language, config-driven validation, the project loader. Other tools (e.g.
    speconaut) depend on this, not on `mindtape-cli`.],
  "mindtape-cli": [The thin `mt` binary (clap) over `mindtape`.],
  "typst-world": [A configurable eval-only `typst::World` over a file-provider
    trait, plus a Typst-free value tree — the substrate every eval-only tool
    in this list sits on, so none of them re-implements `World`.],
  "typst-harvest": [Evaluates a Typst file and collects the data it marked for
    export, treating Typst as a data format. Never lays out or renders a
    page.],
  "typst-cst": [Reads a file's structure (its heading tree, with source
    spans) by parsing alone, without evaluating it.],
  "typst-edit": [Rewrites a Typst file in place at known spans, applying a
    batch of edits as one change.],
  "typst-cleanup": [Consumer-blind orchestration for retiring a tracked
    artifact: a dirty-tree guard, SWHID tombstone minting, and atomic
    inbound-link rewriting.],
  "swhid-mint": [Mints and resolves Software Heritage IDs (SWHID, ISO/IEC
    18670:2025) for content-addressed artifact identity; a VCS-agnostic core
    with a git backend.],
  "artifact-gate": [A plane-agnostic structural rule engine: match a node
    tree against require/forbid/order/allow path rules and emit diagnostics —
    the engine `mt check`'s own structural rules run on.],
)
