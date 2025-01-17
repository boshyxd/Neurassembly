
pub struct DataPreprocessor {
	pub normalize_registers: bool,
	pub remove_comments: bool,
}

impl DataPreprocessor {
	pub fn new() -> Self {
		Self {
			normalize_registers: true,
			remove_comments: true,
		}
	}

	pub fn preprocess(&self, assembly: &str) -> String {
		let mut result = assembly.to_string();
		
		if self.remove_comments {
			result = self.remove_comments(&result);
		}
		
		if self.normalize_registers {
			result = self.normalize_registers(&result);
		}
		
		result
	}

	fn remove_comments(&self, assembly: &str) -> String {
		assembly.lines()
			.filter_map(|line| {
				let line = line.trim();
				if line.starts_with(';') || line.starts_with('#') {
					None
				} else {
					Some(line.split(';').next().unwrap_or("").trim().to_string())
				}
			})
			.collect::<Vec<_>>()
			.join("\n")
	}

	fn normalize_registers(&self, assembly: &str) -> String {
		// Basic register normalization (can be expanded)
		assembly.replace("eax", "rax")
			   .replace("ebx", "rbx")
			   .replace("ecx", "rcx")
			   .replace("edx", "rdx")
	}
}