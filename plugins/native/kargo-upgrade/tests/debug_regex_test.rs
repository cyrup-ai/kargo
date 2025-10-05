// Run this with: cargo test --test debug_regex_test -- --nocapture

#[test]
fn debug_cargo_deps_regex() {
    use regex::Regex;
    
    let content = r#"#!/usr/bin/env rust-script
// cargo-deps: anyhow="1.0.0", tokio="1.0.0", regex

fn main() {
    println!("Hello world!");
}
    "#;

    let regex = Regex::new(r"(?m)//\s*cargo-deps:\s*(.+)$").unwrap();
    
    println!("\n=== REGEX DEBUG ===");
    println!("Pattern: {:?}", regex.as_str());
    println!("\nContent to search:");
    println!("{}", content);
    println!("\nContent lines:");
    for (i, line) in content.lines().enumerate() {
        println!("  Line {}: {:?}", i, line);
        if line.contains("cargo-deps") {
            println!("    ^ THIS LINE CONTAINS cargo-deps");
            println!("    Line bytes: {:?}", line.as_bytes());
        }
    }
    
    let matches: Vec<_> = regex.captures_iter(content).collect();
    println!("\nNumber of matches: {}", matches.len());
    
    for (i, capture) in matches.iter().enumerate() {
        println!("\n--- Match {} ---", i);
        println!("  Full match: {:?}", capture.get(0).map(|m| m.as_str()));
        println!("  Capture group 1: {:?}", capture.get(1).map(|m| m.as_str()));
        if let Some(m) = capture.get(0) {
            println!("  Match range: {}..{}", m.start(), m.end());
        }
    }
    
    // Also test if the line matches when isolated
    let test_line = "// cargo-deps: anyhow=\"1.0.0\", tokio=\"1.0.0\", regex";
    println!("\n=== TESTING ISOLATED LINE ===");
    println!("Line: {:?}", test_line);
    if let Some(cap) = regex.captures(test_line) {
        println!("MATCHES!");
        println!("  Captured: {:?}", cap.get(1).map(|m| m.as_str()));
    } else {
        println!("NO MATCH on isolated line");
    }
}
