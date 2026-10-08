# Build
FROM rust:1.98-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release --locked && rm -rf src

COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs && cargo build --release --locked

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