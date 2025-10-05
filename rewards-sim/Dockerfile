# syntax=docker/dockerfile:1

# ---------- Builder ----------
FROM rust:1 AS builder
WORKDIR /app

# Copy manifests and prebuild deps with a dummy main to cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release || true

# Now bring in the full source and build the actual binary
COPY . .
RUN cargo build --release

# ---------- Runtime ----------
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Copy binary
COPY --from=builder /app/target/release/rewards-simulator /app/rewards-simulator

# App serves static assets via include_str!/runtime FS from repo path; copy web assets if read at runtime
COPY src/web ./src/web

# Railway provides $PORT. The binary reads PORT or BIND_ADDR.
ENV PORT=35025

EXPOSE 35025

CMD ["/app/rewards-simulator"]


