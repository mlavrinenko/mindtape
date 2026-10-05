<!-- Generated from docs/src/contributing/documentation.typ by `just docs md`. Do not edit; edit the source. -->

# Writing the documentation

The doc gates catch mechanical errors. `just docs check` finds a dead link, `just check-doc-facts` a derived fact with no entry, `just check-doc-recipes` a hand-typed recipe name, `just check-doc-prose` history and shouting, and `just check-doc-blocks` a hand-typed code block. No gate finds duplicated prose or a page that does not help its reader. The rules below cover what no gate finds.

The span, recipe, prose and block gates read the Typst sources through one extractor in `.just/scripts/_doc-blocks-lib.sh`. It parses each file with `typstyle --ast` and prints every inline raw, block raw line, comment line and line of prose with its file and line, so a fence written inside a string hides nothing. The same file lists the sources the gates read and fails when a list entry matches no file.

The span gate hands each `mt` command line it finds in a span, after a `$ ` prompt, `VAR=value` words or `&&`, to `mt _cli --check-argv`. That runs the `@name` rewrite, clap and the status-verb parser that `mt` itself runs, so the gate holds no copy of the command grammar. A missing required argument passes, because a span shows part of a command.

## Where the docs live

`www/content/*.typ` is the docs site, and `www/utils/*.typ` holds the notes its generated references render. `docs/src/readme.typ`, `docs/src/agents.typ` and `docs/src/skill.typ` render to README.md, AGENTS.md and `skills/mindtape/SKILL.md`. `docs/src/contributing.typ` renders CONTRIBUTING.md, and `docs/src/contributing/*.typ` render `docs/contributing/*.md`. Edit the source, never the render: `just docs md` overwrites it.

## Rewrite the whole file

Before you change a file, decide who reads it and what they can do after. Then rewrite the whole file for that reader. A file can pass every gate and still waste its reader’s time, because each paragraph that stayed was cheaper to keep than to delete. A file at its line cap covers too much, so trimming words to fit does not fix it. Rewrite it whole, or move the argument to a `docs/contributing/*.md` file and leave a link in the always-loaded source.

## State each fact once

A generated fact (`www/generated/*.json`, read by both render paths) cannot drift. A hand-written sentence can. Two pages state the same fact in different words, one of them changes, and no gate fails. So state a fact on one page and link to it from the others. Content that both render paths show, like the tagline and problem list that open README and the site root, lives once in a module both call (`www/utils/demo.typ`).

The `just dup-prose` audit reports two kinds of repeat that are deliberate. A document an agent reads whole (AGENTS.md, the skill and its references) keeps its own copy of what it needs, because its reader may have no site access. Shared code, such as a render loop or a crate table, may be extracted or copied. Never paraphrase it, because reworded code is worse than either.

## Use checked examples

A code block in the docs comes from a real file under `examples/` that `mt check` validates, from a capture of a real run, or from a derived fact. `www/generate-examples.sh` cuts the lines between `// docs:slice <id>` and `// docs:end` in a fixture into `www/generated/`, and the page renders them with `www/utils/slices.typ`. The DSL page’s field table is `lib/prelude.typ`’s own signature. A hand-typed snippet goes stale when the code changes, and the reader cannot tell.

A capture is a record `www/capture.sh` writes. It holds each command the build ran in a terminal, what the command printed and its exit code. A page shows a record with `terminal(<slug>)`, or `commands(<slug>)` for the commands alone. Both print each command from the record, so a page cannot show a command that did not run. `exit-code(<slug>)` prints the exit code of a refusal the page shows. A step that exits with an unexpected code fails the build, which keeps a refusal the page shows from passing unnoticed. Every `mt` error exits 1, so a refusal capture also names its reason with `--expect <regex>`. The build fails unless the output, without its colour, matches the regex, and it fails on a non-zero `--exit` that has no `--expect`.

`just check-doc-blocks` fails on a fenced block, or a block `raw(...)`, whose text is written in the source. A block no fixture can hold, such as a grammar sketch, carries `// docs:hand-typed <reason>` on the line above it. The gate lists the sources not yet converted. A listed source with no hand-typed block left fails, so the list only shrinks.

## Plain prose

Write for a busy engineer who reads each sentence once. The rules apply to comments in doc sources too: a comment says what the code does and why, in as few lines as it takes.

1. **State the rule and its reason.** A reader who knows the reason can apply the rule to a case it does not name.
   - Bad: “Keep the recipe and the mmz `examples` scope in step.”
   - Good: “Keep the recipe and the mmz `examples` scope in step, because mmz re-runs this gate only when a file in that scope changes.”
2. **Leave out history.** The reader needs today’s behaviour, and `git log` holds how it got there.
   - Bad: “the word-splitting that `[positional-arguments]` used to guard that recipe against, refused at dispatch now instead.”
   - Good: “a value containing a space, which the pattern refuses at dispatch, so the recipe never word-splits it.”
3. **Use the literal word.** A reader has to translate a figure of speech, and two readers translate it differently.
   - Bad: “`.just/docs.just`’s currently userless one is load-bearing and cannot be tidied away”
   - Good: “`.just/docs.just` keeps its own even though none of its recipes uses it yet”
4. **Make the actor the subject.** Files, fields and values do not know, want or own anything. Name the program or person that acts.
   - Bad: “against what git knows”
   - Good: “against the TOML files git lists, tracked or not yet added”
5. **Drop the “not Y” half when no reader asked about Y.** It makes the reader hold Y in mind only to discard it.
   - Bad: “this recipe is only the enforcement point, not a separate lint surface.”
   - Good: “this recipe runs them.”
6. **Put one idea in each sentence.** A reader takes in one idea at a time, so split a sentence at its semicolons and at a colon that announces a claim.
   - Bad: “Editing any source stales the lcov, so it skips on the commits that matter; it catches the free case only.”
   - Good: “It runs only against a fresh lcov. Editing any source makes the lcov stale, so it skips on the commits that matter.”
7. **Do not shout.** All-caps words and adverbs like “crucially” add emphasis. Emphasis on many lines teaches the reader to skip it.
   - Bad: “An unknown bare name is refused naming what IS available”
   - Good: “An unknown bare name is refused with a list of the available packs”

`just check-doc-prose` enforces two of these rules. It fails on the history words `now`, `used to`, `no longer`, `anymore` and `formerly`, and on an all-caps word outside its acronym list. A shell constant passes only as `$NAME`, `${NAME}` or `NAME=`, and a lone capital beside an all-caps word counts as shouting, so wrap a constant’s bare name in backticks. It skips raw spans, fenced blocks and double-quoted text, so a quoted bad example passes. It applies the same rules to the full-line `#` comments of every shell script, YAML file, Justfile and `.just` module that git lists, tracked or not yet added, and to every `description` in `schema/config.schema.json`. So a new script is read the day it lands, and a new violation fails the gate outright. The word lists are in `.just/scripts/check-doc-prose.sh`.

## The duplicate-prose audit

`just dup-prose` reads the doc sources, strips markup and code, and reports sentences that two files share, worst pair first. It is advisory: the threshold is a judgement call, and you decide where each fact lives. Run it before and after a refactor. The algorithm, its settings and why it is written in jq are in `contrib/dup-prose.jq`.
