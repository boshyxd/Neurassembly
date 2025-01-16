use crate::model::{
    encoder::AssemblyToken,
    optimizer::{OptimizationModel, OptimizationConfig},
};
use tch::{Device, Tensor, nn::OptimizerConfig};
use std::{time::Instant, path::PathBuf};

/// Configuration for the training process
#[derive(Debug, Clone)]
pub struct TrainingConfig {
    pub learning_rate: f64,
    pub batch_size: usize,
    pub num_epochs: usize,
    pub save_dir: PathBuf,
    pub checkpoint_interval: usize,
    pub device: Device,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            learning_rate: 1e-4,
            batch_size: 32,
            num_epochs: 100,
            save_dir: PathBuf::from("checkpoints"),
            checkpoint_interval: 10,
            device: Device::Cpu,
        }
    }
}

/// Training example consisting of input and target assembly sequences
#[derive(Debug, Clone)]
pub struct TrainingExample {
    pub input_tokens: Vec<AssemblyToken>,
    pub target_tokens: Vec<AssemblyToken>,
}

/// Handles model training and optimization
pub struct ModelTrainer {
    model: OptimizationModel,
    config: TrainingConfig,
    optimizer: tch::nn::Adam,
    current_epoch: usize,
    best_loss: f64,
}

impl ModelTrainer {
    pub fn new(model: OptimizationModel, config: TrainingConfig) -> Self {
        let optimizer = tch::nn::Adam::default().build(&model.var_store, config.learning_rate).unwrap();
        
        Self {
            model,
            config,
            optimizer,
            current_epoch: 0,
            best_loss: f64::INFINITY,
        }
    }

    /// Train the model on a batch of examples
    fn train_batch(&mut self, batch: &[TrainingExample]) -> f64 {
        // Prepare batch data
        let (input_tokens, target_tokens): (Vec<_>, Vec<_>) = batch.iter()
            .map(|example| (&example.input_tokens, &example.target_tokens))
            .unzip();

        // Forward pass
        let mut total_loss = 0.0;
        for (input, target) in input_tokens.iter().zip(target_tokens.iter()) {
            let output = self.model.forward(input);
            
            // Calculate loss (cross entropy between output and target)
            let target_tensor = self.prepare_target_tensor(target);
            let loss = output.cross_entropy_loss(&target_tensor, None, tch::Reduction::Mean);
            
            // Backward pass
            self.optimizer.backward_step(&loss);
            
            total_loss += f64::from(loss);
        }

        total_loss / batch.len() as f64
    }

    /// Convert target tokens to tensor format
    fn prepare_target_tensor(&self, target_tokens: &[AssemblyToken]) -> Tensor {
        // Convert tokens to indices
        let indices: Vec<i64> = target_tokens.iter()
            .map(|t| t.value.parse::<i64>().unwrap_or(0))
            .collect();

        Tensor::from_slice(&indices)
            .to_device(self.config.device)
    }

    /// Train the model for the specified number of epochs
    pub fn train(&mut self, training_data: Vec<TrainingExample>) -> Result<(), Box<dyn std::error::Error>> {
        let num_batches = (training_data.len() + self.config.batch_size - 1) / self.config.batch_size;
        
        for epoch in 0..self.config.num_epochs {
            let start_time = Instant::now();
            let mut epoch_loss = 0.0;
            
            // Process mini-batches
            for batch_idx in 0..num_batches {
                let start_idx = batch_idx * self.config.batch_size;
                let end_idx = (start_idx + self.config.batch_size).min(training_data.len());
                let batch = &training_data[start_idx..end_idx];
                
                let batch_loss = self.train_batch(batch);
                epoch_loss += batch_loss;
                
                // Log progress
                if (batch_idx + 1) % 10 == 0 {
                    tracing::info!(
                        "Epoch {}/{}, Batch {}/{}, Loss: {:.4}",
                        epoch + 1,
                        self.config.num_epochs,
                        batch_idx + 1,
                        num_batches,
                        batch_loss
                    );
                }
            }
            
            epoch_loss /= num_batches as f64;
            let elapsed = start_time.elapsed();
            
            tracing::info!(
                "Epoch {}/{} completed in {:.2?}, Average Loss: {:.4}",
                epoch + 1,
                self.config.num_epochs,
                elapsed,
                epoch_loss
            );
            
            // Save checkpoint if loss improved
            if epoch_loss < self.best_loss {
                self.best_loss = epoch_loss;
                self.save_checkpoint(epoch + 1, "best_model.pt")?;
            }
            
            // Regular checkpoint saving
            if (epoch + 1) % self.config.checkpoint_interval == 0 {
                self.save_checkpoint(epoch + 1, &format!("checkpoint_epoch_{}.pt", epoch + 1))?;
            }
            
            self.current_epoch = epoch + 1;
        }
        
        Ok(())
    }

    /// Save a model checkpoint
    fn save_checkpoint(&self, epoch: usize, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&self.config.save_dir)?;
        let checkpoint_path = self.config.save_dir.join(filename);
        self.model.save(&checkpoint_path)?;
        
        tracing::info!("Saved checkpoint to {}", checkpoint_path.display());
        Ok(())
    }

    /// Load a model checkpoint
    pub fn load_checkpoint(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let checkpoint_path = self.config.save_dir.join(filename);
        self.model.load(&checkpoint_path)?;
        
        tracing::info!("Loaded checkpoint from {}", checkpoint_path.display());
        Ok(())
    }

    /// Get the current training epoch
    pub fn get_current_epoch(&self) -> usize {
        self.current_epoch
    }

    /// Get the best loss achieved during training
    pub fn get_best_loss(&self) -> f64 {
        self.best_loss
    }
} 