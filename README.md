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
   * [Running Tests](#running-tests)
* [Usage](#usage)
   * [Basic Usage](#basic-usage)
      * [Basic Example](#basic-example)
         * [Tabular Input Data](#tabular-input-data)
         * [Awk Program](#awk-program)
         * [Further Reading](#further-reading)
* [Why the Funny Name?](#why-the-funny-name)
* [Architecture](#architecture)
   * ["Library" Files](#library-files)
      * [The Parser](#the-parser)
      * [The AST Interpreter](#the-ast-interpreter)
   * ["Binary" Files](#binary-files)
* [Difference From Awk: Regular Expressions](#difference-from-awk-regular-expressions)
   * [Example](#example)
* [Implementation Difficulties](#implementation-difficulties)
   * [Parsing String Concatenation vs. Function Calls](#parsing-string-concatenation-vs-function-calls)
      * [String Concatenation](#string-concatenation)
      * [Function Calls](#function-calls)
      * [The Problem](#the-problem)
      * [A Solution](#a-solution)
* [Possible Future Improvements](#possible-future-improvements)
   * [Error Messages](#error-messages)
   * [Associative Arrays](#associative-arrays)
   * [Built-in Functions](#built-in-functions)
   * [CSV Parsing and Separator Strings](#csv-parsing-and-separator-strings)
<!--te-->

# Build Requirements

- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- Unix tools like `make`

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

## Running Tests

To run the test suite for `chawk`, you should be able to use
```bash
make test
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
  online](https://pubs.opengroup.org/onlinepubs/009604499/utilities/awk.html)
  or via man page (`man p awk`).

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

We'll touch on some of these items later in the document. But for a more
extensive look at the features provided by `awk`[^grymoire_note], check out
the [awk grymoire](https://www.grymoire.com/Unix/Awk.html).

[^grymoire_note]: Since `chawk` does not support all of the features provided
  by `awk`, large portions of the awk grymoire will not apply to `chawk`.

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

### The Parser

The parser itself has two broad phases:
1. Conversion from a string to a concrete syntax tree (CST).
2. Conversion from the concrete syntax tree to an abstract syntax tree (AST).

The parser's functionality is implemented in two key files:
1. `src/chawk.pest` provides a formal grammar, compatible with
   [pest](https://github.com/pest-parser/pest).
2. `src/parser.rs` implements both phases of the parser:
    1. It automatically generates the implementation for the string-to-CST
       phase, using a macro from the [pest](https://github.com/pest-parser/pest)
       library.
    2. It then manually implements the CST-to-AST phase, using recursion over
       the CST.

### The AST Interpreter

After parsing the input string, we perform recursion over the resulting
abstract syntax tree to interpret the program.

This AST interpretation functionality is implemented in `src/interpreter.rs`.

`src/interpreter.rs` also provides a convenience function (`Interpreter::run`)
which both parses and interprets a given `chawk` program.

## "Binary" Files

This project produces 4 different executable binaries:
1. The actual `chawk` binary — source code in `src/bin/main.rs`.
2. The `parser` binary, which parses program text into an abstract syntax tree — source
   code in `src/bin/parser.rs`.
3. The `raw-parser`, which parses program text into a concrete syntax tree — source
   code in `src/bin/raw_parser.rs`.
4. The `tester` binary, which runs the test suite — source code in
   `src/bin/tester.rs`.

The `parser`, `raw-parser`, and `tester` binaries are primarily used for
debugging and testing. From a "normal user"'s point of view, only the actual
`chawk` binary is useful.

After building the project, these binaries can be found in the `target/debug/`
directory by default.

# Difference From Awk: Regular Expressions

According to the POSIX standard[^awk_spec], awk should use "[extended regular
expressions](https://en.wikipedia.org/wiki/Regular_expression#Standards),"
which adhere to a specific regular expression syntax specified by POSIX.

Most programming languages use regular expression syntaxes which differ from
the extended regular expressions seen in awk. These syntaxes are largely
influenced by the regular expressions in the aptly-named [Perl-Compatible
Regular
Expression](https://en.wikipedia.org/wiki/Perl_Compatible_Regular_Expressions)
library. Despite its name, this library is not currently 1-to-1 compatible with
[Perl](https://en.wikipedia.org/wiki/Perl)'s default regular expressions,
though it is heavily influenced by them.

`chawk` in particular uses a Rust library known simply as
[regex](https://github.com/rust-lang/regex) for its regular expressions. This
library is modeled after Google's regular expression library for C++, known as
[RE2](https://github.com/google/re2). RE2, in turn, accepts roughly a subset
of the PCRE syntax.

## Example

The following lines will show one of the differences between `awk`'s extended
regular expression syntax and `chawk`'s regular expressions.

```bash
awk '/[[:digit:]]/' test/temperature.txt
```

```bash
./chawk '/\d/' test/temperature.txt
```

Both of these lines should show the same output, printing lines from
`test/temperature.txt` which contain at least a single digit in
them.[^like_grep]

[^like_grep]: These uses of `awk` and `chawk` highly resemble how you might
  use `grep`. In fact, if all you wanted to do was find lines with at least a
  single digit, you probably should use `grep`. We're only using `awk` and
  `chawk` here to illustrate the differences between their regular expression
  syntax.

In both cases, all we're really doing is creating a regular expression that
searches that matches on a single digit. This is easily achievable with a
[character
class](https://en.wikipedia.org/wiki/Regular_expression#Character_classes),
specifically the one which matches on digits.

However, the exact syntax for a digit character class differs between
POSIX extended regular expression (ERE) syntax and `chawk`'s PCRE-influenced syntax.

With ERE syntax, the character class for a digit is `[:digit:]`. However, to
use a character class, we must place it in a [bracket
expression](https://en.wikipedia.org/wiki/Regular_expression#POSIX_basic_and_extended)
first. To do this, we just need to place `[:digit:]` in brackets, resulting in
`[[:digit]]`.

With `chawk`'s PCRE-influenced syntax, the character class for a digit is
`\d`, and we don't need a bracket expression to access the character class,
resulting in just `\d`.

# Implementation Difficulties

By far, the most annoying aspect of this project was parsing the program input
in the first place. Approximately half of all time spent developing the
interpreter was spent on the parser.

## Parsing String Concatenation vs. Function Calls

In particular, it was especially annoying to disambiguate string concatenation
from function calls.

### String Concatenation

In `awk` (and `chawk`), we can conveniently use string concatenation by
placing two values next to each other.

For example,
```bash
awk '{ print "The first column is: " $1 }' test/temperature.txt
```
will concatenate the string "The first column is: " with the value of the
first column and then print out the result, doing so for each row in
`test/temperature.txt`.

### Function Calls

Somewhat similarly, we can use C-style syntax to call functions.

For example, let's use the test file `test/function.awk`, which has the
following contents:

```awk
function say_hello(name) {
  print "Hello, " name "!"
}

END {
  say_hello("Chris")
}
```

We can execute this file with the following invocation of `awk`:
```bash
awk -f test/function.awk test/short_data.txt
```

It doesn't really matter what data file we use, as the source `awk`
program will always have the same result: printing out "Hello, Chris!" via a
function call to `say_hello`.

### The Problem

<!-- With the definition of `say_hello` given above, what should be the result of -->
<!-- the following expression in `awk`? -->

With this syntax in mind, what should be the result of the following
expression in `awk`?

```awk
say_hello ("Chris")
```

Since we can use parentheses to evaluate an expression early, this looks like
a potential example of string concatenation, combining the string values of a
`say_hello` variable and the result of the parenthesized expression
`("Chris")`.

However, this could also be a function call to `say_hello` with the argument
`"Chris"`. If we were using the definition of the `say_hello` function shown
above, this would seem like the correct choice.

A static type-checker could resolve this conflict, as it would know at
compile-time whether or not `say_hello` is a variable (in which case it would
use string concatenation) or a function (in which case it would use a function
call). However, `awk` (and `chawk`) does not have a static type checker, so
this solution is unavailable.

### A Solution

I spent an uncomfortable amount of time trying to figure out how to resolve
this conflict. Maybe it's specified in the POSIX standard for `awk` (and I
just missed it), but I was unable to find any clear directions on how to
disambiguate these two language operations.

Eventually, after reading part of the GNU `awk` manual, I found a viable
solution. According to its section regarding [function
calls](https://www.gnu.org/software/gawk/manual/html_node/Function-Calls.html),
there should be **no whitespace** between the function name and its opening
parenthesis. If there is whitespace between the name and
the opening parenthesis, then the operation in question is actually string
concatenation, not a function call.[^function_whitespace]

[^function_whitespace]: Technically, there should only be no whitespace
  between a *user-defined* function name and its opening parenthesis (at least
  in GNU `awk`). The POSIX standard specifies a number of built-in functions
  for `awk`, and the GNU implementation of `awk` allows for whitespace between
  the names of these built-in functions and their opening parentheses. `chawk`
  does not currently implement any built-in functions for the sake of
  simplicity, so it doesn't need to make this distinction.

Thus the `awk` expression
```awk
say_hello ("Chris")
```
should be an example of *string concatenation*, due to the whitespace
between the function name and the opening parentheses.

I won't dive into the implementation details necessary to make this
distinction in the parser, as it relies on the specific use of the features
provided by [pest](https://pest.rs/), such as explicit whitespace and compound
atomic rules. However, I want to reiterate that this was an especially
annoying case to disambiguate while parsing.

# Possible Future Improvements

Though `chawk` is usable for some of the use cases of `awk`, there are many
ways to improve it.

## Error Messages

Currently the error messages are very poorly formatted.

For example, this line will result in a parsing error, as it has an
unnecessary `}` at the end:

```bash
./chawk '{ print $0 } }' test/temperature.txt
```

The resulting error message will look like:

```awk
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: Error { variant: ParsingError { positives: [EOI, TopItem], negatives: [] }, location: Pos(13), line_col: Pos((1, 14)), path: None, line: "{ print $0 } }", continued_line: None }', src/interpreter.rs:25:46
```

This error does contain some useful information, but it's formatted incredibly
poorly. For example, you can see that the parse error occurs on line 1, column
14, but you have to know to look at the `line_col` field.

We could benefit significantly from formatting the error messages in a more
readable way, as well as possibly showing the locations of errors more
visually. To achieve this, the [ariadne](https://github.com/zesterer/ariadne)
library could be useful.

## Associative Arrays

As per the POSIX standard, `awk` provides associative arrays, similar to
dictionaries in Python.

Let's look at a trivial example:

```awk
END {
  arr["a key"] = "a value"

  arr[38.3] = "another value"

  for (key in arr) {
    print key " corresponds to " arr[key]
  }
}
```

This `awk` creates an associative array and adds two key-value pairs to the
array. Specifically, it associates `"a key"` with `"a value"`, and it
associates `38.3` with `"another value"`. Then it prints out the values for
each of these pairs, using special `for` loop syntax which iterates through
all of the keys in an associative array.

Notably, associative arrays do not need to be explicitly declared or
initialized, with their default value being an empty array.

`chawk` does not implement associative arrays, so this example would not work
with it. However, adding these feature to the language could be useful. For
starters, an implementation in `chawk` could probably leverage the
[`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html)
data structure provided by the Rust standard library.

For a more detailed overview, you can check out the [associative arrays
section](https://www.grymoire.com/Unix/Awk.html#uh-22) of the awk grymoire.

## Built-in Functions

As mentioned in one of the footnotes from earlier,[^function_whitespace]
POSIX `awk` provides a number of built-in functions.

For example, the built-in `sub` function allows you to replace part of a
string with some other text.

```awk
END {
  str = "Good morning!"

  sub("morning", "night", str)

  print str
}
```

This example `awk` program will print out `Good night!` when run with any input
data.

The `sub` function is also capable of matching on regular expressions and
acting implicitly on the current line of input:

```awk
{
  sub(/C$/, "Celsius")
  sub(/F$/, "Fahrenheit")

  print $0
}
```

To see this `awk` program in action, you can store it in a file called
`sub_more.awk` and then run the following line in a unix shell:

```bash
awk -f sub_more.awk test/temperature.txt
```

This `awk` program replaces the `C` and `F` characters at the end of each line
with the text `Celsius` and `Fahrenheit`, respectively.

Additionally, `awk` provides other built-in functions capable of finding the
length of a string or splitting up a string based on a separator. For more
information on these built-in functions, you can check out their
[corresponding sections](https://www.grymoire.com/Unix/Awk.html#uh-41) in the
awk grymoire.

`chawk` does not currently provide any built-in functions, and implementing
some of the ones found in `awk` could be useful.

## CSV Parsing and Separator Strings

You may have noticed some resemblance between the whitespace-separated tabular
data expected by `awk` and the CSV (column-separated values) format commonly
used to store data.

POSIX `awk` provides a command-line flag which allows you to change the
separator used to locate the columns of the input data. You can achieve
something similar to parsing CSVs with something like the following
invocation:

```bash
awk -F, -f prog_file.awk input_data.csv
```

**Note**: `chawk` does not currently implement this command-line flag, and
adding it to the language could be useful.

However, using this command-line flag fails handle to certain edge cases. If a
field itself contains a comma, then the field should be wrapped in double
quotes, assuming the file is compliant with [RFC
4180](https://datatracker.ietf.org/doc/html/rfc4180).

To handle cases like this, it could be useful to implement an actual CSV
parser, which could then be used in place of the default parser via its own
command-line flag.

For an example of an `awk`-like language which has built-in support for
parsing CSVs, you can check out [frawk](https://github.com/ezrosent/frawk).
frawk also has other fascinating features, such as a JIT compiler which
produces LLVM-IR and is supported by static type inference. Features which at
this level of sophistication are probably out of scope for a project like
`chawk`, but they are fascinating directions to take a language like `awk`.

<!-- vim: shiftwidth=4
    -->
