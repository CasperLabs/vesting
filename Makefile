PINNED_TOOLCHAIN := $(shell cat rust-toolchain)

prepare:
	rustup target add wasm32-unknown-unknown
	rustup component add clippy --toolchain ${PINNED_TOOLCHAIN}
	rustup component add rustfmt --toolchain ${PINNED_TOOLCHAIN}

build-contract:
	cargo build --release -p contract --target wasm32-unknown-unknown

test-only:
	cargo test -p tests

copy-wasm-file-to-test:
	mkdir -p tests/wasm
	cp target/wasm32-unknown-unknown/release/contract.wasm tests/wasm/
	cp target/wasm32-unknown-unknown/release/deposit.wasm tests/wasm/

test: build-contract copy-wasm-file-to-test test-only

clippy:
	cargo clippy --all-targets --all -- -D warnings -A renamed_and_removed_lints

check-lint: clippy
	cargo fmt --all -- --check

lint: clippy
	cargo fmt --all
	
clean:
	cargo clean
	rm -rf tests/wasm/*.wasm
	rmdir tests/wasm
