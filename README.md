# Form
![crates.io badge](https://img.shields.io/crates/v/form.svg)

A library for splitting apart a large file with multiple modules into the idiomatic rust directory structure, intended for use with svd2rust.
Creates a lib.rs as well as a subdirectory structure in the target directory. It does NOT create the cargo project or the cargo manifest file.

It's advised (but not necessary) to use rustfmt afterwards.
## Usage:
Arguments:
```
    -i, --input FILE    OPTIONAL: input file to read, defaults to stdin
    -o, --outdir DIR    set output directory
    -h, --help          print this help menu
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
