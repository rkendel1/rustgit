FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin server

# Use a runtime base with glibc >= 2.39 so the built binary starts on Fly.
FROM debian:trixie-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/server /usr/local/bin
ENTRYPOINT ["/usr/local/bin/server"]
