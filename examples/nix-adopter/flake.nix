# A project that takes `mt` from Mindtape's flake into its dev shell, as an
# adopter does. `just smoke-flake` locks it against the working tree.
{
  inputs = {
    mindtape.url = "github:mlavrinenko/mindtape";
    # Mindtape's own pin, so the lock gains nothing more and the build is the
    # one in Mindtape's binary cache.
    nixpkgs.follows = "mindtape/nixpkgs";
  };

  outputs =
    { mindtape, nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAll = f: nixpkgs.lib.genAttrs systems (system: f system nixpkgs.legacyPackages.${system});
    in
    {
      devShells = forAll (
        system: pkgs: {
          default = pkgs.mkShell { packages = [ mindtape.packages.${system}.default ]; };
        }
      );
    };
}
