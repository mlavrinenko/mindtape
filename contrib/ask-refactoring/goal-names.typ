// contrib/ask-refactoring/goal-names.typ: the goal names a swept task rates,
// read from the registries that declare them.
//
// `contrib/ask-refactoring/goals.sh` queries this file with `typst query`, so
// the `== Goals` block every arm hands to `mt add` carries each goal the
// project declares. Importing the registries instead of matching their source
// means a reformatted registry still reads, and a key no marker could spell
// fails the query rather than printing a name the arm's task cannot rate.
//
// A query driver, not a task: nothing imports it, and `mt` never loads it. Its
// imports resolve `@local/mindtape` through the package path the caller
// passes, the way `docs/generate-md.sh` renders a page that imports a registry.
#import "/truth/goals.typ": goals as product-goals
#import "/truth/process-goals.typ": goals as process-goals

#metadata(product-goals.keys() + process-goals.keys()) <sweep-goals>
