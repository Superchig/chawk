all: chawk

chawk: target/debug/chawk
	ln -sf target/debug/chawk

# Our sentinel value for all cargo binaries
target/debug/chawk: $(shell find src -type f) Cargo.toml
	cargo build

test: target/debug/chawk $(shell find test -type f)
	cargo run --bin tester

clean:
	cargo clean
	rm -f chawk
