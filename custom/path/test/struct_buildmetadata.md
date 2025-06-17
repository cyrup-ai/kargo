# Struct `BuildMetadata`

Optional build metadata identifier. This comes after `+` in a SemVer
version, as in `0.8.1+zstd.1.5.0`.
# Examples
Some real world build metadata idioms drawn from crates.io:
- **[libgit2-sys]** <code>0.12.20+<b>1.1.0</b></code> &mdash; for this
crate, the build metadata indicates the version of the C libgit2 library
that the Rust crate is built against.
- **[mashup]** <code>0.1.13+<b>deprecated</b></code> &mdash; just the word
"deprecated" for a crate that has been superseded by another. Eventually
people will take notice of this in Cargo's build output where it lists the
crates being compiled.
- **[google-bigquery2]** <code>2.0.4+<b>20210327</b></code> &mdash; this
library is automatically generated from an official API schema, and the
build metadata indicates the date on which that schema was last captured.
- **[fbthrift-git]** <code>0.0.6+<b>c7fcc0e</b></code> &mdash; this crate is
published from snapshots of a big company monorepo. In monorepo
development, there is no concept of versions, and all downstream code is
just updated atomically in the same commit that breaking changes to a
library are landed. Therefore for crates.io purposes, every published
version must be assumed to be incompatible with the previous. The build
metadata provides the source control hash of the snapshotted code.
[libgit2-sys]: https://crates.io/crates/libgit2-sys
[mashup]: https://crates.io/crates/mashup
[google-bigquery2]: https://crates.io/crates/google-bigquery2
[fbthrift-git]: https://crates.io/crates/fbthrift-git
# Syntax
Build metadata is a series of dot separated identifiers immediately
following the patch or pre-release version. Identifiers must comprise only
ASCII alphanumerics and hyphens: `0-9`, `A-Z`, `a-z`, `-`. Identifiers must
not be empty. Leading zeros *are* allowed, unlike any other place in the
SemVer grammar.
# Total ordering
Build metadata is ignored in evaluating `VersionReq`; it plays no role in
whether a `Version` matches any one of the comparison operators.
However for comparing build metadatas among one another, they do have a
total order which is determined by lexicographic ordering of dot-separated
components. Identifiers consisting of only digits are compared numerically.
Otherwise, identifiers are compared in ASCII sort order. Any numeric
identifier is always less than any non-numeric identifier.
Example:&ensp;`demo`&ensp;&lt;&ensp;`demo.85`&ensp;&lt;&ensp;`demo.90`&ensp;&lt;&ensp;`demo.090`&ensp;&lt;&ensp;`demo.200`&ensp;&lt;&ensp;`demo.1a0`&ensp;&lt;&ensp;`demo.a`&ensp;&lt;&ensp;`memo`

## Fields

Field information will be available in a future version.

