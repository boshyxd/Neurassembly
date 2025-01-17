use crate::model::encoder::AssemblyToken;
use std::collections::HashSet;

pub struct OptimizationValidator {
	pub check_correctness: bool,
	pub check_performance: bool,
}

impl OptimizationValidator {
	pub fn new() -> Self {
		Self {
			check_correctness: true,
			check_performance: true,
		}
	}

	pub fn validate(&self, original: &[AssemblyToken], optimized: &[AssemblyToken]) -> ValidationResult {
		let mut result = ValidationResult::default();

		if self.check_correctness {
			result.semantically_equivalent = self.check_semantic_equivalence(original, optimized);
		}

		if self.check_performance {
			result.performance_improved = self.check_performance_improvement(original, optimized);
		}

		result
	}

	fn check_semantic_equivalence(&self, original: &[AssemblyToken], optimized: &[AssemblyToken]) -> bool {
		// Basic semantic check (can be expanded)
		let original_regs = self.extract_registers(original);
		let optimized_regs = self.extract_registers(optimized);
		
		// Check if the same registers are modified
		original_regs == optimized_regs
	}

	fn check_performance_improvement(&self, original: &[AssemblyToken], optimized: &[AssemblyToken]) -> bool {
		// Basic performance check (can be expanded)
		optimized.len() <= original.len()
	}

	fn extract_registers(&self, tokens: &[AssemblyToken]) -> HashSet<String> {
		tokens.iter()
			.filter(|token| token.token_type == crate::model::encoder::TokenType::Register)
			.map(|token| token.value.clone())
			.collect()
	}
}

#[derive(Debug, Default)]
pub struct ValidationResult {
	pub semantically_equivalent: bool,
	pub performance_improved: bool,
}