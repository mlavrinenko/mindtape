# `iai-callgrind-runner` is the binary `iai-callgrind`'s bench harness shells
# out to: the library encodes each case and pipes it to this runner, which
# drives valgrind and reports the counts. No package carries it, and the
# harness looks it up on PATH by name, so the dev shell builds it from the
# crates.io release.
#
# The version must equal the `iai-callgrind` dev-dependency in
# crates/mindtape/Cargo.toml: the library passes its own version as the
# runner's first argument and the runner refuses a mismatch. Bump both together.
{
  lib,
  rustPlatform,
  fetchCrate,
}:
rustPlatform.buildRustPackage {
  pname = "iai-callgrind-runner";
  version = "0.16.1";

  src = fetchCrate {
    pname = "iai-callgrind-runner";
    version = "0.16.1";
    hash = "sha256-wJTwaqAz8GWCJ/l9GRXYBVBkpPYrWxN4VQ7GdRFXmzM=";
  };

  cargoHash = "sha256-4N7P23bCeeJee/Cm3sSORByh+HzflOENqYqpu629mpA=";

  # The crate's own suite reads fixture files the published tarball does not
  # carry, so it fails on every case that parses one. The binary is what this
  # derivation is for, and `just bench::instructions` exercises it end to end.
  doCheck = false;

  meta = {
    description = "The runner behind iai-callgrind's instruction-count benchmarks";
    homepage = "https://github.com/iai-callgrind/iai-callgrind";
    license = lib.licenses.mit;
    mainProgram = "iai-callgrind-runner";
  };
}
