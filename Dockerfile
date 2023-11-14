FROM fedora:latest
WORKDIR /usr/src/intelli
COPY . .

RUN dnf -y update && \
    dnf -y install openssl-devel ca-certificates pkgconfig protobuf-compiler rust cargo mold && \
    dnf clean all

RUN cargo update
RUN RUSTFLAGS="-C link-arg=-fuse-ld=mold -C target-cpu=native" cargo build --release
RUN ["cp", "./target/release/intelli", "/usr/local/bin/intelli"]

CMD ["intelli"]