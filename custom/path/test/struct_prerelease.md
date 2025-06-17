# Struct `Prerelease`

Optional pre-release identifier on a version string. This comes after `-` in
a SemVer version, like `1.0.0-alpha.1`
# Examples
Some real world pre-release idioms drawn from crates.io:
- **[mio]** <code>0.7.0-<b>alpha.1</b></code> &mdash; the most common style
for numbering pre-releases.
- **[pest]** <code>1.0.0-<b>beta.8</b></code>,&ensp;<code>1.0.0-<b>rc.0</b></code>
&mdash; this crate makes a distinction between betas and release
candidates.
- **[sassers]** <code>0.11.0-<b>shitshow</b></code> &mdash; ???.
- **[atomic-utils]** <code>0.0.0-<b>reserved</b></code> &mdash; a squatted
crate name.
[mio]: https://crates.io/crates/mio
[pest]: https://crates.io/crates/pest
[atomic-utils]: https://crates.io/crates/atomic-utils
[sassers]: https://crates.io/crates/sassers
*Tip:* Be aware that if you are planning to number your own pre-releases,
you should prefer to separate the numeric part from any non-numeric
identifiers by using a dot in between. That is, prefer pre-releases
`alpha.1`, `alpha.2`, etc rather than `alpha1`, `alpha2` etc. The SemVer
spec's rule for pre-release precedence has special treatment of numeric
components in the pre-release string, but only if there are no non-digit
characters in the same dot-separated component. So you'd have `alpha.2` &lt;
`alpha.11` as intended, but `alpha11` &lt; `alpha2`.
# Syntax
Pre-release strings are a series of dot separated identifiers immediately
following the patch version. Identifiers must comprise only ASCII
alphanumerics and hyphens: `0-9`, `A-Z`, `a-z`, `-`. Identifiers must not be
empty. Numeric identifiers must not include leading zeros.
# Total ordering
Pre-releases have a total order defined by the SemVer spec. It uses
lexicographic ordering of dot-separated components. Identifiers consisting
of only digits are compared numerically. Otherwise, identifiers are compared
in ASCII sort order. Any numeric identifier is always less than any
non-numeric identifier.
Example:&ensp;`alpha`&ensp;&lt;&ensp;`alpha.85`&ensp;&lt;&ensp;`alpha.90`&ensp;&lt;&ensp;`alpha.200`&ensp;&lt;&ensp;`alpha.0a`&ensp;&lt;&ensp;`alpha.1a0`&ensp;&lt;&ensp;`alpha.a`&ensp;&lt;&ensp;`beta`

## Fields

Field information will be available in a future version.

