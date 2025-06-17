# Struct `Error`

Error parsing a SemVer version or version requirement.
# Example
```
use semver::Version;
fn main() {
let err = Version::parse("1.q.r").unwrap_err();
// "unexpected character 'q' while parsing minor version number"
eprintln!("{}", err);
}
```

## Fields

Field information will be available in a future version.

