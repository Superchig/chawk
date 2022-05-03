all: chawk

chawk: target/debug/chawk
	ln -sf target/debug/chawk

# Our sentinel value for all cargo binaries
target/debug/chawk: $(shell find src -type f) Cargo.toml
	cargo build

# https://www.gnu.org/software/make/manual/html_node/Force-Targets.html
FORCE:

test: target/debug/chawk $(shell find test -type f) FORCE
	cargo run --bin tester

clean:
	cargo clean
	rm -f chawk
