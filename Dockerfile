FROM rust:slim-bullseye AS buildstage
WORKDIR /build
ENV PROTOC_NO_VENDOR 1
RUN /bin/sh -c set -eux;\
    rustup component add rustfmt;\
    apt-get update;\
    apt-get install -y --no-install-recommends git librocksdb-dev libssl-dev pkg-config clang protobuf-compiler;\
    rm -rf /var/lib/apt/lists/*;
COPY . /build/
RUN cargo build --release
FROM debian:bullseye-slim
COPY --from=buildstage /build/target/release/cloud-op /usr/bin/
CMD ["controller"]
