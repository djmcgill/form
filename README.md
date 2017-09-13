A library for splitting apart a large file with multiple modules into the idiomatic rust directory structure, intended for use with svd2rust.
First argument is the directory that you want the lib.rs file in. The file contents are received via stdin.
Creates a lib.rs as well as a subdirectory structure in the target directory.
It's advised (but not necessary) to use scalafmt afterwards.

Does not create the cargo project or the cargo manifest file.

TODO:
    - proper command line arguments
    - better error handling
