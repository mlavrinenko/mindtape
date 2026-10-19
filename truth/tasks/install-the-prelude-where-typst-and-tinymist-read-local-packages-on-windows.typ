#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Install the prelude where Typst and tinymist read @local packages on Windows",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  estimate: (src-files: 2),
  tags: ("found", "tooling"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a Windows user's tinymist finds the prelude that `mt install` wrote, so the autocompletion the Quickstart names works],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the Quickstart says tinymist finds the installed prelude, which holds on every platform],
    ),
  ),
  assignee: "agent",
  status: wip(2026, 10, 19),
)

== Problem

+ Checking that tinymist finds the prelude `mt install` writes
+ 🔴 #raw("crates/mindtape/src/loader/app_dirs.rs"): `data_home()` uses `dirs::data_local_dir()`, `%LOCALAPPDATA%` on Windows, but Typst and tinymist read `@local` from `dirs::data_dir()`, `%APPDATA%`

```
app_dirs.rs:42   resolve(std::env::var_os(DATA_HOME_ENV), dirs::data_local_dir)
packages.rs:169  dirs::data_dir().map(|dir| FsPackages::new(dir.join("typst/packages")))
http.rs:81       .or_else(|| Some(dirs::data_dir()?.join(DEFAULT_PACKAGES_SUBDIR).into())),
```

- https://github.com/typst/typst/blob/bd5ebec285505889e8ff1d4f68804ab147a699f3/crates/typst-kit/src/packages.rs#L166-L169
- https://github.com/Myriad-Dreamin/tinymist/blob/45b50f0d928e825248fa1c2fcbcd68549e957b5d/crates/tinymist-package/src/registry/http.rs#L78-L81
- https://github.com/dirs-dev/dirs-rs/blob/1c2e3efad531aa67a5656eaedf53fdb8fa9094f7/src/win.rs#L10-L11

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/loader/app_dirs.rs").

== TO-BE

- 🟢 With `$MINDTAPE_DATA_HOME` unset, `mt install` writes the prelude under the directory Typst and tinymist read `@local` packages from, and `mt` reads it back from there. That's `$TYPST_PACKAGE_PATH` when set, else `dirs::data_dir()` joined with `typst/packages`, so `%APPDATA%\typst\packages` on Windows.
- 🟢 `$MINDTAPE_DATA_HOME` still moves both to its own `typst/packages`, and set but empty still resolves no global package.
- 🟢 The `allowed/` records stay under `data_home()`, which keeps `dirs::data_local_dir()`.

== Changes

- `Cargo.toml`, `crates/mindtape/Cargo.toml`, `Cargo.lock`: require typst-world 0.5.5, whose `DiskProvider` reads `@local` where Typst does and whose `platform_package_root()` names that directory.
- `crates/mindtape/src/loader/app_dirs.rs`: `package_root()` moves here and falls back to `typst_world::platform_package_root()` rather than `data_home()`, so the directory has one source.
- `crates/mindtape-cli/src/cmd/init/mod.rs`: `package_dir()` builds on `package_root()`.
- `crates/mindtape/src/loader/cache.rs`: the build key folds `package_root()`, the directory a read resolves, so moving `$TYPST_PACKAGE_PATH` busts it as moving `$MINDTAPE_DATA_HOME` does. The cache stays in `.mindtape/cache/`.
- `.just/scripts/_typst-package.sh`: the dev symlink follows `$TYPST_PACKAGE_PATH` too.
- `www/content/config.typ`: name the directory and both variables.

== Rollback

- Revert the commit. A Windows machine that already ran `mt install` keeps the old copy under `%LOCALAPPDATA%` until `mt install` runs again.

== Out of scope

- The global config directory, which `config_home()` resolves and no Typst tool reads.
- Per-editor setup, which tinymist's own docs cover.

== Dogfood

Not applicable, this repo's contributors run Linux and macOS, where the two directories match.
