#import "../../utils/cli-ref.typ": cli-command, cli-page

#show: cli-page.with("schema")

`mt schema` prints the JSON Schema of a machine output, generated from the
same Rust types that print it. `mt schema --write` records the copies under
`schema/output/`, and `mt schema --check` fails when one differs, so a test
can compare the tree with the binary. It works outside a project.

#cli-command("schema")
