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
  };

  outputs =
    {
      ejectest,
      flake-utils,
      linecop,
      outdatty,
      naersk,
      nixpkgs,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) { inherit system; };

        naersk' = pkgs.callPackage naersk { };

        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        version = cargoToml.workspace.package.version;
      in
      {
        # For `nix build` & `nix run` (produces the `mindtape` binary):
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
