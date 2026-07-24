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
      }
    );
}
