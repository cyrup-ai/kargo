# Module `semver`

[![github]](https://github.com/dtolnay/semver)&ensp;[![crates-io]](https://crates.io/crates/semver)&ensp;[![docs-rs]](https://docs.rs/semver)
[github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
<br>
A parser and evaluator for Cargo's flavor of Semantic Versioning.
Semantic Versioning (see <https://semver.org>) is a guideline for how
version numbers are assigned and incremented. It is widely followed within
the Cargo/crates.io ecosystem for Rust.
<br>
# Example
```
use semver::{BuildMetadata, Prerelease, Version, VersionReq};
fn main() {
let req = VersionReq::parse(">=1.2.3, <1.8.0").unwrap();
// Check whether this requirement matches version 1.2.3-alpha.1 (no)
let version = Version {
major: 1,
minor: 2,
patch: 3,
pre: Prerelease::new("alpha.1").unwrap(),
build: BuildMetadata::EMPTY,
};
assert!(!req.matches(&version));
// Check whether it matches 1.3.0 (yes it does)
let version = Version::parse("1.3.0").unwrap();
assert!(req.matches(&version));
}
```
<br><br>
# Scope of this crate
Besides Cargo, several other package ecosystems and package managers for
other languages also use SemVer:&ensp;RubyGems/Bundler for Ruby, npm for
JavaScript, Composer for PHP, CocoaPods for Objective-C...
The `semver` crate is specifically intended to implement Cargo's
interpretation of Semantic Versioning.
Where the various tools differ in their interpretation or implementation of
the spec, this crate follows the implementation choices made by Cargo. If
you are operating on version numbers from some other package ecosystem, you
will want to use a different semver library which is appropriate to that
ecosystem.
The extent of Cargo's SemVer support is documented in the *[Specifying
Dependencies]* chapter of the Cargo reference.
[Specifying Dependencies]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html

## Contents

* **Struct** `Version` - **SemVer version** as defined by <https://semver.org>....
* **Struct** `VersionReq` - **SemVer version requirement** describing the intersection of some version comparators, such as `>=1.2.3, <1.8`....
* **Struct** `Comparator` - A pair of comparison operator and partial version, such as `>=1.2`. Forms one piece of a VersionReq.
* **Enum** `Op` - SemVer comparison operator: `=`, `>`, `>=`, `<`, `<=`, `~`, `^`, `*`....
* **Struct** `Prerelease` - Optional pre-release identifier on a version string. This comes after `-` in a SemVer version, like `1.0.0-alpha.1`...
* **Struct** `BuildMetadata` - Optional build metadata identifier. This comes after `+` in a SemVer version, as in `0.8.1+zstd.1.5.0`....

