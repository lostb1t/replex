run-tests:
	cargo test -- --nocapture

build:
	cargo build --release

docker-build:
	docker build -t ghcr.io/sarendsen/replex:latest --target replex . -f docker/Dockerfile

# docker-build:
# 	docker buildx build -t ghcr.io/sarendsen/replexnonginx:latest --platform linux/amd64 --target replex . -f docker/Dockerfile

docker-run:
	docker run --rm -it -p 3001:80 -e REPLEX_HOST="http://46.4.30.217:42405" ghcr.io/sarendsen/replex

# push-docker:
# 	docker push ghcr.io/sarendsen/replex:latest

# release: build-docker push-docker

# run:
# 	REPLEX_HOST=http://46.4.30.217:42405 REPLEX_NEWRELIC_API_KEY="NRAK-URAH851PRF8TQX5U69OOPDY4T8U" RUST_LOG="info,replex=debug" cargo run

run:
	REPLEX_CACHE_TTL=0 REPLEX_HOST=http://46.4.30.217:42405 REPLEX_NEWRELIC_API_KEY="eu01xx2d3c6a5e537373a8f8b52003b3FFFFNRAL" RUST_LOG="debug,replex=debug" cargo watch -x run


# run:
# 	REPLEX_CACHE_TTL=0 REPLEX_HOST=http://46.4.30.217:42405 RUST_LOG="debug,replex=info" cargo watch -x run


fix:
	cargo fix

# cargo-update:
# 	cargo install-update -a