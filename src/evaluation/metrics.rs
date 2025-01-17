use crate::model::encoder::AssemblyToken;
use std::{
    collections::HashMap,
    time::Duration,
    process::{Command, Stdio},
};
use serde::{Serialize, Deserialize};

/// Performance metrics for assembly code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Number of instructions
    pub instruction_count: usize,
    /// Estimated cycles
    pub estimated_cycles: u64,
    /// Memory operations
    pub memory_ops: usize,
    /// Register pressure (number of unique registers used)
    pub register_pressure: usize,
    /// Code size in bytes
    pub code_size: usize,
    /// Execution time (if measured)
    pub execution_time: Option<Duration>,
}

/// Configuration for performance measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Number of times to run benchmarks
    pub benchmark_iterations: usize,
    /// Whether to include execution time measurements
    pub measure_execution_time: bool,
    /// Temporary directory for compiled code
    pub temp_dir: std::path::PathBuf,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            benchmark_iterations: 100,
            measure_execution_time: true,
            temp_dir: std::env::temp_dir().join("neurassembly"),
        }
    }
}

/// Evaluates assembly code performance
pub struct PerformanceEvaluator {
    config: MetricsConfig,
    instruction_costs: HashMap<String, u64>,
}

impl PerformanceEvaluator {
    pub fn new(config: MetricsConfig) -> Self {
        let mut evaluator = Self {
            config,
            instruction_costs: HashMap::new(),
        };
        evaluator.initialize_instruction_costs();
        evaluator
    }

    /// Initialize estimated cycle costs for common instructions
    fn initialize_instruction_costs(&mut self) {
        let costs = [
            // Basic arithmetic
            ("mov", 1), ("add", 1), ("sub", 1), ("inc", 1), ("dec", 1),
            ("and", 1), ("or", 1), ("xor", 1), ("not", 1),
            // Memory operations
            ("push", 3), ("pop", 3), ("load", 4), ("store", 4),
            // Control flow
            ("jmp", 2), ("je", 2), ("jne", 2), ("call", 3), ("ret", 3),
            // Complex operations
            ("mul", 3), ("div", 15), ("idiv", 15),
            // SIMD operations
            ("movaps", 1), ("addps", 3), ("mulps", 4),
        ];

        for (inst, cost) in costs {
            self.instruction_costs.insert(inst.to_string(), cost);
        }
    }

    /// Calculate metrics for a sequence of assembly tokens
    pub fn calculate_metrics(&self, tokens: &[AssemblyToken]) -> PerformanceMetrics {
        let mut metrics = PerformanceMetrics {
            instruction_count: 0,
            estimated_cycles: 0,
            memory_ops: 0,
            register_pressure: 0,
            code_size: tokens.len(),
            execution_time: None,
        };

        let mut used_registers = std::collections::HashSet::new();
        let mut current_mnemonic = None;

        for token in tokens {
            match token.token_type {
                crate::model::encoder::TokenType::Mnemonic => {
                    metrics.instruction_count += 1;
                    current_mnemonic = Some(token.value.to_lowercase());
                    
                    // Add estimated cycles
                    if let Some(cost) = self.instruction_costs.get(&token.value.to_lowercase()) {
                        metrics.estimated_cycles += cost;
                    }
                }
                crate::model::encoder::TokenType::Register => {
                    used_registers.insert(token.value.clone());
                }
                crate::model::encoder::TokenType::Memory => {
                    if let Some(mnemonic) = &current_mnemonic {
                        if mnemonic.contains("mov") || mnemonic.contains("load") || 
                           mnemonic.contains("store") || mnemonic.contains("push") || 
                           mnemonic.contains("pop") {
                            metrics.memory_ops += 1;
                        }
                    }
                }
                _ => {}
            }
        }

        metrics.register_pressure = used_registers.len();
        metrics
    }

    /// Measure actual execution time of compiled assembly
    pub fn measure_execution_time(&self, assembly: &str) -> Result<Duration, Box<dyn std::error::Error>> {
        if !self.config.measure_execution_time {
            return Ok(Duration::from_secs(0));
        }

        // Create temporary files
        std::fs::create_dir_all(&self.config.temp_dir)?;
        let asm_file = self.config.temp_dir.join("test.s");
        let _obj_file = self.config.temp_dir.join("test.o");
        let exe_file = self.config.temp_dir.join("test");

        // Write assembly to file
        std::fs::write(&asm_file, assembly)?;

        // Compile assembly
        Command::new("gcc")
            .args(&["-o", exe_file.to_str().unwrap(), asm_file.to_str().unwrap()])
            .output()?;

        // Run multiple times and take average
        let mut total_time = Duration::new(0, 0);
        for _ in 0..self.config.benchmark_iterations {
            let start = std::time::Instant::now();
            Command::new(&exe_file)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .output()?;
            total_time += start.elapsed();
        }

        Ok(total_time / self.config.benchmark_iterations as u32)
    }

    /// Compare two versions of assembly code
    pub fn compare_metrics(&self, original: &[AssemblyToken], optimized: &[AssemblyToken]) -> MetricsComparison {
        let original_metrics = self.calculate_metrics(original);
        let optimized_metrics = self.calculate_metrics(optimized);

        MetricsComparison {
            instruction_reduction: percentage_change(
                original_metrics.instruction_count as u64,
                optimized_metrics.instruction_count as u64,
            ),
            cycle_reduction: percentage_change(
                original_metrics.estimated_cycles,
                optimized_metrics.estimated_cycles,
            ),
            memory_ops_reduction: percentage_change(
                original_metrics.memory_ops as u64,
                optimized_metrics.memory_ops as u64,
            ),
            register_pressure_change: percentage_change(
                original_metrics.register_pressure as u64,
                optimized_metrics.register_pressure as u64,
            ),
            code_size_reduction: percentage_change(
                original_metrics.code_size as u64,
                optimized_metrics.code_size as u64,
            ),
            execution_time_reduction: match (original_metrics.execution_time, optimized_metrics.execution_time) {
                (Some(original), Some(optimized)) => {
                    Some(percentage_change(
                        original.as_nanos() as u64,
                        optimized.as_nanos() as u64,
                    ))
                }
                _ => None,
            },
        }
    }
}

/// Comparison between original and optimized metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsComparison {
    /// Percentage reduction in instruction count
    pub instruction_reduction: f64,
    /// Percentage reduction in estimated cycles
    pub cycle_reduction: f64,
    /// Percentage reduction in memory operations
    pub memory_ops_reduction: f64,
    /// Percentage change in register pressure
    pub register_pressure_change: f64,
    /// Percentage reduction in code size
    pub code_size_reduction: f64,
    /// Percentage reduction in execution time (if measured)
    pub execution_time_reduction: Option<f64>,
}

/// Calculate percentage change between two values
fn percentage_change(original: u64, new: u64) -> f64 {
    if original == 0 {
        return 0.0;
    }
    ((original as f64 - new as f64) / original as f64) * 100.0
} 