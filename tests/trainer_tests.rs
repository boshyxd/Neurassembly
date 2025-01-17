use neurassembly::model::{
    encoder::AssemblyEncoder,
    optimizer::{OptimizationModel, OptimizationConfig},
    trainer::{ModelTrainer, TrainingConfig, TrainingExample},
};
use tempfile::tempdir;
use tch::Device;

fn create_dummy_training_data(encoder: &mut AssemblyEncoder) -> Vec<TrainingExample> {
    let input_assembly = vec![
        "mov rax, rbx",
        "add rax, 1",
        "push rax",
    ];
    
    let target_assembly = vec![
        "mov rax, rbx",
        "inc rax",
        "push rax",
    ];
    
    input_assembly.into_iter()
        .zip(target_assembly)
        .map(|(input, target)| TrainingExample {
            input_tokens: encoder.encode(input),
            target_tokens: encoder.encode(target),
        })
        .collect()
}

#[test]
fn test_trainer_creation() {
    let mut encoder = AssemblyEncoder::new();
    let model_config = OptimizationConfig {
        vocab_size: encoder.get_vocabulary_size() as i64,
        ..Default::default()
    };
    let model = OptimizationModel::new(model_config);
    let training_config = TrainingConfig::default();
    
    let trainer = ModelTrainer::new(model, training_config);
    // Just testing that trainer creation doesn't panic
}

#[test]
fn test_training_loop() -> Result<(), Box<dyn std::error::Error>> {
    let mut encoder = AssemblyEncoder::new();
    let training_data = create_dummy_training_data(&mut encoder);
    
    let model_config = OptimizationConfig {
        vocab_size: encoder.get_vocabulary_size() as i64,
        ..Default::default()
    };
    let model = OptimizationModel::new(model_config);
    
    // Create temporary directory for checkpoints
    let temp_dir = tempdir()?;
    let training_config = TrainingConfig {
        num_epochs: 2, // Reduce epochs for testing
        batch_size: 2,
        save_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };
    
    let mut trainer = ModelTrainer::new(model, training_config);
    trainer.train(training_data)?;
    
    assert_eq!(trainer.get_current_epoch(), 2);
    assert!(trainer.get_best_loss() < f64::INFINITY);
    
    Ok(())
}

#[test]
fn test_checkpoint_save_load() -> Result<(), Box<dyn std::error::Error>> {
    let mut encoder = AssemblyEncoder::new();
    let model_config = OptimizationConfig {
        vocab_size: encoder.get_vocabulary_size() as i64,
        ..Default::default()
    };
    let model = OptimizationModel::new(model_config.clone());
    
    // Create temporary directory for checkpoints
    let temp_dir = tempdir()?;
    let training_config = TrainingConfig {
        save_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };
    
    let mut trainer = ModelTrainer::new(model, training_config.clone());
    
    // Save a checkpoint
    trainer.save_checkpoint("test_checkpoint.pt")?;
    
    // Create a new trainer and load the checkpoint
    let model = OptimizationModel::new(model_config);
    let mut new_trainer = ModelTrainer::new(model, training_config);
    new_trainer.load_checkpoint("test_checkpoint.pt")?;
    
    Ok(())
} 