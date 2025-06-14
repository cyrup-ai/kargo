## cargo-hakari

`cargo hakari` is a command-line application to manage workspace-hack crates. Use it to speed up local `cargo build` and `cargo check` commands by up to **100x**, and cumulatively by up to **1.7x** or more.

For an explanation of what workspace-hack packages are and how they make your builds faster, see the [`about` module](https://docs.rs/cargo-hakari/latest/cargo_hakari/about).

## Examples

The `cargo-guppy` repository uses a workspace-hack crate managed by `cargo hakari`. [See the generated `Cargo.toml`.](https://github.com/guppy-rs/guppy/blob/main/workspace-hack/Cargo.toml)

## Platform support

- **Unix platforms**: Hakari works and is supported.
- **Windows**: Hakari works and outputs file paths with forward slashes for consistency with Unix. CRLF line endings are not supported in the workspace-hack's `Cargo.toml`. Within Git repositories, `cargo hakari init` automatically does this for you. [Here's how to do it manually.](https://stackoverflow.com/a/10017566) (Pull requests to improve this are welcome.)

## Installation

### Release binaries

Release binaries are available on [GitHub Releases](https://github.com/guppy-rs/guppy/releases?q=cargo-hakari&expanded=true), via [`cargo binstall`](https://github.com/cargo-bins/cargo-binstall):

```
cargo binstall cargo-hakari
```

In GitHub Actions CI, use [`taiki-e/install-action`](https://github.com/taiki-e/install-action), which uses `cargo binstall` under the hood:

```
- name: Install cargo-hakari
  uses: taiki-e/install-action@v2
   with:
     tool: cargo-hakari
```

### Installing from source

To install or update `cargo-hakari`, run:

```
cargo install cargo-hakari --locked
```

If `$HOME/.cargo/bin` is in your `PATH`, the `cargo hakari` command will be available.

## Usage

### Getting started

There are four steps you *must* take for `cargo hakari` to work properly.

#### 1\. Check in your Cargo.lock

For hakari to work correctly, you *must* [add your `Cargo.lock` to version control](https://doc.rust-lang.org/cargo/faq.html#why-do-binaries-have-cargolock-in-version-control-but-not-libraries), even if you don't have any binary crates. This is because patch version bumps in dependencies can add or remove features or even entire transitive dependencies.

#### 2\. Initialize the workspace-hack

Initialize a workspace-hack crate for a workspace at path `my-workspace-hack`:

```
cargo hakari init my-workspace-hack
```

![](https://user-images.githubusercontent.com/180618/135726175-dc00dd0c-68a1-455f-a13d-0dd24f545ca6.png)

#### 3\. Generate the Cargo.toml

Generate or update the contents of a workspace-hack crate:

```
cargo hakari generate
```

#### 4\. Add dependencies to the workspace-hack

Add the workspace-hack crate as a dependency to all other workspace crates:

```
cargo hakari manage-deps
```

![](https://user-images.githubusercontent.com/180618/135725773-c71fc4cd-8b7d-4a8e-b97c-d84a2b3b3662.png)

### Making hakari work well

These are things that are not absolutely necessary to do, but will make `cargo hakari` work better.

#### 1\. Update the hakari config

Open up `.config/hakari.toml`, then:

- uncomment or add commonly-used developer platforms
- read the note about the resolver, and strongly consider [setting `resolver = "2"`](https://blog.rust-lang.org/2021/03/25/Rust-1.51.0.html#cargos-new-feature-resolver) in your workspace's `Cargo.toml`.

Remember to run `cargo hakari generate` after changing the config.

#### 2\. Keep the workspace-hack up-to-date in CI

Run the following commands in CI:

```
cargo hakari generate --diff  # workspace-hack Cargo.toml is up-to-date
cargo hakari manage-deps --dry-run  # all workspace crates depend on workspace-hack
```

If either of these commands exits with a non-zero status, you can choose to fail CI or produce a warning message.

For an example, see [this GitHub action used by `cargo-guppy`](https://github.com/guppy-rs/guppy/blob/main/.github/workflows/hakari.yml).

All `cargo hakari` commands take a `--quiet` option to suppress output, though showing diff output in CI is often useful.

#### 3\. Consider a patch directive

If your workspace is depended on as a Git or path dependency, it is **strongly recommended** that you follow the instructions in the [`patch` directive section](https://docs.rs/cargo-hakari/latest/cargo_hakari/patch_directive).

### Information about the workspace-hack

The commands in this section provide information about components in the workspace-hack.

#### Why is a dependency in the workspace-hack?

Print out information about why a dependency is present in the workspace-hack:

```
cargo hakari explain <dependency-name>
```

![](https://user-images.githubusercontent.com/180618/144933657-c45cf719-ecaf-49e0-b2c7-c8d12adf11c0.png)

#### Does the workspace-hack ensure that each dependency is built with exactly one feature set?

```
cargo hakari verify
```

If some dependencies are built with more than one feature set, this command will print out details about them. **This is always a bug** ---if you encounter it, [a bug report](https://github.com/guppy-rs/guppy/issues/new) with more information would be greatly appreciated!

### Publishing a crate

If you publish crates to `crates.io` or other registries, see the [`publishing` module](https://docs.rs/cargo-hakari/latest/cargo_hakari/publishing).

### Disabling and uninstalling

Disable the workspace-hack crate temporarily by removing generated lines from `Cargo.toml`. (Re-enable by running `cargo hakari generate`.)

```
cargo hakari disable
```

Remove the workspace-hack crate as a dependency from all other workspace crates:

```
cargo hakari remove-deps
```

![](https://user-images.githubusercontent.com/180618/135726181-9fe86782-6471-4a1d-a511-a6c55dffbbd7.png)

## Configuration

`cargo hakari` is configured through `.config/hakari.toml` at the root of the workspace. Running `cargo hakari init` causes a new file to be created at this location.

Example configuration:

```toml
# The name of the package used for workspace-hack unification.
hakari-package = "workspace-hack"
# Cargo resolver version in use -- version 2 is highly recommended.
resolver = "2"

# Format for \`workspace-hack = ...\` lines in other Cargo.tomls.
dep-format-version = "4"

# Add triples corresponding to platforms commonly used by developers here.
# https://doc.rust-lang.org/rustc/platform-support.html
platforms = [
    # "x86_64-unknown-linux-gnu",
    # "x86_64-apple-darwin",
    # "x86_64-pc-windows-msvc",
]

# Write out exact versions rather than specifications. Set this to true if version numbers in
# \`Cargo.toml\` and \`Cargo.lock\` files are kept in sync, e.g. in some configurations of
# https://dependabot.com/.
# exact-versions = false
```

For more options, including how to exclude crates from the output, see the [`config` module](https://docs.rs/cargo-hakari/latest/cargo_hakari/config).

## Stability guarantees

`cargo-hakari` follows semantic versioning, where the public API is the command-line interface.

Within a given series, the command-line interface will be treated as append-only.

The generated `Cargo.toml` will also be kept the same unless:

- a new config option is added, in which case the different output will be gated on the new option, or
- there is a bugfix involved.

## Contributing

See the [CONTRIBUTING](https://github.com/guppy-rs/guppy/blob/HEAD/CONTRIBUTING.md) file for how to help out.

## License

This project is available under the terms of either the [Apache 2.0 license](https://github.com/guppy-rs/guppy/blob/HEAD/LICENSE-APACHE) or the [MIT license](https://github.com/guppy-rs/guppy/blob/HEAD/LICENSE-MIT).

## Install

Running the above command will globally install the cargo-hakari binary.

### Install as library

Run the following Cargo command in your project directory:

Or add the following line to your Cargo.toml:

## Documentation

[docs.rs/cargo-hakari/0.9.36](https://docs.rs/cargo-hakari/0.9.36)

## Repository

[github.com/guppy-rs/guppy](https://github.com/guppy-rs/guppy)

## Owners

- [Rain](https://crates.io/users/sunshowers)

## Categories

[Report crate](https://crates.io/support?crate=cargo-hakari&inquire=crate-violation)

The action has been successful
