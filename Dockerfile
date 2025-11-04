# Stage 1: Build
FROM rust:latest as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN RUST_MIN_STACK=16777216 cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/messagerie /app/messagerie
COPY .env .env
EXPOSE 3000
CMD ["./messagerie"]

