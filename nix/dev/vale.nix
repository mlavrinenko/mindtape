# Vale 3.26 is the newest release the pinned Go toolchain builds: 3.27 asks
# for Go 1.27.3. `typst2vast` support needs 3.18, and the pinned nixpkgs
# carries 3.14. The recipe is nixpkgs' own, with the version and both hashes
# pinned here so a gate reads one tool version in every checkout.
{
  lib,
  buildGoModule,
  fetchFromGitHub,
}:
buildGoModule (finalAttrs: {
  pname = "vale";
  version = "3.26.0";

  src = fetchFromGitHub {
    owner = "vale-cli";
    repo = "vale";
    tag = "v${finalAttrs.version}";
    hash = "sha256-mUABzCStq/gA8vZmqAo+1cuW5vjNyd6XRyH7VYTO4tw=";
  };

  # go-tree-sitter/php ships headers only, which `go mod vendor` prunes.
  proxyVendor = true;
  vendorHash = "sha256-BaJcmBJj86Sohgs5mZtRnlv1h121dUD2CoKvPU4mTEA=";
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
