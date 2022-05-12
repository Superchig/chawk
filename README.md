# Chawk

An implementation of a subset of [awk](https://en.wikipedia.org/wiki/AWK).

# Build Requirements

- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

# Build Instructions

## With make

You should be able to build the project with:
```bash
make
```

This will automatically symlink the `chawk` binary to your current directory,
so you should be able to run `chawk` with:
```bash
./chawk
```

# Usage

`chawk` implements a commonly used subset of `awk`'s command-line flags.

## Basic Usage

Like `awk`, the `chawk` program has two primary inputs:
1. an awk program, and
2. tabular input data.

In its simplest usage, `chawk` takes the awk program directly as a
command-line argument and reads the tabular input data in from a file,
specified on the command line.

Let's use this example:
```bash
./chawk '{ print $1 }' test/temperature.txt
```
