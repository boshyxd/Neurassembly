use neurassembly::data::collector::{AssemblyCollector, CollectorConfig};
use std::{fs, path::PathBuf};
use tempfile::tempdir;

fn create_test_source_file(dir: &PathBuf) -> std::io::Result<()> {
    let source = r#"
        int factorial(int n) {
            if (n <= 1) return 1;
            return n * factorial(n - 1);
        }

        int fibonacci(int n) {
            if (n <= 1) return n;
            return fibonacci(n - 1) + fibonacci(n - 2);
        }
    "#;

    fs::write(dir.join("test.c"), source)?;
    Ok(())
}

#[test]
fn test_collector_creation() {
    let config = CollectorConfig::default();
    let collector = AssemblyCollector::new(config);
    // Just testing that collector creation doesn't panic
}

#[test]
fn test_source_file_collection() -> Result<(), Box<dyn std::error::Error>> {
    // Create temporary directories
    let source_dir = tempdir()?;
    let output_dir = tempdir()?;

    // Create test source file
    create_test_source_file(&source_dir.path().to_path_buf())?;

    let config = CollectorConfig {
        source_dir: source_dir.path().to_path_buf(),
        output_dir: output_dir.path().to_path_buf(),
        optimization_levels: vec!["-O0".to_string()], // Just test with one optimization level
        source_extensions: vec!["c".to_string()],
        max_jobs: 1,
    };

    let mut collector = AssemblyCollector::new(config);
    let examples = collector.collect()?;

    // We should have at least one example (might have more due to multiple functions)
    assert!(!examples.is_empty());

    Ok(())
}

#[test]
fn test_multiple_optimization_levels() -> Result<(), Box<dyn std::error::Error>> {
    // Create temporary directories
    let source_dir = tempdir()?;
    let output_dir = tempdir()?;

    // Create test source file
    create_test_source_file(&source_dir.path().to_path_buf())?;

    let config = CollectorConfig {
        source_dir: source_dir.path().to_path_buf(),
        output_dir: output_dir.path().to_path_buf(),
        optimization_levels: vec!["-O0".to_string(), "-O2".to_string()],
        source_extensions: vec!["c".to_string()],
        max_jobs: 1,
    };

    let mut collector = AssemblyCollector::new(config);
    let examples = collector.collect()?;

    // We should have examples from both optimization levels
    assert!(examples.len() >= 2);

    Ok(())
}

#[test]
fn test_encoder_consistency() -> Result<(), Box<dyn std::error::Error>> {
    // Create temporary directories
    let source_dir = tempdir()?;
    let output_dir = tempdir()?;

    // Create test source file
    create_test_source_file(&source_dir.path().to_path_buf())?;

    let config = CollectorConfig {
        source_dir: source_dir.path().to_path_buf(),
        output_dir: output_dir.path().to_path_buf(),
        optimization_levels: vec!["-O0".to_string()],
        source_extensions: vec!["c".to_string()],
        max_jobs: 1,
    };

    let mut collector = AssemblyCollector::new(config);
    let examples = collector.collect()?;

    // Check that all examples have both input and target tokens
    for example in examples {
        assert!(!example.input_tokens.is_empty());
        assert!(!example.target_tokens.is_empty());
        assert_eq!(example.input_tokens.len(), example.target_tokens.len());
    }

    Ok(())
} 