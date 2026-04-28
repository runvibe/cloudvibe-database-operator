FROM rust:1.95-slim AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

FROM debian:trixie-slim

RUN groupadd --system --gid 10001 cloudvibe \
    && useradd --system --uid 10001 --gid 10001 --create-home cloudvibe
USER cloudvibe
WORKDIR /home/cloudvibe

COPY --from=builder /app/target/release/cloudvibe-database-operator /usr/local/bin/cloudvibe-database-operator

EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/cloudvibe-database-operator"]
