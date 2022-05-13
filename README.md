# Chawk

An implementation of a subset of [awk](https://en.wikipedia.org/wiki/AWK).

[`awk`](https://en.wikipedia.org/wiki/AWK) is a domain-specific (but
turing-complete) language aimed at text processing and textual data
manipulation, commonly included as a unix command-line utility.

`chawk` implements a subset of `awk`'s language and command-line interface,
extending this subset of the language by adding local variables.

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

According to the POSIX awk specification[^awk spec], this is
best described as a "sequence of records." Each temperature would be a record,
with the numerical value of the temperature being one field and the unit
(described as C or F) being another field.

[^awk spec]: The POSIX awk specification is [available
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

For a more extensive look at the features provided by `awk`[^grymoire note],
check out the [awk grymoire](https://www.grymoire.com/Unix/Awk.html).

[^grymoire note]: Since `chawk` does not support all of the features provided
  by `awk`, large portions of the awk grymoire will not apply to `chawk`.

# Differences From Chawk

## Regular Expressions

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
