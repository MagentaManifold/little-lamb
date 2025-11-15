use std::fs;
use std::path::Path;
use std::process::Command;

/// Integration test to verify all example files in the examples/ directory
/// (excluding the errors/ subdirectory) execute successfully.
#[test]
fn test_all_examples() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let examples_dir = Path::new(manifest_dir).join("examples");
    let errors_dir = examples_dir.join("errors");

    let mut example_files = Vec::new();

    // TODO: support nested directories
    if let Ok(entries) = fs::read_dir(examples_dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path == errors_dir {
                continue;
            }

            if path.is_file() && path.extension().is_some_and(|ext| ext == "lil") {
                example_files.push(path);
            }
        }
    }

    assert!(
        !example_files.is_empty(),
        "No example files found in examples directory"
    );

    let total_files = example_files.len();

    for example_file in &example_files {
        println!("Testing example: {}", example_file.display());

        let output = Command::new("cargo")
            .args(["run", "--", example_file.to_str().unwrap()])
            .current_dir(manifest_dir)
            .output()
            .expect("Failed to execute cargo run");

        assert!(
            output.status.success(),
            "Example {} failed to run successfully. Stderr: {}",
            example_file.display(),
            String::from_utf8_lossy(&output.stderr)
        );

        println!("✅ {} ran successfully", example_file.display());
    }

    println!("All {} example files ran successfully!", total_files);
}
