<!-- Generated from docs/src/contributing/gates.typ by `just docs md`. Do not edit; edit the source. -->

# Quality gates

How `just check` runs its gates, what stops two gates from breaking each other, and what the pre-commit hook asserts. The rules for contributors are in CONTRIBUTING.md.

## How check runs

`just check` runs its gates in parallel. Each arm is `just mmz <gate>`, which runs `mmz just <gate>`, so mmz caches each gate under the recipe’s name. An arm whose declared inputs are unchanged since its last pass is skipped. `.mmz/config.yaml` declares the inputs and tags every gate `gate`. Then `mmz --is-fresh --tag gate` asserts that every gate passed against the current tree without running any of them. The pre-commit hook asserts it, and so does `mt flip` into a terminal status, through an `[[on.flip]]` hook.

No arm is wrapped in `chronic`, because mmz already keeps a green re-run quiet. A cache hit prints nothing (`on_hit: ""` in `.mmz/config.yaml`), a miss streams its output live, and a failure exits through just’s `[parallel]`. `just test <filter>` runs outside mmz, so a filtered run never records a pass for the full suite.

Every rule reads the `gates-meta` scope, which includes the `Justfile`, so a `Justfile` edit re-runs every arm. A script under `.just/scripts/` has a scope of its own, so editing it re-runs the arms that reach it and the few that read the whole directory.

### Inputs a glob cannot name

`just check-crap` reads `target/coverage/lcov.info`, which git ignores. Its `lcov` scope sets `gitignore: false`, because mmz drops ignored paths from every other scope. With the lcov in its inputs, the arm reruns when `just cover` writes a new lcov, so a cached skip never outlives the missing file.

`just check-secrets` scans the whole tree, but `.mmz/config.yaml` has no negative glob, and no gate may name `truth/tasks/**`. `mt flip` writes the task file it gates, so such an input would make the gate stale in the middle of the flip. The arm’s inputs are therefore one glob per top-level directory, and `.just/scripts/check-secrets-manifest.sh` fails naming any tracked file those globs miss.

### Arms must not read what another arm writes

The arms share one working tree. An arm that reads files another arm writes must stay out of them or take the writer’s lock.

- `www/generate.sh` deletes and recreates `www/generated/` on every run. Every reader and writer of that directory holds the `target/www-generated.lock` flock for its whole run.
- `just site-pages-fixture-check` copies `www/` into a temporary replica. It skips `public/` and `.tola/`, which the `just docs check` arm writes, and regenerates `www/generated/` inside the lock.
- `just check-secrets` derives a gitleaks allowlist from the paths git ignores, so the scan skips `www/generated/` and `www/.tola/`. gitleaks still stats the skipped directory itself, so the scan also holds the lock. `.just/scripts/check-secrets.sh` has the measurements.
- mmz hashes a rule’s inputs before it runs the rule, and aborts with exit 8 if a file vanishes during the hash. `www/.gitignore` ignores the temporary files the favicon conversion writes for that reason, and both mmz and the gitleaks bound read that file.

### One mt binary, one feature set

The `just test` arm runs `target/debug/mt` (`CARGO_BIN_EXE_mt`), and every arm that needs `mt` builds it. `mindtape-cli`’s dev-dependencies turn on the `test-support` features of `mindtape` and `swhid-mint`. So `cargo test --no-run` and a plain `cargo build -p mindtape-cli` build two different binaries for the same path. Cargo hard-links the newest build to `target/debug/mt`, and the swap deletes the file for about 85 ms. A test that spawns `mt` in that window fails with `No such file or directory`.

So every arm that builds `mt` runs `cargo build -p mindtape-cli --tests`, which builds the same binary as the test arm. `just check-examples` builds that way and runs `./target/debug/mt`, because `cargo run` takes no `--tests`. A build with other features writes to another target directory, as `just build-mt` does with `target/mt-dev`. `just check-rrule` builds the `rrule` features into `target/debug` and is safe only because it is not a `just check` arm. CI runs it alone.

`just check-gate-drift` fails naming file and line on any build of `mindtape-cli` that a `just check` arm reaches without `--tests` or a flag that writes elsewhere.

### Why each script has its own scope

`.mmz/config.yaml` names one scope per file under `.just/scripts/`. A single glob over the directory would re-run every gate when one script changed. A list only covers the scripts someone added to it, so `just script-scope-check` fails on a script no scope names, on a rule that reaches a script its inputs miss, and on a script that is neither an `outdatty.yaml` `dev-docs` source nor exempt. `.just/scripts/script-scope-check.sh` explains each arm.

## The formatter path lists

`just fmt` and `just fmt-check` read one list per formatter from the `Justfile`, so a path added there reaches both. `just check-gate-drift` fails when `TOML_FMT_PATHS` misses a TOML file git lists.

- `JUST_FMT_PATHS`: `.just/docs.just`
- `TOML_FMT_PATHS`: `.cargo/mutants.toml`, `.config/nextest.toml`, `.gitleaks.toml`, `.mindtape/config.toml`, `Cargo.toml`, `clippy.toml`, `crates/mindtape-cli/Cargo.toml`, `crates/mindtape-cli/src/axes/headers_spec.toml`, `crates/mindtape-cli/src/axes/workflow_flow.toml`, `crates/mindtape-cli/src/axes/workflow_gated.toml`, `crates/mindtape-cli/src/axes/workflow_personal.toml`, `crates/mindtape-cli/src/axes/workflow_todo_done.toml`, `crates/mindtape-cli/src/presets/flow/config.toml`, `crates/mindtape-cli/src/presets/gated/config.toml`, `crates/mindtape-cli/src/presets/personal/config.toml`, `crates/mindtape-cli/src/presets/todone/config.toml`, `crates/mindtape/Cargo.toml`, `crates/mindtape/src/defaults.toml`, `crates/mindtape/src/frameworks.toml`, `crates/mindtape/src/packs/mindtape-tasks.toml`, `examples/api-service/.mindtape/config.toml`, `examples/derived-tags/.mindtape/config.toml`, `examples/dsl/.mindtape/config.toml`, `examples/estimates/.mindtape/config.toml`, `examples/large/.mindtape/config.toml`, `examples/links/.mindtape/config.toml`, `examples/minimal/.mindtape/config.toml`, `examples/named-sets/.mindtape/config.toml`, `examples/priority/.mindtape/config.toml`, `examples/recurring/.mindtape/config.toml`, `examples/round-trip/.mindtape/config.toml`, `examples/rules/.mindtape/config.toml`, `examples/self-validating/.mindtape/config.toml`, `examples/statuses/.mindtape/config.toml`, `lib/typst.toml`, `rust-toolchain.toml`, `rustfmt.toml`, `tombi.toml`, `www/tola.toml`
- `TYPST_FMT_PATHS`: `lib`, `truth`, `.mindtape/templates`, `examples/api-service/tasks`, `examples/api-service/.mindtape/templates`, `examples/estimates/.mindtape/templates`, `examples/self-validating/.mindtape/templates`, `examples/named-sets/.mindtape/templates`, `examples/derived-tags/api`, `examples/derived-tags/web`, `examples/links/tasks`, `examples/named-sets/tasks`, `examples/priority/tasks`, `examples/recurring/tasks`, `examples/round-trip/tasks`, `examples/dsl/tasks`, `examples/rules`, `examples/statuses/tasks`, `www/content`, `www/templates`, `www/utils`, `docs/src`

just’s `--fmt` flag alone formats the root `Justfile` and skips every `mod` file (verified on just 1.46). So each file in `JUST_FMT_PATHS` gets its own `--justfile <file> --fmt` pass, and each needs `set unstable := true` of its own. A module file that moves without its entry drops out of both recipes.

## What the pre-commit hook asserts

`.githooks/pre-commit` runs `mmz --is-fresh --tag gate`, then `typstyle --check truth` and `mt check`. It refuses the commit when any of them fails.

`MINDTAPE_GATE_SCOPE=<reason>` skips the first assertion only. The other two still run, and `.githooks/commit-msg` refuses the commit unless its message carries a matching `Gate-Scope: <reason>` trailer. So `git log` shows which commits skipped the gate. When a commit may use it is in CONTRIBUTING.md.
