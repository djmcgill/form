# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

## [v0.12.1]
* Format result files with `prettyplease` (when `-f` option passed), update `syn`

## [v0.11.1] - 2024-01-03
* Bump env_logger to 0.10.1 which changes: `Resolved soundness issue by switching from atty to is-terminal`

## [v0.11.0] - 2023-10-29
* Files and directories whose name excluding extensions is a Windows reserved name are renamed by appending an underscore. The module structure and the directory structure remain intact. This is a BREAKING change to the modules that are produced if they have names that now get underscores appended.

## [v0.10.0] - 2022-07-27
* Update deps including security advisories and switching failure to anyhow

## [v0.9.0] - 2022-07-26
* Add GHA CI

[Unreleased]: https://github.com/djmcgill/form/compare/v0.10.0...HEAD
