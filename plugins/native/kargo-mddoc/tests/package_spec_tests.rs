use kargo_mddoc::PackageSpec;

#[test]
fn test_parse_package_name_only() {
    let spec = PackageSpec::parse("tokio").expect("Failed to parse package name");
    assert_eq!(spec.name, "tokio");
    assert_eq!(spec.version, None);
}

#[test]
fn test_parse_package_with_version() {
    let spec = PackageSpec::parse("tokio@1.28.0").expect("Failed to parse package with version");
    assert_eq!(spec.name, "tokio");
    assert_eq!(spec.version, Some("1.28.0".to_string()));
}

#[test]
fn test_invalid_package_name() {
    assert!(PackageSpec::parse("").is_err());
    assert!(PackageSpec::parse("1invalid").is_err());
    assert!(PackageSpec::parse("invalid@1.0@extra").is_err());
}

#[test]
fn test_version_spec() {
    let spec1 = PackageSpec::parse("tokio").expect("Failed to parse tokio package");
    assert_eq!(spec1.version_spec(), "\"*\"");

    let spec2 = PackageSpec::parse("tokio@1.28.0").expect("Failed to parse tokio@1.28.0");
    assert_eq!(spec2.version_spec(), "\"1.28.0\"");
}

#[test]
fn test_output_filenames() {
    let spec1 = PackageSpec::parse("tokio").expect("Failed to parse tokio for filename test");
    assert_eq!(spec1.json_filename(), "tokio.json");
    assert_eq!(spec1.markdown_filename(), "tokio.md");

    let spec2 =
        PackageSpec::parse("tokio@1.28.0").expect("Failed to parse tokio@1.28.0 for filename test");
    assert_eq!(spec2.json_filename(), "tokio-1.28.0.json");
    assert_eq!(spec2.markdown_filename(), "tokio-1.28.0.md");
}
