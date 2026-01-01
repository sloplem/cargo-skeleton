use std::process::Command;
use std::fs;

#[test]
fn test_output_format() {
    // Build the binary first
    let build_status = Command::new("cargo")
        .arg("build")
        .arg("--quiet")
        .status()
        .expect("Failed to build binary");

    assert!(build_status.success(), "Build failed");

    // Run the cargo-skeleton tool
    let output = Command::new("./target/debug/cargo-skeleton")
        .output()
        .expect("Failed to run cargo-skeleton");

    assert!(output.status.success(), "cargo-skeleton failed: {}",
            String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8(output.stdout)
        .expect("Output was not valid UTF-8");

    // Read the actual src/main.rs file
    let main_rs_contents = fs::read_to_string("src/main.rs")
        .expect("Failed to read src/main.rs");

    // Verify output starts with #line directive for main.rs
    assert!(stdout.starts_with("#line 1 \"src/main.rs\"\n"),
            "Output should start with #line directive for src/main.rs");

    // Verify the contents of main.rs appear after the #line directive
    let expected_start = format!("#line 1 \"src/main.rs\"\n{}", main_rs_contents);

    assert!(stdout.starts_with(&expected_start),
            "Output should contain src/main.rs contents after #line directive");

    // Verify that we have multiple #line directives (one per file)
    let line_directive_count = stdout.matches("#line 1 \"").count();
    assert!(line_directive_count >= 3,
            "Expected at least 3 files (main.rs, parser/mod.rs, parser/depfile.rs), found {}",
            line_directive_count);
}

#[test]
fn test_strip_flag() {
    // Build the binary first
    let build_status = Command::new("cargo")
        .arg("build")
        .arg("--quiet")
        .status()
        .expect("Failed to build binary");

    assert!(build_status.success(), "Build failed");

    // Run the cargo-skeleton tool with --strip
    let output = Command::new("./target/debug/cargo-skeleton")
        .arg("--strip")
        .output()
        .expect("Failed to run cargo-skeleton");

    assert!(output.status.success(), "cargo-skeleton --strip failed: {}",
            String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8(output.stdout)
        .expect("Output was not valid UTF-8");

    // Verify output starts with #line directive for main.rs
    assert!(stdout.starts_with("#line 1 \"src/main.rs\"\n"),
            "Output should start with #line directive for src/main.rs");

    // Verify that function signatures are present but bodies are stripped
    // The main function should be present
    assert!(stdout.contains("fn main()"),
            "Should contain main function signature");

    // Verify function bodies are empty blocks
    // Look for pattern: function signature followed by empty braces
    assert!(stdout.contains("fn main() {}"),
            "main() should have empty body");

    // Verify the run function signature is present
    assert!(stdout.contains("fn run() -> Result<(), String>"),
            "Should contain run function signature with return type");

    // Verify struct definitions are preserved
    assert!(stdout.contains("struct BodyStripper"),
            "Should preserve struct definitions");

    // Verify impl blocks are present
    assert!(stdout.contains("impl VisitMut for BodyStripper"),
            "Should preserve impl blocks");

    // Verify method signatures in impl blocks are present
    assert!(stdout.contains("fn visit_item_fn_mut"),
            "Should preserve method signatures");

    // Verify use statements are preserved
    assert!(stdout.contains("use std::process::{Command, exit}"),
            "Should preserve use statements");

    // Verify mod declarations are preserved
    assert!(stdout.contains("mod parser"),
            "Should preserve mod declarations");

    // Verify multiple files are included
    let line_directive_count = stdout.matches("#line 1 \"").count();
    assert!(line_directive_count >= 3,
            "Expected at least 3 files with --strip, found {}",
            line_directive_count);
}
