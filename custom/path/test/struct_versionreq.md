# Struct `VersionReq`

**SemVer version requirement** describing the intersection of some version
comparators, such as `>=1.2.3, <1.8`.
# Syntax
- Either `*` (meaning "any"), or one or more comma-separated comparators.
- A [`Comparator`] is an operator ([`Op`]) and a partial version, separated
by optional whitespace. For example `>=1.0.0` or `>=1.0`.
- Build metadata is syntactically permitted on the partial versions, but is
completely ignored, as it's never relevant to whether any comparator
matches a particular version.
- Whitespace is permitted around commas and around operators. Whitespace is
not permitted within a partial version, i.e. anywhere between the major
version number and its minor, patch, pre-release, or build metadata.

## Fields

Field information will be available in a future version.

