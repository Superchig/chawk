all: chawk

chawk: target/debug/chawk
	ln -sf target/debug/chawk

# Our sentinel value for all cargo binaries
target/debug/chawk: src/** Cargo.toml
	cargo build

test: target/debug/chawk test/**
	cargo run --bin tester

clean:
	cargo clean
	rm -f chawk
