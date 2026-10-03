# Filing a batch — `mt import`

Reference for the mindtape skill. Read this when importing a whole backlog at
once rather than filing tasks one by one.

`mt import` reads canonical task-JSON (`mt ls --json`'s own schema). It checks
the batch as one set, so links between records resolve in any order. One
behaviour it does not announce quietly produces a batch you have to redo:

- **The filename stem comes from `id`, else from `title`.** A record with an
  `id` is written to that stem; one without, as the jq adapters emit, is named
  from its title, so a non-ASCII title yields a non-ASCII stem
  (`аудит-поисковых-запросов.typ`) and the `links:` targets must spell it. Give
  ASCII titles when you want ASCII paths. A taken stem gets a `-2` suffix, and
  links to it do not follow.

`--dry-run` writes nothing and prints `would file` per record. The command
exits 1 when it skipped any record, after it wrote the rest.
