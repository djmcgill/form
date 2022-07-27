[![crates.io](https://img.shields.io/crates/d/form.svg)](https://crates.io/crates/form)
[![crates.io](https://img.shields.io/crates/v/form.svg)](https://crates.io/crates/form)
[![CircleCI](https://circleci.com/gh/djmcgill/form/tree/main.svg?style=svg)](https://circleci.com/gh/djmcgill/form/tree/main)
[![CI](https://github.com/djmcgill/form/workflows/CI/badge.svg?branch=main)](https://github.com/djmcgill/form)

# Form

A library for splitting apart a large file with multiple modules into the idiomatic rust directory structure, intended for use with svd2rust.
Creates a lib.rs as well as a subdirectory structure in the target directory. It does NOT create the cargo project or the cargo manifest file.

It's advised (but not necessary) to use rustfmt afterwards.
## Usage:
Arguments:
```
    -i, --input FILE    OPTIONAL: input file to read, defaults to stdin
    -o, --outdir DIR    set output directory
    -h, --help          print this help menu
    -v, --version       print version information
```


Intended usage (using `svd2rust` 0.12.1 and before):
```bash
svd2rust -i FOO.svd | form -o ~/private/code/form/test/src
```
Usage with `svd2rust` 0.13.0 and later can be found in [svd2rust's documentation](https://docs.rs/svd2rust/).

Advanced usage:
```bash
cargo install form
export RUST_LOG=form=debug
export RUST_BACKTRACE=1
form -i ~/private/code/form/resources/full-lib.rs -o ~/private/code/form/test/src
```
