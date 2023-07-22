runtest:
	cargo test -- --nocapture

build:
	cargo build --release

build-docker:
	docker build -t ghcr.io/sarendsen/replex-test:latest --target replex-nginx .

push-docker:
	docker push ghcr.io/sarendsen/replex:latest

release: build-docker push-docker

clean:
	cargo fix
	cargo machete