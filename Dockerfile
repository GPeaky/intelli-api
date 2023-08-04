FROM rust:1.70.0-slim-bookworm
WORKDIR /usr/src/telemety-api

COPY src ./src
COPY Cargo.toml .
COPY certs/ ./certs
COPY templates/ ./templates

RUN apt-get update && apt-get install -y libssl-dev ca-certificates pkg-config && rm -rf /var/lib/apt/lists/*
RUN cargo build --release

RUN ["cp", "./target/release/intelli-telemetry-f123", "/usr/local/bin/intelli-telemetry-f123"]
CMD ["intelli-telemetry-f123"]
