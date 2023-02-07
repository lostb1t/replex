runtest:
	cargo test -- --nocapture

build:
	cargo build --release

build-docker:
	docker build -t ghcr.io/sarendsen/plex_proxy:latest .

push-docker:
	docker push ghcr.io/sarendsen/plex_proxy:latest