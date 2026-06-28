{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    ejectest = {
      url = "github:mlavrinenko/ejectest";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    linecop = {
      url = "github:mlavrinenko/linecop";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    outdatty = {
      url = "github:mlavrinenko/outdatty";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    # Dedicated, newer nixpkgs pin used only to compile the jscpd copy-paste
    # detector (below). jscpd 5.0.5's bundled Cargo.lock pins oxc 0.133.0,
    # which needs rustc >= 1.93; the workspace deliberately stays on the
    # rustc 1.92 of `nixpkgs` (rust-toolchain.toml). Isolating the tool's
    # compiler here keeps that MSRV pin intact. Rev proven by speconaut.
    nixpkgs-jscpd.url = "github:NixOS/nixpkgs/e80236013dc8b77aa49ca90e7a12d86f5d8d64c9";
  };

  outputs =
    {
      ejectest,
      flake-utils,
      linecop,
      outdatty,
      naersk,
      nixpkgs,
      nixpkgs-jscpd,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) { inherit system; };

        naersk' = pkgs.callPackage naersk { };

        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        version = cargoToml.workspace.package.version;

        # jscpd (Rust copy/paste detector, gates `just check-dry`) isn't in
        # nixpkgs yet; build the published crate from crates.io so the dev
        # shell stays self-contained. naersk does the vendoring —
        # nixpkgs' fetch-cargo-vendor is UA-blocked by crates.io's
        # data-access policy — and `singleStep` skips naersk's dummy-src
        # deps build, which fails to parse this crate's manifest.
        #
        # Built against `nixpkgs-jscpd` (rustc >= 1.93), not the workspace
        # `nixpkgs` (rustc 1.92): jscpd's locked oxc 0.133.0 raises the MSRV
        # past 1.92. Keeping its own pkgs leaves the project toolchain pin
        # (rust-toolchain.toml = 1.92.0) untouched.
        pkgsJscpd = import nixpkgs-jscpd { inherit system; };
        naerskJscpd = pkgsJscpd.callPackage naersk { };
        jscpd = naerskJscpd.buildPackage {
          src = pkgsJscpd.fetchCrate {
            pname = "jscpd";
            version = "5.0.5";
            hash = "sha256-U0unnpWU8gxudX+YVuvE+uBk6hkNWiMjJxXDBAGzqiA=";
          };
          singleStep = true;
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
          # sccache as the rustc wrapper is a dev-shell-only speedup. Kept here
          # (not in a committed .cargo/config.toml) so it never leaks into the
          # `nix build` sandbox, where sccache is absent and cargo would fail
          # trying to exec it. Loaded via direnv (`use flake`).
          RUSTC_WRAPPER = "sccache";
          nativeBuildInputs =
            [
              jscpd
              ejectest.packages.${system}.default
              linecop.packages.${system}.default
              outdatty.packages.${system}.default
            ]
            ++ (with pkgs; [
              rustc
              cargo
              cargo-machete
              cargo-tarpaulin
              clippy
              rustfmt
              just
              moreutils
              nixd
              rust-analyzer
              sccache
              typst
              typstyle
            ]);
          buildInputs = with pkgs; [ openssl ];
          shellHook = ''
            just install-hooks > /dev/null 2>&1
          '';
        };
      }
    );
}
