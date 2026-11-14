# Vale 3.27 reads code comments through tree-sitter grammars, shipped as
# WebAssembly modules (see vale-grammars.nix). `typst2vast` support needs
# 3.18, and this build needs Go 1.27.3, which the pinned nixpkgs-tombi
# carries. The recipe is nixpkgs' own, with the version and both hashes
# pinned here so a gate reads one tool version in every checkout.
{
  lib,
  buildGoModule,
  fetchFromGitHub,
}:
buildGoModule (finalAttrs: {
  pname = "vale";
  version = "3.27.0";

  src = fetchFromGitHub {
    owner = "vale-cli";
    repo = "vale";
    tag = "v${finalAttrs.version}";
    hash = "sha256-jV8Oe8DCWijjhHMBhnTn0NehG5EjEPdjC/JDTSkZSNc=";
  };

  # go-tree-sitter/php ships headers only, which `go mod vendor` prunes.
  proxyVendor = true;
  vendorHash = "sha256-XDvUiBTTpO4K3bHdXNBoLOCxwx37wQJVY8VAEwzWglw=";
  subPackages = [ "cmd/vale" ];
  ldflags = [
    "-s"
    "-X main.version=${finalAttrs.version}"
  ];
  doCheck = false;

  meta = {
    description = "Syntax-aware linter for prose";
    homepage = "https://vale.sh";
    license = lib.licenses.mit;
    mainProgram = "vale";
  };
})
