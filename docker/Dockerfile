FROM rust:1.75-bookworm as builder

WORKDIR /app/src
RUN USER=root cargo new --bin replex
COPY Cargo.toml Cargo.lock ./replex/

WORKDIR /app/src/replex
RUN cargo build --release

# COPY ./ ./
COPY Cargo.toml Cargo.lock ./
COPY ./src ./src
RUN cargo build --release

from debian:bookworm-slim as replex
RUN apt update \
    && apt install -y openssl ca-certificates \
    && apt clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*
COPY --from=builder /app/src/replex/target/release/replex /app/

WORKDIR /app

EXPOSE 80
CMD ["/app/replex"]