mod parser;

use std::process::{Command, exit};
use std::path::PathBuf;
use std::fs;
use std::io::{self, BufRead};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        exit(1);
    }
}

fn run() -> Result<(), String> {
    eprintln!("Generating dependency files...");
    generate_depfiles()?;

    eprintln!("Parsing dependency files...");
    let source_files = parse_depfiles()?;

    eprintln!("Found {} source files", source_files.len());

    // Output all source files with #line directives
    for file in &source_files {
        output_file(file)?;
    }

    Ok(())
}

fn output_file(path: &PathBuf) -> Result<(), String> {
    let contents = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    // Output #line directive
    println!("#line 1 \"{}\"", path.display());

    // Output file contents
    print!("{}", contents);

    // Ensure there's a newline at the end
    if !contents.ends_with('\n') {
        println!();
    }

    Ok(())
}

fn generate_depfiles() -> Result<(), String> {
    let output = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .output()
        .map_err(|e| format!("Failed to run cargo check: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("cargo check failed:\n{}", stderr));
    }

    Ok(())
}

fn parse_depfiles() -> Result<Vec<PathBuf>, String> {
    let target_dir = PathBuf::from("target/debug/deps");

    if !target_dir.exists() {
        return Err(format!("Target directory not found: {}", target_dir.display()));
    }

    let mut all_sources = Vec::new();

    // Find all .d files in target/debug/deps/
    let entries = fs::read_dir(&target_dir)
        .map_err(|e| format!("Failed to read target directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("d") {
            let sources = parse_single_depfile(&path)?;
            all_sources.extend(sources);
        }
    }

    // Deduplicate
    all_sources.sort();
    all_sources.dedup();

    Ok(all_sources)
}

fn parse_single_depfile(path: &PathBuf) -> Result<Vec<PathBuf>, String> {
    let file = fs::File::open(path)
        .map_err(|e| format!("Failed to open {}: {}", path.display(), e))?;

    let reader = io::BufReader::new(file);
    let mut sources = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read line: {}", e))?;

        // The first line has format: "target: dep1.rs dep2.rs dep3.rs"
        // We want to extract the dependencies after the colon
        if let Some(deps_part) = line.split(':').nth(1) {
            for dep in deps_part.split_whitespace() {
                let dep_path = PathBuf::from(dep);
                // Only include .rs files (not .rmeta or other artifacts)
                if dep_path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    sources.push(dep_path);
                }
            }
            // We only care about the first line with the colon
            break;
        }
    }

    Ok(sources)
}
