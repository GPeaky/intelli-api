FROM rust:1.70.0-slim-bookworm
WORKDIR /usr/src/telemety-api

COPY src ./src
COPY Cargo.toml .

RUN apt-get update && apt-get install -y ca-certificates libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*
RUN cargo build --release

RUN ["cp", "./target/release/intelli-telemetry-f123", "/usr/local/bin/intelli-telemetry-f123"]
RUN ["rm", "-rf", "../telemety-api"]
CMD ["intelli-telemetry-f123"]