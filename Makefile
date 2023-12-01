TAG := bitcoinconsensus

DOCKER_RUN := docker run --interactive --rm \
	-v ${PWD}:/bitcoinconsensus \

build-emscripten: builder
	$(DOCKER_RUN) --tty ${TAG} cargo build --target wasm32-unknown-emscripten

build-unknown: builder
	$(DOCKER_RUN) --tty ${TAG} cargo build --target wasm32-unknown-unknown

build-wasi: builder
	$(DOCKER_RUN) --tty ${TAG} cargo build --target wasm32-wasi

wasm-pack: builder
	$(DOCKER_RUN) --tty ${TAG} wasm-pack build --target web

builder:
	docker build --tag ${TAG} \
		--build-arg UID="$(shell id -u)" \
		.

