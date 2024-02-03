#!/bin/sh
rm -rf target || true
docker run --rm -v ./:/src -v ./../src/:/bindings -e CARGO_HTTP_MULTIPLEXING=false -e RUSTFLAGS='-C target-feature=-crt-static' -e BINDINGS_DIR=/bindings -w /src -it libvips-builder sh -c "cargo update"
docker run --rm -v ./:/src -v ./../src/:/bindings -e CARGO_HTTP_MULTIPLEXING=false -e RUSTFLAGS='-C target-feature=-crt-static' -e BINDINGS_DIR=/bindings -w /src -it libvips-builder sh -c "cargo build"
