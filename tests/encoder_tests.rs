use neurassembly::model::encoder::{AssemblyEncoder, TokenType};

#[test]
fn test_basic_instruction_encoding() {
    let mut encoder = AssemblyEncoder::new();
    let assembly = "mov rax, rbx";
    let tokens = encoder.encode(assembly);

    assert_eq!(tokens.len(), 3); // mnemonic + register + register
    assert_eq!(tokens[0].token_type, TokenType::Mnemonic);
    assert_eq!(tokens[0].value, "mov");
    assert_eq!(tokens[1].token_type, TokenType::Register);
    assert_eq!(tokens[1].value, "rax");
    assert_eq!(tokens[2].token_type, TokenType::Register);
    assert_eq!(tokens[2].value, "rbx");
}

#[test]
fn test_memory_operand_encoding() {
    let mut encoder = AssemblyEncoder::new();
    let assembly = "mov dword ptr [rax + rbx*4 + 0x10], ecx";
    let tokens = encoder.encode(assembly);

    // Verify memory operand components
    let memory_tokens: Vec<_> = tokens.iter()
        .filter(|t| matches!(t.token_type, TokenType::Memory | TokenType::Register | TokenType::Immediate))
        .collect();

    assert!(tokens.iter().any(|t| t.value == "dword")); // Size prefix
    assert!(memory_tokens.iter().any(|t| t.value == "rax")); // Base register
    assert!(memory_tokens.iter().any(|t| t.value == "rbx")); // Index register
    assert!(memory_tokens.iter().any(|t| t.value == "4")); // Scale
    assert!(memory_tokens.iter().any(|t| t.value == "0x10")); // Displacement
    assert!(memory_tokens.iter().any(|t| t.value == "ecx")); // Source register
}

#[test]
fn test_vocabulary_building() {
    let mut encoder = AssemblyEncoder::new();
    
    // Get IDs for some tokens
    let mov_id = encoder.get_token_id("mov");
    let rax_id = encoder.get_token_id("rax");
    let rbx_id = encoder.get_token_id("rbx");

    // Verify we can get the tokens back
    assert_eq!(encoder.get_token(mov_id), Some("mov"));
    assert_eq!(encoder.get_token(rax_id), Some("rax"));
    assert_eq!(encoder.get_token(rbx_id), Some("rbx"));

    // Verify vocabulary size
    assert_eq!(encoder.get_vocabulary_size(), 3);
} 