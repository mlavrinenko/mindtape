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
      qahq,
      tola,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) { inherit system; };

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

        # cargo-crap (the CRAP-metric gate behind `just crap`) is not in
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
              qahq.packages.${system}.jscpd
              qahq.packages.${system}.ejectest
              qahq.packages.${system}.linecop
              qahq.packages.${system}.outdatty
              tola.packages.${system}.default
            ]
            ++ (with pkgs; [
              rustc
              cargo
              cargo-machete
              cargo-llvm-cov
              cargo-mutants
              clippy
              rustfmt
              mold
              hyperfine
              jq
              just
              librsvg
              moreutils
              nixd
              pagefind
              rust-analyzer
              # `tombi` and `typstyle` are the project formatters: Tombi is
              # the canonical TOML formatter (Tombi 0.9+ ships `format` /
              # `format --check` / `lint`); typstyle is the Typst formatter
              # `just fmt` runs after `cargo fmt`. Both stay on the project
              # PATH so editor-on-save and `just fmt-check` are byte-identical.
              tombi
              typst
              typstyle
              util-linux
              yq
            ]);
          buildInputs = with pkgs; [ openssl ];
          shellHook = ''
            export LLVM_COV=${llvm}/bin/llvm-cov
            export LLVM_PROFDATA=${llvm}/bin/llvm-profdata
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
