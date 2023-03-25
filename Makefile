## test : Run project tests
test: Cargo.toml src/*
	cargo test

## build : Build project executable (in debug mode)
build: target/debug/sleep-on-lan
target/debug/sleep-on-lan: Cargo.toml src/*
	cargo build

## release : Build project executable (in release mode)
release: target/release/sleep-on-lan
target/release/sleep-on-lan: Cargo.toml src/*
	cargo build --release

## package : Package project executable and other derivatives into archive
package: sleep-on-lan.tar.gz
sleep-on-lan.tar.gz: release dist/*
	tar --create --gzip --verbose --file sleep-on-lan.tar.gz \
		--directory dist/ $(shell ls dist) \
		--directory ../target/release/ sleep-on-lan

## clean : Clean project artifacts
clean:
	rm -r target/ sleep-on-lan.tar.gz

## help : Print this help
help:
	@grep -E "^##" Makefile | cut --characters 4-

.PHONY: clean help