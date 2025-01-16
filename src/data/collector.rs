use crate::model::{
    encoder::{AssemblyEncoder, AssemblyToken},
    trainer::TrainingExample,
};
use std::{
    path::{Path, PathBuf},
    fs,
    io::{self, BufRead, BufReader},
    process::Command,
};
use iced_x86::{Decoder, DecoderOptions};
use rayon::prelude::*;
use serde::{Serialize, Deserialize};

/// Configuration for data collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorConfig {
    /// Directory containing source files to compile
    pub source_dir: PathBuf,
    /// Directory to store compiled binaries and assembly
    pub output_dir: PathBuf,
    /// Optimization levels to collect (-O0, -O1, -O2, -O3)
    pub optimization_levels: Vec<String>,
    /// File extensions to process
    pub source_extensions: Vec<String>,
    /// Maximum number of parallel jobs
    pub max_jobs: usize,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            source_dir: PathBuf::from("sources"),
            output_dir: PathBuf::from("compiled"),
            optimization_levels: vec!["-O0".to_string(), "-O2".to_string(), "-O3".to_string()],
            source_extensions: vec!["c".to_string(), "cpp".to_string()],
            max_jobs: num_cpus::get(),
        }
    }
}

/// Handles collection and processing of assembly code examples
pub struct AssemblyCollector {
    config: CollectorConfig,
    encoder: AssemblyEncoder,
}

impl AssemblyCollector {
    pub fn new(config: CollectorConfig) -> Self {
        Self {
            config,
            encoder: AssemblyEncoder::new(),
        }
    }

    /// Collect assembly examples from source files
    pub fn collect(&mut self) -> Result<Vec<TrainingExample>, Box<dyn std::error::Error>> {
        // Create output directory if it doesn't exist
        fs::create_dir_all(&self.config.output_dir)?;

        // Find all source files
        let source_files = self.find_source_files()?;
        tracing::info!("Found {} source files", source_files.len());

        // Process files in parallel
        let examples: Vec<TrainingExample> = source_files
            .par_iter()
            .flat_map(|source_file| {
                self.process_source_file(source_file)
                    .unwrap_or_else(|e| {
                        tracing::error!("Error processing {}: {}", source_file.display(), e);
                        vec![]
                    })
            })
            .collect();

        tracing::info!("Collected {} training examples", examples.len());
        Ok(examples)
    }

    /// Find all source files in the source directory
    fn find_source_files(&self) -> io::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        for entry in fs::read_dir(&self.config.source_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if self.config.source_extensions.iter().any(|e| e == ext) {
                        files.push(path);
                    }
                }
            }
        }
        Ok(files)
    }

    /// Process a single source file and generate training examples
    fn process_source_file(&mut self, source_file: &Path) -> Result<Vec<TrainingExample>, Box<dyn std::error::Error>> {
        let mut examples = Vec::new();
        let file_stem = source_file.file_stem().unwrap().to_str().unwrap();

        // Compile with different optimization levels
        for opt_level in &self.config.optimization_levels {
            let asm_path = self.config.output_dir.join(format!("{}_{}.s", file_stem, opt_level));
            let obj_path = self.config.output_dir.join(format!("{}_{}.o", file_stem, opt_level));

            // Compile to assembly
            self.compile_to_assembly(source_file, &asm_path, opt_level)?;

            // Extract function pairs from assembly
            let function_pairs = self.extract_function_pairs(&asm_path)?;

            // Create training examples from function pairs
            for (unopt_func, opt_func) in function_pairs {
                if let Ok(example) = self.create_training_example(&unopt_func, &opt_func) {
                    examples.push(example);
                }
            }
        }

        Ok(examples)
    }

    /// Compile source file to assembly
    fn compile_to_assembly(&self, source: &Path, output: &Path, opt_level: &str) -> io::Result<()> {
        let status = Command::new("gcc")
            .arg("-S")
            .arg(opt_level)
            .arg("-o")
            .arg(output)
            .arg(source)
            .status()?;

        if !status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Compilation failed for {}", source.display()),
            ));
        }

        Ok(())
    }

    /// Extract function pairs from assembly file
    fn extract_function_pairs(&self, asm_file: &Path) -> io::Result<Vec<(String, String)>> {
        let file = fs::File::open(asm_file)?;
        let reader = BufReader::new(file);
        let mut pairs = Vec::new();
        let mut current_function = String::new();
        let mut in_function = false;

        for line in reader.lines() {
            let line = line?;
            if line.starts_with('.') && line.contains(':') {
                // New function starts
                if in_function {
                    // Store previous function
                    if !current_function.is_empty() {
                        pairs.push((current_function.clone(), current_function.clone()));
                    }
                    current_function.clear();
                }
                in_function = true;
            }

            if in_function {
                current_function.push_str(&line);
                current_function.push('\n');
            }
        }

        // Don't forget the last function
        if in_function && !current_function.is_empty() {
            pairs.push((current_function.clone(), current_function.clone()));
        }

        Ok(pairs)
    }

    /// Create a training example from a pair of functions
    fn create_training_example(&mut self, unopt_func: &str, opt_func: &str) -> Result<TrainingExample, Box<dyn std::error::Error>> {
        let input_tokens = self.encoder.encode(unopt_func);
        let target_tokens = self.encoder.encode(opt_func);

        Ok(TrainingExample {
            input_tokens,
            target_tokens,
        })
    }

    /// Get the encoder for external use
    pub fn get_encoder(&self) -> &AssemblyEncoder {
        &self.encoder
    }

    /// Get a mutable reference to the encoder
    pub fn get_encoder_mut(&mut self) -> &mut AssemblyEncoder {
        &mut self.encoder
    }
} 