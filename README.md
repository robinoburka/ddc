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

### How to Read the Output
The output includes two key columns to help you evaluate the projects on your disk:
- **Last Change**: Shows the most recent modification within the entire directory tree. This helps you identify stale projects.
- **Size**: Displays the total size of the directory and all its subdirectories. This helps you identify large projects that may be consuming unnecessary space.

Both columns use color coding - green, yellow, and red - to convey the following meanings:
- **Last Change**
  - 🟢 Green: The project was updated very recently.
  - 🟡 Yellow: The project was updated in the recent past.
  - 🔴 Red: The project hasn’t been updated in a long time.
- **Size**
  - 🟢 Green: The project takes up little space.
  - 🟡 Yellow: The project occupies a moderate amount of space.
  - 🔴 Red: The project takes up a large amount of space.

This color coding makes it easy to quickly assess your projects and decide which ones to keep or clean up. For example:
- 🟢 Green – 🔴 Red: The project is actively developed and takes up a lot of space. You likely want to keep it.
- 🔴 Red – 🟢 Green: The project is stale but doesn’t use much space. It’s probably fine to keep.
- 🔴 Red – 🔴 Red: The project is both stale and large. Consider cleaning it up.
- Other combinations should be evaluated on a case-by-case basis.

## Configuration

`ddc` identifies well-known paths used by popular tools. However, it cannot automatically determine where you store your projects. That’s why a configuration file is necessary.

### Configuration file structure

You need to specify the path to your main development directory. For example:

```toml
[[paths]]
path = "projects/"
```

Multiple directories are allowed. See the example configuration.

### Default Discovery Definitions

To see the paths that `ddc` scans by default, run:

```shell
ddc show-definitions
```

## Interactive browser

_This feature is currently experimental._

For interactive browsing of the results, you can use:

```shell
ddc browse
```

The interactive browser can be used to inspect results in more interactive manner. It enables you to jump to a reported path. Also, it enables you to jump to a parent of a reported path. It's useful to check the sizes of the project files overall.

See the basic help in the footer, or use `?` to display the full UI help window.
