set quiet := true
set unstable := true
set working-directory := '..'
set script-interpreter := ["bash", "-eo", "pipefail"]

[default]
[private]
list:
    @just --list outdatty

[doc("Fail if a source changed without its dependents reconfirmed")]
[group("gate")]
check:
    bash .just/scripts/outdatty-check.sh

[arg('GROUP', pattern='[a-z][a-z0-9-]*', help='outdatty group to re-confirm — the names .config/outdatty.yaml declares')]
[arg('DEPENDENTS', pattern='[A-Za-z0-9._/-]+', help='record only these dependents of GROUP, leaving the rest pending')]
[doc("Re-confirm one dependency group, or only the named dependents in it, into .config/outdatty.lock")]
update GROUP *DEPENDENTS:
    outdatty update --group {{ GROUP }} {{ prepend('--dependent ', DEPENDENTS) }}

[doc("Re-record every group: claims a review of every group's dependents")]
update-all:
    outdatty update

[doc("Lock the Docs-Unaffected trailer check against a fixture repo")]
[group("gate")]
trailer-fixture:
    bash .just/scripts/outdatty-review-trailer-fixture-check.sh
