runtest:
	cargo test -- --nocapture

build:
	cargo build --release

build-docker:
	docker build -t ghcr.io/sarendsen/httplex:latest .

push-docker:
	docker push ghcr.io/sarendsen/httplex:latest

release: build-docker push-docker
