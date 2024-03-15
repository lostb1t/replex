run-tests:
	cargo test -- --nocapture

build:
	cargo build --release

docker-build:
	docker build -t ghcr.io/lostb1t/replex:nginx-test --target nginx . -f docker/Dockerfile

docker-run:
	docker run --rm -it -p 80:80 \
		-e REPLEX_REDIRECT_STREAMS=1 -e RUST_LOG="info,replex=info" \
		ghcr.io/lostb1t/replex:nginx-test

# push-docker:
# 	docker push ghcr.io/lostb1t/replex:latest

# release: build-docker push-docker

run:
	REPLEX_HOST="https://16191.trae.dgjei.com" \
	REPLEX_DISABLE_CONTINUE_WATCHING=1 \
	REPLEX_REDIRECT_STREAMS=1 \
	REPLEX_PRIORITY_HUBS="Continue watching,Jump back in" \
	REPLEX_CACHE_ROWS=0 \
	REPLEX_CACHE_TTL=0 \
	REPLEX_PORT=80 \
	RUST_LOG="info" \
	cargo watch -x run

fix:
	cargo fix

# cargo-update:
# 	cargo install-update -a

# REPLEX_VIDEO_TRANSCODE_FALLBACK_FOR="4k" \
# REPLEX_AUTO_SELECT_VERSION=0 \
# REPLEX_FORCE_MAXIMUM_QUALITY=1 \
# REPLEX_HERO_ROWS="home.movies.recent,movies.recent,movie.recentlyadded,movie.topunwatched,movie.recentlyviewed,hub.movie.recentlyreleased,home.television.recent,tv.inprogress,tv.recentlyaired" \
# REPLEX_INCLUDE_WATCHED=0 \
#	REPLEX_DISABLE_RELATED=0 \
#	REPLEX_DISABLE_LEAF_COUNT=0 \
#	REPLEX_DISABLE_USER_STATE=0 \
#	REPLEX_ENABLE_CONSOLE=0 \
#	REPLEX_CACHE_TTL=0 \
# RUST_LOG="info,replex=debug" \