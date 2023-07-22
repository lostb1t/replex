runtest:
	cargo test -- --nocapture

build:
	cargo build --release

docker-build:
	docker build -t ghcr.io/sarendsen/replex-test:latest --target nginx .

docker-run:
	docker run --rm -it -p 80:80 -e REPLEX_HOST="http://46.4.30.217:42405" ghcr.io/sarendsen/replex-test:latest

# push-docker:
# 	docker push ghcr.io/sarendsen/replex:latest

# release: build-docker push-docker

fix:
	cargo fix