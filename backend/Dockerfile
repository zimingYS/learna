# ===== Stage 1: 构建 =====
FROM rust:1.98.0-slim AS builder
WORKDIR /app

# 依赖
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src \
    && echo "" > src/lib.rs \
    && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src

# 源码
COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs src/lib.rs && cargo build --release

# ===== Stage 2: 运行时 =====
FROM debian:trixie-slim

RUN apt-get update\
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/backend /usr/local/bin/backend
CMD ["backend"]
