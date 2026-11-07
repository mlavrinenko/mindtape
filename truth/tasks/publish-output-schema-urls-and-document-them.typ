#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Publish output schema URLs and document them",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("docs", "www", "cli", "stateless"),
  x-goals: (
    product: verdicts(
      (self-describing: helps[`$schema` resolves wherever the JSON is read]),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        docs-match-code: helps[the published URL is checked against the committed schema],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: wip(2026, 11, 8),
  links: depends-on("generate-a-schema-for-every-json-output.typ")
    + depends-on("add-schema-pointer-to-object-json-outputs.typ")
    + depends-on("wrap-collection-json-outputs-in-an-object-envelope.typ"),
)

== AS-IS

+ Run `mt install --force` and read `www/content/schema.typ`.
+ 🔴 Install writes only `config.schema.json` and `at.schema.json`; the page documents the task record but names no output schema or `$schema` URL; no docs gate checks a `$schema` URL against a file.

== TO-BE

- 🟢 `mt install` writes the current output schemas beside the two editor schemas, and `mt schema <name>` prints any of them.
- 🟢 `www/content/schema.typ` lists every `mt --json` output, its stable `main` URL and the version in that URL, plus how a local install resolves when offline.
- 🟢 A working-tree build names its checkout through `MINDTAPE_SCHEMA_BASE`; a released binary names the canonical `main` base.
- 🟢 `mt schema --check` runs in the test suite and CI, so a committed schema that differs fails the build; `just docs::check` fails when a `$schema` URL has no matching file.

```
$ mt schema ls | jq -r '.$id'
https://raw.githubusercontent.com/mlavrinenko/mindtape/main/schema/output/ls/1.schema.json
```

== Changes

- `crates/mindtape-cli/src/cmd/install/mod.rs` and `tests.rs`: write the output schemas under the XDG schema directory, with `--no-schema` and `--force`, and pin their bytes to `SCHEMAS`.
- `www/content/schema.typ`: the output list from a real `mt schema` capture, the `$schema` URL rule and the local fallback.
- `www/generate-import-export.sh`: capture `mt schema`; `www/generate.sh` keeps `MINDTAPE_SCHEMA_BASE` pinned empty so the transcript names the released URL.
- `.just/scripts/check-schema-urls.sh` (new), wired into `docs::check`: map each registered schema and each docs URL to `schema/output/...` and fail when the file is missing.
- `docs/src/agents.typ`, `CHANGELOG.md`: the new contract.

No release-hook schema write and no `release.yml` schema step. `schema_e2e::recorded_schemas_match_the_binary` runs `mt schema --check`, `ci.yml` runs `just check`, and `release.yml` requires a green run on the tag's commit, so drift fails a normal gate.

== Out of scope

- Changing any output shape; this task publishes what the earlier tasks built.

== Dogfood

The docs build reads the same generated schemas `mt install` writes, and a broken URL fails here.
