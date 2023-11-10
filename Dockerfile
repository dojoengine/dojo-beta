FROM debian:buster-slim as base

LABEL description="Dojo is a provable game engine and toolchain for building onchain games and autonomous worlds with Cairo" \
    authors="tarrence <tarrence@cartridge.gg>" \
    source="https://github.com/dojoengine/dojo" \
    documentation="https://book.dojoengine.org/"

FROM base as amd64
COPY target/x86_64-unknown-linux-gnu/release/katana /usr/local/bin/katana
COPY target/x86_64-unknown-linux-gnu/release/sozo /usr/local/bin/sozo
COPY target/x86_64-unknown-linux-gnu/release/torii /usr/local/bin/torii

FROM base as arm64
COPY target/aarch64-unknown-linux-gnu/release/katana /usr/local/bin/katana
COPY target/aarch64-unknown-linux-gnu/release/sozo /usr/local/bin/sozo
COPY target/aarch64-unknown-linux-gnu/release/torii /usr/local/bin/torii
