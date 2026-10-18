set quiet := true
set unstable := true
set working-directory := '..'
set script-interpreter := ["bash", "-eo", "pipefail"]

[default]
[private]
list:
    @just --list bench

[arg('N', pattern='\d+', help='number of deterministic task files to generate')]
[doc("Regenerate the gitignored examples/large benchmark tasks")]
gen N='2000':
    examples/large/generate.sh {{ N }}

[arg('N', pattern='\d+', help='number of generated task files to benchmark against')]
[doc("Benchmark mt ls and mt check against a large task set")]
run N='2000':
    cargo build --release -p mindtape-cli
    examples/large/generate.sh {{ N }}
    export MINDTAPE_NOW=2026-07-16 && hyperfine \
        --warmup 1 --runs 3 \
        --command-name "mt ls ({{ N }} tasks)" "./target/release/mt -w examples/large ls" \
        --command-name "mt check ({{ N }} tasks)" "./target/release/mt -w examples/large check"
