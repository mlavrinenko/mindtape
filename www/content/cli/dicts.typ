#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("dicts")

`mt dicts` lists every dictionary the project can read: the `[dicts.<name>]`
sections it declares and the built-ins `mt` provides. Each line gives the name,
the entry count it resolved to, whether a lookup folds case, and where the
entries come from. `inline` covers `values`, a `path` file and a built-in.
`include:` names the dictionaries it unions in, so a dictionary built from both
built-in word lists shows their count together.

`mt dicts <name>` prints one dictionary's entries, one per line, after `mt`
merges `include`, `values` and `path`. That's the list a rule reading the
dictionary accepts, and a `fold` dictionary prints it lowercase. To declare a
dictionary, or to see how its entries merge, read
#link(u("/named-sets/#dictionaries"))[Views & dictionaries].

#cli-command("dicts")
