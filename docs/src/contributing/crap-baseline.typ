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

What #just.crap-baseline() writes to `.config/crap-baseline.json`, and what it
checks first. The ratchet that reads the baseline is in
#link("docs/contributing/coverage-crap.md")[docs/contributing/coverage-crap.md].

= Why the paths are repo-relative

`cargo crap --workspace` reports absolute paths, and cargo-crap 0.3.1 has no
flag to shorten them. So #just.crap-baseline() strips the workspace root
before it writes, and the tracked file contains no machine's layout.

Relative paths do not weaken the gate. cargo-crap first matches a baseline
entry by file, function and line, but the live scan uses absolute paths, so
that match never succeeds. It then falls back to name and path suffix, which
cannot distinguish two functions with the same name in one file, such as the
eleven `Diag::from` in `from_lib.rs`. So #just.crap-check() gives the baseline
the scan's root. It also gives each such function the line of the one at the
same position in the scan. A line shift then changes no pairing, and a rise in
any of them fails.
`.just/scripts/_crap-ratchet-lib.sh` says how cargo-crap pairs and why.

#just.crap-baseline-check() fails on an absolute path in the baseline. A mint
by hand writes them back, and a reviewer does not read a thousand-entry JSON
diff line by line. It also fails on a baseline that scores no function. Such a
baseline would pass the ratchet by treating every function as new. It runs a
positive control first, because a filter that matches nothing reports a clean
file.

gitleaks cannot check this. Its default ruleset ignores any finding under
`/home/<user>/`, and a config that extends it can only add allowlists.
#just.lint-secrets() uses gitleaks for credentials.

= What the mint checks first

#just.crap-baseline() runs the threshold check before it writes. That check
fails on any function scoring above 30, so the mint can't record a bad score
as acceptable. It also refuses a stale lcov, so the committed scores are ones
a clean run reproduces.

The mint doesn't run the ratchet. The ratchet compares scores against the
file the mint is about to rewrite, so a deliberate rise in complexity fails
it. That rise is when
#link("docs/contributing/coverage-crap.md")[coverage-crap.md] tells you to
mint, so running the ratchet first would refuse that mint.
