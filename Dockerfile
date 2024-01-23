FROM fedora:latest
WORKDIR /usr/src/intelli
COPY . .

RUN dnf -y update && \
    dnf -y install openssl-devel ca-certificates pkgconfig protobuf-compiler mold mimalloc && \
    dnf clean all

RUN dnf -y groupinstall "Development Tools" && \
    dnf clean all

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup update nightly && \
    rustup default nightly

RUN cargo fetch

RUN RUSTFLAGS="-C link-arg=-fuse-ld=mold -C target-cpu=native" cargo build --release

RUN ["cp", "./target/release/intelli", "/usr/local/bin/intelli"]

CMD ["intelli"]