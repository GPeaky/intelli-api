FROM fedora:latest
WORKDIR /usr/src/intelli
COPY . .

RUN dnf -y update && dnf -y install openssl-devel ca-certificates pkgconfig protobuf-compiler rust cargo && dnf clean all

RUN cargo update \
    && cargo build --release

RUN ["cp", "./target/release/intelli", "/usr/local/bin/intelli"]
CMD ["intelli"]