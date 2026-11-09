# Filing a batch with `mt import`

Reference for the mindtape skill. Read this before you import a whole backlog
at once instead of filing tasks one by one.

`mt import` reads canonical task JSON, the schema `mt ls --json` prints. It
checks the batch as one set, so links between records resolve in any order.

Set each record's stem before you import. `mt import` derives a stem by these
rules:

- A record with an `id` is written to that stem.
- A record without one, as the jq adapters emit, is named from its title. A
  non-ASCII title gives a non-ASCII stem (`аудит-поисковых-запросов.typ`), and
  the `links:` targets must spell it. For ASCII paths, give ASCII titles or
  set `id`.
- `mt import` appends `-2` to a taken stem, then `-3`. Links naming the
  original stem do not follow the rename; give the record an unused `id` or
  update those links.

Run `--dry-run` first: it writes nothing and prints `would file` per record.
The command writes the rest, then exits 1 if it skipped any record.
