FROM rust:1.67 as builder

WORKDIR /app/src
RUN USER=root cargo new --bin plex_proxy
COPY Cargo.toml Cargo.lock ./plex_proxy/

WORKDIR /app/src/plex_proxy
RUN cargo build --release

COPY ./ ./
RUN cargo build --release

# alpine needs musl, too much work
FROM debian:stable-slim
WORKDIR /app
RUN apt update \
    && apt install -y openssl ca-certificates \
    && apt clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*
COPY --from=builder /app/src/plex_proxy/target/release/plex_proxy /app
CMD ["/app/plex_proxy"]