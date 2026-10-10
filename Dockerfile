# Dependencies (cached, rebuilt only when Cargo.toml/Cargo.lock change)
FROM rust:1.98-bookworm AS deps
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release --locked && rm -rf src

# Build
FROM deps AS builder
COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs && cargo build --release --locked

# Development (live reload via cargo-watch)
FROM deps AS dev
RUN cargo install cargo-watch --locked
EXPOSE 3000
CMD ["sh", "-c", "cargo run --bin migrate && cargo run --bin seed && cargo watch --ignore target --ignore .git -x run"]

# Run
FROM debian:bookworm-slim AS final
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && adduser --disabled-password --gecos "" appuser

COPY --from=builder /app/target/release/workout-tracker-rust /usr/local/bin/
COPY --from=builder /app/target/release/migrate /usr/local/bin/

USER appuser
ENV LISTEN_ADDR=0.0.0.0:3000
EXPOSE 3000
CMD ["workout-tracker-rust"]
