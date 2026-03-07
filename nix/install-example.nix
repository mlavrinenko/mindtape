# Example: adding MindTape to a NixOS flake configuration.
#
# 1. Add the input to your flake.nix:
#
#   inputs.mindtape.url = "github:mlavrinenko/mindtape";
#   inputs.mindtape.inputs.nixpkgs.follows = "nixpkgs";
#
# 2. Import the module and configure the service:

{ mindtape, ... }:

{
  imports = [ mindtape.nixosModules.default ];

  services.mindtape = {
    enable = true;
    user = "tank";
    group = "users";
    watchPaths = [
      { path = "/home/tank/notes"; }
    ];
  };
}
