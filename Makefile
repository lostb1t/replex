runtest:
	cargo test -- --nocapture

build:
	cargo build --release

build-docker:
	docker build -t ghcr.io/sarendsen/replex:latest .

push-docker:
	docker push ghcr.io/sarendsen/replex:latest

release: build-docker push-docker
