#!/usr/bin/env bash

set -o pipefail
set -o nounset
set -o errexit

cargo build

qemu-system-aarch64 \
    -machine raspi3b \
    -display none \
    -serial null \
    -serial stdio \
    -kernel ./target/aarch64-unknown-none-softfloat/debug/kernel
