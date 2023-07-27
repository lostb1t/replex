FROM rust:1.71 as builder

RUN update-ca-certificates

WORKDIR /app/src
RUN USER=root cargo new --bin replex
COPY Cargo.toml Cargo.lock ./replex/

WORKDIR /app/src/replex
RUN cargo build --release

COPY ./ ./
RUN cargo build --release

FROM debian:stable-slim as replex
COPY --from=builder /app/src/replex/target/release/replex /app/
RUN apt update \
    && apt install -y openssl ca-certificates \
    && apt clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*
EXPOSE 80
CMD ["/app/replex"]
