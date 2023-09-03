FROM rust:1.71.1-slim-bookworm
WORKDIR /usr/src/telemety-api

COPY src ./src
COPY Cargo.toml .
COPY askama.toml .
COPY certs/ ./certs
COPY views/ ./views

RUN apt-get update && apt-get install -y libssl-dev ca-certificates pkg-config && rm -rf /var/lib/apt/lists/*
RUN cargo build --release

RUN ["cp", "./target/release/intelli-api", "/usr/local/bin/intelli-api"]
CMD ["intelli-api"]