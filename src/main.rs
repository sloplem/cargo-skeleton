mod parser;

use std::process::{Command, exit};
use std::path::PathBuf;
use std::fs;
use std::io::{self, BufRead};
use syn::visit_mut::{self, VisitMut};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        exit(1);
    }
}

fn run() -> Result<(), String> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let strip = args.contains(&"--strip".to_string());

    if strip {
        eprintln!("Stripping mode enabled");
    }

    eprintln!("Generating dependency files...");
    generate_depfiles()?;

    eprintln!("Parsing dependency files...");
    let source_files = parse_depfiles()?;

    eprintln!("Found {} source files", source_files.len());

    // Output all source files with #line directives
    for file in &source_files {
        output_file(file, strip)?;
    }

    Ok(())
}

fn output_file(path: &PathBuf, strip: bool) -> Result<(), String> {
    let contents = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    // Output #line directive
    println!("#line 1 \"{}\"", path.display());

    if strip {
        // Parse and strip function bodies
        let stripped = strip_function_bodies(&contents)?;
        print!("{}", stripped);
    } else {
        // Output file contents as-is
        print!("{}", contents);
    }

    // Ensure there's a newline at the end
    if !contents.ends_with('\n') {
        println!();
    }

    Ok(())
}

fn strip_function_bodies(source: &str) -> Result<String, String> {
    let mut ast = syn::parse_file(source)
        .map_err(|e| format!("Failed to parse Rust source: {}", e))?;

    let mut stripper = BodyStripper;
    stripper.visit_file_mut(&mut ast);

    Ok(quote::quote!(#ast).to_string())
}

struct BodyStripper;

impl VisitMut for BodyStripper {
    fn visit_item_fn_mut(&mut self, node: &mut syn::ItemFn) {
        // Replace function body with empty block
        node.block = Box::new(syn::parse_quote!({ /* ... */ }));

        // Continue visiting nested items
        visit_mut::visit_item_fn_mut(self, node);
    }

    fn visit_impl_item_fn_mut(&mut self, node: &mut syn::ImplItemFn) {
        // Replace method body with empty block
        node.block = syn::parse_quote!({ /* ... */ });

        // Continue visiting nested items
        visit_mut::visit_impl_item_fn_mut(self, node);
    }

    fn visit_trait_item_fn_mut(&mut self, node: &mut syn::TraitItemFn) {
        // Replace trait method body with empty block (for default implementations)
        if node.default.is_some() {
            node.default = Some(syn::parse_quote!({ /* ... */ }));
        }

        // Continue visiting nested items
        visit_mut::visit_trait_item_fn_mut(self, node);
    }
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

    // Filter to only include files in src/ (our own source files, not dependencies)
    all_sources.retain(|path| path.starts_with("src/"));

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
