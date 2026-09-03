# Filing a batch — `mt import`

Reference for the mindtape skill. Read this when importing a whole backlog at
once rather than filing tasks one by one.

`mt import` reads canonical task-JSON (`mt ls --json`'s own schema). Two
behaviours it does not announce, each of which quietly produces a batch you
have to redo:

- **The filename stem comes from `title`, not `id`.** An input `id` is ignored
  for naming, so a non-ASCII title yields a non-ASCII stem
  (`аудит-поисковых-запросов.typ`), and `links:` targets being stems, every
  edge in the batch must spell it. Give ASCII titles when you want ASCII paths.
- **Intra-batch `links:` resolve only backwards.** A target is checked against
  files existing when its record is written, so a task must appear AFTER
  everything it depends on. Order topologically and one pass files the lot;
  otherwise dependents report `skipped: link target ... not found` while the
  rest are filed, leaving a half-imported backlog. A forward edge that cannot
  be reordered (the reverse `related` of a pair) is a hand edit afterwards.

`--dry-run` writes nothing, so every intra-batch edge reports as missing; a
batch with internal edges has no clean dry run — read the real run's report.