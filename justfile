build:
    #!/usr/bin/env bash
    set -euxo pipefail
    PATH="$PATH:$HOME/.cargo/bin"
    cargo objcopy --release --bin kernel -- -O binary kernel8.img
