use criterion::{black_box, criterion_group, criterion_main, Criterion};
use neurassembly::model::{
	encoder::{AssemblyEncoder, AssemblyToken},
	optimizer::{OptimizationModel, OptimizationConfig},
};

fn optimize_benchmark(c: &mut Criterion) {
	let config = OptimizationConfig::default();
	let model = OptimizationModel::new(config);
	let mut encoder = AssemblyEncoder::new();

	// Sample assembly code for benchmarking
	let assembly = "mov rax, rbx\nadd rax, 42\n";
	let tokens = encoder.encode(assembly);

	c.bench_function("optimize_simple_sequence", |b| {
		b.iter(|| {
			model.optimize(black_box(&tokens))
		});
	});
}

criterion_group!(benches, optimize_benchmark);
criterion_main!(benches);