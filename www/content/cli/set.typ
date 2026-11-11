#import "../../utils/cli-ref.typ": cli-command, cli-page

#show: cli-page.with("set")

`mt set` writes the core fields with an unambiguous literal: `title`,
`priority`, `difficulty`, `kind`, `assignee`, `waits-on`, `tags`, `due` and
`start`. It shares the field table, the `=`/`+=` grammar and the renderer with
`mt add --set`, so a value means the same on a filed task and a new one.
`+=` appends to a list field, and `=` replaces it. A directory is not a field:
`mt set dir=…` refuses, because `mt mv` only renames a filed task and `--dir`
chooses the folder only when adding.

#cli-command("set")
