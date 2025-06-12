build:
    #!/usr/bin/env bash
    set -euxo pipefail
    PATH="$PATH:$HOME/.cargo/bin"
    cargo objcopy --release --bin kernel -- -O binary kernel8.img

run: build
    #!/usr/bin/env bash
    set -euxo pipefail
    export RUSTFLAGS="-Ctarget-cpu=native"
    export CARGO_BUILD_TARGET="x86_64-unknown-linux-gnu"
    cargo run --release --bin chainload-push
