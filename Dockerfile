FROM rust:1.67 as builder

WORKDIR /app/src
RUN USER=root cargo new --bin replex
COPY Cargo.toml Cargo.lock ./replex/

WORKDIR /app/src/replex
RUN cargo build --release

COPY ./ ./
RUN cargo build --release

# alpine needs musl, too much work
FROM debian:stable-slim as standalone
WORKDIR /app
RUN apt update \
    && apt install -y openssl ca-certificates \
    && apt clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*
COPY --from=builder /app/src/replex/target/release/replex /app
EXPOSE 3001
CMD ["/app/replex"]

FROM nginx as nginx
COPY --from=builder /app/src/replex/target/release/replex /app
COPY nginx.conf.template /etc/nginx/templates

# FROM rust:1.61.0 as builder
# WORKDIR /usr/src/myapp
# COPY . .
# RUN cargo install --path .
# FROM debian:buster-slim
# RUN apt-get update & apt-get install -y extra-runtime-dependencies & rm -rf /var/lib/apt/lists/*
# COPY --from=builder /usr/local/cargo/bin/myapp /usr/local/bin/myapp
# CMD ["myapp"]