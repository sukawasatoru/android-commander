SHELL = /bin/bash
.SUFFIXES:

ifeq ($(OS),Windows_NT)
    MV = REN
endif

all: ../server/app/build/outputs/android-commander-server
	cargo build --release

../server/app/build/outputs/android-commander-server:
	$(MAKE) -C ../server

.PHONY: debug
debug: ../server/app/build/outputs/android-commander-server
	cargo build

.PHONY: lint
lint: fmt ../server/app/build/outputs/android-commander-server
	cargo clippy

.PHONY: clean
clean:
	cargo clean

.PHONY: distclean
distclean: clean
	$(MAKE) -C ../server clean

.PHONY: prepare
prepare:
	rustup component add clippy rustfmt

.PHONY: fmt
fmt:
	cargo fmt -v -- --check
