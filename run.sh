#!/usr/bin/env bash

set -o pipefail
set -o nounset
set -o errexit

cargo build --release --bin chainload-pull --features qemu

cd chainload-push
cargo build --release --bin chainload-push
cd -

# cargo build --release --bin kernel
cargo build --bin kernel

qemu-system-aarch64 \
    -machine raspi3b \
    -display none \
    -serial stdio \
    -serial pty \
    -d int \
    -kernel ./target/aarch64-unknown-none-softfloat/release/chainload-pull
