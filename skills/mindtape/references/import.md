# Filing a batch with `mt import`

Reference for the mindtape skill. Read this before you import a whole backlog
at once instead of filing tasks one by one.

`mt import` reads canonical task JSON, the schema `mt ls --json` prints. It
checks the batch as one set, so links between records resolve in any order.

Set each record's stem before you import, because `mt import` does not
announce how it picks one:

- A record with an `id` is written to that stem.
- A record without one, as the jq adapters emit, is named from its title. A
  non-ASCII title gives a non-ASCII stem (`аудит-поисковых-запросов.typ`), and
  the `links:` targets must spell it. For ASCII paths, give ASCII titles or
  set `id`.
- A taken stem gets a `-2` suffix, and links to it do not follow.

Run `--dry-run` first: it writes nothing and prints `would file` per record.
The command exits 1 when it skipped any record, after it wrote the rest.
