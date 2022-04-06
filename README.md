# Chawk

An implementation of a subset of [awk](https://en.wikipedia.org/wiki/AWK).

# Build Requirements

- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

# Steps To Build

You should be able to build the project with:
```bash
cargo build
```

This will create an executable binary for the project at
`target/debug/chawk`.

To run the binary, you can use this binary:
```bash
# This will print the first column in examples/temperature.txt
target/debug/chawk '{ print $1 }' examples/temperature.txt
```

Alternatively, you can symlink the binary to the project root, and then run
that binary:
```bash
ln -s target/debug/chawk
./chawk '{ print $1 }' examples/temperature.txt
```
