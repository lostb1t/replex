run-tests:
	cargo test -- --nocapture

build:
	cargo build --release

docker-build:
	docker build -t ghcr.io/sarendsen/replex:nginx-test --target nginx . -f docker/Dockerfile

# docker-build:
# 	docker buildx build -t ghcr.io/sarendsen/replexnonginx:latest --platform linux/amd64 --target replex . -f docker/Dockerfile

# docker-run:
# 	docker run --rm -it -p 3001:80 -e REPLEX_HOST="http://46.4.30.217:42405" ghcr.io/sarendsen/replex:test

docker-run:
	docker run --rm -it -p 80:80 \
		-e REPLEX_REDIRECT_STREAMS=1 -e RUST_LOG="info,replex=info" -e REPLEX_TMDB_API_KEY=0d73e0cb91f39e670b0efa6913afbd58 \
		-e REPLEX_HOST="https://46-4-30-217.01b0839de64b49138531cab1bf32f7c2.plex.direct:42405" ghcr.io/sarendsen/replex:nginx-test

# push-docker:
# 	docker push ghcr.io/sarendsen/replex:latest

# release: build-docker push-docker

# run:
# 	REPLEX_HOST=http://46.4.30.217:42405 REPLEX_NEWRELIC_API_KEY="NRAK-URAH851PRF8TQX5U69OOPDY4T8U" RUST_LOG="info,replex=debug" cargo run

# run:
# 	REPLEX_CACHE_TTL=0 REPLEX_HOST=https://46-4-30-217.01b0839de64b49138531cab1bf32f7c2.plex.direct:42405 REPLEX_NEWRELIC_API_KEY="eu01xx2d3c6a5e537373a8f8b52003b3FFFFNRAL" RUST_LOG="debug,replex=debug" cargo watch -x run


run:
	REPLEX_VIDEO_TRANSCODE_FALLBACK_FOR="4k" REPLEX_AUTO_SELECT_VERSION=1 REPLEX_FORCE_MAXIMUM_QUALITY=1 REPLEX_CACHE_ROWS=0 REPLEX_HERO_ROWS="movies.recent,movie.recentlyadded,continueWatching" REPLEX_PORT=80 REPLEX_INCLUDE_WATCHED=1 REPLEX_REDIRECT_STREAMS=0 REPLEX_DISABLE_RELATED=0 REPLEX_DISABLE_LEAF_COUNT=0 REPLEX_DISABLE_USER_STATE=0 REPLEX_ENABLE_CONSOLE=0 REPLEX_CACHE_TTL=0 REPLEX_HOST="https://46-4-30-217.01b0839de64b49138531cab1bf32f7c2.plex.direct:42405/" RUST_LOG="info,replex=debug" cargo watch -x run


# run:
# 	REPLEX_ENABLE_CONSOLE=0 REPLEX_CACHE_TTL=0 REPLEX_HOST=https://46-4-30-217.01b0839de64b49138531cab1bf32f7c2.plex.direct:42405 RUST_LOG="info" cargo run


fix:
	cargo fix

# cargo-update:
# 	cargo install-update -a