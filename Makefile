runtest:
	cargo test -- --nocapture

build:
	cargo build --release

docker-build:
	docker build -t ghcr.io/sarendsen/replex-test:latest --target nginx . -f docker/Dockerfile

docker-run:
	docker run --rm -it -p 3001:80 -e REPLEX_HOST="http://46.4.30.217:42405" ghcr.io/sarendsen/replex-test:latest

# push-docker:
# 	docker push ghcr.io/sarendsen/replex:latest

# release: build-docker push-docker

run:
	REPLEX_HOST=http://46.4.30.217:42405 cargo run

fix:
	cargo fix