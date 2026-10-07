// docs/src/contributing/crap-baseline.typ renders
// docs/contributing/crap-baseline.md through docs/generate-md.sh (typlite;
// docs/src/lib.typ lists the constraints).
//
// The path stripping and the pre-mint check are in
// .just/scripts/crap-baseline.sh and crap-baseline-check.sh. The ratchet the
// baseline feeds is explained in coverage-crap.typ.
#import "../lib.typ": doc-title, just

#metadata((
  output: "docs/contributing/crap-baseline.md",
)) <mindtape-md>

#doc-title[The CRAP baseline]

What #just.crap-baseline() writes to `crap-baseline.json`, and what it checks
first. The ratchet that reads the baseline is in
#link("docs/contributing/coverage-crap.md")[docs/contributing/coverage-crap.md].

= Why the paths are repo-relative

`cargo crap --workspace` reports absolute paths, and cargo-crap 0.3.1 has no
flag to shorten them. So #just.crap-baseline() strips the workspace root
before it writes, and the tracked file holds no machine's layout.

Relative paths lose no matching. cargo-crap first matches an entry on file,
function and line. Any checkout at another root, CI's included, misses that
pass and pairs entries by path suffix instead. With relative paths, every
machine uses the suffix pass. Measured on a relative mint, 1056 of 1058
entries paired as unchanged. The other two are one file's two functions named
`fake_editor`, which no suffix can tell apart on any machine.

#just.crap-baseline-check() fails on an absolute path in the baseline. A mint
by hand writes them back, and nobody reviews a thousand-entry JSON diff line
by line. It also fails on a baseline that scores no function, which would
pass the ratchet by treating every function as new. It runs a positive
control first, because a filter that matches nothing reports a clean file.

gitleaks can't do this job. Its default ruleset ignores any finding under
`/home/<user>/`, and a config that extends it can only add allowlists.
#just.check-secrets() uses gitleaks for credentials.

= What the mint checks first

#just.crap-baseline() runs the threshold check before it writes. That check
fails on any function scoring above 30, so the mint can't record a bad score
as acceptable. It also refuses a stale lcov, so the committed scores are ones
a clean run reproduces.

The mint doesn't run the ratchet. The ratchet compares scores against the
file the mint is about to rewrite, so a deliberate rise in complexity fails
it. That rise is exactly when
#link("docs/contributing/coverage-crap.md")[coverage-crap.md] tells you to
mint, so running the ratchet first would refuse the mint's own use case.
