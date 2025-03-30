# `ddc` - Dev Dir Cleaner

A tiny tool to keep a developer's disk tidy.

Regular development across multiple projects often results in directories scattered all over the disk. This tool helps identify these directories and clean them up when needed.

## Installation

This project is in the preview stage and is not yet published through standard channels. You can install it from source.

Since this project requires iterating over BTree nodes, it needs the `nightly` version of Rust. Make sure to install the appropriate toolchain:

```shell
rustup toolchain install nightly
```

Use the standard compilation command to build the project:

```shell
cargo +nightly build --release
```

Then, copy the resulting binary to a preferred location covered by `$PATH`.

Alternatively, use the `cargo install` command:

```shell
cargo +nightly install --path .
```

## Usage

The basic usage requires simply running the command:

```shell
ddc
```

By default, this executes the `analyze` subcommand:

```shell
ddc analyze
```

A configuration file is required. To generate an example configuration file, run:

```shell
ddc generate-config
```

Then, follow the instructions in the file to set it up.

## Configuration

`ddc` identifies well-known paths used by popular tools. However, it cannot automatically determine where you store your projects. That’s why a configuration file is necessary.

### Minimal Configuration

As a minimum, you need to specify the path to your main development directory. For example:

```toml
[[paths]]
path = "projects/"
discovery = true
```

### Record Structure

Each record follows this structure:

```toml
[[paths]]
name = "Custom virtualenv location"
path = ".virtualenvs/"
discovery = false
language = "python"
```

Where:

- `name` *(optional)* – A name for the path. Displayed in the tool’s output if `discovery` is set to `false`.
- `path` – The directory path. All paths are assumed to be relative to your home directory.
- `discovery` – If `true`, auto-discovery is enabled. This means `ddc` will try to identify paths such as virtual environments or Rust build directories.
- `language` *(optional)* – The associated programming language. This setting is only useful when `discovery` is set to `false` and improves visual representation.

### Default Discovery Definitions

To see the paths that `ddc` scans by default, run:

```shell
ddc analyze --show-definitions
```
