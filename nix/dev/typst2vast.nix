# `typst2vast` is Vale's Typst reader: it parses a document with the Typst
# compiler's own syntax tree and emits the HTML Vale lints. No package carries
# it, and Vale looks it up on PATH by name, so the dev shell builds it from the
# tagged source. The version tracks the Cargo.lock the upstream tag ships.
{
  lib,
  rustPlatform,
  fetchFromGitHub,
}:
rustPlatform.buildRustPackage {
  pname = "typst2vast";
  version = "0.1.0";

  src = fetchFromGitHub {
    owner = "jdkato";
    repo = "typst2vast";
    rev = "e7c051015f76c987aa7f08598a7aea15be895a00";
    hash = "sha256-ifgUVUdrA9vihj4s0d44VkY/4i3YxXMek4KSZJrMvkY=";
  };

  cargoHash = "sha256-Uodxp2ejlYtL3W5n63XoiEp5gK16w38OhoCGPR3vCLY=";

  meta = {
    description = "Convert Typst to HTML for Vale, parsing without evaluating";
    homepage = "https://github.com/jdkato/typst2vast";
    license = lib.licenses.mit;
    mainProgram = "typst2vast";
  };
}
