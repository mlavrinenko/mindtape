# The tree-sitter grammars Vale 3.27 reads comments through, one WebAssembly
# module per language. The modules ship as release assets of the Vale version
# the shell pins; `VALE_GRAMMARS_PATH` in shell.nix points here, so a lint run
# needs no network and no `vale sync`.
{
  lib,
  runCommandLocal,
  fetchurl,
  unzip,
  vale,
}:
let
  hashes = {
    bash = "sha256-OD9B2se6f/a2IxCukPvsrVhytCHhY2JCjoX20zgygqQ=";
    css = "sha256-78mCupKIKN9p06Co0X/0CVtYkNn5GMQc40rf+UDTFSk=";
    rust = "sha256-BAeIUAx0FLPnifCPAoHk0S/c8ZdyC+sjjicTm95RFjE=";
    yaml = "sha256-gYSmzyMp9svGdI+/iAeuW1jeerIPmhnxMB79HDakuXE=";
  };
in
runCommandLocal "vale-grammars-${vale.version}" { nativeBuildInputs = [ unzip ]; } ''
  mkdir -p "$out"
  ${lib.concatMapStringsSep "\n" (name: ''
    unzip -j ${fetchurl {
      url = "https://github.com/vale-cli/vale/releases/download/v${vale.version}/grammar-${name}.zip";
      hash = hashes.${name};
    }} "grammar-${name}/styles/config/grammars/${name}.wasm" -d "$out"
  '') (lib.attrNames hashes)}
''
