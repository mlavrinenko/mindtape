#import "../../utils/cli-ref.typ": cli-command, cli-page

#show: cli-page.with("templates")

`mt templates` lists every template `mt add` accepts, one line each: the
template's name, the `kind` it sets, how many slots it declares, and `default`
on the one `mt add` picks when you name none. The `kind` column reads `any` for
a template that leaves the field to a slot, as the built-in `task` does. The
listing has no status column, because `mt add` stamps the status itself.

The listing reads the same template files `mt add` reads, so a template added
to a directory in `[templates] dirs` appears without a config change. To see
what one template's slots are, run `mt add -t <name> --describe`.

#cli-command("templates")

`--json` wraps the list in `templates` under `$schema`, each with its `name`,
`kind`, `slots` and `default`.
