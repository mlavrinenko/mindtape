<!-- Generated from docs/src/contributing/docs-site-build.typ by `just docs md`. Do not edit; edit the source. -->

# The docs site build

Why `just docs build` and `just docs check` rebuild the site under `www/content/` from scratch every time.

## Why the build is always clean

Both recipes pass `--clean` to `tola build`. tola does not track a `json()` read as a page dependency, so an incremental build can keep a page’s old HTML after the fact it reads changes. Every page that reads a generated fact is exposed, and `just docs check` would then validate HTML that no current source produces. With `--clean`, every page recompiles on every build.

`just docs serve` keeps the incremental build, because it is not a gate and a full rebuild on each keystroke would make hot reload useless.
