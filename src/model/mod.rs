pub mod encoder;
pub mod optimizer;
pub mod trainer;

// Re-export main types
pub use encoder::AssemblyEncoder;
pub use optimizer::OptimizationModel;
pub use trainer::ModelTrainer; 