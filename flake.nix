{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # Memoized command runner. `mt flip` into a terminal status gates on
    # `mmz --is-fresh --tag gate`, which asserts every gate-tagged rule in
    # .mmz/config.yaml already passed against this worktree instead of
    # running any of them again.
    mmz.url = "github:mlavrinenko/mmz";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    # A second nixpkgs, read for `tombi` and `worktrunk`. The main pin stays
    # where the tinymist/Typst note below wants it; it predates worktrunk's
    # package, and tombi >= 1.5 is the first with per-schema `format.rules` that
    # switch key and array-value sorting off (see .config/tombi.toml). Move it on
    # its own: `nix flake update nixpkgs-tombi`.
    nixpkgs-tombi.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    qahq.url = "github:mlavrinenko/qahq";
    # Typst-native static site generator that builds the docs under www/.
    tola.url = "github:tola-rs/tola-ssg/v0.7.1";
  };

  outputs =
    {
      flake-utils,
      mmz,
      naersk,
      nixpkgs,
      nixpkgs-tombi,
      qahq,
      tola,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) { inherit system; };
        # Only `tombi` and `worktrunk` are read from this one; see the input's
        # comment.
        pkgs-tombi = (import nixpkgs-tombi) { inherit system; };

        naersk' = pkgs.callPackage naersk { };

        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        version = cargoToml.workspace.package.version;

        # nixpkgs' rustc ships no `llvm-tools-preview` component, so
        # cargo-llvm-cov cannot find llvm-cov/llvm-profdata in the sysroot.
        # Point it at a standalone LLVM of the SAME major as rustc's (1.92 →
        # LLVM 21) — a mismatched llvm-profdata rejects the profraw format.
        llvm = pkgs.llvmPackages_21.llvm;

        # Windows cross-check toolchain (see `devShells.windows`).
        mingw = pkgs.pkgsCross.mingwW64;
        mingwBin = "${mingw.stdenv.cc}/bin/${mingw.stdenv.cc.targetPrefix}";

        # cargo-crap (the CRAP-metric gate behind `just crap::check`) is not in
        # nixpkgs. Build it from the crates.io tarball with naersk `singleStep`
        # — crates.io' data-access policy 403s the UA that nixpkgs'
        # `fetch-cargo-vendor` sends, so `buildRustPackage` cannot compute a
        # vendor hash; naersk vendors through cargo itself and sidesteps it.
        cargo-crap = naersk'.buildPackage {
          pname = "cargo-crap";
          version = "0.3.1";
          src = pkgs.fetchCrate {
            pname = "cargo-crap";
            version = "0.3.1";
            hash = "sha256-3qvyS5+7kQgmfk8Sl+29VJq+u+aECoh6n9A/9i0fRyY=";
          };
          singleStep = true;
          doCheck = false;
        };
      in
      {
        # For `nix build` & `nix run` (produces the `mt` binary):
        packages.default = naersk'.buildPackage {
          src = ./.;
          inherit version;
          doCheck = false;
        };

        # For `nix develop`:
        devShells.default = pkgs.mkShell {
          # RUSTC_WRAPPER comes from the host session (global kache); the dev
          # shell inherits it and it never reaches the `nix build` sandbox.
          nativeBuildInputs =
            [
              cargo-crap
              mmz.packages.${system}.default
              qahq.packages.${system}.canibump
              qahq.packages.${system}.jscpd
              qahq.packages.${system}.ejectest
              qahq.packages.${system}.linecop
              qahq.packages.${system}.outdatty
              tola.packages.${system}.default
              pkgs-tombi.tombi
              # `wt` forks and lands worktrees: `just fork` and `just land` run
              # it, and .config/wt.toml holds the hooks it fires. The main pin
              # predates worktrunk's nixpkgs package, so it comes from the
              # second nixpkgs too.
              pkgs-tombi.worktrunk
            ]
            ++ (with pkgs; [
              # .config/outdatty.lock records blake3 hashes, and the lock's
              # merge driver (.just/scripts/outdatty-lock-merge.sh) hashes a
              # file from the merged tree to pick the side that recorded it.
              b3sum
              rustc
              cargo
              cargo-machete
              cargo-llvm-cov
              # The release bump, CHANGELOG dating, commit and tag behind
              # `just release`; .config/release.toml configures it.
              cargo-release
              cargo-mutants
              # cargo-nextest is a GATE dependency, not a dev aid: `just test`
              # and `just cover` both run through it, so a second runner is
              # dead config. nextest runs no doctests, and the separate
              # `cargo test --doc` pass cratemplate keeps for them is absent
              # here deliberately: the tree has zero doctests (see the
              # `test *ARGS` recipe's comment for the verification), so a
              # doctest added later must come with its own gate.
              cargo-nextest
              clippy
              rustfmt
              mold
              # `just lint::workflows`: actionlint runs shellcheck over each
              # workflow `run:` block when it finds it on PATH.
              actionlint
              shellcheck
              # The secret scanner behind `just lint::secrets`. Pinned here
              # rather than installed per-machine so CI (`nix develop --command
              # just check`) and a local run read the same ruleset version —
              # the default ruleset is compiled into the binary, so a version
              # bump changes what the gate finds.
              gitleaks
              # `dot` draws the docs site's diagrams: www/capture.sh's
              # render_dot lays out the DOT that `mt graph` and the status
              # legend produce. Typst finds its packages only through the
              # network at build time, so a Typst Universe renderer such as
              # diagraph would make the docs build depend on packages.typst.org.
              graphviz
              hyperfine
              # `ps`: commit/amend.rs falls back to it where /proc is absent,
              # and its test calls it on Linux too. A bare host may lack it.
              procps
              jq
              just
              librsvg
              moreutils
              nixd
              pagefind
              # www/lsp-capture.py speaks LSP to tinymist for the Overview's
              # language server blocks. yq's wrapper also puts a python3 on
              # PATH, but a yq bump could drop it, so the docs build names it.
              python3
              rust-analyzer
              # tinymist has two users. Its bundled `typlite`
              # (`$out/bin/typlite`) renders the docs/src/*.typ sources into
              # the repo-root Markdown (`docs/generate-md.sh`) and the site's
              # Markdown copies. Its language server is what
              # www/lsp-capture.py records for the Overview. The closure is
              # about 196 MiB, and the nixpkgs binary cache holds it under
              # this flake's pinned input, so nothing builds locally.
              # Pin note: tinymist 0.14.10 (this pin) embeds Typst 0.14,
              # matching the `typst` below. Verify that still holds after any
              # nixpkgs bump, since a later tinymist moved to Typst 0.15.
              tinymist
              # `tombi` and `typstyle` are the project formatters: Tombi is
              # the canonical TOML formatter (`format` / `format --check` /
              # `lint`); typstyle is the Typst formatter `just fmt` runs after
              # `cargo fmt`. Both stay on the project PATH so editor-on-save
              # and `just fmt-check` are byte-identical. tombi comes from
              # `nixpkgs-tombi` (>= 1.5, needed to turn sorting off), the rest
              # from the main pin.
              typst
              typstyle
              util-linux
              yq
            ]);
          buildInputs = with pkgs; [ openssl ];
          # The dev `mt` is a binary on PATH, not a wrapper: `just build-mt`
          # writes `target/mt-dev/debug/mt`, and the worktrunk hooks in
          # .config/wt.toml call that recipe at the events that invalidate
          # the binary, and nothing rebuilds it per call. Its own target dir
          # keeps it off the lock `just check` holds on `target/`, and off
          # `target/debug/mt`, which the test arm links with test features
          # this plain build would relink away.
          shellHook = ''
            export LLVM_COV=${llvm}/bin/llvm-cov
            export LLVM_PROFDATA=${llvm}/bin/llvm-profdata
            # mold links the host target's builds: an edit to `mindtape`
            # relinks the test binaries in 4.6 s instead of 10.2 s with ld.
            # mold is ELF-only, so Linux only. A target variable, not
            # RUSTFLAGS, so a recipe that sets RUSTFLAGS (mutants) wins whole.
            ${pkgs.lib.optionalString pkgs.stdenv.isLinux ''
              export CARGO_TARGET_${pkgs.stdenv.hostPlatform.rust.cargoEnvVarTarget}_RUSTFLAGS="-Clink-arg=-fuse-ld=mold"
            ''}
            # Prepended idempotently: direnv reloads a shell it has already
            # loaded, and a second copy would only grow PATH.
            mt_root=$(git rev-parse --show-toplevel 2>/dev/null || echo "$PWD")
            case ":$PATH:" in
              *":$mt_root/target/mt-dev/debug:"*) ;;
              *) export PATH="$mt_root/target/mt-dev/debug:$PATH" ;;
            esac
            # One line when the binary is missing, naming the recipe and what
            # `mt` currently resolves to. stderr must be a terminal, so CI's
            # `nix develop --command` stays silent.
            if [ ! -x "$mt_root/target/mt-dev/debug/mt" ] && [ -t 2 ]; then
              echo "mindtape dev shell: no dev mt yet — run \`just build-mt\` (mt currently resolves to $(command -v mt || echo 'nothing'))" >&2
            fi
            just install-hooks > /dev/null 2>&1
          '';
        };

        # For `just check-windows` — cross-check the workspace against Windows
        # from Linux, so a `cfg(windows)` arm cannot rot between CI runs.
        #
        # Why a separate shell: nixpkgs' plain `rustc` ships std for the host,
        # wasm and bpf only, so `cargo check --target x86_64-pc-windows-gnu`
        # has nothing to compile against. `pkgsCross.mingwW64`'s rustc carries
        # std for BOTH x86_64-pc-windows-gnu (the target) and
        # x86_64-unknown-linux-gnu (the host — build scripts and proc macros
        # still run there), which is exactly the pair a cross `cargo check`
        # needs. rustup is the usual way to add a target and is deliberately
        # not used: it is not on this project's dev path, and a
        # rustup-managed target would drift from the pinned toolchain, whereas
        # pkgsCross comes from the same nixpkgs input as the default shell and
        # so is the same rustc version as rust-toolchain.toml for free.
        #
        # Why -gnu when the release ships -msvc: the two share one
        # `cfg(windows)` surface, which is all this codebase branches on, and
        # msvc's C dependencies (stacker, psm) cannot be built from Linux at
        # all. The Windows CI job is what proves -msvc and runtime behaviour;
        # this shell only moves compile breakage earlier.
        devShells.windows = pkgs.mkShell {
          nativeBuildInputs = [
            mingw.buildPackages.cargo
            mingw.buildPackages.rustc
            pkgs.just
          ];
          # The `cc` crate builds the C and assembly bits of stacker/psm for
          # the TARGET during a plain `cargo check` (build scripts run), so it
          # needs the mingw compiler by store path. The host toolchain stays
          # whatever mkShell's own stdenv provides.
          CC_x86_64_pc_windows_gnu = "${mingwBin}cc";
          AR_x86_64_pc_windows_gnu = "${mingwBin}ar";
          CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = "${mingwBin}cc";
          # sccache is inherited from the host session and is not part of this
          # toolchain's closure; a stale wrapper breaks the cross build.
          RUSTC_WRAPPER = "";
        };
      }
    );
}
