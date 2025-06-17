# Struct `Version`

**SemVer version** as defined by <https://semver.org>.
# Syntax
- The major, minor, and patch numbers may be any integer 0 through u64::MAX.
When representing a SemVer version as a string, each number is written as
a base 10 integer. For example, `1.0.119`.
- Leading zeros are forbidden in those positions. For example `1.01.00` is
invalid as a SemVer version.
- The pre-release identifier, if present, must conform to the syntax
documented for [`Prerelease`].
- The build metadata, if present, must conform to the syntax documented for
[`BuildMetadata`].
- Whitespace is not allowed anywhere in the version.
# Total ordering
Given any two SemVer versions, one is less than, greater than, or equal to
the other. Versions may be compared against one another using Rust's usual
comparison operators.
- The major, minor, and patch number are compared numerically from left to
right, lexicographically ordered as a 3-tuple of integers. So for example
version `1.5.0` is less than version `1.19.0`, despite the fact that
"1.19.0" &lt; "1.5.0" as ASCIIbetically compared strings and 1.19 &lt; 1.5
as real numbers.
- When major, minor, and patch are equal, a pre-release version is
considered less than the ordinary release:&ensp;version `1.0.0-alpha.1` is
less than version `1.0.0`.
- Two pre-releases of the same major, minor, patch are compared by
lexicographic ordering of dot-separated components of the pre-release
string.
- Identifiers consisting of only digits are compared
numerically:&ensp;`1.0.0-pre.8` is less than `1.0.0-pre.12`.
- Identifiers that contain a letter or hyphen are compared in ASCII sort
order:&ensp;`1.0.0-pre12` is less than `1.0.0-pre8`.
- Any numeric identifier is always less than any non-numeric
identifier:&ensp;`1.0.0-pre.1` is less than `1.0.0-pre.x`.
Example:&ensp;`1.0.0-alpha`&ensp;&lt;&ensp;`1.0.0-alpha.1`&ensp;&lt;&ensp;`1.0.0-alpha.beta`&ensp;&lt;&ensp;`1.0.0-beta`&ensp;&lt;&ensp;`1.0.0-beta.2`&ensp;&lt;&ensp;`1.0.0-beta.11`&ensp;&lt;&ensp;`1.0.0-rc.1`&ensp;&lt;&ensp;`1.0.0`

## Fields

Field information will be available in a future version.

