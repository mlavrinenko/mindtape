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
  status: proposed(2026, 11, 7),
  links: depends-on("generate-a-schema-for-every-json-output.typ")
    + depends-on("add-schema-pointer-to-object-json-outputs.typ")
    + depends-on("wrap-collection-json-outputs-in-an-object-envelope.typ"),
)

== AS-IS

+ Run `mt install --force` and read `www/content/schema.typ`.
+ 🔴 Install writes only `config.schema.json` and `at.schema.json`; the page documents the task record but names no output schema or `$schema` URL; the release workflow never builds the output schemas.

== TO-BE

- 🟢 `mt install` writes the current output schemas beside the two editor schemas, and `mt schema <name>` prints any of them.
- 🟢 `www/content/schema.typ` lists every `mt --json` output, its `$schema` URL and the version in that URL, plus how a local install resolves when offline.
- 🟢 The release workflow regenerates the tagged schemas and refuses a tag when a committed schema differs; a docs gate fails when a `$schema` URL has no matching file.

```
$ mt schema ls | jq -r '.$id'
https://raw.githubusercontent.com/mlavrinenko/mindtape/v0.5.0/schema/output/ls/1.schema.json
```

== Changes

- `crates/mindtape-cli/src/cmd/install/mod.rs`: write the output schemas under the XDG schema directory, still with `--no-schema` and `--force`.
- `www/content/schema.typ`: the output table, the `$schema` rule and the local fallback.
- `www/generate.sh`: capture the schema list, and pin `MINDTAPE_SCHEMA_BASE` for stable docs output.
- `.github/workflows/release.yml`: run `mt schema --write` and fail on a diff before tagging.
- `.just/scripts/check-schema-urls.sh` (new): map each `$schema` URL to `schema/output/...` and fail when the file is missing.
- `docs/src/agents.typ`, `CHANGELOG.md`: the new contract.

== Out of scope

- Changing any output shape; this task publishes what the earlier tasks built.

== Dogfood

The docs build reads the same generated schemas `mt install` writes, and a broken URL fails here.
