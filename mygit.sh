#!/bin/sh
exec cargo run \
    --quiet \
    --target-dir=/tmp/basic-rust-git \
    --manifest-path $(dirname $0)/Cargo.toml "$@"
