set quiet := true
set unstable := true
set working-directory := '..'
set script-interpreter := ["bash", "-eo", "pipefail"]

[default]
[doc("Install everything: mt, the Typst lib, schema, skill")]
all: bin lib
    bash .just/scripts/install.sh

[doc("Build mt in release mode and install it into $CARGO_HOME/bin")]
bin:
    bash .just/scripts/install-bin.sh

[doc("Install the mindtape Typst library for local #import (dev symlink)")]
lib:
    bash .just/scripts/install-lib.sh

[doc("Remove the dev symlink installed by lib")]
uninstall-lib:
    bash .just/scripts/uninstall-lib.sh
