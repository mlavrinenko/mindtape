# The prose gates stay on Vale 3.26: 3.27's sentence segmenter joins a
# wrapped Typst paragraph into one sentence, which turns 147 existing prose
# alerts on across docs and task bodies. A backfill task cleans them, then
# this pin goes and the prose gates call `vale` (3.27, in vale.nix) too. The
# recipe is nixpkgs' own, with the version and both hashes pinned here so a
# gate reads one tool version in every checkout.
{
  lib,
  buildGoModule,
  fetchFromGitHub,
}:
buildGoModule (finalAttrs: {
  pname = "vale-prose";
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
  # The prose gates call this binary by name, and the 3.27 `vale` owns
  # `bin/vale`. A second `bin/vale` would collide in the shell's buildEnv.
  postInstall = ''
    mv $out/bin/vale $out/bin/vale-prose
  '';
  ldflags = [
    "-s"
    "-X main.version=${finalAttrs.version}"
  ];
  doCheck = false;

  meta = {
    description = "Syntax-aware linter for prose, pinned for the 3.26 prose gates";
    homepage = "https://vale.sh";
    license = lib.licenses.mit;
    mainProgram = "vale-prose";
  };
})
