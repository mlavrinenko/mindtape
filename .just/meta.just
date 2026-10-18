set quiet := true
set unstable := true
set working-directory := '..'
set script-interpreter := ["bash", "-eo", "pipefail"]

[default]
[private]
list:
    @just --list meta

[doc("Lock the arg-attribute patterns against known-bad and known-good input")]
[group("gate")]
arg-pattern:
    bash .just/scripts/arg-pattern-check.sh

[doc("Lock positional-argument forwarding on the four passthrough recipes")]
[group("gate")]
positional-args:
    bash .just/scripts/positional-args-check.sh

[doc("Lock the script attribute and each justfile's strict-mode interpreter")]
[group("gate")]
script-attr:
    bash .just/scripts/script-attr-check.sh

[doc("Lock every gate script into a named mmz scope and outdatty's script groups")]
[group("gate")]
script-scope:
    bash .just/scripts/script-scope-check.sh

[doc("Fail if TOML_FMT_PATHS drifts from git, or a gate builds mt without --tests")]
[group("gate")]
gate-drift:
    bash .just/scripts/check-gate-drift.sh

[doc("Lock the formatter gates against a mis-formatted fixture")]
[group("gate")]
fmt-fixture:
    bash .just/scripts/fmt-fixture-check.sh

[doc("Lock the size gate's scan scope against an oversized hidden-dir fixture")]
[group("gate")]
size-fixture:
    bash .just/scripts/size-fixture-check.sh
