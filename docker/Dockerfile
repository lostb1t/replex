FROM rust:latest as builder

WORKDIR /app/src
RUN USER=root cargo new --bin replex
COPY Cargo.toml Cargo.lock ./replex/

WORKDIR /app/src/replex
RUN RUSTFLAGS="--cfg tokio_unstable" cargo build --release

COPY ./ ./
RUN RUSTFLAGS="--cfg tokio_unstable" cargo build --release

FROM debian:bullseye-slim as replex
RUN apt update \
    && apt install -y openssl ca-certificates \
    && apt clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

WORKDIR /app
COPY --from=builder /app/src/replex/target/release/replex ./

EXPOSE 80
CMD ["/app/replex"]
