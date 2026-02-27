build-all: build-linux-x64 build-linux-arm64 build-linux-armv7

build-linux-x64:
    cargo build --release --target=x86_64-unknown-linux-gnu

build-linux-arm64:
    cargo build --release --target=aarch64-unknown-linux-musl

build-linux-armv7:
    cargo build --release --target=armv7-unknown-linux-musleabihf