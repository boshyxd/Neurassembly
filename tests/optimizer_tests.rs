use neurassembly::model::{
    encoder::{AssemblyEncoder, AssemblyToken},
    optimizer::{OptimizationModel, OptimizationConfig},
};

#[test]
fn test_model_creation() {
    let config = OptimizationConfig::default();
    let model = OptimizationModel::new(config);
    // Just testing that model creation doesn't panic
}

#[test]
fn test_model_forward_pass() {
    let mut encoder = AssemblyEncoder::new();
    let assembly = "mov rax, rbx";
    let tokens = encoder.encode(assembly);

    let config = OptimizationConfig {
        vocab_size: encoder.get_vocabulary_size() as i64,
        ..Default::default()
    };
    let model = OptimizationModel::new(config);

    let output = model.forward(&tokens);
    assert_eq!(output.size(), &[1, tokens.len() as i64, config.vocab_size]);
}

#[test]
fn test_model_optimization() {
    let mut encoder = AssemblyEncoder::new();
    let assembly = "mov rax, rbx\nadd rax, 1";
    let tokens = encoder.encode(assembly);

    let config = OptimizationConfig {
        vocab_size: encoder.get_vocabulary_size() as i64,
        ..Default::default()
    };
    let model = OptimizationModel::new(config);

    let optimized_tokens = model.optimize(&tokens);
    assert!(!optimized_tokens.is_empty());
}

#[test]
fn test_model_save_load() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use tempfile::tempdir;

    let config = OptimizationConfig::default();
    let model = OptimizationModel::new(config.clone());

    // Create a temporary directory for the model
    let dir = tempdir()?;
    let model_path = dir.path().join("model.pt");

    // Save the model
    model.save(&model_path)?;
    assert!(model_path.exists());

    // Load the model
    let mut loaded_model = OptimizationModel::new(config);
    loaded_model.load(&model_path)?;

    Ok(())
} 