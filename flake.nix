{
  # Only what `packages.default` builds from. The dev shell's inputs sit in
  # nix/dev/flake.nix, so a flake that takes Mindtape as an input locks these
  # and nothing else; `just smoke-flake` caps that count.
  inputs = {
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
      # naersk's own toolchain option, unused here: the build takes nixpkgs'
      # rustc and cargo.
      inputs.fenix.follows = "";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ flake-parts.flakeModules.partitions ];

      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];

      # `nix develop` loads the dev partition, and with it nix/dev's inputs;
      # `nix build` and `nix run` never do.
      partitionedAttrs.devShells = "dev";
      partitions.dev = {
        extraInputsFlake = ./nix/dev;
        module = ./nix/dev/shell.nix;
      };

      perSystem =
        { pkgs, ... }:
        let
          naersk' = pkgs.callPackage inputs.naersk { };
          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        in
        {
          # For `nix build` & `nix run` (produces the `mt` binary):
          packages.default = naersk'.buildPackage {
            # naersk would name a virtual workspace `rust-workspace`, and `nix run`
            # runs `bin/<name>` unless `mainProgram` says otherwise.
            name = "mindtape";
            src = ./.;
            inherit (cargoToml.workspace.package) version;
            doCheck = false;
            meta.mainProgram = "mt";
          };
        };
    };
}
