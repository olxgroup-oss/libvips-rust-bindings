#!/bin/sh

docker run --rm -v $(pwd):/src -v $(pwd)/../src/:/bindings -e RUSTFLAGS='-C target-feature=-crt-static' -e BINDINGS_DIR=/bindings -w /src -it libvips-builder cargo build