FROM fedora:latest
WORKDIR /usr/src/intelli
COPY . .

RUN dnf -y update && dnf -y install openssl-devel ca-certificates pkgconfig flatbuffers-compiler rust cargo && dnf clean all

RUN cargo update \
    && cargo build --release

CMD ["/target/build/intelli"]