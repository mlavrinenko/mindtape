{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    qahq.url = "github:mlavrinenko/qahq";
    # Typst-native static site generator that builds the docs under www/.
    tola.url = "github:tola-rs/tola-ssg/v0.7.1";
  };

  outputs =
    {
      flake-utils,
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
              cargo-tarpaulin
              clippy
              rustfmt
              hyperfine
              jq
              just
              moreutils
              nixd
              pagefind
              rust-analyzer
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
