# syntax=docker/dockerfile:1.7

# Alpine's Rust image targets musl by default, so the binary is static and can
# run on distroless/static with no libc to copy along.
FROM rust:1-alpine AS build
RUN apk add --no-cache musl-dev
WORKDIR /src
COPY rust ./rust
ARG VERSION=dev
# CAPY_VERSION is read at compile time for `capy version`; it replaces the Go
# build's -ldflags "-X main.version=...".
RUN CAPY_VERSION="${VERSION}" cargo build --release \
    --manifest-path rust/Cargo.toml -p capy-cli \
 && cp rust/target/release/capy /out-capy

FROM gcr.io/distroless/static:nonroot
COPY --from=build /out-capy /usr/local/bin/capy
USER nonroot:nonroot
ENTRYPOINT ["/usr/local/bin/capy"]
CMD ["help"]
