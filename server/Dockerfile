FROM rust:1.94-slim AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY common/ common/
COPY server/ server/
COPY client/Cargo.toml client/Cargo.toml
RUN mkdir -p client/src && echo "fn main() {}" > client/src/main.rs

RUN cargo build --release -p skycraft-server

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/skycraft-server /app/skycraft-server

EXPOSE 35565/udp

CMD ["/app/skycraft-server"]
