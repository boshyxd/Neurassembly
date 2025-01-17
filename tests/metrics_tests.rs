use neurassembly::{
    model::encoder::AssemblyEncoder,
    evaluation::metrics::{PerformanceEvaluator, MetricsConfig},
};
use tempfile::tempdir;

#[test]
fn test_metrics_calculation() {
    let mut encoder = AssemblyEncoder::new();
    let evaluator = PerformanceEvaluator::new(MetricsConfig::default());

    // Simple assembly sequence
    let assembly = "mov rax, rbx\nadd rax, 1\npush rax";
    let tokens = encoder.encode(assembly);
    
    let metrics = evaluator.calculate_metrics(&tokens);
    
    assert_eq!(metrics.instruction_count, 3);
    assert!(metrics.estimated_cycles > 0);
    assert_eq!(metrics.memory_ops, 1); // One push operation
    assert!(metrics.register_pressure > 0);
    assert_eq!(metrics.code_size, tokens.len());
}

#[test]
fn test_metrics_comparison() {
    let mut encoder = AssemblyEncoder::new();
    let evaluator = PerformanceEvaluator::new(MetricsConfig::default());

    // Original code: add 1 using addition
    let original = "mov rax, 0\nadd rax, 1";
    let original_tokens = encoder.encode(original);

    // Optimized code: add 1 using inc
    let optimized = "mov rax, 0\ninc rax";
    let optimized_tokens = encoder.encode(optimized);

    let comparison = evaluator.compare_metrics(&original_tokens, &optimized_tokens);

    // The optimized version should have fewer instructions and cycles
    assert!(comparison.instruction_reduction >= 0.0);
    assert!(comparison.cycle_reduction >= 0.0);
}

#[test]
fn test_memory_operations_counting() {
    let mut encoder = AssemblyEncoder::new();
    let evaluator = PerformanceEvaluator::new(MetricsConfig::default());

    let assembly = "mov rax, [rbx]\npush rax\npop rcx\nmov [rdx], rax";
    let tokens = encoder.encode(assembly);
    
    let metrics = evaluator.calculate_metrics(&tokens);
    
    // Should count 4 memory operations: load, push, pop, store
    assert_eq!(metrics.memory_ops, 4);
}

#[test]
fn test_register_pressure() {
    let mut encoder = AssemblyEncoder::new();
    let evaluator = PerformanceEvaluator::new(MetricsConfig::default());

    let assembly = "mov rax, rbx\nmov rcx, rdx\nadd rax, rcx";
    let tokens = encoder.encode(assembly);
    
    let metrics = evaluator.calculate_metrics(&tokens);
    
    // Should count 4 unique registers: rax, rbx, rcx, rdx
    assert_eq!(metrics.register_pressure, 4);
}

#[test]
fn test_execution_time_measurement() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let config = MetricsConfig {
        benchmark_iterations: 1, // Reduce iterations for testing
        measure_execution_time: true,
        temp_dir: temp_dir.path().to_path_buf(),
    };
    let evaluator = PerformanceEvaluator::new(config);

    // Simple assembly program
    let assembly = r#"
        .global main
        main:
            mov $0, %rax
            ret
    "#;

    let duration = evaluator.measure_execution_time(assembly)?;
    assert!(duration.as_nanos() > 0);

    Ok(())
} 