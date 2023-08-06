FROM rust:1.71-bookworm as builder

WORKDIR /app/src
RUN USER=root cargo new --bin replex
COPY Cargo.toml Cargo.lock ./replex/

WORKDIR /app/src/replex
RUN cargo build --release

COPY ./ ./
RUN cargo build --release

FROM nginx:bookworm as nginx
RUN apt update \
    && apt install -y openssl ca-certificates \
    && apt clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*
COPY --from=builder /app/src/replex/target/release/replex /app/

WORKDIR /app

COPY docker/default.conf.template /etc/nginx/templates/
RUN echo "daemon off;" >> /etc/nginx/nginx.conf
# RUN rm /etc/nginx/conf.d/default.conf
COPY docker/start.sh start.sh
STOPSIGNAL SIGQUIT
CMD ./start.sh