FROM rust:1.73-slim-bookworm
WORKDIR /usr/src/telemety-api

COPY . .

RUN apt-get update && apt-get install -y libssl-dev ca-certificates pkg-config flatbuffers-compiler  && rm -rf /var/lib/apt/lists/*
RUN cargo update
RUN cargo build --release

RUN ["cp", "./target/release/intelli", "/usr/local/bin/intelli"]
CMD ["intelli"]