FROM rust:1.72-slim-bookworm
WORKDIR /usr/src/telemety-api

COPY src ./src
COPY Cargo.toml .
COPY certs/ ./certs

RUN apt-get update && apt-get install -y libssl-dev ca-certificates pkg-config && rm -rf /var/lib/apt/lists/*
RUN cargo build --release

RUN ["cp", "./target/release/intelli", "/usr/local/bin/intelli"]
CMD ["intelli"]