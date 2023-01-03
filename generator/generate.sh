#!/bin/sh

docker run --rm -v $(pwd):/src -v $(pwd)/../src/:/bindings -e CARGO_HTTP_MULTIPLEXING=false -e RUSTFLAGS='-C target-feature=-crt-static' -e BINDINGS_DIR=/bindings -w /src -it libvips-builder cargo build