FROM rust:1.75 as builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY build.rs ./

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rust-llm-runner /usr/local/bin/

RUN mkdir -p /root/.rust-llm-runner

EXPOSE 11434

CMD ["rust-llm-runner", "serve", "--host", "0.0.0.0", "--port", "11434"]
