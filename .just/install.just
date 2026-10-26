set quiet := true
set unstable := true
set working-directory := '..'
set script-interpreter := ["bash", "-eo", "pipefail"]

[default]
[doc("Install a snapshot of everything: mt, the Typst lib, schema, skill")]
all: bin lib
    bash .just/scripts/install.sh

[doc("Build mt in release mode and install it into $CARGO_HOME/bin")]
bin:
    bash .just/scripts/install-bin.sh

[doc("Install the installed mt's own copy of the Typst library for local #import")]
lib: bin
    bash .just/scripts/install-lib.sh

[doc("Remove the Typst library installed by lib")]
uninstall-lib:
    bash .just/scripts/uninstall-lib.sh
