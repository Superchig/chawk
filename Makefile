all: chawk

chawk: target/debug/chawk
	ln -sf target/debug/chawk

target/debug/chawk: src/**.rs Cargo.toml
	cargo build

test: target/debug/chawk test/**
	cargo run --bin tester

clean:
	cargo clean
	rm -f chawk
