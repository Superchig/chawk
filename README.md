# Chawk

An implementation of a subset of [awk](https://en.wikipedia.org/wiki/AWK).

[`awk`](https://en.wikipedia.org/wiki/AWK) is a domain-specific (but
turing-complete) language aimed at text processing and textual data
manipulation, commonly included as a unix command-line utility.

`chawk` implements a subset of `awk`'s language and command-line interface,
extending this subset of the language by adding local variables.

## Table of Contents

<!--ts-->
* [Chawk](#chawk)
   * [Table of Contents](#table-of-contents)
* [Build Requirements](#build-requirements)
* [Build Instructions](#build-instructions)
   * [With Make](#with-make)
      * [Build Errors](#build-errors)
* [Usage](#usage)
   * [Basic Usage](#basic-usage)
      * [Basic Example](#basic-example)
         * [Tabular Input Data](#tabular-input-data)
         * [Awk Program](#awk-program)
         * [Further Reading](#further-reading)
* [Differences From Chawk](#differences-from-chawk)
   * [Regular Expressions](#regular-expressions)
* [Why the Funny Name?](#why-the-funny-name)
<!--te-->

# Build Requirements

- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

# Build Instructions

## With Make

You should be able to build the project with:
```bash
make
```

If you see a build error like this:
```
error: package `chawk v0.1.0 (/home/chiggie/projects/chawk)` cannot be built because it requires rustc 1.59 or newer, while the currently active rustc version is 1.57.0
```
then you have an older version of the rust compiler than necessary.

To solve this, you can update your rust compiler version with:
```
rustup update
```

Running `make` should automatically symlink the `chawk` binary to your current
directory, so you should be able to run `chawk` with:
```bash
./chawk
```

### Build Errors

# Usage

`chawk` implements a commonly used subset of `awk`'s command-line flags.

## Basic Usage

Like `awk`, the `chawk` program has two primary inputs:
1. an awk program, and
2. tabular input data.

In its simplest usage, `chawk` takes the awk program directly as a
command-line argument and reads the tabular input data in from a file,
specified on the command line.

### Basic Example

Consider this example:
```bash
./chawk '{ print $1 }' test/temperature.txt
```

If you run the above line in a unix shell, you should see this output:
```
temp
26.1
78.1
23.1
25.7
76.3
77.3
24.2
79.3
27.9
75.1
25.9
79.0
```

Why is this? Let's break down this line to see why.

In this example, the first argument `'{ print $1 }'` is an awk program, and
the second argument `test/temperature.txt` is the file containing the input
data. We'll examine these two inputs in more detail below, starting with the
tabular input data.

#### Tabular Input Data

The `test/temperature.txt` file looks like this:
```
temp	unit
26.1	C
78.1	F
23.1	C
25.7	C
76.3	F
77.3	F
24.2	C
79.3	F
27.9	C
75.1	F
25.9	C
79.0	F
```

As you can see, it describes a series of temperatures with their corresponding
units, resembling a table.

According to the POSIX awk specification[^awk_spec], this is
best described as a "sequence of records." Each temperature would be a record,
with the numerical value of the temperature being one field and the unit
(described as C or F) being another field.

[^awk_spec]: The POSIX awk specification is [available
  online](https://pubs.opengroup.org/onlinepubs/9699919799/) or via man page
  (`man p awk`).

#### Awk Program

In short, the awk program (which was just `{ print $1 }`) prints the first
column of every record.

Specifically, the `print $1` statement will print the first column of the
"current record."

Since that statement is within the curly braces (`{` and `}`), it's executed
for each record.

Thus, When we run this awk program on the input data found in
`test/temperature.txt`, we obtain the output seen previously.

**Exercise**: The `chawk` program, like `awk`, has built-in support for
regular expressions. Try running `./chawk '/C$/ { print $1 }'
test/temperature.txt`. Can you figure out how the regular expression `/C$/` is
being used?

**Exercise**: The `chawk` program, like `awk`, can read its program source
code in from a file, which is useful with larger programs. As a trivial
example, try writing the awk program `{ print $1 }` in a file named `prog.awk`
and then running `./chawk -f prog.awk test/temperature.txt` in a unix shell.

**Exercise**: The `chawk` program, like `awk`, can read its tabular input data
in from standard input. As a trivial example, try running `cat
test/temperature.txt | ./chawk '{ print $1 }'`.

#### Further Reading

`awk` (and `chawk`) have many of the features you might expect, such as:
- C-style flow control (`if`/`while`/`for`)
- Function calls and definitions.
- String concatenation
- Floating point arithmetic

For a more extensive look at the features provided by `awk`[^grymoire_note],
check out the [awk grymoire](https://www.grymoire.com/Unix/Awk.html).

[^grymoire_note]: Since `chawk` does not support all of the features provided
  by `awk`, large portions of the awk grymoire will not apply to `chawk`.

# Architecture

Source files for `chawk` are split into two broad categories:
1. "Library" files, which implement the underlying functionality of `chawk`,
   and
2. "Binary" files, which hook up command-line flags to the functionality from
   "library" files. These files *produce* binary executables when compiled.
   They are not themselves binary executables.

These two categories of files are located in different directories:
1. "Library" files can be found directly in the first level of `src/`.
   For example, `src/interpreter.rs` is a "library" file.
2. "Binary" files are found in `src/bin`. For example, `src/bin/main.rs` is a
   "binary" file.

## "Library" Files

As with many programming languages, `chawk` is split into a parser and an
actual interpreter.

The parser itself has two broad phases:
1. Conversion from a string to a concrete syntax tree.
2. Conversion from the concrete syntax tree to an abstract syntax tree.

Reflecting these two phases, the parser's functionality is implemented by two
files:
1. `src/chawk.pest`, which provides a formal grammar.
2. `src/parser.rs`, which uses a Rust macro to generate a...

## "Binary" Files

TODO(Chris): Finish this part of the report.

# Differences From Chawk

## Regular Expressions

According to the POSIX standard[^awk_spec], awk should use "[extended regular
expressions](https://en.wikipedia.org/wiki/Regular_expression#Standards),"
which adhere to a specific regular expression syntax specified by POSIX.

Most programming languages use regular expression syntaxes which differ from
the extended regular expressions seen in awk. These syntaxes are largely
influenced by that of the [Perl](https://en.wikipedia.org/wiki/Perl)
programming language, though these syntaxes are usually not 1-to-1 compatible
with Perl's.

TODO: Finish this part of the report.

# Why the Funny Name?

Fun fact: the original `awk` is [named after the three people who created
it](https://en.wikipedia.org/wiki/AWK): Aho, Weinberger, and Kernighan. Plus,
it resembles the bird [auk](https://en.wikipedia.org/wiki/Auk), which appears
on the [cover](https://en.wikipedia.org/wiki/The_AWK_Programming_Language) of
the book describing the language from its creators.

`chawk` is similarly named after its creator: me. Obviously, I'm nowhere near
to Aho, Weinberger, or Kernighan in terms of my accomplishments or
understanding of computer science, so maybe it's a little conceited to use the
first two letters of my name. After all, the authors of awk only used one
letter from each of their names.
