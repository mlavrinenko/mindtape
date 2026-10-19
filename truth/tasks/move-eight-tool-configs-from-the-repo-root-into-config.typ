#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Move eight tool configs from the repo root into .config/",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 40),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[each moved config is proven read at its new path, and the TOML configs `fmt-check` formats become its inputs],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 19),
)

== Problem

+ Run `ls -A` in the repo root.
+ 🔴 It lists 52 entries. Eight are one tool's config each: `tombi.toml`, `canibump.toml`, `.gitleaks.toml`, `.shellcheckrc`, `crap-baseline.json`, `.jscpd-baseline.json` and `release.toml`, plus `.cargo/mutants.toml`, the one tracked file in an ignored folder. Every one of those tools can read its config from `.config/`.


== TO-BE

- 🟢 The eight files live in `.config/`, so the root lists what a reader opens.
- 🟢 Each gate reads its config at the new path. A probe whose output depends on that config prints the same before and after the move, and prints something else with the config absent.
- 🟢 Each moved file is an input of the gate that reads it, so an edit re-runs that gate, and both baselines keep their merge drivers.


== Changes

- `.config/`: the eight files. `gitleaks.toml`, `shellcheckrc` and `jscpd-baseline.json` lose the leading dot.
- `Justfile`, `.just/lint.just`, `.just/crap.just`: `TOML_FMT_PATHS`, `canibump -m`, `shellcheck --rcfile`, jscpd's `--baseline` and the merge driver names.
- `.just/scripts/`: the `config` variable of `check-secrets.sh`, the crap scripts' baseline path, `cargo mutants --config` from `_mutants-env.sh`, `cargo release -c` in `release.sh`, and every comment naming a moved file.
- `.mmz/config.yaml`: each scope's path. `toml-fmt` takes `.config/*.toml`, so the canibump and release configs become `fmt-check` inputs.
- `.gitattributes`, `.gitignore`, `.mindtape/config.toml`, `outdatty.yaml`, `flake.nix`: the new paths. `.gitignore` drops its `.cargo/mutants.toml` exception.
- `docs/src/contributing.typ`, `docs/src/contributing/*.typ`, `.mindtape/templates/sweep-*.typ`, `contrib/ask-refactoring/*.sh`: the new paths, with the rendered docs.
- `crates/mindtape-cli/tests/it/*_baseline_merge_e2e.rs`: the baselines' new paths.


== Rollback

- Revert the commits. No tool reads both paths, so the old layout works again once the files are back.


== Out of scope

- Every other root file, `clippy.toml`, `rustfmt.toml`, `.linecop.yaml` and `outdatty.yaml` included. Each tool finds its config its own way and needs its own check.


== Dogfood

This repo's `just check` reads every moved config from `.config/`, and `ls -A` in the root lists eight fewer entries.

